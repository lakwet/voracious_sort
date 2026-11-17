//! Integration tests of the v1 public API, called as a user of the crate
//! calls it: the `RadixSort` trait on every primitive type, on slices and
//! on `Vec`, on custom structs, and the sort functions exported at the
//! crate root. Each result is checked against the std sort.
//!
//! Runs with and without `voracious_multithread` (the multithread sorts
//! are tested only with the feature). The crate's own tests (src/tests)
//! need the feature: these are the only tests of the default build.

use std::cmp::Ordering;
use std::fmt::Debug;

use voracious_radix_sort::*;

/// Sizes: empty, tiny, around the small-array fallback (128), around a
/// few chunk sizes, and large enough for the radix passes.
const SIZES: &[usize] = &[0, 1, 2, 3, 10, 127, 128, 129, 200, 1000, 4095, 4097, 20_000, 50_000];

/// xorshift64*: deterministic, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Raw 64-bit patterns for each shape: random, few distinct, all equal,
/// ascending, descending, only the extremes (all bits set or clear, and
/// their neighbours).
fn shapes(n: usize, seed: u64) -> Vec<(&'static str, Vec<u64>)> {
    let mut r = Rng(seed | 1);
    let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
    let few: Vec<u64> = (0..n).map(|_| r.next() % 7).collect();
    let ext: Vec<u64> = (0..n)
        .map(|_| match r.next() % 4 {
            0 => 0,
            1 => 1,
            2 => u64::MAX,
            _ => u64::MAX - 1,
        })
        .collect();
    let mut asc = random.clone();
    asc.sort_unstable();
    let desc: Vec<u64> = asc.iter().rev().copied().collect();
    vec![
        ("random", random),
        ("few", few),
        ("equal", vec![0x5A5A_5A5A_5A5A_5A5A; n]),
        ("asc", asc),
        ("desc", desc),
        ("extremes", ext),
    ]
}

/// Every way the trait sorts a copy of `a`: slice and `Vec`, unstable and
/// stable, and the multithread sort with the feature.
fn trait_sorts<T, K>(a: &[T]) -> Vec<(&'static str, Vec<T>)>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let mut out = Vec::new();
    let mut v = a.to_vec();
    v[..].voracious_sort();
    out.push(("slice voracious_sort", v));
    let mut v = a.to_vec();
    v[..].voracious_stable_sort();
    out.push(("slice voracious_stable_sort", v));
    let mut v = a.to_vec();
    v.voracious_sort();
    out.push(("Vec voracious_sort", v));
    let mut v = a.to_vec();
    v.voracious_stable_sort();
    out.push(("Vec voracious_stable_sort", v));
    #[cfg(feature = "voracious_multithread")]
    for &threads in [1usize, 4].iter() {
        let mut v = a.to_vec();
        v.voracious_mt_sort(threads);
        out.push(("voracious_mt_sort", v));
    }
    out
}

/// Integers, `char`, `bool`: the result must equal the std sort.
fn check_total<T, K>(name: &str, a: &[T])
where
    T: Radixable<K> + Ord + Debug,
    K: RadixKey,
{
    let mut expected = a.to_vec();
    expected.sort();
    for (how, got) in trait_sorts(a) {
        assert!(got == expected, "{} {} n={}: wrong order", name, how, a.len());
    }
}

macro_rules! int_test {
    ($test:ident, $($t:ty),*) => {
        #[test]
        fn $test() {
            $(
                for &n in SIZES.iter() {
                    for (shape, raw) in shapes(n, n as u64 + 11) {
                        let a: Vec<$t> = raw.iter().map(|&x| x as $t).collect();
                        check_total(&format!("{} {}", stringify!($t), shape), &a);
                    }
                }
                // MIN and MAX mixed with values close to them.
                let mut r = Rng(99);
                for &n in [129usize, 1000, 20_000].iter() {
                    let a: Vec<$t> = (0..n)
                        .map(|_| {
                            let d = (r.next() % 300) as $t;
                            if r.next() % 2 == 0 {
                                <$t>::MIN.wrapping_add(d)
                            } else {
                                <$t>::MAX.wrapping_sub(d)
                            }
                        })
                        .collect();
                    check_total(&format!("{} min/max", stringify!($t)), &a);
                }
            )*
        }
    };
}

int_test!(trait_unsigned, u8, u16, u32, u64, u128, usize);
int_test!(trait_signed, i8, i16, i32, i64, i128, isize);

#[test]
fn trait_char_bool() {
    for &n in SIZES.iter() {
        for (shape, raw) in shapes(n, n as u64 + 5) {
            // Every code point, surrogates skipped.
            let a: Vec<char> = raw
                .iter()
                .map(|&x| std::char::from_u32((x % 0x11_0000) as u32).unwrap_or('\u{FFFD}'))
                .collect();
            check_total(&format!("char {}", shape), &a);
            let b: Vec<bool> = raw.iter().map(|&x| x & 1 == 1).collect();
            check_total(&format!("bool {}", shape), &b);
        }
    }
}

