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

use super::super::dedicated::ravenous_sort::ravenous_sort;

/// Array sizes of the benchmarks: every correctness test covers them too.
const BENCH_SIZES: [usize; 10] = [1_000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000, 3_000_000, 10_000_000];
use super::super::generators::float_64::*;

/// ravenous_sort must give exactly (bit for bit) the `total_cmp` order.
fn check_total_order_f32(sort: &dyn Fn(&mut [f32])) {
    for size in [0, 1, 2, 33, 50, 64, 65, 100, 128, 129, 350, 351, 600, 601, 5_000, 8_191, 8_192, 1_048_575, 1_048_576].iter().chain(BENCH_SIZES.iter()) {
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
    for size in [0, 1, 2, 33, 50, 64, 65, 100, 128, 129, 700, 701, 800, 801, 3_000, 3_001, 8_192, 262_143, 262_144, 1_500_000].iter().chain(BENCH_SIZES.iter()) {
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
fn test_ded_ravenous_sort_f32() {
    check_total_order_f32(&|a| ravenous_sort(a));
}

#[test]
fn test_ded_ravenous_sort_f64() {
    check_total_order_f64(&|a| ravenous_sort(a));
}

/// Special values: signed zeros, infinities, subnormals (no NaN: not
/// supported yet). `-0.0 == +0.0` for `<`, so the check is "sorted for
/// `partial_cmp` + same multiset of bits", not a bitwise `total_cmp` order.
#[test]
fn test_ded_ravenous_sort_special_values() {
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
        ravenous_sort(&mut arr);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "special f64 size {} not sorted", size);
        let mut a: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        a.sort_unstable();
        assert!(a == expected, "special f64 size {} multiset", size);

        let mut arr32: Vec<f32> = (0..size)
            .map(|i| if i % 3 == 0 { base_f64[i % base_f64.len()] as f32 } else { no_nan32(f32::from_bits((i as u32).wrapping_mul(0x9E37_79B9))) })
            .collect();
        let mut expected32: Vec<u32> = arr32.iter().map(|x| x.to_bits()).collect();
        expected32.sort_unstable();
        ravenous_sort(&mut arr32);
        assert!(arr32.windows(2).all(|w| w[0] <= w[1]), "special f32 size {} not sorted", size);
        let mut a: Vec<u32> = arr32.iter().map(|x| x.to_bits()).collect();
        a.sort_unstable();
        assert!(a == expected32, "special f32 size {} multiset", size);
    }
}

/// Low-cardinality detectors: integer ranges, grids, few values, with late
/// outliers, signed zeros and negative values, around the size limits.
#[test]
fn test_ded_ravenous_sort_detectors() {
    // Flush the detector lane counters often, to exercise the flush.
    crate::dedicated::float_detect::FLUSH_OVERRIDE.store(4096, std::sync::atomic::Ordering::Relaxed);
    fn check64(mut arr: Vec<f64>, what: &str) {
        let mut expected: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        expected.sort_unstable();
        ravenous_sort(&mut arr);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{} {} not sorted", what, arr.len());
        let mut got: Vec<u64> = arr.iter().map(|x| x.to_bits()).collect();
        got.sort_unstable();
        assert!(got == expected, "{} {} multiset", what, arr.len());
        // f32 version of the same values
        let mut a32: Vec<f32> = arr.iter().map(|&x| x as f32).collect();
        let mut e32: Vec<u32> = a32.iter().map(|x| x.to_bits()).collect();
        e32.sort_unstable();
        ravenous_sort(&mut a32);
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
    use super::super::dedicated::ravenous_sort::{ravenous_sort_f32_radix_mode, ravenous_sort_f64_radix_mode};
    for mode in [Digits::Auto, Digits::Bytes, Digits::Wide] {
        check_total_order_f32(&|a| ravenous_sort_f32_radix_mode(a, mode));
        check_total_order_f64(&|a| ravenous_sort_f64_radix_mode(a, mode));
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
fn test_ded_ravenous_sort_aliased_buckets() {
    for &n in [100_003usize, 1_000_003].iter() {
        let teeth: Vec<f32> = (0..n).map(|i| (i % (n / 4)) as f32).collect();
        let saw = helper_random_array_asc_sawtooth_f32(n);
        let ints: Vec<f64> = (0..n).map(|i| ((i * 7919) % (n / 3)) as f64).collect();
        for v in [teeth, saw].iter() {
            let mut a = v.clone();
            let mut e = v.clone();
            e.sort_unstable_by(|x, y| x.total_cmp(y));
            ravenous_sort(&mut a);
            assert!(a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())), "f32 {}", n);
        }
        let mut a = ints.clone();
        let mut e = ints.clone();
        e.sort_unstable_by(|x, y| x.total_cmp(y));
        ravenous_sort(&mut a);
        assert!(a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())), "f64 {}", n);
    }
}

