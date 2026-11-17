//! Detectors: distributions sorted faster than by a radix sort (narrow
//! ranges counted, few distinct values, long runs of equal keys, heavy
//! hitters, dense ranges with outliers), and the cluster rule choosing the
//! engine. Each one checks a sample first, then verifies before writing.

#[macro_use]
mod dense;
#[macro_use]
mod few;
#[macro_use]
mod range;

pub(crate) mod floats;
pub(crate) mod heavy;
pub(crate) mod plateaus;

/// Spread sample shared by the integer detectors.
pub(crate) const SAMPLE: usize = 64;
/// From this size the few-distinct-values counting and the ranges are tried.
pub(crate) const FEW_MIN: usize = 129;
/// Narrow-range counting: at most this many integers spanned...
pub(crate) const RANGE_MAX: usize = 1 << 16;
/// ...and at most one counter per this many keys (the table is zeroed,
/// scanned and written on top of the keys; u16 random: 21 % slower than the
/// LSD at 1.5 keys per counter, 24 % faster at 3).
pub(crate) const RANGE_KEYS_PER_COUNTER: usize = 2;
/// Wide-range counting (counters within half the L2): at least this many
/// keys per counter (char over its 21 bits: count / LSD 1.11 at 2.7 keys
/// per counter, 0.59 at 5.5).
pub(crate) const WIDE_KEYS_PER_COUNTER: usize = 4;
/// Offset sort: bytes saved at least.
pub(crate) const OFFSET_BYTES: u32 = 1;
/// Shortest array tried for a dense range with outliers or heavy hitters.
pub(crate) const DENSE_MIN: usize = 4096;
/// Most distinct values counted, and the hash table size (at most half full).
pub(crate) const FEW_MAX: usize = 256;
pub(crate) const FEW_TABLE_BITS: u32 = 9;
pub(crate) const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;
/// Fewest keys for the 4-times larger few table (many values in the sample).
pub(crate) const FEW_BIG_MIN: usize = 16384;

