use super::super::{Radixable, RadixableForContainer};
use super::comparative_sort::insertion_sort;
use super::utils::{get_histogram, prefix_sums, swap, Params};

fn serial_swap<T>(
    arr: &mut [T],
    heads: &mut Vec<usize>,
    tails: &[usize],
    p: &Params,
    mask: <[T] as RadixableForContainer>::KeyType,
    shift: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType> + Copy,
    [T]: RadixableForContainer,
{
    for i in 0..(p.radix_range) - 1 {
        while heads[i] < tails[i] {
            unsafe {
                let mut bucket =
                    arr.get_unchecked(heads[i]).get_key(mask, shift);
                while bucket != i {
                    swap(arr, heads[i], heads[bucket]);
                    heads[bucket] += 1;
                    bucket = arr.get_unchecked(heads[i]).get_key(mask, shift);
                }
                heads[i] += 1;
            }
        }
    }
}

pub fn serial_radixsort_rec<T>(arr: &mut [T], p: Params)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= 64 {
        insertion_sort(arr);
        return;
    }

    let (mask, shift) = arr.get_mask_and_shift(&p);
    let histogram = get_histogram(arr, &p, mask, shift);
    let (p_sums, mut heads, tails) = prefix_sums(&histogram);

    serial_swap(arr, &mut heads, &tails, &p, mask, shift);

    let mut rest = arr;
    if p.level < p.max_level - 1 {
        for i in 0..(p.radix_range) {
            let bucket_end = p_sums[i + 1] - p_sums[i];
            let (first_part, second_part) = rest.split_at_mut(bucket_end);
            rest = second_part;
            if histogram[i] > 1 {
                let new_params = p.new_level(p.level + 1);
                serial_radixsort_rec(first_part, new_params);
            }
        }
    }
}

pub fn american_flag_sort<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= 64 {
        insertion_sort(arr);
        return;
    }

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    serial_radixsort_rec(arr, params);
}