/// Small arrays (sorting networks up to 32): every length up to 40, every
/// generator, plus special values (signed zeros, infinities, NaN bits).
#[test]
fn test_ded_ravenous_sort_tiny() {
    for size in 0..=40usize {
        for (gen, name) in generators_f64().iter() {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.sort_unstable_by(|a, b| a.total_cmp(b));
            ravenous_sort(&mut arr);
            assert!(arr.iter().map(|x| x.to_bits()).eq(expected.iter().map(|x| x.to_bits())), "f64 {} size {}", name, size);
        }
        for (gen, name) in generators_f32().iter() {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.sort_unstable_by(|a, b| a.total_cmp(b));
            ravenous_sort(&mut arr);
            assert!(arr.iter().map(|x| x.to_bits()).eq(expected.iter().map(|x| x.to_bits())), "f32 {} size {}", name, size);
        }
        let specials = [0.0f64, -0.0, 1.0, -1.0, f64::INFINITY, f64::NEG_INFINITY, f64::MAX, f64::MIN, 1e-310, -1e-310, 2.5];
        let mut arr: Vec<f64> = (0..size).map(|i| specials[(i * 7) % specials.len()]).collect();
        let mut expected = arr.clone();
        expected.sort_unstable_by(|a, b| a.total_cmp(b));
        ravenous_sort(&mut arr);
        assert!(arr.iter().map(|x| x.to_bits()).eq(expected.iter().map(|x| x.to_bits())), "f64 specials size {}", size);
        let mut arr32: Vec<f32> = arr.iter().map(|&x| x as f32).collect();
        let mut expected32 = arr32.clone();
        expected32.sort_unstable_by(|a, b| a.total_cmp(b));
        ravenous_sort(&mut arr32);
        assert!(arr32.iter().map(|x| x.to_bits()).eq(expected32.iter().map(|x| x.to_bits())), "f32 specials size {}", size);
    }
}

fn check_devour_sort<T: crate::DevourSort + std::fmt::Debug>(gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extremes: &[T], ty: &str) {
    let sizes: Vec<usize> = (0..=70)
        .chain([128, 129, 350, 351, 700, 701, 149, 150, 151, 199, 200, 201, 5_000, 1_048_575, 1_048_576, 1_500_000, 4_194_304, 12_582_911, 12_582_912].iter().copied())
        .chain(BENCH_SIZES.iter().copied())
        .collect();
    for &size in sizes.iter() {
        for (gen, name) in gens.iter() {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.sort_unstable();
            crate::devour_sort(&mut arr);
            assert!(arr == expected, "{} {} size {}", ty, name, size);
        }
        let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
        let mut expected = arr.clone();
        expected.sort_unstable();
        crate::devour_sort(&mut arr);
        assert!(arr == expected, "{} extremes size {}", ty, size);
    }
}

