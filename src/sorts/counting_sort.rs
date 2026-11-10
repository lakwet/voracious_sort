use super::super::{RadixKey, Radixable};
use super::utils::Params;

fn counting_sort_aux<T, K>(arr: &mut [T], p: Params)
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mask = dummy.default_mask(p.radix);
    // Counts: u32 in the hot loop (as fast as before), added to usize totals
    // every 2^31 keys, so that a value present more than 4 billion times
    // cannot overflow (the counters were inferred as i32 before: wrong
    // result beyond 2^31 equal keys).
    let mut histogram = vec![0usize; p.radix_range];
    let mut block_counts = vec![0u32; p.radix_range];
    for block in arr.chunks(1 << 31) {
        let quotient = block.len() / 4;
        let remainder = block.len() % 4;
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let bucket0 = block.get_unchecked(i).extract(mask, 0);
                let bucket1 = block.get_unchecked(i + 1).extract(mask, 0);
                let bucket2 = block.get_unchecked(i + 2).extract(mask, 0);
                let bucket3 = block.get_unchecked(i + 3).extract(mask, 0);
                block_counts[bucket0] += 1;
                block_counts[bucket1] += 1;
                block_counts[bucket2] += 1;
                block_counts[bucket3] += 1;
            }
        }
        let offset = quotient * 4;
        for i in 0..remainder {
            unsafe {
                let bucket = block.get_unchecked(offset + i).extract(mask, 0);
                block_counts[bucket] += 1;
            }
        }
        for (total, c) in histogram.iter_mut().zip(block_counts.iter_mut()) {
            *total += *c as usize;
            *c = 0;
        }
    }

    let dummy = arr[0];

    let mut position = 0;
    histogram.iter().enumerate().for_each(|(value, count)| {
        if *count > 0 {
            let quotient = *count / 4;
            let remainder = count % 4;
            let v = dummy.to_generic(value);
            for _ in 0..quotient {
                unsafe {
                    *arr.get_unchecked_mut(position) = v;
                    *arr.get_unchecked_mut(position + 1) = v;
                    *arr.get_unchecked_mut(position + 2) = v;
                    *arr.get_unchecked_mut(position + 3) = v;
                }
                position += 4;
            }
            for _ in 0..remainder {
                unsafe {
                    *arr.get_unchecked_mut(position) = v;
                    position += 1;
                }
            }
        }
    });
}

/// # Counting sort
///
/// An implementation of the
/// [Counting sort](https://en.wikipedia.org/wiki/Counting_sort)
/// algorithm.
///
/// Counting sort is very fast for inputs with a small bit representation.
///
/// This Counting sort has been a bit customized since it takes a radix input.
pub fn counting_sort<T, K>(arr: &mut [T], radix: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    if arr.len() < 2 {
        return;
    }

    let offset = 0;
    let level = 0;
    let max_level = 1;
    let params = Params::new(level, radix, offset, max_level);

    counting_sort_aux(arr, params);
}
