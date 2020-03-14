use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, Radixable};
use super::lsd_sort::lsd_radixsort_body;
use super::msd_sort::copy_by_histogram;
use super::utils::{
    copy_nonoverlapping, get_full_histogram_except_for_last_level,
    only_one_bucket_filled, prefix_sums, Params,
};

pub fn thiel_radixsort_body<T, K>(arr: &mut [T], p: Params)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let size = arr.len();
    let dummy = arr[0];
    let histograms = get_full_histogram_except_for_last_level(arr, &p);

    let estimated_size = ((size >> p.radix) + size % p.radix + 1) as usize;
    let mut buffer: Vec<T> = vec![arr[0]; estimated_size * p.radix_range];

    let t1 = arr;
    let t2 = &mut buffer;
    let t2 = t2.as_mut_slice();

    let source = t1;
    let destination = t2;

    /*** first pass with estimated buckets size ***/
    let mut first_histogram = vec![0; p.radix_range];
    let mut overflow_histogram = vec![0; p.radix_range];
    let mut estimated_heads = Vec::with_capacity(p.radix_range);
    for i in 0..p.radix_range {
        estimated_heads.push(i * estimated_size);
    }
    let mut estimated_tails = estimated_heads.to_vec();
    estimated_tails.push(p.radix_range * estimated_size);
    estimated_tails.remove(0);

    let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(p.max_level - 1));
    let mut overflow_index = 0;

    for i in 0..size {
        let target_bucket = source[i].extract(mask, shift);
        first_histogram[target_bucket] += 1;
        // if the bucket buffer is not full, put the element in the buffer
        if estimated_heads[target_bucket] < estimated_size {
            destination[estimated_heads[target_bucket]] = source[i];
            estimated_heads[target_bucket] += 1;
        } else {
            // otherwise use the source input array
            source.swap(i, overflow_index);
            overflow_histogram[target_bucket] += 1;
            overflow_index += 1;
        }
    }

    /*** second LSD radix sort pass and to handle overflow elements ***/
    // copy overflow element in another bucket...
    let mut overflow_buckets = Vec::<T>::new();
    let (_, mut overflow_heads, overflow_tails) =
        prefix_sums(&overflow_histogram);
    let overflow_heads_copy = overflow_heads.to_vec();
    if overflow_index > 0 {
        overflow_buckets = vec![destination[0]; overflow_index];
        for item in source.iter().take(overflow_index) {
            let target_bucket = item.extract(mask, shift);
            overflow_buckets[overflow_heads[target_bucket]] = *item;
            overflow_heads[target_bucket] += 1;
        }
    }

    // perform second pass while taking into account overflow_buckets
    let level = p.max_level - 2;
    let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));
    let (_, mut heads, _) = prefix_sums(&histograms[level]);

    for prev_bucket in 0..p.radix_range {
        for item in overflow_buckets
            .iter()
            .take(overflow_tails[prev_bucket])
            .skip(overflow_heads_copy[prev_bucket])
        {
            let target_bucket = item.extract(mask, shift);
            source[heads[target_bucket]] = *item;
            heads[target_bucket] += 1;
        }
        for item in destination
            .iter()
            .take(estimated_heads[prev_bucket])
            .skip(prev_bucket * estimated_size)
        {
            let target_bucket = item.extract(mask, shift);
            source[heads[target_bucket]] = *item;
            heads[target_bucket] += 1;
        }
    }

    let mut t1 = source;
    let mut t2 = destination;

    /*** then use regular LSD radix sort ***/
    let mut index = 0;
    for level in (0..(p.max_level - 2)).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));
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

    if index == 1 {
        copy_nonoverlapping(t2, t1, size);
    }
}

fn thiel_radixsort_aux<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let (offset, _) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    if params.max_level < 2 {
        lsd_radixsort_body(arr, params);
    } else {
        thiel_radixsort_body(arr, params);
    }
}

/// # Thiel sort
///
/// An implementation of the
/// [Fast radix sort](https://github.com/AwardOfSky/Fast-Radix-Sort)
/// algorithm. (Same author as the DFR sort)
///
/// We haven't managed to reproduce the Fast radix sort performance. Maybe
/// the generic implementation prevent us from reaching the same runtimes as
/// the original algorithm which has a dedicated implementation for integers.
///
/// The Verge sort pre-processing heuristic is also added.
///
/// The Thiel sort is an out of place unstable radix sort. The original sort
/// is stable, but because the fallback has been replaced by an unstable sort,
/// the entire algorithm is then unstable.
pub fn thiel_radixsort<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators =
        verge_sort_preprocessing(arr, radix, &thiel_radixsort_aux);
    k_way_merge(arr, &mut separators);
}
