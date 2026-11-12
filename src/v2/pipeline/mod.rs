//! The radix path shared by all key types, after the presorted check:
//!
//! 1. sorted random runs (sorted sources interleaved): straight to step 4;
//! 2. repair of nearly sorted arrays;
//! 3. the detectors of the key type (few values, ranges, heavy hitters...);
//! 4. Verge: long runs kept, the rest sorted by the engine (`sort`), the
//!    segments merged — or the whole array sorted when the merges would be
//!    unpredictable (they then cost more than a radix sort).

pub(crate) mod merge;
mod near_sorted;
pub(crate) mod presorted;
pub(crate) mod verge;

use merge::{merge_runs, predictable_merge};
use presorted::{repair_nearly_sorted, Unsorted};
use verge::{compute_big_enough_run, explore_forward_asc, verge_sort_preprocessing_fast};

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
    let random_runs = check_random_runs && starts_with_random_runs(arr, known.ascending_prefix);
    let (sorted, equal_pairs) = if random_runs {
        (false, known.probes.map_or(0, |p| p.equal()))
    } else {
        repair_nearly_sorted(arr, known)
    };
    if sorted || detect(arr, equal_pairs) {
        return;
    }
    if random_runs || can_skip_verge(arr) {
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
    if merges_unpredictable(arr, &separators, &chunks) {
        sort(arr);
        return;
    }
    for &(start, len) in chunks.iter() {
        sort(&mut arr[start..start + len]);
    }
    merge_runs(arr, &mut separators);
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
/// than one unpredictable merge round. Chunks are not sorted yet: only
/// run-run pairs are sampled.
fn merges_unpredictable<T: Copy + PartialOrd>(arr: &[T], separators: &[usize], chunks: &[(usize, usize)]) -> bool {
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
        if !predictable_merge(&arr[s..e], m - s) {
            unpredictable += 1;
        }
    }
    sampled > 0 && 2 * unpredictable > sampled
}
