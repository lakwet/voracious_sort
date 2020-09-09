use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::super::{RadixKey, Radixable};
use super::utils::offset_from_bits;

pub fn compute_offset_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
) -> (usize, usize) {
    let dummy = arr[0];
    let max = arr.par_iter().map(|item| item.into_key_type()).max().unwrap();

    offset_from_bits(
        arr,
        max,
        radix,
        dummy.type_size(),
        dummy.default_key(),
        dummy.one(),
    )
}

pub fn aggregate_histograms(histograms: &[Vec<usize>]) -> Vec<usize> {
    let mut global_histogram = vec![0; histograms[0].len()];

    histograms.iter().for_each(|histogram| {
        histogram.iter().enumerate().for_each(|(i, v)| {
            global_histogram[i] += v;
        });
    });

    global_histogram
}
