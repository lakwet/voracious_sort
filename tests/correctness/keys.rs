//! Integers and `char`: `devour_sort` and `RadixSort::voracious_sort`
//! against the standard sort.

use rayon::slice::ParallelSliceMut;

use crate::{quick_max, BENCH_SIZES, THRESHOLD_SIZES};

/// `devour_sort` and `voracious_sort` on every integer type and `char`:
/// every generator, the thresholds and the benchmark sizes, extreme values.
#[test]
fn integers() {
    use crate::generators::char::generators_char;
    use crate::generators::int_extra::{extra_i32, extra_i64, extra_u32, extra_u64};
    use crate::generators::narrow_extra::{extra_char, extra_i16, extra_i8, extra_u16, extra_u8};
    use crate::generators::signed_i16::generators_i16;
    use crate::generators::signed_i32::generators_i32;
    use crate::generators::signed_i64::generators_i64;
    use crate::generators::signed_i8::generators_i8;
    use crate::generators::unsigned_u16::generators_u16;
    use crate::generators::unsigned_u32::generators_u32;
    use crate::generators::unsigned_u64::generators_u64;
    use crate::generators::unsigned_u8::generators_u8;
    use voracious_radix_sort::{devour_sort, DevourSort, RadixSort, Sortable};
    fn check<T: DevourSort + Sortable + std::fmt::Debug + Send>(
        mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>,
        extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>,
        extremes: &[T],
        ty: &str,
    ) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain(THRESHOLD_SIZES.iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.par_sort_unstable();
                let mut by_trait = arr.clone();
                devour_sort(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
                by_trait.voracious_sort();
                assert!(by_trait == expected, "voracious_sort {} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> =
                (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.par_sort_unstable();
            devour_sort(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    check(
        generators_u32(),
        extra_u32::generators(),
        &[0u32, 1, u32::MAX, u32::MAX - 1, 1 << 31, (1 << 31) - 1],
        "u32",
    );
    check(
        generators_u64(),
        extra_u64::generators(),
        &[0u64, 1, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 32) - 1, 1 << 32],
        "u64",
    );
    check(generators_i32(), extra_i32::generators(), &[0i32, -1, 1, i32::MIN, i32::MAX], "i32");
    check(
        generators_i64(),
        extra_i64::generators(),
        &[0i64, -1, 1, i64::MIN, i64::MAX, 1 << 32, -(1 << 32)],
        "i64",
    );
    check(generators_u16(), extra_u16::generators(), &[0u16, 1, u16::MAX], "u16");
    check(generators_i16(), extra_i16::generators(), &[0i16, -1, i16::MIN, i16::MAX], "i16");
    check(generators_u8(), extra_u8::generators(), &[0u8, 1, u8::MAX], "u8");
    check(generators_i8(), extra_i8::generators(), &[0i8, -1, i8::MIN, i8::MAX], "i8");
    check(
        generators_char(),
        extra_char::generators(),
        &['\0', 'a', '\u{10FFFF}', '\u{D7FF}', '\u{E000}'],
        "char",
    );
}

/// `devour_sort` on 128-bit keys: every generator, the sizes of the other
/// integer tests (thresholds and benchmarks), extreme values.
#[test]
fn integers_128() {
    use crate::generators::int128_extra::{extra_i128, extra_u128};
    use crate::generators::signed_i128::generators_i128;
    use crate::generators::unsigned_u128::generators_u128;
    use voracious_radix_sort::{devour_sort, DevourSort, RadixSort, Sortable};
    fn check<T: DevourSort + Sortable + std::fmt::Debug + Send>(
        mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>,
        extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>,
        extremes: &[T],
        ty: &str,
    ) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain(THRESHOLD_SIZES.iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.par_sort_unstable();
                let mut by_trait = arr.clone();
                devour_sort(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
                by_trait.voracious_sort();
                assert!(by_trait == expected, "voracious_sort {} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> =
                (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.par_sort_unstable();
            devour_sort(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    check(
        generators_u128(),
        extra_u128::generators(),
        &[
            0u128,
            1,
            u128::MAX,
            u128::MAX - 1,
            1 << 127,
            1 << 64,
            (1 << 64) - 1,
            u64::MAX as u128 + 7,
        ],
        "u128",
    );
    check(
        generators_i128(),
        extra_i128::generators(),
        &[
            0i128,
            -1,
            1,
            i128::MIN,
            i128::MAX,
            1 << 64,
            -(1 << 64),
            i64::MIN as i128,
            i64::MAX as i128,
        ],
        "i128",
    );
}