/// Floats: no total order in `PartialOrd`. The result must hold the same
/// values (bit for bit), the negative NaNs first, the positive NaNs last,
/// and the rest in order (-0.0 and +0.0 compare equal: either order).
macro_rules! float_test {
    ($test:ident, $t:ty, $bits:ty) => {
        #[test]
        fn $test() {
            let specials: [$t; 12] = [
                <$t>::NAN,
                -<$t>::NAN,
                0.0,
                -0.0,
                <$t>::INFINITY,
                <$t>::NEG_INFINITY,
                <$t>::MIN,
                <$t>::MAX,
                <$t>::MIN_POSITIVE,
                -<$t>::MIN_POSITIVE,
                <$t>::EPSILON,
                <$t>::from_bits(1), // smallest subnormal
            ];
            let mut r = Rng(7);
            for &n in SIZES.iter() {
                for &rate in [0u64, 1, 10, 50].iter() {
                    let a: Vec<$t> = (0..n)
                        .map(|_| {
                            if r.next() % 100 < rate {
                                specials[(r.next() % 12) as usize]
                            } else {
                                // Finite values: a random float would be NaN
                                // or infinite now and then.
                                ((r.next() as i32) as $t) / 7.0
                            }
                        })
                        .collect();
                    for (how, got) in trait_sorts(&a) {
                        let what = format!("{} {} n={} specials {}%", stringify!($t), how, n, rate);
                        check_float(&what, &a, &got);
                    }
                }
            }
        }
    };
}

fn check_float<T: Copy + PartialOrd + Debug + FloatBits>(what: &str, a: &[T], got: &[T]) {
    let mut x: Vec<u64> = a.iter().map(|v| v.bits()).collect();
    let mut y: Vec<u64> = got.iter().map(|v| v.bits()).collect();
    x.sort_unstable();
    y.sort_unstable();
    assert!(x == y, "{}: values lost or changed", what);
    let neg = got.iter().take_while(|v| v.is_nan() && v.negative()).count();
    let pos = got.iter().rev().take_while(|v| v.is_nan() && !v.negative()).count();
    let nans = a.iter().filter(|v| v.is_nan()).count();
    assert_eq!(neg + pos, nans, "{}: NaN not at the ends", what);
    let rest = &got[neg..got.len() - pos];
    for w in rest.windows(2) {
        assert!(
            w[0].partial_cmp(&w[1]) != Some(Ordering::Greater),
            "{}: {:?} before {:?}",
            what,
            w[0],
            w[1]
        );
    }
}

trait FloatBits {
    fn bits(&self) -> u64;
    fn is_nan(&self) -> bool;
    fn negative(&self) -> bool;
}

impl FloatBits for f32 {
    fn bits(&self) -> u64 { self.to_bits() as u64 }
    fn is_nan(&self) -> bool { f32::is_nan(*self) }
    fn negative(&self) -> bool { self.is_sign_negative() }
}

impl FloatBits for f64 {
    fn bits(&self) -> u64 { self.to_bits() }
    fn is_nan(&self) -> bool { f64::is_nan(*self) }
    fn negative(&self) -> bool { self.is_sign_negative() }
}

float_test!(trait_f32, f32, u32);
float_test!(trait_f64, f64, u64);

/// A custom struct sorted by a `u32` key, with its position as payload:
/// the stable sort keeps equal keys in their first order.
#[derive(Copy, Clone, Debug)]
struct Record {
    key: u32,
    pos: usize,
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<Ordering> { self.key.partial_cmp(&other.key) }
}

impl PartialEq for Record {
    fn eq(&self, other: &Record) -> bool { self.key == other.key }
}

impl Radixable<u32> for Record {
    type Key = u32;
    fn key(&self) -> u32 { self.key }
}

/// The struct of the crate documentation: an `f32` key.
#[derive(Copy, Clone, Debug)]
struct Custom {
    value: f32,
    other: usize,
}

