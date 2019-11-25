use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::{
    explore_simple_forward, verge_sort_preprocessing, Orientation,
};
use super::super::{Radixable, RadixableForContainer};
use super::counting_sort::counting_sort;
use super::msd_sort::msd_radixsort_rec;
use super::ska_sort::ska_swap;
use super::utils::{get_histogram, prefix_sums, Params};

pub fn voracious_sort_rec<T>(
    arr: &mut [T],
    p: Params,
    zipf_heuristic_count: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    // Small optimization, use PDQ sort (sort implemented in Std Rust Unstable)
    // instead of insertion sort for small size array.
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }
    // Main optimization is here: better diversion handling.
    // Maybe it seems not important, but, really, it is very important.
    // What is important is to have a sort that is as fast as possible
    // for small size array. msd_radixsort has been designed that way.
    // It is unusual to have an out of place msd radix sort (usually msd radix
    // sort are in place).
    // The threshold has been found by experimental tests.
    if arr.len() <= 30_000 {
        msd_radixsort_rec(arr, p);
        return;
    }

    let (mask, shift) = arr.get_mask_and_shift(&p);
    let histogram = get_histogram(arr, &p, mask, shift);
    let (p_sums, mut heads, tails) = prefix_sums(&histogram);

    ska_swap(arr, &mut heads, &tails, mask, shift);

    let mut rest = arr;
    if p.level < p.max_level - 1 {
        for i in 0..(p.radix_range) {
            let bucket_end = p_sums[i + 1] - p_sums[i];
            let (first_part, second_part) = rest.split_at_mut(bucket_end);
            rest = second_part;
            if histogram[i] > 1 {
                // skip slice with only 1 items (already sorted)
                let new_params = p.new_level(p.level + 1);
                // Other optimization, it costs almost nothing to perform this
                // check, and it allows to gain time on some data distributions.
                if zipf_heuristic_count > 0 {
                    match explore_simple_forward(first_part) {
                        Orientation::IsAsc => (),
                        Orientation::IsDesc => {
                            first_part.reverse();
                        }
                        Orientation::IsPlateau => (),
                        Orientation::IsNone => {
                            voracious_sort_rec(
                                first_part,
                                new_params,
                                zipf_heuristic_count - 1,
                            );
                        }
                    }
                } else {
                    voracious_sort_rec(first_part, new_params, 0);
                }
            }
        }
    }
}

fn voracious_sort_aux<T>(
    arr: &mut [T],
    radix: usize,
    heuristic: bool,
    min_cs2: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    let size = arr.len();
    if size <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    if heuristic {
        // we could add more heuristics, but the idea is to keep an MSD radix
        // sort, so there is no additional memory requirement
        if max_level == 1 {
            counting_sort(arr, 8);
        } else if max_level == 2 && size >= min_cs2 {
            counting_sort(arr, 16);
        } else {
            voracious_sort_rec(arr, params, 2);
        }
    } else {
        voracious_sort_rec(arr, params, 2);
    }
}

pub fn voracious_sort<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        voracious_sort_aux(arr, radix, false, 0)
    });
    k_way_merge(arr, &mut separators);
}

pub fn voracious_sort_heu<T>(arr: &mut [T], radix: usize, min_cs2: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        voracious_sort_aux(arr, radix, true, min_cs2)
    });
    k_way_merge(arr, &mut separators);
}
