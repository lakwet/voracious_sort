use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{Radixable, RadixableForContainer};
use super::comparative_sort::insertion_sort_try;
use super::msd_sort::copy_by_histogram;
use super::utils::{
    copy_nonoverlapping, get_full_histograms_fast, get_partial_histograms_fast,
    only_one_bucket_filled, prefix_sums, Params,
};
use super::voracious_sort::voracious_sort_rec;

const EFST: f64 = 0.1; // Estimated Final Size Threshold
const NRT: f64 = 0.35; // Next Radix Threshold

fn get_best_radix_size_and_runs(size: usize) -> (usize, usize) {
    let mut results = Vec::new();

    for r in 7..10 {
        let diversion_threshold = (2usize.pow(r as u32) as f64) as usize;
        let mut required_bytes =
            (((size as f64) / (diversion_threshold as f64)).log2() / r as f64)
                .ceil() as usize;
        let mut estimated_final_size = (size as f64)
            / 2usize.pow(r as u32).pow(required_bytes as u32) as f64;
        if estimated_final_size > 1.0 {
            required_bytes += 1;
            estimated_final_size = (size as f64)
                / 2usize.pow(r as u32).pow(required_bytes as u32) as f64;
        }
        results.push((required_bytes, estimated_final_size, r));
    }

    results.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if results[0].0 == results[2].0 && results[2].1 <= EFST {
        (results[2].2, results[2].0)
    } else if (results[0].0 == results[1].0 && results[1].1 <= EFST)
        || results[0].1 > NRT
    {
        (results[1].2, results[1].0)
    } else {
        (results[0].2, results[0].0)
    }
}

pub fn dlsd_radixsort_body<T>(
    arr: &mut [T],
    p: Params,
    rbd: usize, // runs before diversion
    diversion: bool,
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

    let mut index = 0;

    let mut buffer: Vec<T> = vec![arr[0]; size];

    let histograms = if diversion {
        get_partial_histograms_fast(arr, &p, rbd)
    } else {
        get_full_histograms_fast(arr, &p)
    };

    let mut t1 = arr;
    let t2 = &mut buffer;
    let mut t2 = t2.as_mut_slice();

    /* Swap elements the right amount of time to reach diversion threshold */
    for level in (p.level..p.max_level).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = if diversion {
            source.get_mask_and_shift_for_partial(&p.new_level(level))
        } else {
            source.get_mask_and_shift(&p.new_level(level))
        };
        let (_, mut heads, _) = prefix_sums(&histograms[level]);

        copy_by_histogram(
            size,
            &mut source,
            &mut destination,
            &mut heads,
            mask,
            shift,
        );

        index = 1 - index;

        if index == 1 {
            t1 = source;
            t2 = destination;
        } else {
            t2 = source;
            t1 = destination;
        }
    }

    /* Ensure data is at the right place */
    if index == 1 {
        copy_nonoverlapping(t2, t1, size);
    }

    if diversion && t1.element_bit_size() - p.offset >= p.radix * p.max_level {
        let unsorted_parts = insertion_sort_try(&mut t1, &p);

        let new_level = 0;
        let std_radix = 8;
        let new_raw_offset = p.offset + p.max_level * p.radix;
        let new_max_level = t1.compute_max_level(new_raw_offset, std_radix);
        let new_offset = t1.element_bit_size() - new_max_level * std_radix;
        let new_params =
            Params::new(new_level, std_radix, new_offset, new_max_level);

        unsorted_parts.iter().for_each(|(start, end)| {
            voracious_sort_rec(&mut t1[*start..*end], new_params, 2);
        });
    }
}

pub fn dlsd_radixsort_aux<T>(arr: &mut [T], radix: usize)
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

    let (sugg_radix, required_bytes) = get_best_radix_size_and_runs(arr.len());
    let (_, sugg_raw_offset) = arr.compute_offset(sugg_radix);

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);

    let (params, diversion, rbd) = if required_bytes < max_level {
        (
            Params::new(0, sugg_radix, sugg_raw_offset, required_bytes),
            true,
            required_bytes,
        )
    } else {
        (Params::new(0, radix, offset, max_level), false, max_level)
    };

    dlsd_radixsort_body(arr, params, rbd, diversion);
}

pub fn dlsd_radixsort<T>(arr: &mut [T], radix: usize)
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
        dlsd_radixsort_aux(arr, radix)
    });
    k_way_merge(arr, &mut separators);
}