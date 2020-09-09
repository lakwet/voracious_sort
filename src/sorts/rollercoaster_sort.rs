use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::{
    explore_simple_forward, verge_sort_preprocessing, Orientation,
};
use super::super::{RadixKey, RadixSort, Radixable};
use super::counting_sort::counting_sort;
use super::dlsd_sort::dlsd_radixsort_body;
use super::lsd_sort::lsd_radixsort_body;
use super::ska_sort::ska_swap;
use super::utils::{get_histogram, prefix_sums, Params};

pub fn fallback<T: Radixable<K>, K: RadixKey>(arr: &mut [T], p: Params) {
    let size = arr.len();
    // It is a fallback, we don't want to sort big array.
    assert!(size <= 128_000);

    if size <= 256 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let remaining_level = p.max_level - p.level;
    if remaining_level < 4 {
        let new_offset = ((p.offset / p.radix) * p.radix) + (p.level * p.radix);
        let p = Params::new(0, p.radix, new_offset, remaining_level);
        lsd_radixsort_body(arr, p);
    } else {
        let new_offset = p.level * p.radix + p.offset;
        let new_max_level = if size <= 65_536 { 2 } else { 3 };
        let new_params = Params::new(0, p.radix, new_offset, new_max_level);
        dlsd_radixsort_body(arr, new_params, new_max_level, true);
    }
}

fn rollercoaster_sort_rec<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: Params,
    zhc: usize, // zipf heuristic count
    first_pass: bool,
) {
    if !first_pass && arr.len() <= 128_000 {
        fallback(arr, p);
        return;
    }

    let dummy = arr[0];
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
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
                // Heuristic for signed integer with at least or more than
                // 64bits.
                // The idea is to skip all the non used bits after the signed
                // bit.
                if first_pass && p.radix == 8 && i == 128 && dummy.is_i32() {
                    unsafe {
                        let arr_u32 =
                            &mut *(first_part as *mut [T] as *mut [u32]);
                        arr_u32.voracious_sort();
                    }
                } else if first_pass
                    && p.radix == 8
                    && i == 128
                    && dummy.is_i64()
                {
                    unsafe {
                        let arr_u64 =
                            &mut *(first_part as *mut [T] as *mut [u64]);
                        arr_u64.voracious_sort();
                    }
                } else if first_pass
                    && p.radix == 8
                    && i == 128
                    && dummy.is_i128()
                {
                    unsafe {
                        let arr_u128 =
                            &mut *(first_part as *mut [T] as *mut [u128]);
                        arr_u128.voracious_sort();
                    }
                } else {
                    let new_params = p.new_level(p.level + 1);
                    if zhc > 0 {
                        match explore_simple_forward(first_part) {
                            Orientation::IsAsc => (),
                            Orientation::IsDesc => {
                                first_part.reverse();
                            },
                            Orientation::IsPlateau => (),
                            Orientation::IsNone => {
                                rollercoaster_sort_rec(
                                    first_part,
                                    new_params,
                                    zhc - 1,
                                    false,
                                );
                            },
                        }
                    } else {
                        rollercoaster_sort_rec(
                            first_part, new_params, 0, false,
                        );
                    }
                }
            }
        }
    }
}

fn rollercoaster_sort_aux<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
    heuristic: bool,
    min_cs2: usize,
) {
    let size = arr.len();
    if size <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let (_, raw_offset) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(raw_offset, radix);

    let (offset_reg, _) = dummy.compute_offset(arr, 8);
    let max_level_reg = dummy.compute_max_level(offset_reg, 8);

    if max_level == 0 {
        return;
    }

    let params = Params::new(0, radix, raw_offset, max_level);

    if heuristic {
        if max_level_reg == 1 {
            counting_sort(arr, 8);
        } else if max_level_reg == 2 && arr.len() >= min_cs2 {
            counting_sort(arr, 16);
        } else {
            rollercoaster_sort_rec(arr, params, 2, true);
        }
    } else {
        rollercoaster_sort_rec(arr, params, 2, true);
    }
}

/// # Rollercoaster sort
///
/// This sort is this crate's author invention. This is a Voracious sort (in its
/// single thread version) which immediatly fallbacks on a DLSD sort. A new
/// fallback strategy for small chunks has been found.
///
/// The name is because this sort can switch between a LSD or a MSD strategy.
///
/// The Verge sort pre-processing heuristic is also added.
///
/// This Rollercoaster sort is an out of place unstable radix sort.
pub fn rollercoaster_sort<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        rollercoaster_sort_aux(arr, radix, false, 0)
    });
    k_way_merge(arr, &mut separators);
}

pub fn rollercoaster_sort_heu<T, K>(arr: &mut [T], radix: usize, min_cs2: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        rollercoaster_sort_aux(arr, radix, true, min_cs2)
    });
    k_way_merge(arr, &mut separators);
}
