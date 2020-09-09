use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::sorts::utils::{
    copy_nonoverlapping, get_empty_histograms, only_one_bucket_filled,
    prefix_sums, Params,
};
use super::super::Radixable;

const UNROLL_SIZE: usize = 4;

pub fn copy_by_histogram(
    size: usize,
    source: &mut [u32],
    destination: &mut [u32],
    heads: &mut Vec<usize>,
    mask: u32,
    shift: usize,
) {
    let source = &mut source[0..size];
    let remainder = size % UNROLL_SIZE;
    let (source_fst, source_snd) = source.split_at_mut(size - remainder);

    source_fst.chunks_exact(UNROLL_SIZE).for_each(|chunk| unsafe {
        let b0 = chunk.get_unchecked(0).extract(mask, shift);
        let b1 = chunk.get_unchecked(1).extract(mask, shift);
        let b2 = chunk.get_unchecked(2).extract(mask, shift);
        let b3 = chunk.get_unchecked(3).extract(mask, shift);

        let d0 = *heads.get_unchecked(b0);
        heads[b0] += 1;
        let d1 = *heads.get_unchecked(b1);
        heads[b1] += 1;
        let d2 = *heads.get_unchecked(b2);
        heads[b2] += 1;
        let d3 = *heads.get_unchecked(b3);
        heads[b3] += 1;

        destination[d0] = *chunk.get_unchecked(0);
        destination[d1] = *chunk.get_unchecked(1);
        destination[d2] = *chunk.get_unchecked(2);
        destination[d3] = *chunk.get_unchecked(3);
    });

    for item in source_snd.iter() {
        let target_bucket = item.extract(mask, shift);
        destination[heads[target_bucket]] = *item;
        heads[target_bucket] += 1;
    }
}

fn get_full_histograms(arr: &mut [u32], p: &Params) -> Vec<Vec<usize>> {
    let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
    let default_mask = arr[0].default_mask(p.radix);
    let shift = p.radix as u32;

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    let offset = quotient * 4;

    if p.max_level == 1 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let v0 = arr.get_unchecked(i);
                let v1 = arr.get_unchecked(i + 1);
                let v2 = arr.get_unchecked(i + 2);
                let v3 = arr.get_unchecked(i + 3);
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let v = arr.get_unchecked(offset + i);
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }
    } else if p.max_level == 2 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = *arr.get_unchecked(i);
                let mut v1 = *arr.get_unchecked(i + 1);
                let mut v2 = *arr.get_unchecked(i + 2);
                let mut v3 = *arr.get_unchecked(i + 3);
                histograms[1][(v0 & default_mask) as usize] += 1;
                histograms[1][(v1 & default_mask) as usize] += 1;
                histograms[1][(v2 & default_mask) as usize] += 1;
                histograms[1][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = *arr.get_unchecked(offset + i);
                histograms[1][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }
    } else if p.max_level == 3 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = *arr.get_unchecked(i);
                let mut v1 = *arr.get_unchecked(i + 1);
                let mut v2 = *arr.get_unchecked(i + 2);
                let mut v3 = *arr.get_unchecked(i + 3);
                histograms[2][(v0 & default_mask) as usize] += 1;
                histograms[2][(v1 & default_mask) as usize] += 1;
                histograms[2][(v2 & default_mask) as usize] += 1;
                histograms[2][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[1][(v0 & default_mask) as usize] += 1;
                histograms[1][(v1 & default_mask) as usize] += 1;
                histograms[1][(v2 & default_mask) as usize] += 1;
                histograms[1][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = *arr.get_unchecked(offset + i);
                histograms[2][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[1][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }
    } else if p.max_level == 4 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = *arr.get_unchecked(i);
                let mut v1 = *arr.get_unchecked(i + 1);
                let mut v2 = *arr.get_unchecked(i + 2);
                let mut v3 = *arr.get_unchecked(i + 3);
                histograms[3][(v0 & default_mask) as usize] += 1;
                histograms[3][(v1 & default_mask) as usize] += 1;
                histograms[3][(v2 & default_mask) as usize] += 1;
                histograms[3][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[2][(v0 & default_mask) as usize] += 1;
                histograms[2][(v1 & default_mask) as usize] += 1;
                histograms[2][(v2 & default_mask) as usize] += 1;
                histograms[2][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[1][(v0 & default_mask) as usize] += 1;
                histograms[1][(v1 & default_mask) as usize] += 1;
                histograms[1][(v2 & default_mask) as usize] += 1;
                histograms[1][(v3 & default_mask) as usize] += 1;
                v0 >>= shift;
                v1 >>= shift;
                v2 >>= shift;
                v3 >>= shift;
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = *arr.get_unchecked(offset + i);
                histograms[3][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[2][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[1][(v & default_mask) as usize] += 1;
                v >>= shift;
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }
    }

    histograms
}

pub fn lsd_radixsort_body(arr: &mut [u32], p: Params) {
    let size = arr.len();

    if size <= 128 {
        arr.sort_unstable();
        return;
    }

    let dummy = arr[0];
    let mut buffer: Vec<u32> = vec![0; size];
    let mut index = 0;

    let histograms = get_full_histograms(arr, &p);

    let mut t1 = arr;
    let t2 = &mut buffer;
    let mut t2 = t2.as_mut_slice();
    for level in (p.level..p.max_level).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));
        let (_, mut heads, _) = prefix_sums(&histograms[level]);

        copy_by_histogram(
            source.len(),
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

pub fn lsd_u32(arr: &mut [u32]) {
    if arr.len() <= 128 {
        arr.sort_unstable();
        return;
    }

    let radix = 8;
    let dummy = arr[0];
    let (offset, _) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(offset, radix);

    if max_level == 0 {
        return;
    }

    let params = Params::new(0, radix, offset, max_level);

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, _| {
        lsd_radixsort_body(arr, params);
    });
    k_way_merge(arr, &mut separators);
}
