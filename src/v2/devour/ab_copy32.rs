//! Dev only (feature `ab`): A/B copy of the u32 / i32 chains (`u32.rs`,
//! `i32.rs`); edit here, measured against the library by `examples/ab_u32.rs`
//! (`copy`). Trial: the DLSD as the engine below `dlsd_max()` keys
//! (`AB_DLSD_MAX`, default 262 144); its scratch on the stack up to 4096 keys
//! unless `AB_DLSD_HEAP` is set.

/// Below this many keys a chunk goes to the DLSD (env `AB_DLSD_MAX`).
fn dlsd_max() -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("AB_DLSD_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(262_144)
    })
}

/// The DLSD with its scratch on the heap (env `AB_DLSD_HEAP`).
fn dlsd_heap() -> bool {
    static H: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *H.get_or_init(|| std::env::var("AB_DLSD_HEAP").is_ok())
}

macro_rules! chain32_ab {
    ($name:ident, $t:ty, $c:ident, $small:expr, $ls:ident, $dl:ident, $st:ident, $ro:ident) => {
        pub(crate) mod $name {
        use super::{dlsd_heap, dlsd_max};
        use crate::v2::engines::dlsd::$st;
        use std::mem::MaybeUninit;
        use crate::v2::cache::{budget, Budget};
        use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
        use crate::v2::decide::shape::{globally_ordered, locally_ordered};
        use crate::v2::devour::keys::*;
        use crate::v2::devour::DevourSort;
        use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
        use crate::v2::engines::{dlsd::$dl, lsd::$ls, roller::$ro};
        use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
        use crate::v2::runs::merge::{self, merge_runs};
        use crate::v2::runs::verge::verge_sort_preprocessing_fast;
        use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
        use crate::v2::special::plateaus::plateaus;
        use crate::v2::special::{spread, DENSE_MIN, SAMPLE};

        /// Up to this size: `sort_unstable` (measured where it crosses the radix
        /// path).
        const SMALL: usize = $small;

        /// The rollercoaster from this size (the L2 bytes; for its memory, n instead of 2n).
        fn roller_min() -> usize { budget(Budget::Roller32) }

        // The range, dense and few detectors of this type, in module `detectors`.
        int_detectors!(detectors, $t, u32, $c, $c, super::roller_min);

        /// Sorts `arr`: a sorting network up to `NETWORK_MAX` keys, else the chain.
        pub(crate) fn sort(arr: &mut [$t]) {
            if arr.len() <= NETWORK_MAX {
                trace!("network");
                network_sort(bits_of(arr), $c, $c);
            } else {
                sort_large(arr);
            }
        }

        /// Arrays beyond the sorting network.
        #[inline(never)]
        fn sort_large(arr: &mut [$t]) {
            let n = arr.len();
            chain! {
                "merge_small" if n <= MERGE_MAX => do { merge_sort(bits_of(arr), $c, $c) };
            }
            let known = match presorted(arr) {
                Ok(()) => {
                    trace!("presorted");
                    return;
                },
                Err(known) => known,
            };
            chain! {
                "std_small" if n <= SMALL => do { arr.sort_unstable() };
            }
            let buffer = std::cell::RefCell::new(Vec::new());
            let sort_chunk = |chunk: &mut [$t]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
            path(arr, known, &detect, &sort_chunk);
        }

        /// The detectors, on one spread sample (sorted once, only when the detectors
        /// that need it in order are reached). `equal_pairs`: equal pairs among the
        /// probes of the near-sorted repair.
        fn detect(arr: &mut [$t], equal_pairs: usize) -> bool {
            let mut sample = spread(arr);
            chain! {
                return true;
                "range" => try detectors::range(arr, &sample);
                "plateaus" => try plateaus(arr, equal_pairs);
                // One dominant value among spread ones, short array: the std
                // partition settles the keys equal to its pivot in one pass.
                "dominant" if arr.len() < DENSE_MIN
                    && dominant(&sample, |x, y| x == y)
                    => do { arr.sort_unstable() };
                "few" => try detectors::few(arr, &sample);
            }
            // Heavy keys and dense ranges: the sample in order.
            if arr.len() < HEAVY_MIN || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y)) {
                return false;
            }
            sample.sort_unstable();
            chain! {
                return true;
                "heavy" => try heavy_sorted_by(
                    arr,
                    &sample,
                    HEAVY_MIN,
                    &|r: &mut [$t]| sort(r),
                    |x, y| x == y,
                    |x, y| x < y,
                );
                "dense" if arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
            }
            false
        }

        /// The engine of one chunk (the whole array, or a gap between Verge's runs).
        #[inline(always)]
        fn sort_chunk(bits: &mut [u32], _: &mut Vec<MaybeUninit<u32>>) {
            // The rollercoaster beyond the cache only on ordered keys (the MSD pass
            // then writes in few sequential streams; random keys go 15 to 45 %
            // faster through the LSD from 30M to 1G).
            if bits.len() < dlsd_max() {
                trace!("dlsd");
                if dlsd_heap() {
                    $dl::dlsd(bits);
                } else {
                    $st::dlsd(bits);
                }
            } else if bits.len() >= roller_min() && (locally_ordered(bits) || globally_ordered(bits)) {
                trace!("roller");
                $ro::roller(bits);
            } else {
                trace!("lsd");
                $ls::lsd(bits);
            }
        }

        /// The radix path (copy of `decide::path`); the first matching
        /// rule wins.
        fn path(
            arr: &mut [$t],
            known: Unsorted,
            detect: &dyn Fn(&mut [$t], usize) -> bool,
            sort: &dyn Fn(&mut [$t]),
        ) {
            let n = arr.len();
            let p = known.ascending_prefix;
            chain! {
                // A sorted prefix of at least half the array: the tail alone goes
                // through the path, then one merge. A short tail: sorted (or
                // reversed) as it is, else the engine.
                "tail_first" if tail_first(n, p) => do {
                    let tail = &mut arr[p..];
                    if tail.len() >= TAIL_PATH_MIN {
                        path(tail, Unsorted::default(), detect, sort);
                    } else if presorted(tail).is_err() {
                        sort(tail);
                    }
                    merge_runs(arr, &mut vec![0, p, n]);
                };
            }
            let simd_rule = merge::simd_rule_applies::<$t>(n);
            let random_runs = !simd_rule && starts_with_random_runs(arr, p);
            let (sorted, equal_pairs) = if random_runs {
                (false, known.probes.map_or(0, |p| p.equal()))
            } else {
                repair_nearly_sorted(arr, known)
            };
            if random_runs {
                trace!("random_runs");
            }
            chain! {
                "repaired" if sorted => do {};
                _ => try detect(arr, equal_pairs);
                "no_verge" if random_runs || can_skip_verge(arr) => do { sort(arr) };
            }
            // Verge: the unsorted chunks are recorded, sorted after the merge rule.
            let base = arr.as_ptr() as usize;
            let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
            let mut separators =
                verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [$t], _| {
                    let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<$t>();
                    chunks.borrow_mut().push((start, chunk.len()));
                });
            let chunks = chunks.into_inner();
            // SIMD merges: merging beats the radix sort up to a few merge levels.
            let segments = separators.len().saturating_sub(1);
            let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
            let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
            let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
            let sort_chunks = |arr: &mut [$t]| {
                for &(start, len) in chunks.iter() {
                    sort(&mut arr[start..start + len]);
                }
            };
            chain! {
                "verge_whole" if unpredictable && !simd_merge => do { sort(arr) };
                "verge_simd_merge" if unpredictable => do {
                    sort_chunks(arr);
                    merge::merge_runs_simd(arr, &mut separators);
                };
                "verge_merge" => do {
                    sort_chunks(arr);
                    merge_runs(arr, &mut separators);
                };
            }
        }

        /// Tails from this length go through the whole path (repair, detectors);
        /// shorter ones go to the engine.
        const TAIL_PATH_MIN: usize = 4096;

        /// A sorted prefix of `p` keys out of `n`: the tail-first path.
        fn tail_first(n: usize, p: usize) -> bool { p >= n / 2 && p < n }
        }
    };
}

chain32_ab!(cu32, u32, id32, 200, wu32, du32, su32, ru32);
chain32_ab!(ci32, i32, flip32, 150, wi32, di32, si32, ri32);
