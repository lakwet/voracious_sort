use rayon::prelude::*;

use std::time::Instant;

use super::super::{RadixKey, Radixable};

/// Sample standard deviation of the measured times (Bessel's correction:
/// divided by `n - 1`, since the mean is estimated from the same runs).
#[allow(dead_code)]
pub fn std_deviation(data: &[u64], mean: u64) -> f32 {
    if data.len() < 2 {
        return 0.0;
    }
    let variance = data
        .iter()
        .map(|&value| {
            // Square in f64: a difference above ~4.3 s (in ns) would
            // overflow u64 when squared.
            let diff = value as f64 - mean as f64;
            diff * diff
        })
        .sum::<f64>()
        / (data.len() - 1) as f64;

    variance.sqrt() as f32
}

#[allow(dead_code)]
pub fn helper_sort_aux<T, K>(
    sort: &dyn Fn(&mut [T]) -> (),
    runs: usize,
    size: usize,
    generator: &dyn Fn(usize) -> Vec<T>,
    with_check: bool,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    let mut nanos: Vec<u64> = Vec::with_capacity(runs);

    for _ in 0..runs {
        if with_check {
            let mut array = generator(size);
            let mut check = array.to_vec();

            let start = Instant::now();
            sort(&mut array);
            let ns: u64 = start.elapsed().as_nanos() as u64;
            nanos.push(ns);

            check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(check, array);
        } else {
            let mut array = generator(size);

            let start = Instant::now();
            sort(&mut array);
            let ns: u64 = start.elapsed().as_nanos() as u64;
            nanos.push(ns);
        }
    }

    // if nanos.len() > 1 {
    //     nanos.remove(0);
    // }

    // Divide by the number of measures actually kept: dividing by runs - 1
    // only made sense with the first run removed above (commented out), and
    // inflated every mean by runs / (runs - 1), i.e. +11% with 10 runs.
    let sum: u64 = nanos.iter().sum();
    let mean: u64 = sum / nanos.len().max(1) as u64;
    // The deviation is over the runs, not over the array size.
    let std_dev: f32 = std_deviation(&nanos, mean);
    let per_item: f32 = (mean as f64 / size as f64) as f32;

    // \u{1b} => escape for terminal
    // 0 => no color
    // 0;30 => gray
    // 0;31 => red
    // 1;31 => red
    // 0;32 => green
    // 0;33 => brown
    // 0;34 => blue
    // 1;34 => light blue
    // 0;37 => light gray

    // print time, standard deviation and time per item
    print!(
        "\u{1b}[0;32m{}us\u{1b}[0m\t\u{1b}[1;31m{:.0}ns\u{1b}[0m\t(\u{1b}[0;\
         33m{:.2}ns\u{1b}[0m)\t",
        mean / 1000,
        std_dev,
        per_item
    );
}
