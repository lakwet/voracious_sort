//! The radix path shared by all key types, after the presorted check. The
//! first matching line wins (`trace!` label in brackets):
//!
//! | when                                             | then                                         |
//! |--------------------------------------------------|----------------------------------------------|
//! | sorted prefix of at least n / 2 keys             | the tail alone through the path (from        |
//! |                                                  | `TAIL_PATH_MIN` keys), one merge [tail_first] |
//! | two long ascending runs first (n / log2 n),      | no repair, no Verge: detectors, then the     |
//! | merged unpredictably, no SIMD merge rule         | engine on the whole array [random_runs]      |
//! | few breaks of order (presorted probes)           | repaired in place [repaired]                 |
//! | a detector of the key type sorts it              | done [range, few, heavy, dense, plateaus...] |
//! | no monotone run of n / log2 n (`RUN_PROBE`       | the engine on the whole array [no_verge]     |
//! | pairs probed on a grid)                          |                                              |
//! | merges unpredictable (`merges_unpredictable`),   | the engine on the whole array [verge_whole]  |
//! | no SIMD merge                                    |                                              |
//! | else                                             | Verge: runs kept, gaps by the engine, merged |
//! |                                                  | [verge_merge; verge_simd_merge by blocks]    |
//!
//! The engine of each chunk: `decide::chunks`.

use crate::v2::inspect::presorted::{repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs, predictable_merge};
use crate::v2::runs::verge::{compute_big_enough_run, explore_forward_asc, verge_sort_preprocessing_fast};

/// Merge levels from which dense merges cost more than a radix sort (see
/// `merges_unpredictable`).
const DENSE_LEVELS: usize = 4;

/// Tails from this length go through the whole path (repair, detectors);
/// shorter ones go to the engine. Same value as `DENSE_MIN` (special), the
/// size from which the detectors pay off: could be tied to it.
const TAIL_PATH_MIN: usize = 4096;

/// Whether an array of `n` keys with a sorted prefix of `p` keys takes the
/// tail-first path of `radix_path`.
pub(crate) fn tail_first(n: usize, p: usize) -> bool {
    p >= n / 2 && p < n
}

/// Monotone pairs probed at each grid point by `can_skip_verge`.
const RUN_PROBE: usize = 8;

/// Sorts `arr` along the radix path. `known`: what the presorted check
/// learned; `check_random_runs`: step 1 on; `detect`: the detectors (given
/// the number of equal probe pairs), returning whether they sorted the
/// array; `sort`: the radix engine.
pub(crate) fn radix_path<T: Copy + PartialOrd>(
    arr: &mut [T],
    known: Unsorted,
    check_random_runs: bool,
    detect: &dyn Fn(&mut [T], usize) -> bool,
    sort: &dyn Fn(&mut [T]),
) {
    // A sorted prefix of at least half the array (seen by the presorted
    // check): the tail alone goes through the path, then one merge, instead
    // of scanning the prefix again (repair, Verge). u128 sorted + 10 % random
    // tail 100k 2.96 to 2.07 ns per key.
    let n = arr.len();
    let p = known.ascending_prefix;
    if tail_first(n, p) {
        trace!("tail_first");
        // A short tail: sorted (or reversed) as it is, else the engine (the
        // whole path needs more keys).
        let tail = &mut arr[p..];
        if tail.len() >= TAIL_PATH_MIN {
            radix_path(tail, Unsorted::default(), check_random_runs, detect, sort);
        } else if crate::v2::inspect::presorted::presorted(tail).is_err() {
            sort(tail);
        }
        merge_runs(arr, &mut vec![0, p, n]);
        return;
    }
    let simd_rule = merge::simd_rule_applies::<T>(arr.len());
    let random_runs = check_random_runs && !simd_rule && starts_with_random_runs(arr, known.ascending_prefix);
    let (sorted, equal_pairs) = if random_runs {
        (false, known.probes.map_or(0, |p| p.equal()))
    } else {
        repair_nearly_sorted(arr, known)
    };
    if random_runs {
        trace!("random_runs");
    }
    if sorted {
        trace!("repaired");
        return;
    }
    if detect(arr, equal_pairs) {
        return;
    }
    if random_runs || can_skip_verge(arr) {
        trace!("no_verge");
        sort(arr);
        return;
    }
    // Verge: the unsorted chunks are recorded, sorted after the merge rule.
    let base = arr.as_ptr() as usize;
    let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [T], _| {
        let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<T>();
        chunks.borrow_mut().push((start, chunk.len()));
    });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
    if unpredictable && !simd_merge {
        trace!("verge_whole");
        sort(arr);
        return;
    }
    trace!(if unpredictable { "verge_simd_merge" } else { "verge_merge" });
    for &(start, len) in chunks.iter() {
        sort(&mut arr[start..start + len]);
    }
    if unpredictable {
        merge::merge_runs_simd(arr, &mut separators);
    } else {
        merge_runs(arr, &mut separators);
    }
}

