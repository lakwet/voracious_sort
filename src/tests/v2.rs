//! Correctness of the `v2` sorts (same checks as for the work copies in
//! `dedicated.rs`).

use rayon::slice::ParallelSliceMut;
use super::super::generators::unsigned_u32::generators_u32;

/// `V2_QUICK` set: sizes up to 100k only (quick check after each change).
fn quick_max() -> usize {
    if std::env::var("V2_QUICK").is_ok() { 100_000 } else { usize::MAX }
}

const BENCH_SIZES: [usize; 10] = [1_000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000, 3_000_000, 10_000_000];

/// `devour_sort2` (algorithmic work copy): same coverage as `devour_sort`,
/// every type, every generator, every benchmark size and the thresholds.
#[test]
fn test_v2_devour_sort2() {
    use super::super::v2::integers::{devour_sort2, DevourSort2};
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
    fn check<T: DevourSort2 + std::fmt::Debug + Send>(mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, extremes: &[T], ty: &str) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain([128, 129, 150, 151, 199, 200, 201, 255, 256, 257, 350, 351, 700, 701, 5_000, 65_535, 65_536, 131_072, 524_288, 1_048_576, 4_194_304].iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.par_sort_unstable();
                devour_sort2(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.par_sort_unstable();
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
fn test_v2_devour_sort2_128() {
    use super::super::v2::integers::{devour_sort2, DevourSort2};
    use super::super::generators::int128_extra::{extra_i128, extra_u128};
    use super::super::generators::signed_i128::generators_i128;
    use super::super::generators::unsigned_u128::generators_u128;
    fn check<T: DevourSort2 + std::fmt::Debug + Send>(mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>, extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, extremes: &[T], ty: &str) {
        gens.extend(extra);
        let sizes: Vec<usize> = (0..=70)
            .chain([128, 129, 255, 256, 257, 700, 701, 2_047, 2_048, 4_095, 4_096, 5_000, 65_536, 131_072, 524_288, 1_048_576].iter().copied())
            .chain(BENCH_SIZES.iter().copied())
            .collect();
        for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
            for (gen, name) in gens.iter() {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.par_sort_unstable();
                devour_sort2(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
            let mut arr: Vec<T> = (0..size).map(|i| extremes[(i * 7919) % extremes.len()]).collect();
            let mut expected = arr.clone();
            expected.par_sort_unstable();
            devour_sort2(&mut arr);
            assert!(arr == expected, "{} extremes size {}", ty, size);
        }
    }
    check(generators_u128(), extra_u128::generators(), &[0u128, 1, u128::MAX, u128::MAX - 1, 1 << 127, 1 << 64, (1 << 64) - 1, u64::MAX as u128 + 7], "u128");
    check(generators_i128(), extra_i128::generators(), &[0i128, -1, 1, i128::MIN, i128::MAX, 1 << 64, -(1 << 64), i64::MIN as i128, i64::MAX as i128], "i128");
}

/// Sizes for the record tests: small ones, the thresholds of the sort
/// (few-keys count from 128, Verge from 4096, the cache limit of the record
/// size: half the L2), the benchmark sizes up to `max`.
fn record_sizes(record: usize, max: usize) -> Vec<usize> {
    let cache = super::super::v2::cache::l2_bytes() / 2 / record;
    let mut sizes: Vec<usize> = (0..=40)
        .chain([64, 65, 127, 128, 129, 255, 256, 257, 1_000, 4_095, 4_096, 4_097, 65_536].iter().copied())
        .chain([cache - 1, cache, cache + 1].iter().copied())
        .chain(BENCH_SIZES.iter().copied())
        .filter(|&n| n <= max)
        .collect();
    sizes.sort_unstable();
    sizes.dedup();
    sizes
}

/// `devour_sort_by_key` checked against the stable `sort_by` on the key
/// (records carry their original index, so any reordering of equal keys
/// fails).
fn check_by_key<T: Clone + PartialEq + std::fmt::Debug + Send, K: super::super::v2::by_key::SortKey>(gens: &super::super::generators::records::Gens<T>, key: fn(&T) -> K, sizes: &[usize], ty: &str) {
    use super::super::v2::by_key::{devour_sort_by_key, SortKey};
    for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
        for (gen, name) in gens.iter() {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.par_sort_by(|a, b| key(a).encode().cmp(&key(b).encode()));
            devour_sort_by_key(&mut arr, key);
            assert!(arr == expected, "{} {} size {}", ty, name, size);
        }
    }
}

/// `devour_sort_by_key` on the key distributions: keys of 16 to 64 bits and
/// char, records of 8 to 128 bytes, key first or last, a `String` record, a
/// composite key; up to 1M.
#[test]
fn test_v2_sort_by_key() {
    use super::super::generators::records::*;
    macro_rules! sizes {
        ($keys:expr, $k:ty, $w8:literal, $w32:literal, $w128:literal, $name:literal) => {{
            if $w8 > 0 {
                check_by_key(&wrap($keys, rec::<$k, $w8>), |r| r.key, &record_sizes(8, 1_000_000), concat!($name, " 8 B"));
            }
            check_by_key(&wrap($keys, rec::<$k, $w32>), |r| r.key, &record_sizes(32, 1_000_000), concat!($name, " 32 B"));
            check_by_key(&wrap($keys, rec::<$k, $w128>), |r| r.key, &record_sizes(128, 1_000_000), concat!($name, " 128 B"));
            check_by_key(&wrap($keys, rec_end::<$k, 3>), |r| r.key, &record_sizes(16, 1_000_000), concat!($name, " key last"));
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
    check_by_key(&wrap(keys_u64(), rec_string), |r| r.key, &record_sizes(32, 1_000_000), "string");
    check_by_key(&wrap(keys_u64(), rec_pair), |r| r.key(), &record_sizes(16, 1_000_000), "pair");
    // The LSD on the elements, forced.
    use super::super::v2::by_key::BY_KEY_ENGINE;
    BY_KEY_ENGINE.store(1, std::sync::atomic::Ordering::Relaxed);
    sizes!(keys_u16(), u16, 1, 7, 31, "lsd u16");
    sizes!(keys_i32(), i32, 1, 7, 31, "lsd i32");
    sizes!(keys_f64(), f64, 0, 6, 30, "lsd f64");
    check_by_key(&wrap(keys_u64(), rec_string), |r| r.key, &record_sizes(32, 1_000_000), "lsd string");
    BY_KEY_ENGINE.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// Stability on keys built for it (many equal keys in every shape the sort
/// treats apart), every key type and record size, every size up to 1M.
#[test]
fn test_v2_sort_by_key_stability() {
    use super::super::generators::records::stability::as_keys;
    use super::super::generators::records::*;
    macro_rules! stab {
        ($k:ty, $conv:expr, $w8:literal, $w32:literal, $w128:literal, $name:literal) => {{
            let keys = || as_keys::<$k>($conv);
            if $w8 > 0 {
                check_by_key(&wrap(keys(), rec::<$k, $w8>), |r| r.key, &record_sizes(8, 1_000_000), concat!($name, " 8 B"));
            }
            check_by_key(&wrap(keys(), rec::<$k, $w32>), |r| r.key, &record_sizes(32, 1_000_000), concat!($name, " 32 B"));
            check_by_key(&wrap(keys(), rec::<$k, $w128>), |r| r.key, &record_sizes(128, 1_000_000), concat!($name, " 128 B"));
            check_by_key(&wrap(keys(), rec_end::<$k, 3>), |r| r.key, &record_sizes(16, 1_000_000), concat!($name, " key last"));
        }};
    }
    stab!(u16, |v| v as u16, 1, 7, 31, "u16");
    stab!(i16, |v| (v as i16).wrapping_sub(1000), 1, 7, 31, "i16");
    stab!(u32, |v| v as u32, 1, 7, 31, "u32");
    stab!(i32, |v| v as i32 - 1000, 1, 7, 31, "i32");
    stab!(f32, |v| v as f32 - 0.5, 1, 7, 31, "f32");
    stab!(char, |v| char::from_u32((v % 0xD000) as u32).unwrap(), 1, 7, 31, "char");
    stab!(u64, |v| v, 0, 6, 30, "u64");
    stab!(i64, |v| v as i64 - 1000, 0, 6, 30, "i64");
    stab!(f64, |v| v as f64 * -0.25, 0, 6, 30, "f64");
    check_by_key(&wrap(as_keys::<u64>(|v| v), rec_string), |r| r.key, &record_sizes(32, 1_000_000), "stab string");
    check_by_key(&wrap(as_keys::<u64>(|v| v), rec_pair), |r| r.key(), &record_sizes(16, 1_000_000), "stab pair");
}

/// Stability at the largest benchmark sizes (3M, 10M), records of 8 and 32
/// bytes.
#[test]
fn test_v2_sort_by_key_stability_large() {
    use super::super::generators::records::stability::as_keys;
    use super::super::generators::records::*;
    let large: Vec<usize> = BENCH_SIZES.iter().copied().filter(|&n| n > 1_000_000).collect();
    check_by_key(&wrap(as_keys::<u32>(|v| v as u32), rec::<u32, 1>), |r| r.key, &large, "stab u32 8 B");
    check_by_key(&wrap(as_keys::<u64>(|v| v), rec::<u64, 6>), |r| r.key, &large, "stab u64 32 B");
    check_by_key(&wrap(as_keys::<f64>(|v| v as f64), rec::<f64, 2>), |r| r.key, &large, "stab f64 16 B");
}

/// `ravenous_sort2` (algorithmic work copy): bit for bit `total_cmp` order,
/// same sizes as `ravenous_sort`.
#[test]
fn test_v2_ravenous_sort2() {
    use super::super::v2::floats::ravenous_sort2;
    super::dedicated::check_total_order_f32(&|a| ravenous_sort2(a));
    super::dedicated::check_total_order_f64(&|a| ravenous_sort2(a));
}
