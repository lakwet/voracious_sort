use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, Radixable};
use super::msd_sort::copy_by_histogram;
use super::utils::{get_histogram, prefix_sums, Params};

fn msd_stable_radixsort_rec<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: Params,
) {
    if arr.len() <= 128 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    let histogram = get_histogram(arr, &p, mask, shift);
    let (p_sums, mut heads, _) = prefix_sums(&histogram);

    let mut buffer = arr.to_vec();

    copy_by_histogram(arr.len(), &mut buffer, arr, &mut heads, mask, shift);

    let mut rest = arr;
    if p.level < p.max_level - 1 {
        for i in 0..(p.radix_range) {
            let bucket_end = p_sums[i + 1] - p_sums[i];
            let (first_part, second_part) = rest.split_at_mut(bucket_end);
            rest = second_part;
            if histogram[i] > 1 {
                let new_params = p.new_level(p.level + 1);
                msd_stable_radixsort_rec(first_part, new_params);
            }
        }
    }
}

fn msd_stable_radixsort_aux<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
) {
    if arr.len() <= 128 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let (offset, _) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(offset, radix);

    if max_level == 0 {
        return;
    }

    let params = Params::new(0, radix, offset, max_level);

    msd_stable_radixsort_rec(arr, params);
}

/// # MSD stable sort
///
/// An implementation of the
/// [MSD sort](https://en.wikipedia.org/wiki/Radix_sort)
/// algorithm.
///
/// Implementation has been deeply optimized:
/// - Small preliminary check to skip prefix zero bits.
/// - Use vectorization.
///
/// We choose to use an out of place implementation to have a fast radix sort
/// for small input. This sort is used as a fallback for other radix sort from
/// this crate.
///
/// The Verge sort pre-processing heuristic is also added.
///
/// This MSD sort is an out of place stable radix sort.
pub fn msd_stable_radixsort<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
) {
    if arr.len() <= 128 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators =
        verge_sort_preprocessing(arr, radix, &msd_stable_radixsort_aux);
    k_way_merge(arr, &mut separators);
}