#[test]
fn test_ded_devour_sort() {
    use super::super::generators::signed_i32::generators_i32;
    use super::super::generators::signed_i64::generators_i64;
    use super::super::generators::unsigned_u64::generators_u64;
    check_devour_sort(generators_u32(), &[0u32, 1, u32::MAX, u32::MAX - 1, 1 << 31, (1 << 31) - 1, 12345], "u32");
    check_devour_sort(generators_u64(), &[0u64, 1, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 63) - 1, 12345], "u64");
    check_devour_sort(generators_i32(), &[0i32, 1, -1, i32::MIN, i32::MAX, i32::MIN + 1, -12345], "i32");
    check_devour_sort(generators_i64(), &[0i64, 1, -1, i64::MIN, i64::MAX, i64::MIN + 1, -12345], "i64");
    use super::super::generators::int_extra::*;
    check_devour_sort(extra_u32::generators(), &[0u32], "u32 extra");
    check_devour_sort(extra_u64::generators(), &[0u64], "u64 extra");
    check_devour_sort(extra_i32::generators(), &[0i32], "i32 extra");
    check_devour_sort(extra_i64::generators(), &[0i64], "i64 extra");
}

/// Narrow-range counting window: ranges at the ends of the key domain, keys
/// outside the sample window early, late, below and above it (count
/// resumed in a wider window, or given up beyond the limit).
#[test]
fn test_ded_devour_sort_window() {
    use rand::RngExt;
    macro_rules! window {
        ($t:ty) => {
            let mut rng = rand::rng();
            let bases: [$t; 4] = [<$t>::MIN, <$t>::MAX - 60, 0, (<$t>::MAX / 3) as $t];
            for &n in [129usize, 1_000, 5_000, 100_000, 300_000].iter() {
                for &base in bases.iter() {
                    for case in 0..8 {
                        let mut arr: Vec<$t> = (0..n).map(|_| base.wrapping_add(rng.random_range(0..60) as $t)).collect();
                        let pos = [n / 50, n * 95 / 100, n - 1, 0][case % 4];
                        let off: $t = if case < 4 { 3_000 as $t } else { (1u64 << 20) as $t };
                        arr[pos] = if base == <$t>::MAX - 60 { base.wrapping_sub(off) } else { base.wrapping_add(off) };
                        if case % 2 == 1 {
                            arr[n / 3] = base.wrapping_add(200 as $t);
                        }
                        let mut expected = arr.clone();
                        expected.sort_unstable();
                        crate::devour_sort(&mut arr);
                        assert!(arr == expected, "{} n {} base {} case {}", stringify!($t), n, base, case);
                    }
                }
            }
        };
    }
    window!(u32);
    window!(u64);
    window!(i32);
    window!(i64);
}

/// The Verge skip is exact: whenever it says skip, the Verge pre-processing
/// keeps no run (one chunk, the whole array, left untouched).
#[test]
fn test_ded_skip_verge_exact() {
    use super::super::algo::verge_sort_fast::verge_sort_preprocessing_fast;
    use super::super::dedicated::ravenous_sort::skip_verge;
    fn check<T: Copy + PartialOrd + std::fmt::Debug>(gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, ty: &str) {
        let mut skipped = 0;
        let mut total = 0;
        for &n in [129usize, 150, 200, 351, 1_000, 4_096, 10_000, 65_537, 200_000].iter() {
            for (gen, name) in gens.iter() {
                for _ in 0..3 {
                    let arr = gen(n);
                    total += 1;
                    if !skip_verge(&arr) {
                        continue;
                    }
                    skipped += 1;
                    let mut work = arr.clone();
                    let chunks = std::cell::RefCell::new(Vec::new());
                    let seps = verge_sort_preprocessing_fast::<_, false>(&mut work, 8, &|c: &mut [T], _| chunks.borrow_mut().push(c.len()));
                    assert_eq!(seps, vec![0, n], "{} {} {}", ty, name, n);
                    assert_eq!(chunks.into_inner(), vec![n], "{} {} {}", ty, name, n);
                    assert!(work.iter().zip(arr.iter()).all(|(a, b)| a.partial_cmp(b) == Some(std::cmp::Ordering::Equal) || (a != a && b != b)), "{} {} {}", ty, name, n);
                }
            }
        }
        assert!(skipped > 0 && skipped < total, "{} {}/{}", ty, skipped, total);
    }
    use super::super::generators::int_extra::*;
    let mut g = generators_u32();
    g.extend(extra_u32::generators());
    check(g, "u32");
    let mut g = super::super::generators::unsigned_u64::generators_u64();
    g.extend(extra_u64::generators());
    check(g, "u64");
    let mut g = super::super::generators::signed_i64::generators_i64();
    g.extend(extra_i64::generators());
    check(g, "i64");
    check(super::super::generators::float_64::generators_f64(), "f64");
    check(super::super::generators::float_32::generators_f32(), "f32");
}