/// The array starts with two long ascending runs (the first one, `prefix`
/// keys, already scanned) whose merge would be unpredictable: Verge would
/// end by sorting the whole array anyway, so the repair and Verge are
/// skipped. Long: Verge's length, n / log2(n).
fn starts_with_random_runs<T: Copy + PartialOrd>(arr: &[T], prefix: usize) -> bool {
    let n = arr.len();
    let long = compute_big_enough_run(n);
    if prefix < long || prefix >= n {
        return false;
    }
    let end = explore_forward_asc(arr, prefix);
    end - prefix >= long && !predictable_merge(&arr[..end], prefix)
}

/// Whether Verge would keep no run (it keeps monotone runs of at least
/// n / log2(n) keys): such a run would contain a point of a grid of half
/// that step followed by `RUN_PROBE` monotone pairs. Random keys fail each
/// probe within a couple of pairs.
fn can_skip_verge<T: Copy + PartialOrd>(arr: &[T]) -> bool {
    let n = arr.len();
    let step = compute_big_enough_run(n) / 2;
    if step <= RUN_PROBE {
        return false;
    }
    let mut p = 0;
    while p + RUN_PROBE < n {
        let window = &arr[p..=p + RUN_PROBE];
        let ascending = window.windows(2).all(|x| x[0] <= x[1]);
        let descending = window.windows(2).all(|x| x[0] >= x[1]);
        if ascending || descending {
            return false;
        }
        p += step;
    }
    true
}

/// Whether merging the Verge segments would cost more than sorting the
/// whole array: most first-round merges between two runs unpredictable for
/// a branchy merge (about as costly as a whole radix sort each). From 3
/// segments; from 2 for keys of at most 16 bits, whose radix sort is cheaper
/// than one unpredictable merge round. Dense merges (one element per
/// comparison) count as unpredictable from `DENSE_LEVELS` merge levels, for
/// arrays beyond a quarter of the largest cache (L2 / 4 alone sent 100k
/// floats to the radix on the Ryzen's 512 KiB L2: f32 saw 2.5 to 7.7): float sawtooth, 18-21 teeth, f32 10M
/// 6.6 to 3.7 ns per key, f64 600k 6.4 to 4.4; in the cache the merges stay
/// cheaper (f32 600k 4.9 against 7.2, f64 100k 4.6 against 6.5). Chunks are
/// not sorted yet: only run-run pairs are sampled.
fn merges_unpredictable<T: Copy + PartialOrd>(arr: &[T], separators: &[usize], chunks: &[(usize, usize)], levels: usize) -> bool {
    let beyond = std::mem::size_of_val(arr) > crate::v2::cache::largest_bytes() / 4;
    let dense_levels = beyond && levels >= DENSE_LEVELS;
    let narrow = std::mem::size_of::<T>() <= 2;
    if separators.len() < if narrow { 3 } else { 4 } {
        return false;
    }
    let is_chunk = |s: usize| chunks.iter().any(|&(c, _)| c == s);
    let (mut sampled, mut unpredictable) = (0, 0);
    for w in separators.windows(3).step_by(2) {
        let (s, m, e) = (w[0], w[1], w[2]);
        if is_chunk(s) || is_chunk(m) || arr[m - 1] <= arr[m] {
            continue;
        }
        sampled += 1;
        if !predictable_merge(&arr[s..e], m - s) || (dense_levels && merge::dense_merge(&arr[s..e], m - s)) {
            unpredictable += 1;
        }
    }
    sampled > 0 && 2 * unpredictable > sampled
}
