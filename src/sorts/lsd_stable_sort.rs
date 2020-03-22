use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, Radixable};
use super::lsd_sort::lsd_radixsort_aux;

/// # LSD stable sort
///
/// An implementation of the
/// [LSD sort](https://en.wikipedia.org/wiki/Radix_sort)
/// algorithm.
///
/// Implementation has been deeply optimized:
/// - Small preliminary check to skip prefix zero bits.
/// - Use ping pong copy.
/// - Use vectorization.
/// - Compute histograms in one pass.
/// - Check the number of non-empty buckets, if only one bucket is non-empty,
/// then skip the `copy_by_histogram`.
///
/// The Verge sort pre-processing heuristic is also added.
///
/// This LSD stable sort is an out of place stable radix sort.
pub fn lsd_stable_radixsort<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        if arr.len() <= 128 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort_aux(arr, radix, false, 0)
        }
    });
    k_way_merge(arr, &mut separators);
}