/// Repair of nearly sorted arrays: misplaced elements alone, side by side
/// (large or small, up to 10 in a row), at both ends; random arrays. Sorted
/// and a permutation of the input when it succeeds, a permutation otherwise.
#[test]
fn test_ded_repair_near_sorted() {
    use super::super::algo::near_sorted::repair_near_sorted;
    use rand::RngExt;
    let mut rng = rand::rng();
    for &n in [64usize, 100, 1_000, 10_000, 100_000].iter() {
        for case in 0..40 {
            let mut arr: Vec<u32> = (0..n as u32).map(|v| v * 4).collect();
            if case == 39 {
                arr.iter_mut().for_each(|v| *v = rng.random());
            } else {
                let groups = 1 + case % 5;
                for _ in 0..groups {
                    let len = 1 + rng.random_range(0..10usize.min(n / 8));
                    let at = match case % 4 { 0 => 0, 1 => n - len, _ => rng.random_range(0..n - len) };
                    for k in 0..len {
                        arr[at + k] = if case % 3 == 0 { rng.random_range(0..4 * n as u32) } else if case % 3 == 1 { 4 * n as u32 + k as u32 } else { k as u32 };
                    }
                }
            }
            if case % 2 == 1 {
                arr.reverse();
            }
            let mut expected = arr.clone();
            expected.sort_unstable();
            let mut work = arr.clone();
            let ok = repair_near_sorted(&mut work);
            let mut perm = work.clone();
            perm.sort_unstable();
            assert_eq!(perm, expected, "n {} case {}: not a permutation", n, case);
            if ok {
                assert_eq!(work, expected, "n {} case {}: not sorted", n, case);
            }
        }
    }
}

