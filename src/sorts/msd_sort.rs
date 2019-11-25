use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{Radixable, RadixableForContainer};
use super::utils::{get_histogram, prefix_sums, Params};

const UNROLL_SIZE: usize = 4;

#[inline]
pub fn copy_by_histogram<T>(
    size: usize,
    source: &mut [T],
    destination: &mut [T],
    heads: &mut Vec<usize>,
    mask: <[T] as RadixableForContainer>::KeyType,
    shift: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    let quotient = size / UNROLL_SIZE;
    let remainder = size % UNROLL_SIZE;

    for q in 0..quotient {
        let i = q * UNROLL_SIZE;
        unsafe {
            let b0 = source.get_unchecked(i).get_key(mask, shift);
            let b1 = source.get_unchecked(i + 1).get_key(mask, shift);
            let b2 = source.get_unchecked(i + 2).get_key(mask, shift);
            let b3 = source.get_unchecked(i + 3).get_key(mask, shift);

            let d0 = heads[b0];
            heads[b0] += 1;

            let d1 = heads[b1];
            heads[b1] += 1;

            let d2 = heads[b2];
            heads[b2] += 1;

            let d3 = heads[b3];
            heads[b3] += 1;

            destination[d0] = *source.get_unchecked(i);
            destination[d1] = *source.get_unchecked(i + 1);
            destination[d2] = *source.get_unchecked(i + 2);
            destination[d3] = *source.get_unchecked(i + 3);
        }
    }

    for r in 0..remainder {
        let i = quotient * UNROLL_SIZE + r;
        let target_bucket = source[i].get_key(mask, shift);
        destination[heads[target_bucket]] = source[i];
        heads[target_bucket] += 1;
    }
}

pub fn msd_radixsort_rec<T>(arr: &mut [T], p: Params)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let (mask, shift) = arr.get_mask_and_shift(&p);
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
                msd_radixsort_rec(first_part, new_params);
            }
        }
    }
}

fn msd_radixsort_aux<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    msd_radixsort_rec(arr, params);
}

pub fn msd_radixsort<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators =
        verge_sort_preprocessing(arr, radix, &msd_radixsort_aux);
    k_way_merge(arr, &mut separators);
}