impl PartialOrd for Custom {
    fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl PartialEq for Custom {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

impl Radixable<f32> for Custom {
    type Key = f32;
    fn key(&self) -> f32 { self.value }
}

#[test]
fn trait_custom_structs() {
    for &n in SIZES.iter() {
        for (shape, raw) in shapes(n, n as u64 + 3) {
            let a: Vec<Record> = raw
                .iter()
                .enumerate()
                .map(|(pos, &x)| Record { key: (x % 1000) as u32, pos })
                .collect();
            let mut expected = a.clone();
            expected.sort_by_key(|r| (r.key, r.pos));
            let mut stable = a.clone();
            stable.voracious_stable_sort();
            let same =
                stable.iter().zip(expected.iter()).all(|(x, y)| x.key == y.key && x.pos == y.pos);
            assert!(same, "Record {} n={}: not sorted stably", shape, n);
            for (how, got) in trait_sorts(&a) {
                let mut g: Vec<(u32, usize)> = got.iter().map(|r| (r.key, r.pos)).collect();
                assert!(g.windows(2).all(|w| w[0].0 <= w[1].0), "Record {} {} n={}", shape, how, n);
                g.sort_unstable();
                let e: Vec<(u32, usize)> = expected.iter().map(|r| (r.key, r.pos)).collect();
                assert!(g == e, "Record {} {} n={}: records lost", shape, how, n);
            }

            let c: Vec<Custom> = raw
                .iter()
                .enumerate()
                .map(|(other, &x)| Custom { value: ((x as i32) as f32) / 3.0, other })
                .collect();
            for (how, got) in trait_sorts(&c) {
                assert!(
                    got.windows(2).all(|w| w[0].value <= w[1].value),
                    "Custom {} {} n={}",
                    shape,
                    how,
                    n
                );
                let mut g: Vec<usize> = got.iter().map(|x| x.other).collect();
                g.sort_unstable();
                assert!(
                    g.iter().copied().eq(0..n),
                    "Custom {} {} n={}: records lost",
                    shape,
                    how,
                    n
                );
            }
        }
    }
}

/// The sort functions exported at the crate root, radix 8, on the types
/// they are meant for. Floats are finite here: only the trait and a few
/// functions set NaN aside.
macro_rules! check_fn {
    ($name:expr, $sort:expr, $t:ty, $conv:expr, $sizes:expr) => {
        for &n in $sizes.iter() {
            for (shape, raw) in shapes(n, n as u64 + 17) {
                let a: Vec<$t> = raw.iter().map($conv).collect();
                let mut expected = a.clone();
                expected.sort_by(|x, y| x.partial_cmp(y).unwrap());
                let mut got = a.clone();
                $sort(&mut got[..]);
                assert!(got == expected, "{} {} {} n={}", $name, stringify!($t), shape, n);
            }
        }
    };
}

macro_rules! check_generic_fn {
    ($name:expr, $f:ident) => {
        check_fn!($name, |v: &mut [u32]| $f(v, 8), u32, |&x| x as u32, SIZES);
        check_fn!($name, |v: &mut [u64]| $f(v, 8), u64, |&x| x, SIZES);
        check_fn!($name, |v: &mut [i32]| $f(v, 8), i32, |&x| x as i32, SIZES);
        check_fn!($name, |v: &mut [i64]| $f(v, 8), i64, |&x| x as i64, SIZES);
        check_fn!($name, |v: &mut [f32]| $f(v, 8), f32, |&x| ((x as i32) as f32) / 7.0, SIZES);
        check_fn!($name, |v: &mut [f64]| $f(v, 8), f64, |&x| ((x as i64) as f64) / 7.0, SIZES);
    };
}

#[test]
fn functions_generic() {
    check_generic_fn!("american_flag_sort", american_flag_sort);
    check_generic_fn!("awardofthesky_sort", awardofthesky_sort);
    check_generic_fn!("dlsd_radixsort", dlsd_radixsort);
    check_generic_fn!("lsd_radixsort", lsd_radixsort);
    check_generic_fn!("lsd_stable_radixsort", lsd_stable_radixsort);
    check_generic_fn!("msd_radixsort", msd_radixsort);
    check_generic_fn!("msd_stable_radixsort", msd_stable_radixsort);
    check_generic_fn!("rollercoaster_sort", rollercoaster_sort);
    check_generic_fn!("ska_sort", ska_sort);
    check_generic_fn!("voracious_sort", voracious_sort);
}

#[test]
fn functions_dedicated() {
    // Counting sort: one pass on the low `radix` bits, so the radix is the
    // key width.
    check_fn!("counting_sort", |v: &mut [u8]| counting_sort(v, 8), u8, |&x| x as u8, SIZES);
    check_fn!("counting_sort", |v: &mut [u16]| counting_sort(v, 16), u16, |&x| x as u16, SIZES);
    check_fn!("cs_u16", cs_u16, u16, |&x| x as u16, SIZES);
    check_fn!("lsd_u32", lsd_u32, u32, |&x| x as u32, SIZES);
    check_fn!("lsd_f32", lsd_f32, f32, |&x| ((x as i32) as f32) / 7.0, SIZES);
    check_fn!("boolean_sort", boolean_sort, bool, |&x| x & 1 == 1, SIZES);
    // Insertion sort: quadratic, small arrays only.
    check_fn!("insertion_sort", insertion_sort, u32, |&x| x as u32, [0usize, 1, 2, 10, 100, 500]);
}

#[cfg(feature = "voracious_multithread")]
#[test]
fn functions_peeka() {
    for &blocks in [5000usize, 150_000].iter() {
        for &threads in [1usize, 4].iter() {
            check_fn!(
                "peeka_sort",
                |v: &mut [u64]| peeka_sort(v, 8, blocks, threads),
                u64,
                |&x| x,
                SIZES
            );
            check_fn!(
                "peeka_sort",
                |v: &mut [i32]| peeka_sort(v, 8, blocks, threads),
                i32,
                |&x| x as i32,
                SIZES
            );
            let f = |&x: &u64| ((x as i64) as f64) / 7.0;
            check_fn!(
                "peeka_sort",
                |v: &mut [f64]| peeka_sort(v, 8, blocks, threads),
                f64,
                f,
                SIZES
            );
        }
    }
}
