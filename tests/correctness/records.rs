//! Records sorted by key: `voracious_sort_by_key` (stable) and
//! `voracious_sort_unstable_by_key`.

use rayon::slice::ParallelSliceMut;
use voracious_radix_sort::RadixSort;

use crate::{quick_max, BENCH_SIZES};

/// Sizes for the record tests: small ones, the thresholds of the sort
/// (few-keys count from 128, Verge from 4096, the cache limit of the record
/// size: half the L2 on common L2 sizes), the benchmark sizes up to `max`.
fn record_sizes(record: usize, max: usize) -> Vec<usize> {
    // The cache limit of the record size (half the L2) on common L2 sizes.
    let cache: Vec<usize> = [256 << 10, 512 << 10, 1 << 20, 2 << 20, 12 << 20]
        .iter()
        .flat_map(|&l2: &usize| {
            let n = l2 / 2 / record;
            [n - 1, n, n + 1]
        })
        .collect();
    let mut sizes: Vec<usize> = (0..=40)
        .chain(
            [64, 65, 127, 128, 129, 255, 256, 257, 1_000, 4_095, 4_096, 4_097, 65_536]
                .iter()
                .copied(),
        )
        .chain(cache)
        .chain(BENCH_SIZES.iter().copied())
        .filter(|&n| n <= max)
        .collect();
    sizes.sort_unstable();
    sizes.dedup();
    sizes
}

/// `famished_sort_by_key` checked against the stable `sort_by` on the key
/// (records carry their original index, so any reordering of equal keys
/// fails).
fn check_by_key<T: Clone + PartialEq + std::fmt::Debug + Send, K: voracious_radix_sort::SortKey>(
    gens: &crate::generators::records::Gens<T>,
    key: fn(&T) -> K,
    sizes: &[usize],
    ty: &str,
) {
    for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
        for (gen, name) in gens.iter() {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.par_sort_by(|a, b| key(a).encode().cmp(&key(b).encode()));
            arr.voracious_sort_by_key(key);
            assert!(arr == expected, "{} {} size {}", ty, name, size);
        }
    }
}

/// `records`: the stable sort equal to the stable `sort_by` on the key; the
/// unstable sort with the keys in order and the same records (both sides
/// put in one order: key, then the record, whose payload holds its
/// original index).
fn check_records<
    T: Clone + PartialEq + PartialOrd + std::fmt::Debug + Send,
    K: voracious_radix_sort::SortKey,
