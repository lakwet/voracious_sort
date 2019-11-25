use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{Radixable, RadixableForContainer};
use super::msd_sort::copy_by_histogram;
use super::utils::{copy_nonoverlapping, get_histogram, prefix_sums, Params};

const THRESHOLD: usize = 128;

fn string_radixsort_rec<T>(
    arr: &mut [T],
    buffer: &mut [T],
    index: usize,
    p: Params,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= THRESHOLD {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if index == 1 {
            copy_nonoverlapping(arr, buffer, arr.len());
        }
        return;
    }

    let (mask, shift) = arr.get_mask_and_shift(&p);
    let histogram = get_histogram(arr, &p, mask, shift);
    let (p_sums, mut heads, _) = prefix_sums(&histogram);

    copy_by_histogram(arr.len(), arr, buffer, &mut heads, mask, shift);

    if p.level < p.max_level - 1 {
        let mut rest = arr;
        let mut rest_buffer = buffer;
        for i in 0..(p.radix_range) {
            let bucket_end = p_sums[i + 1] - p_sums[i];
            let (mut first_part, second_part) = rest.split_at_mut(bucket_end);
            let (mut first_part_buf, second_part_buf) =
                rest_buffer.split_at_mut(bucket_end);
            rest = second_part;
            rest_buffer = second_part_buf;
            if histogram[i] > 1 {
                let new_params = p.new_level(p.level + 1);
                string_radixsort_rec(
                    &mut first_part_buf,
                    &mut first_part,
                    1 - index,
                    new_params,
                );
            } else if histogram[i] == 1 && index == 0 {
                first_part[0] = first_part_buf[0];
            }
        }
    }
}

fn string_radixsort_aux<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= THRESHOLD {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut buffer: Vec<T> = vec![arr[0]; arr.len()];

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    string_radixsort_rec(arr, &mut buffer, 0, params);
}

pub fn msd_string_radixsort<T>(arr: &mut [T])
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= THRESHOLD {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators =
        verge_sort_preprocessing(arr, 8, &string_radixsort_aux);
    k_way_merge(arr, &mut separators);
}
