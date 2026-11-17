//! Inspections read by the radix path of every type (the path itself, its
//! rules and thresholds, is copied in each `types` file):
//!
//! - `starts_with_random_runs`: two long ascending runs first whose merge is
//!   unpredictable [random_runs];
//! - `can_skip_verge`: no monotone run of n / log2 n [no_verge];
//! - `merges_unpredictable`: Verge's merges would cost more than sorting the
//!   whole array [verge_whole].

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

/// Whether merging the Verge segments would cost more than sorting the
/// whole array: most first-round merges between two runs unpredictable for
/// a branchy merge (about as costly as a whole radix sort each). From 3
/// segments; from 2 for keys of at most 16 bits, whose radix sort is cheaper
/// than one unpredictable merge round. Dense merges (one element per
/// comparison) count as unpredictable from `DENSE_LEVELS` merge levels, for
/// arrays beyond a quarter of the largest cache (L2 / 4 alone sent 100k
/// floats to the radix on the Ryzen's 512 KiB L2: f32 saw 2.5 to 7.7):
/// float sawtooth, 18-21 teeth, f32 10M 6.6 to 3.7 ns per key, f64 600k 6.4
/// to 4.4; in the cache the merges stay cheaper (f32 600k 4.9 against 7.2,
/// f64 100k 4.6 against 6.5). Chunks are not sorted yet: only run-run pairs
/// are sampled.
pub(crate) fn merges_unpredictable<T: Copy + PartialOrd>(
    arr: &[T],
    separators: &[usize],
    chunks: &[(usize, usize)],
    levels: usize,
) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    /// Random keys hold no run Verge would keep; a sorted array (or one
    /// with a long run in the middle) does.
    #[test]
    fn skip_verge() {
        let mut r = Rng(53);
        for &n in [4096usize, 100_000].iter() {
            let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
            assert!(can_skip_verge(&random), "random n={}", n);
            let mut run = random.clone();
            run[n / 4..3 * n / 4].sort_unstable();
            assert!(!can_skip_verge(&run), "run n={}", n);
            let asc: Vec<u64> = (0..n as u64).collect();
            assert!(!can_skip_verge(&asc), "sorted n={}", n);
        }
    }

    /// Two long ascending runs of random keys start the array: their merge
    /// is unpredictable. Two runs one after the other (or a short prefix)
    /// are not reported.
    #[test]
    fn random_runs() {
        let mut r = Rng(59);
        let n = 100_000;
        let mut a: Vec<u64> = (0..n).map(|_| r.next()).collect();
        a[..n / 2].sort_unstable();
        a[n / 2..].sort_unstable();
        assert!(starts_with_random_runs(&a, n / 2));
        assert!(!starts_with_random_runs(&a, 10));
        let apart: Vec<u64> =
            (0..n as u64).map(|i| if i < n as u64 / 2 { i } else { i + n as u64 }).collect();
        assert!(!starts_with_random_runs(&apart, n / 2));
    }
}
