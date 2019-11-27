use super::super::Radixable;
use super::american_flag_sort::serial_radixsort_rec;
use super::comparative_sort::insertion_sort;
use super::utils::{get_histogram, prefix_sums, swap, Params};

const UNROLL_SIZE: usize = 4;

pub fn ska_swap<T: Radixable + Copy>(
    arr: &mut [T],
    heads: &mut Vec<usize>,
    tails: &[usize],
    mask: <T as Radixable>::KeyType,
    shift: usize,
) {
    let mut buckets_size = Vec::new();
    for i in 0..heads.len() {
        buckets_size.push((i, tails[i] - heads[i]))
    }
    buckets_size.sort_unstable_by_key(|elt| elt.1);
    buckets_size.pop();

    while !buckets_size.is_empty() {
        let mut to_remove = Vec::new();
        for (i, (computed_index, _)) in buckets_size.iter().enumerate() {
            let span = tails[*computed_index] - heads[*computed_index];

            if span > 0 {
                let offset = heads[*computed_index];
                let quotient = span / UNROLL_SIZE;
                let remainder = span % UNROLL_SIZE;

                for q in 0..quotient {
                    let origin = offset + q * UNROLL_SIZE;

                    unsafe {
                        let tb0 =
                            arr.get_unchecked(origin).extract(mask, shift);
                        let dest_index_0 = heads[tb0];
                        heads[tb0] += 1;
                        let tb1 =
                            arr.get_unchecked(origin + 1).extract(mask, shift);
                        let dest_index_1 = heads[tb1];
                        heads[tb1] += 1;
                        let tb2 =
                            arr.get_unchecked(origin + 2).extract(mask, shift);
                        let dest_index_2 = heads[tb2];
                        heads[tb2] += 1;
                        let tb3 =
                            arr.get_unchecked(origin + 3).extract(mask, shift);
                        let dest_index_3 = heads[tb3];
                        heads[tb3] += 1;

                        swap(arr, origin, dest_index_0);
                        swap(arr, origin + 1, dest_index_1);
                        swap(arr, origin + 2, dest_index_2);
                        swap(arr, origin + 3, dest_index_3);
                    }
                }

                let new_off = offset + UNROLL_SIZE * quotient;

                for i in 0..remainder {
                    unsafe {
                        let bucket =
                            arr.get_unchecked(new_off + i).extract(mask, shift);
                        swap(arr, new_off + i, heads[bucket]);
                        heads[bucket] += 1;
                    }
                }
            } else {
                to_remove.push(i);
            }
        }

        to_remove.reverse();
        for i in to_remove.iter() {
            buckets_size.remove(*i);
        }
    }
}

fn ska_sort_rec<T: Radixable + Copy + PartialOrd>(arr: &mut [T], p: Params) {
    if arr.len() <= 64 {
        insertion_sort(arr);
        return;
    }
    if arr.len() <= 1024 {
        serial_radixsort_rec(arr, p);
        return;
    }

    let dummy = arr[0];
    let (mask, shift) = dummy.get_mask_and_shift(&p);
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
                let new_params = p.new_level(p.level + 1);
                ska_sort_rec(first_part, new_params);
            }
        }
    }
}

/// # Ska sort
///
/// An implementation of the
/// [Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
/// algorithm.
///
/// The Ska sort is an in place unstable radix sort.
pub fn ska_sort<T: Radixable + Copy + PartialOrd>(arr: &mut [T], radix: usize) {
    if arr.len() <= 64 {
        insertion_sort(arr);
        return;
    }

    let dummy = arr[0];
    let (offset, _) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    ska_sort_rec(arr, params);
}