/// devour_sort on 8- and 16-bit integers and char: every length up to 70,
/// around the small-path, counting and domain limits, extremes.
#[test]
fn test_ded_devour_sort_narrow_types() {
    use super::super::generators::char::generators_char;
    use super::super::generators::signed_i16::generators_i16;
    use super::super::generators::signed_i8::generators_i8;
    use super::super::generators::unsigned_u16::generators_u16;
    use super::super::generators::unsigned_u8::generators_u8;
    fn check<T: crate::DevourSort + std::fmt::Debug>(gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extremes: &[T], ty: &str) {
        let sizes: Vec<usize> = (0..=70)
            .chain([128, 129, 150, 151, 255, 256, 257, 32_767, 32_768, 65_535, 65_536, 131_071, 131_072, 131_073, 300_000, 524_287, 524_288].iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter() {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.sort_unstable();
                crate::devour_sort(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.sort_unstable();
            crate::devour_sort(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    use super::super::generators::narrow_extra::*;
    fn with<T>(mut g: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>) -> Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)> {
        g.extend(extra);
        g
    }
    check(with(generators_u8(), extra_u8::generators()), &[0u8, 1, 127, 128, 254, 255], "u8");
    check(with(generators_i8(), extra_i8::generators()), &[0i8, 1, -1, i8::MIN, i8::MAX, i8::MIN + 1], "i8");
    check(with(generators_u16(), extra_u16::generators()), &[0u16, 1, 255, 256, u16::MAX, u16::MAX - 1, 1 << 15], "u16");
    check(with(generators_i16(), extra_i16::generators()), &[0i16, 1, -1, i16::MIN, i16::MAX, i16::MIN + 1, -12345], "i16");
    check(with(generators_char(), extra_char::generators()), &['\0', 'a', 'é', '\u{D7FF}', '\u{E000}', '\u{FFFF}', '\u{10000}', char::MAX], "char");
}

/// devour_sort dense range with outliers: most keys in a narrow window, a
/// minority spread below and above (all widths, signed and unsigned).
#[test]
fn test_ded_devour_sort_dense_outliers() {
    use rand::RngExt;
    macro_rules! check {
        ($t:ty, $centre:expr) => {
            let mut rng = rand::rng();
            for &n in [4095usize, 4096, 10_000, 100_000, 300_000].iter() {
                for &inside in [50usize, 60, 75, 90, 99].iter() {
                    let mut arr: Vec<$t> = (0..n)
                        .map(|_| {
                            if rng.random_range(0..100) < inside {
                                ($centre as $t).wrapping_add(rng.random_range(0..200) as $t)
                            } else {
                                rng.random::<$t>()
                            }
                        })
                        .collect();
                    let mut expected = arr.clone();
                    expected.sort_unstable();
                    crate::devour_sort(&mut arr);
                    assert!(arr == expected, "{} n {} inside {}", stringify!($t), n, inside);
                }
            }
        };
    }
    check!(u16, 30_000u16);
    check!(i16, -100i16);
    check!(u32, 1_000_000u32);
    check!(i32, -5_000i32);
    check!(u64, u64::MAX - 1000);
    check!(i64, i64::MIN + 10);
    let mut rng = rand::rng();
    for &n in [5_000usize, 100_000].iter() {
        let mut arr: Vec<char> = (0..n)
            .map(|_| if rng.random_range(0..10) < 9 { (b'a' + rng.random_range(0..26u8)) as char } else { rng.random::<char>() })
            .collect();
        let mut expected = arr.clone();
        expected.sort_unstable();
        crate::devour_sort(&mut arr);
        assert!(arr == expected, "char n {}", n);
    }
}

/// devour_sort on a range wrapping around the end of the domain (values near
/// the minimum and near the maximum of the encoded domain).
#[test]
fn test_ded_devour_sort_wrapped_range() {
    use rand::RngExt;
    macro_rules! check {
        ($t:ty) => {
            let mut rng = rand::rng();
            for &n in [129usize, 1000, 4096, 10_000, 100_000, 300_000].iter() {
                for &low_pct in [0usize, 1, 50, 99, 100].iter() {
                    let mut arr: Vec<$t> = (0..n)
                        .map(|_| {
                            let d = rng.random_range(0..300) as $t;
                            if rng.random_range(0..100) < low_pct { <$t>::MIN.wrapping_add(d) } else { <$t>::MAX.wrapping_sub(d) }
                        })
                        .collect();
                    if n > 1000 {
                        arr[n * 95 / 100] = <$t>::MAX / 2;
                    }
                    let mut expected = arr.clone();
                    expected.sort_unstable();
                    crate::devour_sort(&mut arr);
                    assert!(arr == expected, "{} n {} low {}", stringify!($t), n, low_pct);
                }
            }
        };
    }
    check!(u16);
    check!(i16);
    check!(u32);
    check!(i32);
    check!(u64);
    check!(i64);
}

/// devour_sort on long runs of equal keys (plateaus), around the thresholds of
/// the run compression.
#[test]
fn test_ded_devour_sort_plateaus() {
    use rand::RngExt;
    macro_rules! check {
        ($t:ty) => {
            let mut rng = rand::rng();
            for &n in [1023usize, 1024, 5_000, 100_000].iter() {
                for &run in [1usize, 16, 31, 32, 33, 100, 1000].iter() {
                    let mut arr: Vec<$t> = Vec::with_capacity(n);
                    while arr.len() < n {
                        let v = rng.random::<$t>();
                        let len = (rng.random_range(1..=2 * run)).min(n - arr.len());
                        arr.extend(std::iter::repeat(v).take(len));
                    }
                    let mut expected = arr.clone();
                    expected.sort_unstable();
                    crate::devour_sort(&mut arr);
                    assert!(arr == expected, "{} n {} run {}", stringify!($t), n, run);
                }
            }
        };
    }
    check!(u8);
    check!(i16);
    check!(u32);
    check!(i32);
    check!(u64);
    check!(i64);
}

/// Values at both ends of the domain (offset sort on rotated keys), with and
/// without keys in the middle, every 32/64-bit type.
#[test]
fn test_ded_devour_sort_both_ends() {
    use super::super::devour_sort;
    fn check<T: crate::DevourSort + std::fmt::Debug + Copy>(make: &dyn Fn(u64, usize) -> T, ty: &str) {
        let mut seed = 0x1234_5678_9abc_def1u64;
        for &n in &[200usize, 1000, 5000, 70_000] {
            for &width in &[1u64 << 12, 1 << 20, 1 << 28] {
                let mut v: Vec<T> = (0..n)
                    .map(|i| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        make(seed % width, i)
                    })
                    .collect();
                let mut e = v.clone();
                e.sort_unstable();
                devour_sort(&mut v);
                assert!(v == e, "{} n {} width {}", ty, n, width);
            }
        }
    }
    check(&|v, i| if i % 2 == 0 { v as u32 } else { u32::MAX - v as u32 }, "u32");
    check(&|v, i| if i % 2 == 0 { v } else { u64::MAX - v }, "u64");
    check(&|v, i| if i % 3 == 0 { v as u32 } else if i % 3 == 1 { u32::MAX - v as u32 } else { (v as u32) | 0x8000_0000 }, "u32 three");
    check(&|v, i| if i % 2 == 0 { (v as i32).wrapping_add(i32::MIN) } else { i32::MAX - v as i32 }, "i32");
    check(&|v, i| if i % 2 == 0 { (v as i64).wrapping_add(i64::MIN) } else { i64::MAX - v as i64 }, "i64");
    check(&|v, i| if i % 2 == 0 { -(v as i64) } else { v as i64 }, "i64 around 0");
}

/// `usize` / `isize` (sorted as the integer of the same width): every
/// generator, with the extremes.
#[test]
fn test_ded_devour_sort_pointer_width() {
    use super::super::devour_sort;
    use super::super::generators::signed_i64::generators_isize;
    use super::super::generators::unsigned_u64::generators_usize;
    for &size in &[0usize, 1, 33, 500, 10_000, 200_000] {
        for (gen, name) in generators_usize().iter() {
            let mut a = gen(size);
            if size > 2 {
                a[0] = usize::MAX;
                a[1] = 0;
            }
            let mut e = a.clone();
            e.sort_unstable();
            devour_sort(&mut a);
            assert!(a == e, "usize {} {}", name, size);
        }
        for (gen, name) in generators_isize().iter() {
            let mut a = gen(size);
            if size > 2 {
                a[0] = isize::MAX;
                a[1] = isize::MIN;
            }
            let mut e = a.clone();
            e.sort_unstable();
            devour_sort(&mut a);
            assert!(a == e, "isize {} {}", name, size);
        }
    }
}

/// Wide ranges counted (span beyond 65536 values, at least 4 keys per
/// counter), and the failed bet (one outlier outside the sampled range).
#[test]
fn test_ded_devour_sort_wide_count() {
    use super::super::devour_sort;
    let mut seed = 0x0bad_5eed_1234_5678u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for &(n, span) in &[(300_000usize, 70_000u64), (1_200_000, 300_000)] {
        for outlier in [false, true] {
            let mut a: Vec<u32> = (0..n).map(|_| 1_000_000 + (next() % span) as u32).collect();
            let mut b: Vec<i64> = (0..n).map(|_| (next() % span) as i64 - span as i64 / 2).collect();
            let mut c: Vec<char> = (0..n).map(|_| std::char::from_u32(0x4E00 + (next() % span.min(0x5000)) as u32).unwrap()).collect();
            if outlier {
                a[n / 3] = u32::MAX;
                b[n / 3] = i64::MIN;
                c[n / 3] = '\u{10FFFF}';
            }
            let (mut ea, mut eb, mut ec) = (a.clone(), b.clone(), c.clone());
            ea.sort_unstable();
            eb.sort_unstable();
            ec.sort_unstable();
            devour_sort(&mut a);
            devour_sort(&mut b);
            devour_sort(&mut c);
            assert!(a == ea && b == eb && c == ec, "n {} span {} outlier {}", n, span, outlier);
        }
    }
}

/// `devour_sort2` (algorithmic work copy): same coverage as `devour_sort`,
/// every type, every generator, every benchmark size and the thresholds.
#[test]
fn test_ded_devour_sort2() {
    use super::super::dedicated::devour_sort2::{devour_sort2, DevourSort2};
    use super::super::generators::char::generators_char;
    use super::super::generators::int_extra::{extra_i32, extra_i64, extra_u32, extra_u64};
    use super::super::generators::narrow_extra::{extra_char, extra_i16, extra_i8, extra_u16, extra_u8};
    use super::super::generators::signed_i16::generators_i16;
    use super::super::generators::signed_i32::generators_i32;
    use super::super::generators::signed_i64::generators_i64;
    use super::super::generators::signed_i8::generators_i8;
    use super::super::generators::unsigned_u16::generators_u16;
    use super::super::generators::unsigned_u64::generators_u64;
    use super::super::generators::unsigned_u8::generators_u8;
    fn check<T: DevourSort2 + std::fmt::Debug>(mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, extremes: &[T], ty: &str) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain([128, 129, 150, 151, 199, 200, 201, 255, 256, 257, 350, 351, 700, 701, 5_000, 65_535, 65_536, 131_072, 524_288, 1_048_576, 4_194_304].iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter() {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.sort_unstable();
                devour_sort2(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.sort_unstable();
            devour_sort2(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    check(generators_u32(), extra_u32::generators(), &[0u32, 1, u32::MAX, u32::MAX - 1, 1 << 31, (1 << 31) - 1], "u32");
    check(generators_u64(), extra_u64::generators(), &[0u64, 1, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 32) - 1, 1 << 32], "u64");
    check(generators_i32(), extra_i32::generators(), &[0i32, -1, 1, i32::MIN, i32::MAX], "i32");
    check(generators_i64(), extra_i64::generators(), &[0i64, -1, 1, i64::MIN, i64::MAX, 1 << 32, -(1 << 32)], "i64");
    check(generators_u16(), extra_u16::generators(), &[0u16, 1, u16::MAX], "u16");
    check(generators_i16(), extra_i16::generators(), &[0i16, -1, i16::MIN, i16::MAX], "i16");
    check(generators_u8(), extra_u8::generators(), &[0u8, 1, u8::MAX], "u8");
    check(generators_i8(), extra_i8::generators(), &[0i8, -1, i8::MIN, i8::MAX], "i8");
    check(generators_char(), extra_char::generators(), &['\0', 'a', '\u{10FFFF}', '\u{D7FF}', '\u{E000}'], "char");
}

/// `devour_sort2` on 128-bit keys: every generator, the sizes of the other
/// integer tests (thresholds and benchmarks), extreme values.
#[test]
fn test_ded_devour_sort2_128() {
    use super::super::dedicated::devour_sort2::{devour_sort2, DevourSort2};
    use super::super::generators::int128_extra::{extra_i128, extra_u128};
    use super::super::generators::signed_i128::generators_i128;
    use super::super::generators::unsigned_u128::generators_u128;
    fn check<T: DevourSort2 + std::fmt::Debug>(mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, extremes: &[T], ty: &str) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain([128, 129, 255, 256, 257, 700, 701, 2_047, 2_048, 4_095, 4_096, 5_000, 65_536, 131_072, 524_288, 1_048_576].iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter() {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.sort_unstable();
                devour_sort2(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.sort_unstable();
            devour_sort2(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    check(generators_u128(), extra_u128::generators(), &[0u128, 1, u128::MAX, u128::MAX - 1, 1 << 127, 1 << 64, (1 << 64) - 1, u64::MAX as u128 + 7], "u128");
    check(generators_i128(), extra_i128::generators(), &[0i128, -1, 1, i128::MIN, i128::MAX, 1 << 64, -(1 << 64), i64::MIN as i128, i64::MAX as i128], "i128");
}

/// `devour_sort_by_key`: same result as the stable `sort_by` on the key
/// (records carry their original index): keys of 16 to 64 bits and char,
/// records of 8 to 128 bytes, key first or last, a `String` record, a
/// composite key.
#[test]
fn test_ded_sort_by_key() {
    use super::super::dedicated::by_key::{devour_sort_by_key, SortKey};
    use super::super::generators::records::*;
    fn check<T: Clone + PartialEq + std::fmt::Debug, K: SortKey>(gens: Gens<T>, key: fn(&T) -> K, ty: &str) {
        let sizes: Vec<usize> = (0..=40).chain([64, 65, 127, 128, 255, 256, 1_000, 4_096, 65_536, 100_000, 1_000_000].iter().copied()).collect();
        for &size in sizes.iter() {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.sort_by(|a, b| key(a).encode().cmp(&key(b).encode()));
                devour_sort_by_key(&mut arr, key);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
        }
    }
    macro_rules! sizes {
        ($keys:expr, $k:ty, $w8:literal, $w32:literal, $w128:literal, $name:literal) => {{
            if $w8 > 0 {
                check(wrap($keys, rec::<$k, $w8>), |r| r.key, concat!($name, " 8 B"));
            }
            check(wrap($keys, rec::<$k, $w32>), |r| r.key, concat!($name, " 32 B"));
            check(wrap($keys, rec::<$k, $w128>), |r| r.key, concat!($name, " 128 B"));
            check(wrap($keys, rec_end::<$k, 3>), |r| r.key, concat!($name, " key last"));
        }};
    }
    sizes!(keys_u16(), u16, 1, 7, 31, "u16");
    sizes!(keys_i16(), i16, 1, 7, 31, "i16");
    sizes!(keys_u32(), u32, 1, 7, 31, "u32");
    sizes!(keys_i32(), i32, 1, 7, 31, "i32");
    sizes!(keys_f32(), f32, 1, 7, 31, "f32");
    sizes!(keys_char(), char, 1, 7, 31, "char");
    sizes!(keys_u64(), u64, 0, 6, 30, "u64");
    sizes!(keys_i64(), i64, 0, 6, 30, "i64");
    sizes!(keys_f64(), f64, 0, 6, 30, "f64");
    check(wrap(keys_u64(), rec_string), |r| r.key, "string");
    check(wrap(keys_u64(), rec_pair), |r| r.key(), "pair");
    // The LSD on the elements, forced.
    use super::super::dedicated::by_key::BY_KEY_ENGINE;
    BY_KEY_ENGINE.store(1, std::sync::atomic::Ordering::Relaxed);
    sizes!(keys_u16(), u16, 1, 7, 31, "lsd u16");
    sizes!(keys_i32(), i32, 1, 7, 31, "lsd i32");
    sizes!(keys_f64(), f64, 0, 6, 30, "lsd f64");
    check(wrap(keys_u64(), rec_string), |r| r.key, "lsd string");
    BY_KEY_ENGINE.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// `ravenous_sort2` (algorithmic work copy): bit for bit `total_cmp` order,
/// same sizes as `ravenous_sort`.
#[test]
fn test_ded_ravenous_sort2() {
    use super::super::dedicated::ravenous_sort2::ravenous_sort2;
    check_total_order_f32(&|a| ravenous_sort2(a));
    check_total_order_f64(&|a| ravenous_sort2(a));
}
