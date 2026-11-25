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

/// Questions switched off (the `copy:MASK` sorts of the example): 1 range,
/// 2 plateaus, 4 dominant, 8 few, 16 heavy, 32 dense, 64 every detector
/// (no sample either), 128 Verge (the engine on the whole array), 256 the
/// near-sorted repair, 512 the light chain switched ON (inverted bit: on(512)
/// false means the light chain runs), 1024 the light chain without its
/// sorted-head rule, 2048 without its dominant test, 4096 (inverted) range
/// and few before the near-sorted repair, 8192 no few4 (at most 4 values on
/// the sample, counted by comparisons).
pub(crate) static SKIP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The light chain up to this many keys (env `AB_LIGHT_MAX`, default 8192).
fn light_max() -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("AB_LIGHT_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(8192)
    })
}

thread_local! {
    /// Range and few already asked on this path (mask bit 4096).
    static EARLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `sort_unstable` up to this many keys (env `AB_SMALL`, default 160; the
/// type's own `SMALL` with `AB_SMALL=0`).
fn small_max(own: usize) -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let m = *M.get_or_init(|| std::env::var("AB_SMALL").ok().and_then(|v| v.parse().ok()).unwrap_or(160));
    if m == 0 { own } else { m }
}

/// Whether the question `bit` is asked.
#[inline(always)]
fn on(bit: u32) -> bool { SKIP.load(std::sync::atomic::Ordering::Relaxed) & bit == 0 }

/// The DLSD with its scratch on the heap (env `AB_DLSD_HEAP`).
fn dlsd_heap() -> bool {
    static H: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *H.get_or_init(|| std::env::var("AB_DLSD_HEAP").is_ok())
}

macro_rules! chain32_ab {
    ($name:ident, $t:ty, $c:ident, $small:expr, $ls:ident, $dl:ident, $st:ident, $ro:ident) => {
        pub(crate) mod $name {
        use super::{dlsd_heap, dlsd_max, light_max, on, small_max, EARLY};
        use crate::v2::engines::dlsd::$st;
        use std::mem::MaybeUninit;
        use crate::v2::cache::{budget, Budget};
        use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
        use crate::v2::decide::shape::{globally_ordered, locally_ordered};
        use crate::v2::devour::keys::*;
        use crate::v2::devour::DevourSort;
        use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
        use crate::v2::engines::{dlsd::$dl, lsd::$ls, roller::$ro};
        use crate::v2::inspect::presorted::{presorted, probe_pairs, repair_nearly_sorted, Unsorted};
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
                "std_small" if n <= small_max(SMALL) => do { arr.sort_unstable() };
            }
            // Trial (mask bit 512): the light chain up to `light_max()` keys:
            // range and few on the spread sample, then the engine (no
            // plateaus, dominant, heavy, dense, near-sorted repair, Verge).
            // A sorted head of half the array, or the 32 spread pairs nearly
            // all in one direction (nearly sorted, saw teeth): the whole path.
            // (`presorted` compares the pairs only when the first key is the
            // larger: else they are compared here, 32 comparisons.)
            let ordered = known.ascending_prefix >= n / 2 || {
                let p = known.probes.unwrap_or_else(|| probe_pairs(arr));
                // (Mostly equal pairs: plateaus, Sqrt.)
                p.ascending >= 28 || p.descending >= 28 || p.equal() >= 24
            };
            if !on(512) && n <= light_max() && !(on(1024) && ordered) {
                let sample = spread(arr);
                if on(8192) && few4(arr, &sample) {
                    trace!("light_few4");
                    return;
                }
                if on(1) && detectors::range(arr, &sample) {
                    trace!("light_range");
                    return;
                }
                // One dominant value with a long tail of distinct values
                // (Sparse), on the first 16 sample keys: the majority vote,
                // its count at least 8 and no other value 3 times or more
                // (FakeZipf repeats a second value: left to `few`). Then
                // the std partition (as `detect`). The full `dominant` on
                // the 64 keys cost up to 0.8 ns per key at 250 keys.
                if on(2048) {
                    let s16 = &sample[..16];
                    let (mut v, mut votes) = (s16[0], 0usize);
                    for &x in s16 {
                        if votes == 0 {
                            v = x;
                        }
                        votes = if x == v { votes + 1 } else { votes - 1 };
                    }
                    let count = |u: $t| s16.iter().filter(|&&x| x == u).count();
                    if count(v) >= 8 && s16.iter().all(|&u| u == v || count(u) < 3) {
                        trace!("light_dominant");
                        arr.sort_unstable();
                        return;
                    }
                }
                // Below 512 keys `few` only when the first 16 sample keys
                // hold at most 8 values (it rejects the others itself, on
                // 16 keys there, but after zeroing its table: 3-4 % at
                // 250-300 keys; from 512 keys it accepts more values).
                let mut distinct = 0;
                for i in 0..16 {
                    distinct += !sample[..i].contains(&sample[i]) as usize;
                }
                if on(8) && (distinct <= 8 || n >= 512) && detectors::few(arr, &sample) {
                    trace!("light_few");
                    return;
                }
                sort_chunk(bits_of(arr), &mut Vec::new());
                return;
            }
            let buffer = std::cell::RefCell::new(Vec::new());
            let sort_chunk = |chunk: &mut [$t]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
            path(arr, known, &detect, &sort_chunk);
        }

        /// At most 4 distinct values on the sample (Binary, Few 4): each key
        /// compared with them in one pass (no branch per key), the keys
        /// written back from the counts when they are all among them (else
        /// nothing written). Port of the 16-bit `cheap_few`; idea of
        /// blazing-radix (a sample proposes at most four keys, verified by a
        /// full scan before any write).
        fn few4(arr: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
            let mut reps = [sample[0]; 4];
            let mut k = 1;
            for &v in &sample[1..] {
                if !reps[..k].contains(&v) {
                    if k == 4 {
                        return false;
                    }
                    reps[k] = v;
                    k += 1;
                }
            }
            reps[..k].sort_unstable();
            match k {
                1 => few4_k::<1>(arr, &reps),
                2 => few4_k::<2>(arr, &reps),
                3 => few4_k::<3>(arr, &reps),
                _ => few4_k::<4>(arr, &reps),
            }
        }

        fn few4_k<const KN: usize>(arr: &mut [$t], reps: &[$t; 4]) -> bool {
            let mut c = [0usize; KN];
            for &x in arr.iter() {
                for j in 0..KN {
                    c[j] += (x == reps[j]) as usize;
                }
            }
            if c.iter().sum::<usize>() != arr.len() {
                return false;
            }
            let mut at = 0;
            for j in 0..KN {
                arr[at..at + c[j]].fill(reps[j]);
                at += c[j];
            }
            true
        }

        /// The detectors, on one spread sample (sorted once, only when the detectors
        /// that need it in order are reached). `equal_pairs`: equal pairs among the
        /// probes of the near-sorted repair.
        fn detect(arr: &mut [$t], equal_pairs: usize) -> bool {
            if !on(64) {
                return false;
            }
            let mut sample = spread(arr);
            chain! {
                return true;
                "few4" if on(8192) => try few4(arr, &sample);
                "range" if on(1) && !EARLY.with(|e| e.get()) => try detectors::range(arr, &sample);
                "plateaus" if on(2) => try plateaus(arr, equal_pairs);
                // One dominant value among spread ones, short array: the std
                // partition settles the keys equal to its pivot in one pass.
                "dominant" if on(4) && arr.len() < DENSE_MIN
                    && dominant(&sample, |x, y| x == y)
                    => do { arr.sort_unstable() };
                "few" if on(8) && !EARLY.with(|e| e.get()) => try detectors::few(arr, &sample);
            }
            // Heavy keys and dense ranges: the sample in order.
            if arr.len() < HEAVY_MIN || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y)) {
                return false;
            }
            sample.sort_unstable();
            chain! {
                return true;
                "heavy" if on(16) => try heavy_sorted_by(
                    arr,
                    &sample,
                    HEAVY_MIN,
                    &|r: &mut [$t]| sort(r),
                    |x, y| x == y,
                    |x, y| x < y,
                );
                "dense" if on(32) && arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
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
            // Trial (mask bit 4096): range and few on the sample before the
            // near-sorted repair when the spread pairs show no order (Binary:
            // the repair costs 7-14 %); `detect` then skips them.
            let early = !on(4096)
                && !known.probes.is_some_and(|p| p.ascending >= 28 || p.descending >= 28);
            if early {
                let sample = spread(arr);
                if on(1) && detectors::range(arr, &sample) {
                    trace!("early_range");
                    return;
                }
                if on(8) && detectors::few(arr, &sample) {
                    trace!("early_few");
                    return;
                }
            }
            EARLY.with(|e| e.set(early));
            let simd_rule = merge::simd_rule_applies::<$t>(n);
            let random_runs = !simd_rule && starts_with_random_runs(arr, p);
            let (sorted, equal_pairs) = if random_runs || !on(256) {
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
                "no_verge" if !on(128) || random_runs || can_skip_verge(arr) => do { sort(arr) };
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
