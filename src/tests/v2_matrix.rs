//! Every records chain of the v2 (`v2::types::records`: one per key type and
//! record size) reached and checked, stable and unstable: records of exactly
//! each size class, and between classes (12, 24, 40 bytes), for every key
//! type (bool, u8 ... u128, i128, f32, f64, char, usize, isize). Keys of
//! several shapes (random, narrow, few, heavy, sorted, reversed, sawtooth,
//! random tail, equal) and the special values (MIN, MAX, 0, NaN of both
//! signs, -0.0, +0.0, infinities). Array sizes around the thresholds of the
//! chains, and around the cache budgets of the records (`V2_QUICK`: up to
//! 100k).
//!
//! Stable: the record order equals a stable sort on the encoded key (the
//! payload holds the original index when it fits). Unstable: the keys in
//! order and the same records.

use crate::v2::cache::{budget, Budget};
use crate::v2::sorts::by_key::{devour_sort_by_key, devour_sort_unstable_by_key, SortKey};

/// Key first, then `N` payload bytes (the original index, little endian,
/// in the first up to 4); packed: the record takes exactly
/// `size_of::<K>() + N` bytes.
#[derive(Clone, Copy)]
#[repr(C, packed)]
struct P<K: Copy, const N: usize> {
    key: K,
    pay: [u8; N],
}

/// A key type built from 64 random bits, with its special values.
trait Key: SortKey + Copy + Send + Sync + 'static {
    fn from(x: u64) -> Self;
    fn specials() -> Vec<Self>;
}

macro_rules! int_key {
    ($($t:ty),*) => {$(
        impl Key for $t {
            fn from(x: u64) -> Self {
                x as $t
            }
            fn specials() -> Vec<Self> {
                vec![<$t>::MIN, <$t>::MAX, 0 as $t, 1 as $t, <$t>::MAX / 2, <$t>::MIN / 2]
            }
        }
    )*};
}
int_key!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

impl Key for u128 {
    fn from(x: u64) -> Self { ((x as u128) << 64) | x.wrapping_mul(0x9E37_79B9_7F4A_7C15) as u128 }
    fn specials() -> Vec<Self> { vec![0, 1, u128::MAX, u64::MAX as u128, 1 << 64, u128::MAX / 2] }
}

impl Key for i128 {
    fn from(x: u64) -> Self { <u128 as Key>::from(x) as i128 }
    fn specials() -> Vec<Self> {
        vec![0, -1, 1, i128::MIN, i128::MAX, i64::MIN as i128, i64::MAX as i128]
    }
}

impl Key for bool {
    fn from(x: u64) -> Self { x & 1 == 1 }
    fn specials() -> Vec<Self> { vec![false, true] }
}

impl Key for char {
    fn from(x: u64) -> Self { char::from_u32((x % 0x11_0000) as u32).unwrap_or('\u{FFFD}') }
    fn specials() -> Vec<Self> { vec!['\0', 'a', '\u{D7FF}', '\u{E000}', '\u{10FFFF}'] }
}

impl Key for f32 {
    fn from(x: u64) -> Self {
        // Small inputs (narrow, few shapes): small values; else mostly
        // ordinary values of both signs, some raw bit patterns.
        if x < 1 << 32 {
            x as f32 * 0.5
        } else if x % 16 == 0 {
            f32::from_bits((x >> 8) as u32)
        } else {
            (x as i64 >> 20) as f32 * 0.25
        }
    }
    fn specials() -> Vec<Self> {
        vec![
            f32::NAN,
            -f32::NAN,
            0.0,
            -0.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MIN,
            f32::MAX,
            f32::MIN_POSITIVE,
            -f32::MIN_POSITIVE,
            f32::from_bits(1),
        ]
    }
}

impl Key for f64 {
    fn from(x: u64) -> Self {
        if x < 1 << 32 {
            x as f64 * 0.5
        } else if x % 16 == 0 {
            f64::from_bits(x.rotate_left(17))
        } else {
            (x as i64 >> 8) as f64 * 0.25
        }
    }
    fn specials() -> Vec<Self> {
        vec![
            f64::NAN,
            -f64::NAN,
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
            f64::from_bits(1),
        ]
    }
}