/// Spread sample of `SAMPLE` keys at the positions frac(i x golden ratio) x n.
#[inline]
pub(crate) fn spread<T: Copy>(arr: &[T]) -> [T; SAMPLE] {
    let n = arr.len() as u64;
    let mut sample = [arr[0]; SAMPLE];
    for (i, slot) in sample.iter_mut().enumerate() {
        *slot = arr[((((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n) >> 32) as usize];
    }
    sample
}

/// The integer detectors of one key type (`range`, `dense`, `few`), in
/// module `$m`. `$encode` / `$decode`: the key map of `$t` as `$uint`;
/// `$roller_min`: the rollercoaster size (the offset sort stays below).
macro_rules! int_detectors {
    ($m:ident, $t:ty, $uint:ty, $encode:expr, $decode:expr, $roller_min:path) => {
        pub(crate) mod $m {
            use super::*;
            use crate::v2::cache::{budget, Budget};
            use crate::v2::engines::counting::{
                domain_count, exact_count, window_count, window_count_split,
            };
            use crate::v2::special::*;

            int_range!($t, $uint, $encode, $decode, $roller_min);
            int_dense!($t, $uint, $encode, $decode);
            int_few!($t, $uint, $encode, $decode);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    /// The integer detectors of each type (`range`, `few`, `dense`): when
    /// they answer `true` the array is sorted, else it holds the same
    /// values (`range` and `few` leave it untouched). On the shapes they
    /// exist for (narrow, few) they must answer `true`.
    macro_rules! detectors {
        ($t:ident, $u:ty, $strict:expr) => {{
            use crate::v2::types::$t::detectors as d;
            for_each_case::<$u>(|shape, raw| {
                let a: Vec<$t> = raw.iter().map(|&x| x as $t).collect();
                if a.is_empty() {
                    return;
                }
                let what = format!("{} {} n={}", stringify!($t), shape, a.len());
                // The narrow shape built from the low bits only.
                let narrow = shape == "narrow"
                    && a.len() >= 2000
                    && raw.iter().all(|&x| (x as u64) < 1 << 32);
                let sample = spread(&a);
                let mut got = a.clone();
                if d::range(&mut got, &sample) {
                    assert!(sorted_by(&got, |x| x), "range {}: not sorted", what);
                    assert!(same_values(&got, &a), "range {}: values changed", what);
                } else {
                    assert!(got == a, "range {}: gave up but changed", what);
                    assert!(!($strict && narrow), "range {}: narrow keys missed", what);
                }
                let mut got = a.clone();
                if d::few(&mut got, &sample) {
                    assert!(sorted_by(&got, |x| x), "few {}: not sorted", what);
                    assert!(same_values(&got, &a), "few {}: values changed", what);
                } else {
                    assert!(got == a, "few {}: gave up but changed", what);
                    let few = shape == "few" && a.len() >= FEW_MIN;
                    assert!(!($strict && few), "few {}: few values missed", what);
                }
                let mut got = a.clone();
                if d::dense(&mut got, &sample) {
                    assert!(sorted_by(&got, |x| x), "dense {}: not sorted", what);
                }
                assert!(same_values(&got, &a), "dense {}: values changed", what);
            });
        }};
    }

    #[test]
    fn integer_detectors() {
        detectors!(u8, u8, false);
        detectors!(i8, u8, false);
        detectors!(u16, u16, false);
        detectors!(i16, u16, false);
        detectors!(u32, u32, true);
        detectors!(i32, u32, true);
        detectors!(u64, u64, true);
        detectors!(i64, u64, true);
    }

    /// Long runs of equal keys are rewritten in order; anything else is
    /// left as it was.
    #[test]
    fn plateaus() {
        use crate::v2::inspect::presorted::probe_pairs;
        let mut r = Rng(29);
        for &n in [1024usize, 5000, 20_000].iter() {
            for &run in [1usize, 50, 500].iter() {
                let a: Vec<u32> =
                    (0..n).map(|i| ((i / run) as u64 * 0x9E37_79B9 % 1000) as u32).collect();
                let mut got = a.clone();
                let equal = probe_pairs(&got).equal();
                if plateaus::plateaus(&mut got, equal) {
                    assert!(sorted_by(&got, |x| x), "n={} run={}: not sorted", n, run);
                    assert!(same_values(&got, &a), "n={} run={}: values changed", n, run);
                } else {
                    assert!(got == a, "n={} run={}: gave up but changed", n, run);
                    assert!(run < 500, "n={} run={}: plateaus missed", n, run);
                }
            }
            let a: Vec<u32> = (0..n).map(|_| r.next() as u32).collect();
            assert!(!plateaus::plateaus(&mut a.clone(), 0));
        }
    }

    /// Heavy hitters: half the keys on one value, a fifth on another, the
    /// rest random: extracted, then the rest sorted. Random keys: the
    /// filter says no and the extraction gives up, values kept.
    #[test]
    fn heavy() {
        use heavy::*;
        let mut r = Rng(31);
        let eq = |a: u64, b: u64| a == b;
        let lt = |a: u64, b: u64| a < b;
        let sort = |a: &mut [u64]| a.sort_unstable();
        for &n in [HEAVY_MIN, 10_000, 100_000].iter() {
            let hit: Vec<u64> = (0..n)
                .map(|_| match r.next() % 10 {
                    0..=4 => 42,
                    5 | 6 => 7,
                    _ => r.next(),
                })
                .collect();
            let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
            for (name, a) in [("heavy", hit), ("random", random)].iter() {
                let mut sample = spread(a);
                sample.sort_unstable();
                let mut got = a.clone();
                let done = heavy_sorted_by(&mut got, &sample, HEAVY_MIN, &sort, eq, lt);
                if done {
                    assert!(sorted_by(&got, |x| x), "{} n={}: not sorted", name, n);
                }
                assert!(same_values(&got, a), "{} n={}: values changed", name, n);
                assert_eq!(done, *name == "heavy", "{} n={}", name, n);
                assert_eq!(may_be_heavy(&sample, eq), *name == "heavy", "{} n={}: filter", name, n);
            }
        }
        // One dominant value among spread ones; not when another value holds
        // an eighth of the sample (here on the 8 picks, every 8th key).
        let mut s = [0u64; SAMPLE];
        for (i, v) in s.iter_mut().enumerate() {
            *v = if i % 3 == 0 { i as u64 + 100 } else { 5 };
        }
        assert!(dominant(&s, eq));
        let mut few = [0u64; SAMPLE];
        for (i, v) in few.iter_mut().enumerate() {
            *v = (i % 8 == 0) as u64;
        }
        assert!(!dominant(&few, eq));
    }

    /// Float detectors: constant arrays and integer-valued floats (a range
    /// of 50: at least 2 keys per counter from 129 keys) are sorted by
    /// `detect`, few distinct values by `few`; when they answer
    /// `true` the array is in the total order (NaN, -0.0 included), else
    /// it is untouched.
    macro_rules! floats {
        ($f:ty, $detect:path, $few:path, $key:expr) => {{
            let key = $key;
            let mut r = Rng(37);
            let specials = [<$f>::NAN, -<$f>::NAN, 0.0, -0.0, <$f>::INFINITY, <$f>::NEG_INFINITY];
            for &n in [129usize, 1000, 20_000].iter() {
                let cases: Vec<(&str, Vec<$f>, bool, bool)> = vec![
                    ("constant", vec![2.5; n], true, true),
                    (
                        "integers",
                        (0..n).map(|_| (r.next() % 50) as $f - 25.0).collect(),
                        true,
                        false,
                    ),
                    (
                        "few",
                        (0..n).map(|_| [1.5, -3.0, 1e9][(r.next() % 3) as usize]).collect(),
                        false,
                        true,
                    ),
                    (
                        "specials",
                        (0..n).map(|_| specials[(r.next() % 6) as usize]).collect(),
                        false,
                        true,
                    ),
                    (
                        "random",
                        (0..n).map(|_| <$f>::from_bits(r.next() as _)).collect(),
                        false,
                        false,
                    ),
                ];
                for (name, a, by_detect, by_few) in cases {
                    for &(which, found) in [("detect", by_detect), ("few", by_few)].iter() {
                        let mut got = a.clone();
                        let done = if which == "detect" {
                            $detect(&mut got)
                        } else {
                            $few(&mut got, None)
                        };
                        let what = format!("{} {} {} n={}", stringify!($f), which, name, n);
                        if done {
                            assert!(
                                got.windows(2).all(|w| key(w[0]) <= key(w[1])),
                                "{}: not sorted",
                                what
                            );
                        } else {
                            let same = got.iter().zip(&a).all(|(x, y)| x.to_bits() == y.to_bits());
                            assert!(same, "{}: gave up but changed", what);
                        }
                        assert!(done || !found, "{}: missed", what);
                    }
                }
            }
        }};
    }

    #[test]
    fn float_detectors() {
        floats!(f32, floats::detect32, floats::few32, |x: f32| float_key_32(x.to_bits()));
        floats!(f64, floats::detect64, floats::few64, |x: f64| float_key_64(x.to_bits()));
    }
}
