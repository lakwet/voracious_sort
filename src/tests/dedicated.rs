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
    for size in [0, 1, 2, 100, 128, 129, 3_000, 3_001, 8_192, 10_000, 100_000, 1_048_575, 1_048_576, 1_500_000].iter() {
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

/// Low-cardinality detectors: integer ranges, grids, few values, with late
/// outliers, signed zeros and negative values, around the size limits.
#[test]
fn test_ded_float_sort_detectors() {
    fn check64(mut arr: Vec<f64>, what: &str) {
        let mut expected: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        expected.sort_unstable();
        float_sort(&mut arr);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{} {} not sorted", what, arr.len());
        let mut got: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        got.sort_unstable();
        assert!(got == expected, "{} {} multiset", what, arr.len());
        // f32 version of the same values
        let mut a32: Vec<f32> = arr.iter().map(|&x| x as f32).collect();
        let mut e32: Vec<u32> = a32.iter().map(|x| x.to_bits()).collect();
        e32.sort_unstable();
        float_sort(&mut a32);
        assert!(a32.windows(2).all(|w| w[0] <= w[1]), "{} {} f32 not sorted", what, a32.len());
        let mut g32: Vec<u32> = a32.iter().map(|x| x.to_bits()).collect();
        g32.sort_unstable();
        assert!(g32 == e32, "{} {} f32 multiset", what, a32.len());
    }
    let mut seed = 0x1234_5678_9abc_def0u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for &n in [129usize, 200, 1_000, 4_095, 4_096, 5_000, 70_000].iter() {
        let ints: Vec<f64> = (0..n).map(|_| (next() % 256) as f64 - 128.0).collect();
        check64(ints.clone(), "ints");
        for &pos in [0, n / 2, n - 1, n * 95 / 100, 63, 64, 65.min(n - 1)].iter() {
            let mut v = ints.clone();
            v[pos] = 1e6;
            check64(v, "ints wide outlier");
            let mut v = ints.clone();
            v[pos] = 0.5;
            check64(v, "ints fractional outlier");
            let mut v = ints.clone();
            v[pos] = -0.0;
            check64(v, "ints signed zero");
            let mut v = ints.clone();
            v[pos] = f64::INFINITY;
            check64(v, "ints infinity");
        }
        let grid: Vec<f64> = (0..n).map(|_| ((next() % 256) as f64 - 128.0) * 0.125 + 3.0).collect();
        check64(grid.clone(), "grid");
        let mut v = grid.clone();
        v[n * 95 / 100] = 3.0625;
        check64(v, "grid off-grid");
        let reps = [-7.25f64, -0.0, 0.0, 1e-300, 3.5, 1e300, -1e300, 42.0, 0.1, 0.2, 0.3, 0.4, 5e-324, -5e-324, 99.9, -99.9];
        for &k in [1usize, 2, 4, 5, 16].iter() {
            let few: Vec<f64> = (0..n).map(|_| reps[(next() % k as u64) as usize]).collect();
            check64(few.clone(), "few");
            let mut v = few.clone();
            v[n * 95 / 100] = 1234.5;
            check64(v, "few late");
        }
    }
}

/// Every digit layout of the fused LSD (bytes, 11-bit, automatic choice).
#[test]
fn test_ded_float_lsd_layouts() {
    use super::super::dedicated::float_lsd::Digits;
    use super::super::dedicated::float_sort::{float_sort_f32_radix_mode, float_sort_f64_radix_mode};
    for mode in [Digits::Auto, Digits::Bytes, Digits::Wide] {
        check_total_order_f32(&|a| float_sort_f32_radix_mode(a, mode));
        check_total_order_f64(&|a| float_sort_f64_radix_mode(a, mode));
    }
}

/// The per-step profiling helper of the fused LSD must still sort.
#[test]
fn test_ded_float_lsd_profile() {
    use super::super::dedicated::float_lsd::w32;
    for &wide in [false, true].iter() {
        let mut v = helper_random_array_uniform_f32(10_000);
        let mut expected = v.clone();
        expected.sort_unstable_by(|a, b| a.total_cmp(b));
        let bits: &mut [u32] =
            unsafe { std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut u32, v.len()) };
        let steps = w32::profile(bits, wide);
        assert!(steps.iter().any(|(name, _)| name == "histogram"));
        let a: Vec<u32> = v.iter().map(|x| x.to_bits()).collect();
        let e: Vec<u32> = expected.iter().map(|x| x.to_bits()).collect();
        assert!(a == e, "profile wide={}", wide);
    }
}

/// Integer-valued floats make bucket starts alias in the cache: the staged
/// scatter is used for those passes. Check the result on such data.
#[test]
fn test_ded_float_sort_aliased_buckets() {
    for &n in [100_003usize, 1_000_003].iter() {
        let teeth: Vec<f32> = (0..n).map(|i| (i % (n / 4)) as f32).collect();
        let saw = helper_random_array_asc_sawtooth_f32(n);
        let ints: Vec<f64> = (0..n).map(|i| ((i * 7919) % (n / 3)) as f64).collect();
        for v in [teeth, saw].iter() {
            let mut a = v.clone();
            let mut e = v.clone();
            e.sort_unstable_by(|x, y| x.total_cmp(y));
            float_sort(&mut a);
            assert!(a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())), "f32 {}", n);
        }
        let mut a = ints.clone();
        let mut e = ints.clone();
        e.sort_unstable_by(|x, y| x.total_cmp(y));
        float_sort(&mut a);
        assert!(a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())), "f64 {}", n);
    }
}