>(
    gens: &crate::generators::records::Gens<T>,
    key: fn(&T) -> K,
    sizes: &[usize],
    ty: &str,
) {
    let total = |a: &T, b: &T| {
        key(a)
            .encode()
            .cmp(&key(b).encode())
            .then(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    };
    for &size in sizes.iter().filter(|&&n| n <= quick_max()) {
        for (gen, name) in gens.iter() {
            let arr = gen(size);
            let mut expected = arr.clone();
            expected.par_sort_by(|a, b| key(a).encode().cmp(&key(b).encode()));
            let mut stable = arr.clone();
            stable.voracious_sort_by_key(key);
            assert!(stable == expected, "stable {} {} size {}", ty, name, size);
            let mut unstable = arr;
            unstable.voracious_sort_unstable_by_key(key);
            assert!(
                unstable.windows(2).all(|w| key(&w[0]).encode() <= key(&w[1]).encode()),
                "unstable order {} {} size {}",
                ty,
                name,
                size
            );
            unstable.par_sort_by(total);
            expected.par_sort_by(total);
            assert!(unstable == expected, "unstable records {} {} size {}", ty, name, size);
        }
    }
}

/// `records`, stable and unstable, on the key distributions: keys of 16 to
/// 64 bits and char, records of 8 to 128 bytes, key first or last, a
/// `String` record, a composite key; up to 1M.
#[test]
fn records_by_key() {
    use crate::generators::records::*;
    macro_rules! sizes {
        ($keys:expr, $k:ty, $w8:literal, $w16:literal, $w32:literal, $w128:literal, $name:literal) => {{
            if $w8 > 0 {
                check_records(
                    &wrap($keys, rec::<$k, $w8>),
                    |r| r.key,
                    &record_sizes(8, 1_000_000),
                    concat!($name, " 8 B"),
                );
            }
            check_records(
                &wrap($keys, rec::<$k, $w16>),
                |r| r.key,
                &record_sizes(16, 1_000_000),
                concat!($name, " 16 B"),
            );
            check_records(
                &wrap($keys, rec::<$k, $w32>),
                |r| r.key,
                &record_sizes(32, 1_000_000),
                concat!($name, " 32 B"),
            );
            check_records(
                &wrap($keys, rec::<$k, $w128>),
                |r| r.key,
                &record_sizes(128, 1_000_000),
                concat!($name, " 128 B"),
            );
            check_records(
                &wrap($keys, rec_end::<$k, 3>),
                |r| r.key,
                &record_sizes(16, 1_000_000),
                concat!($name, " key last"),
            );
        }};
    }
    sizes!(keys_u16(), u16, 1, 3, 7, 31, "u16");
    sizes!(keys_i16(), i16, 1, 3, 7, 31, "i16");
    sizes!(keys_u32(), u32, 1, 3, 7, 31, "u32");
    sizes!(keys_i32(), i32, 1, 3, 7, 31, "i32");
    sizes!(keys_f32(), f32, 1, 3, 7, 31, "f32");
    sizes!(keys_char(), char, 1, 3, 7, 31, "char");
    sizes!(keys_u64(), u64, 0, 2, 6, 30, "u64");
    sizes!(keys_i64(), i64, 0, 2, 6, 30, "i64");
    sizes!(keys_f64(), f64, 0, 2, 6, 30, "f64");
    check_records(&wrap(keys_u64(), rec_string), |r| r.key, &record_sizes(32, 1_000_000), "string");
    check_records(&wrap(keys_u64(), rec_pair), |r| r.key(), &record_sizes(16, 1_000_000), "pair");
}

/// Stability on keys built for it (many equal keys in every shape the sort
/// treats apart), every key type and record size, every size up to 1M.
#[test]
fn sort_by_key_stability() {
    use crate::generators::records::stability::as_keys;
    use crate::generators::records::*;
    macro_rules! stab {
        ($k:ty, $conv:expr, $w8:literal, $w32:literal, $w128:literal, $name:literal) => {{
            let keys = || as_keys::<$k>($conv);
            if $w8 > 0 {
                check_by_key(
                    &wrap(keys(), rec::<$k, $w8>),
                    |r| r.key,
                    &record_sizes(8, 1_000_000),
                    concat!($name, " 8 B"),
                );
            }
            check_by_key(
                &wrap(keys(), rec::<$k, $w32>),
                |r| r.key,
                &record_sizes(32, 1_000_000),
                concat!($name, " 32 B"),
            );
            check_by_key(
                &wrap(keys(), rec::<$k, $w128>),
                |r| r.key,
                &record_sizes(128, 1_000_000),
                concat!($name, " 128 B"),
            );
            check_by_key(
                &wrap(keys(), rec_end::<$k, 3>),
                |r| r.key,
                &record_sizes(16, 1_000_000),
                concat!($name, " key last"),
            );
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
    check_by_key(
        &wrap(as_keys::<u64>(|v| v), rec_string),
        |r| r.key,
        &record_sizes(32, 1_000_000),
        "stab string",
    );
    check_by_key(
        &wrap(as_keys::<u64>(|v| v), rec_pair),
        |r| r.key(),
        &record_sizes(16, 1_000_000),
        "stab pair",
    );
}

/// Stability at the largest benchmark sizes (3M, 10M), records of 8 and 32
/// bytes.
#[test]
fn sort_by_key_stability_large() {
    use crate::generators::records::stability::as_keys;
    use crate::generators::records::*;
    let large: Vec<usize> = BENCH_SIZES.iter().copied().filter(|&n| n > 1_000_000).collect();
    check_by_key(
        &wrap(as_keys::<u32>(|v| v as u32), rec::<u32, 1>),
        |r| r.key,
        &large,
        "stab u32 8 B",
    );
    check_by_key(&wrap(as_keys::<u64>(|v| v), rec::<u64, 6>), |r| r.key, &large, "stab u64 32 B");
    check_by_key(
        &wrap(as_keys::<f64>(|v| v as f64), rec::<f64, 2>),
        |r| r.key,
        &large,
        "stab f64 16 B",
    );
}
