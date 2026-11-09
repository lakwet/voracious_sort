use super::super::dedicated::cs_u16::cs_u16;
use super::super::dedicated::lsd_f32::lsd_f32;
use super::super::dedicated::lsd_u32::lsd_u32;

use super::super::generators::float_32::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;

use super::sorts::helper_sort;

#[test]
fn test_ded_sort_lsd_f32() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| lsd_f32(a), generators_f32(), *size);
    }
}

#[test]
fn test_ded_sort_lsd_u32() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| lsd_u32(a), generators_u32(), *size);
    }
}

#[test]
fn test_ded_sort_counting_sort_u16() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| cs_u16(a), generators_u16(), *size);
    }
}

use super::super::dedicated::float_sort::float_sort;
use super::super::generators::float_64::*;

/// float_sort must give exactly (bit for bit) the `total_cmp` order.
fn check_total_order_f32(sort: &dyn Fn(&mut [f32])) {
    for size in [0, 1, 2, 100, 128, 129, 600, 601, 5_000, 8_191, 8_192, 100_000].iter() {
        for (gen, name) in generators_f32().iter() {
            let mut arr = gen(*size);
            let mut expected = arr.clone();
            expected.sort_unstable_by(|a, b| a.total_cmp(b));
            sort(&mut arr);
            let a: Vec<u32> = arr.iter().map(|x| x.to_bits()).collect();
            let e: Vec<u32> = expected.iter().map(|x| x.to_bits()).collect();
            assert!(a == e, "f32 {} size {}", name, size);
        }
    }
}

fn check_total_order_f64(sort: &dyn Fn(&mut [f64])) {
    for size in [0, 1, 2, 100, 128, 129, 3_000, 3_001, 8_192, 10_000, 100_000].iter() {
        for (gen, name) in generators_f64().iter() {
            let mut arr = gen(*size);
            let mut expected = arr.clone();
            expected.sort_unstable_by(|a, b| a.total_cmp(b));
            sort(&mut arr);
            let a: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
            let e: Vec<u64> = expected.iter().map(|x| x.to_bits()).collect();
            assert!(a == e, "f64 {} size {}", name, size);
        }
    }
}

#[test]
fn test_ded_float_sort_f32() {
    check_total_order_f32(&|a| float_sort(a));
}

#[test]
fn test_ded_float_sort_f64() {
    check_total_order_f64(&|a| float_sort(a));
}

/// Special values: signed zeros, infinities, subnormals (no NaN: not
/// supported yet). `-0.0 == +0.0` for `<`, so the check is "sorted for
/// `partial_cmp` + same multiset of bits", not a bitwise `total_cmp` order.
#[test]
fn test_ded_float_sort_special_values() {
    let base_f64 = [
        0.0f64, -0.0, f64::INFINITY, f64::NEG_INFINITY, f64::MIN_POSITIVE, -f64::MIN_POSITIVE,
        f64::from_bits(1), f64::from_bits(1) * -1.0, f64::MAX, f64::MIN, 1.0, -1.0,
    ];
    let no_nan64 = |x: f64| if x.is_nan() { 0.5 } else { x };
    let no_nan32 = |x: f32| if x.is_nan() { 0.5 } else { x };
    for &size in [14usize, 3_000, 50_000].iter() {
        // Many copies of the special values + a deterministic pseudo random fill.
        let mut arr: Vec<f64> = (0..size)
            .map(|i| if i % 3 == 0 { base_f64[i % base_f64.len()] } else { no_nan64(f64::from_bits((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))) })
            .collect();
        let mut expected: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        expected.sort_unstable();
        float_sort(&mut arr);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "special f64 size {} not sorted", size);
        let mut a: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        a.sort_unstable();
        assert!(a == expected, "special f64 size {} multiset", size);

        let mut arr32: Vec<f32> = (0..size)
            .map(|i| if i % 3 == 0 { base_f64[i % base_f64.len()] as f32 } else { no_nan32(f32::from_bits((i as u32).wrapping_mul(0x9E37_79B9))) })
            .collect();
        let mut expected32: Vec<u32> = arr32.iter().map(|x| x.to_bits()).collect();
        expected32.sort_unstable();
        float_sort(&mut arr32);
        assert!(arr32.windows(2).all(|w| w[0] <= w[1]), "special f32 size {} not sorted", size);
        let mut a: Vec<u32> = arr32.iter().map(|x| x.to_bits()).collect();
        a.sort_unstable();
        assert!(a == expected32, "special f32 size {} multiset", size);
    }
}