/// xorshift64*: the same keys on every machine.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// The key shapes of `n` keys (`all`: every shape, else the ones that reach
/// the chains' rules beyond the cache).
fn shapes<K: Key>(n: usize, all: bool) -> Vec<(&'static str, Vec<K>)> {
    let mut r = Rng(n as u64 * 2 + 1);
    let raw: Vec<u64> = (0..n).map(|_| r.next()).collect();
    let rand: Vec<K> = raw.iter().map(|&x| K::from(x)).collect();
    let by_key = |mut v: Vec<K>| {
        v.sort_by_key(|k| k.encode());
        v
    };
    let asc = by_key(rand.clone());
    let mut out =
        vec![("random", rand.clone()), ("few", raw.iter().map(|&x| K::from(x % 5)).collect())];
    let specials = K::specials();
    let special: Vec<K> =
        raw.iter().map(|&x| specials[(x % specials.len() as u64) as usize]).collect();
    out.push(("specials", special));
    // Teeth: the sorted keys dealt into 7 ascending runs.
    let saw: Vec<K> = (0..7).flat_map(|t| asc.iter().skip(t).step_by(7).copied()).collect();
    out.push(("saw", saw));
    if !all {
        return out;
    }
    out.push(("narrow", raw.iter().map(|&x| K::from(x % 1000)).collect()));
    out.push((
        "heavy",
        raw.iter().map(|&x| if x % 10 != 0 { K::from(42) } else { K::from(x) }).collect(),
    ));
    out.push(("desc", asc.iter().rev().copied().collect()));
    let mut tail = asc.clone();
    for (i, x) in tail.iter_mut().enumerate().skip(n * 9 / 10) {
        *x = K::from(raw[i]);
    }
    out.push(("tail", tail));
    out.push(("asc", asc));
    out.push(("equal", vec![K::from(7); n]));
    out
}

/// Array sizes: around the thresholds of the chains (`ends` 64, heavy and
/// Verge 2048 / 4096, in-cache rules 32768, unstable rules 65536), then
/// around the records budgets (half of it: beyond the cache; twice: the MSD
/// limit) for records of `bytes` bytes.
fn sizes(bytes: usize) -> (Vec<usize>, Vec<usize>) {
    let quick = std::env::var("V2_QUICK").is_ok();
    let mut small: Vec<usize> =
        [0usize, 1, 2, 3, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 1000]
            .iter()
            .copied()
            .chain([2047, 2048, 2049, 4095, 4096, 4097, 32767, 32768, 32769, 65535, 65536, 65537])
            .collect();
    let records = budget(Budget::Records);
    let b = bytes.max(1);
    let mut large: Vec<usize> = [records / 2 / b, 2 * records / b]
        .iter()
        .flat_map(|&c| [c - 1, c + 1])
        .chain([4 * records / b])
        .filter(|&n| !quick || n <= 100_000)
        .collect();
    small.sort_unstable();
    large.sort_unstable();
    large.dedup();
    large.retain(|n| !small.contains(n));
    (small, large)
}

/// Checks the chains of `K` keys in records of `size_of::<K>() + N` bytes.
fn check<K: Key, const N: usize>(name: &str) {
    let bytes = std::mem::size_of::<P<K, N>>();
    assert_eq!(bytes, std::mem::size_of::<K>() + N);
    let idx_bytes = N.min(4);
    let (small, large) = sizes(bytes);
    let runs = small.iter().map(|&n| (n, true)).chain(large.iter().map(|&n| (n, false)));
    for (n, all) in runs {
        // The index fits in the payload: stability checked.
        let stable_check = idx_bytes > 0 && (idx_bytes >= 4 || n <= 1 << (8 * idx_bytes));
        for (shape, keys) in shapes::<K>(n, all) {
            let recs: Vec<P<K, N>> = keys
                .iter()
                .enumerate()
                .map(|(i, &key)| {
                    let mut pay = [0u8; N];
                    pay[..idx_bytes].copy_from_slice(&(i as u32).to_le_bytes()[..idx_bytes]);
                    P { key, pay }
                })
                .collect();
            let view = |v: &[P<K, N>]| -> Vec<(u128, [u8; N])> {
                v.iter()
                    .map(|r| {
                        let key = r.key;
                        (key.encode(), r.pay)
                    })
                    .collect()
            };
            let mut expected = view(&recs);
            expected.sort_by_key(|e| e.0);
            let what = |mode: &str| format!("{} {} B {} n {} {}", name, bytes, mode, n, shape);
            let mut a = recs.clone();
            devour_sort_by_key(&mut a, |r| r.key);
            let got = view(&a);
            if stable_check {
                assert!(got == expected, "{}", what("stable"));
            } else {
                assert!(
                    got.iter().map(|e| e.0).eq(expected.iter().map(|e| e.0)),
                    "{}",
                    what("stable")
                );
            }
            let mut b = recs;
            devour_sort_unstable_by_key(&mut b, |r| r.key);
            let mut got = view(&b);
            assert!(
                got.iter().map(|e| e.0).eq(expected.iter().map(|e| e.0)),
                "{}",
                what("unstable")
            );
            got.sort_unstable();
            let mut all_recs = expected;
            all_recs.sort_unstable();
            assert!(got == all_recs, "{}", what("unstable records"));
        }
    }
}

