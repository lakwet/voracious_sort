use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, Radixable};
use super::comparative_sort::insertion_sort_try;
use super::lsd_sort::lsd_radixsort_body;
use super::msd_sort::copy_by_histogram;
use super::utils::{
    copy_nonoverlapping, get_partial_histograms, offset_from_bits,
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

pub fn dlsd_radixsort_body<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: Params,
    rbd: usize, // runs before diversion
    diversion: bool,
) {
    let size = arr.len();

    if size <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let mut index = 0;

    let mut buffer: Vec<T> = vec![arr[0]; size];

    let histograms = if diversion {
        get_partial_histograms(arr, &p, rbd)
    } else {
        dummy.get_full_histograms(arr, &p)
    };

    let mut t1 = arr;
    let t2 = &mut buffer;
    let mut t2 = t2.as_mut_slice();

    // Swap elements the right amount of time to reach diversion threshold
    for level in (p.level..p.max_level).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = if diversion {
            dummy.get_mask_and_shift_from_left(&p.new_level(level))
        } else {
            dummy.get_mask_and_shift(&p.new_level(level))
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

    // Ensure data is at the right place
    if index == 1 {
        copy_nonoverlapping(t2, t1, size);
    }

    if diversion && dummy.type_size() - p.offset >= p.radix * p.max_level {
        let unsorted_parts = insertion_sort_try(&mut t1, &p);

        let radix = 8;
        let raw_offset = p.max_level * p.radix + p.offset;
        let new_max_level = dummy.compute_max_level(raw_offset, radix);

        let new_params_msd = Params::new(0, radix, raw_offset, new_max_level);

        let offset_lsd = dummy.type_size() - new_max_level * radix;
        let new_params_lsd = Params::new(0, radix, offset_lsd, new_max_level);

        unsorted_parts.iter().for_each(|(i, j)| {
            if j - i <= 250 {
                t1[*i..*j].sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            } else if j - i > 3000 && new_max_level <= 4 {
                lsd_radixsort_body(&mut t1[*i..*j], new_params_lsd);
            } else {
                voracious_sort_rec(&mut t1[*i..*j], new_params_msd, 0);
            }
        });
    }
}

fn dlsd_radixsort_aux<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];

    let (sugg_radix, required_bytes) = get_best_radix_size_and_runs(arr.len());

    let max_key = dummy.get_max_key(arr);
    let bits = dummy.type_size();
    let zero = dummy.default_key();
    let one = dummy.one();

    let (_, sugg_raw_offset) =
        offset_from_bits(arr, max_key, sugg_radix, bits, zero, one);
    let (offset, _) = offset_from_bits(arr, max_key, radix, bits, zero, one);
    let max_level = dummy.compute_max_level(offset, radix);

    if max_level == 0 {
        return;
    }

    let sugg_max_level = dummy.compute_max_level(sugg_raw_offset, sugg_radix);

    let (params, diversion, rbd) = if required_bytes < sugg_max_level {
        (
            Params::new(0, sugg_radix, sugg_raw_offset, required_bytes),
            true,
            required_bytes,
        )
    } else if sugg_radix > radix {
        (
            Params::new(0, sugg_radix, sugg_raw_offset, sugg_max_level),
            false,
            sugg_max_level,
        )
    } else {
        (Params::new(0, radix, offset, max_level), false, max_level)
    };

    dlsd_radixsort_body(arr, params, rbd, diversion);
}

/// # DLSD sort: Diverting LSD sort
///
/// A simpler version of the
/// [DFR sort](https://github.com/ramou/dfr)
/// algorithm.
///
/// Several changes have been made. Diversion is different, and only one out of
/// the three ideas from the DFR sort is implemented. So it is less dependent on
/// the uniformly distributed input hypothesis. Moreover a variable radix is
/// added.
///
/// The core idea of this algorithm is, actually, an heuristic. An estimation
/// of the number of required passes is computed, and then diversion occurs.
/// Which is unusual for a LSD sort algorithm.
///
/// The Verge sort pre-processing heuristic is also added.
///
/// The DLSD sort is an out of place unstable radix sort. The core algorithm
/// is stable but fallback and diversion are unstable.
pub fn dlsd_radixsort<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
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
