use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, Radixable};
use super::lsd_sort::lsd_radixsort_body;
use super::msd_sort::copy_by_histogram;
use super::utils::{
    copy_nonoverlapping, get_empty_histograms, only_one_bucket_filled,
    prefix_sums, Params,
};

pub fn get_full_histograms_except_last_chunk<T, K>(
    arr: &mut [T],
    p: &Params,
) -> Vec<Vec<usize>>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut histograms = get_empty_histograms(p.max_level - 1, p.radix_range);
    let default_mask = dummy.default_mask(p.radix);
    let shift = dummy.usize_to_keytype(p.radix);

    let remainder = arr.len() % 4;
    let (arr_fst, arr_remainder) = arr.split_at(arr.len() - remainder);

    if p.max_level == 2 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 3 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 4 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 5 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 6 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 7 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 8 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    }

    histograms
}

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
    let histograms = get_full_histograms_except_last_chunk(arr, &p);

    let estimated_size = ((size >> p.radix) + size % p.radix + 1) as usize;
    let mut buffer: Vec<T> = vec![arr[0]; estimated_size * p.radix_range];

    let t1 = arr;
    let t2 = &mut buffer;
    let t2 = t2.as_mut_slice();

    let source = t1;
    let destination = t2;

    // first pass with estimated buckets size
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

    // second LSD radix sort pass and to handle overflow elements
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

    // then use regular LSD radix sort
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

    if max_level == 0 {
        return;
    }

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
///
/// Since this sort is not used in this crate, it is not fully optimized and
/// implemented. It won't work for u128 and i128.
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

#[test]
fn test_utils_get_full_histograms_except_last_chunk() {
    let p = Params::new(0, 4, 16, 4); // level, radix, offset, max_level
    let mut arr: Vec<u32> = vec![8, 5, 1024, 512, 256, 16_384, 64, 32];
    let histograms = get_full_histograms_except_last_chunk(&mut arr, &p);

    assert_eq!(histograms.len(), 3);
    assert_eq!(histograms[0], vec![
        7, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[1], vec![
        5, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[2], vec![
        6, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
}