/// Records of `K` keys of every size class from the key's own size, and
/// between classes: (payload bytes, record size) listed per key width.
macro_rules! matrix {
    ($k:ty, $name:literal; $($n:literal),*) => {
        $(check::<$k, $n>($name);)*
    };
}

#[test]
fn test_v2_records_matrix_8_16() {
    // Sizes 1, 2, 4, 8, 12, 16, 24, 32, 40, 64, 128.
    matrix!(bool, "bool"; 0, 1, 3, 7, 11, 15, 23, 31, 39, 63, 127);
    matrix!(u8, "u8"; 0, 1, 3, 7, 11, 15, 23, 31, 39, 63, 127);
    matrix!(i8, "i8"; 0, 1, 3, 7, 11, 15, 23, 31, 39, 63, 127);
    // Sizes 2, 4, 8, 12, 16, 24, 32, 40, 64, 128.
    matrix!(u16, "u16"; 0, 2, 6, 10, 14, 22, 30, 38, 62, 126);
    matrix!(i16, "i16"; 0, 2, 6, 10, 14, 22, 30, 38, 62, 126);
}

#[test]
fn test_v2_records_matrix_32() {
    // Sizes 4, 8, 12, 16, 24, 32, 40, 64, 128.
    matrix!(u32, "u32"; 0, 4, 8, 12, 20, 28, 36, 60, 124);
    matrix!(i32, "i32"; 0, 4, 8, 12, 20, 28, 36, 60, 124);
    matrix!(f32, "f32"; 0, 4, 8, 12, 20, 28, 36, 60, 124);
    matrix!(char, "char"; 0, 4, 8, 12, 20, 28, 36, 60, 124);
}

#[test]
fn test_v2_records_matrix_64() {
    // Sizes 8, 12, 16, 24, 32, 40, 64, 128.
    matrix!(u64, "u64"; 0, 4, 8, 16, 24, 32, 56, 120);
    matrix!(i64, "i64"; 0, 4, 8, 16, 24, 32, 56, 120);
    matrix!(f64, "f64"; 0, 4, 8, 16, 24, 32, 56, 120);
    matrix!(usize, "usize"; 0, 8, 24);
    matrix!(isize, "isize"; 0, 8, 24);
}

#[test]
fn test_v2_records_matrix_128() {
    // Sizes 16, 24, 32, 40, 64, 128.
    matrix!(u128, "u128"; 0, 8, 16, 24, 48, 112);
    matrix!(i128, "i128"; 0, 8, 16, 24, 48, 112);
}

/// Floats with their special values (NaN of both signs, -0.0, +0.0,
/// infinities, MIN, MAX, subnormals) mixed at several rates into the other
/// shapes, at the thresholds of the float chains: bit for bit the
/// `total_cmp` order.
#[test]
fn test_v2_floats_specials() {
    use crate::v2::sorts::ravenous::{ravenous_sort2, RavenousSort2};
    fn run<K: Key + RavenousSort2>(name: &str) {
        let (small, _) = sizes(std::mem::size_of::<K>());
        let specials = K::specials();
        for &n in small.iter().chain([300_000, 1_000_000].iter()) {
            for (shape, keys) in shapes::<K>(n, true) {
                for rate in [1u64, 10, 50] {
                    let mut r = Rng(n as u64 + rate);
                    let mut a: Vec<K> = keys
                        .iter()
                        .map(|&k| {
                            let x = r.next();
                            if x % 100 < rate {
                                specials[(x >> 32) as usize % specials.len()]
                            } else {
                                k
                            }
                        })
                        .collect();
                    let mut e: Vec<u128> = a.iter().map(|k| k.encode()).collect();
                    e.sort_unstable();
                    ravenous_sort2(&mut a);
                    let got: Vec<u128> = a.iter().map(|k| k.encode()).collect();
                    assert!(got == e, "{} n {} {} {} %", name, n, shape, rate);
                }
            }
        }
    }
    run::<f32>("f32");
    run::<f64>("f64");
}
