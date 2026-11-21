//! Inspections read by the radix path of every type (the path itself, its
//! rules and thresholds, is copied in each `types` file), with the trace
//! label of the step each one leads to:
//!
//! - `starts_with_random_runs`: two long ascending runs first whose merge is
//!   unpredictable (`random_runs`);
//! - `can_skip_verge`: no monotone run of n / log2 n (`no_verge`);
//! - `merges_unpredictable`: Verge's merges would cost more than sorting the
//!   whole array (`verge_whole`).

use crate::v2::runs::merge::{self, predictable_merge};
use crate::v2::runs::verge::{compute_big_enough_run, explore_forward_asc};

/// Merge levels from which dense merges cost more than a radix sort (see
/// `merges_unpredictable`).
const DENSE_LEVELS: usize = 4;

/// Monotone pairs probed at each grid point by `can_skip_verge`.
const RUN_PROBE: usize = 8;

/// The array starts with two long ascending runs (the first one, `prefix`
/// keys, already scanned) whose merge would be unpredictable: Verge would
/// end by sorting the whole array anyway, so the repair and Verge are
/// skipped. Long: Verge's length, n / log2(n).
pub(crate) fn starts_with_random_runs<T: Copy + PartialOrd>(arr: &[T], prefix: usize) -> bool {
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
pub(crate) fn can_skip_verge<T: Copy + PartialOrd>(arr: &[T]) -> bool {
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

/// Whether merging Verge's segments would cost more than sorting the whole
/// array: most sampled merges of two runs are unpredictable for a branchy
/// merge, each about as costly as a whole radix sort. Needs 3 segments (2
/// for keys of at most 16 bits, whose radix sort is cheap). Beyond a quarter
/// of the largest cache, dense merges (one element per comparison) also
/// count from `DENSE_LEVELS` levels. Only run-run pairs are sampled: the
/// chunks are not sorted yet.
pub(crate) fn merges_unpredictable<T: Copy + PartialOrd>(
    arr: &[T],
    separators: &[usize],
    chunks: &[(usize, usize)],
    levels: usize,
) -> bool {
    // Beyond the cache, the k-way merge takes even unpredictable merges
    // (cheaper than a whole radix sort), except for the 32-bit keys, whose
    // SIMD block merge is faster there.
    let in_cache = std::mem::size_of_val(arr)
        <= crate::v2::cache::budget(crate::v2::cache::Budget::MergeTwoWay) / 2;
    let simd32 = std::mem::size_of::<T>() == 4 && crate::v2::runs::merge::simd_applies::<T>();
    if !in_cache && !simd32 {
        return false;
    }
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
        if !predictable_merge(&arr[s..e], m - s)
            || (dense_levels && merge::dense_merge(&arr[s..e], m - s))
        {
            unpredictable += 1;
        }
    }
    sampled > 0 && 2 * unpredictable > sampled
}

#[cfg(test)] mod tests;
