//! f32 / f64: `ravenous_sort` and `RadixSort::voracious_sort`, bit for bit
//! the `total_cmp` order (NaNs of both signs, -0.0 / +0.0, infinities).

use voracious_radix_sort::{ravenous_sort, RadixSort};

use crate::generators::float_32::*;
use crate::generators::float_64::*;
use crate::{quick_max, BENCH_SIZES};

/// The sort must give exactly (bit for bit) the `total_cmp` order.
fn check_total_order_f32(sort: &dyn Fn(&mut [f32])) {
    for size in [
        0, 1, 2, 33, 50, 64, 65, 100, 128, 129, 350, 351, 600, 601, 2_047, 2_048, 2_049, 4_095,
        4_096, 4_097, 5_000, 8_191, 8_192, 1_048_575, 1_048_576,
    ]
    .iter()
    .chain(BENCH_SIZES.iter())
    .filter(|&&n| n <= quick_max())
    {
        for (gen, name) in generators_f32().iter() {
            let mut arr = gen(*size);
            let mut expected = arr.clone();
            rayon::slice::ParallelSliceMut::par_sort_unstable_by(&mut expected[..], |a, b| {
                a.total_cmp(b)
            });
            sort(&mut arr);
            let a: Vec<u32> = arr.iter().map(|x| x.to_bits()).collect();
            let e: Vec<u32> = expected.iter().map(|x| x.to_bits()).collect();
            assert!(a == e, "f32 {} size {}", name, size);
        }
    }
}

fn check_total_order_f64(sort: &dyn Fn(&mut [f64])) {
    for size in [
        0, 1, 2, 33, 50, 64, 65, 100, 128, 129, 700, 701, 800, 801, 2_047, 2_048, 2_049, 3_000,
        3_001, 4_095, 4_096, 4_097, 8_192, 262_143, 262_144, 1_500_000,
    ]
    .iter()
    .chain(BENCH_SIZES.iter())
    .filter(|&&n| n <= quick_max())
    {
        for (gen, name) in generators_f64().iter() {
            let mut arr = gen(*size);
            let mut expected = arr.clone();
            rayon::slice::ParallelSliceMut::par_sort_unstable_by(&mut expected[..], |a, b| {
                a.total_cmp(b)
            });
            sort(&mut arr);
            let a: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
            let e: Vec<u64> = expected.iter().map(|x| x.to_bits()).collect();
            assert!(a == e, "f64 {} size {}", name, size);
        }
    }
}

#[test]
fn floats() {
    check_total_order_f32(&|a| ravenous_sort(a));
    check_total_order_f64(&|a| ravenous_sort(a));
    check_total_order_f32(&|a| a.voracious_sort());
    check_total_order_f64(&|a| a.voracious_sort());
}
