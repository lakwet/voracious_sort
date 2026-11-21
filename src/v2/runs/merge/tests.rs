//! Unit tests of `v2/runs/merge/mod.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// A key and its first position: compared by the key only, so that the
/// stability shows.
#[derive(Clone, Copy, Debug)]
struct Rec {
    key: u32,
    pos: u32,
}

impl PartialEq for Rec {
    fn eq(&self, o: &Rec) -> bool { self.key == o.key }
}

impl PartialOrd for Rec {
    fn partial_cmp(&self, o: &Rec) -> Option<std::cmp::Ordering> { self.key.partial_cmp(&o.key) }
}

/// Cuts `0..n` into `runs` segments at random places (empty ones kept
/// out): the separators.
fn cuts(n: usize, runs: usize, r: &mut Rng) -> Vec<usize> {
    let mut s: Vec<usize> = (1..runs).map(|_| (r.next() % (n as u64 + 1)) as usize).collect();
    s.push(0);
    s.push(n);
    s.sort_unstable();
    s.dedup();
    s
}

/// Runs (sorted stably), merged: the stable sort of the whole, at every
/// size (in place and ping-pong) and number of runs (2 to 64).
#[test]
fn stable_merges() {
    let mut r = Rng(11);
    for &n in SIZES.iter().chain(LARGE.iter()) {
        for &runs in [2usize, 3, 4, 5, 8, 64].iter() {
            for &distinct in [4u64, 1 << 32].iter() {
                let mut a: Vec<Rec> = (0..n as u32)
                    .map(|pos| Rec { key: (r.next() % distinct) as u32, pos })
                    .collect();
                let mut separators = cuts(n, runs, &mut r);
                for w in separators.windows(2) {
                    a[w[0]..w[1]].sort_by_key(|x| x.key);
                }
                let mut expected = a.clone();
                expected.sort_by_key(|x| x.key);
                merge_runs(&mut a, &mut separators);
                let same = a.iter().zip(&expected).all(|(x, y)| x.key == y.key && x.pos == y.pos);
                assert!(same, "n={} runs={} distinct={}: not the stable order", n, runs, distinct);
            }
        }
    }
}

/// The SIMD merges of every key type they take: sorted runs merged into
/// the sorted array (floats in their total order, without NaN: the
/// float chains set them aside first, and the f64 keys take the scalar
/// merge, which compares with `<`).
macro_rules! simd_case {
    ($t:ty, $from:expr, $key:expr) => {{
        let mut r = Rng(13);
        for &n in SIZES.iter().chain(LARGE.iter()) {
            for &runs in [2usize, 3, 7, 64].iter() {
                let mut a: Vec<$t> = (0..n).map(|_| $from(r.next())).collect();
                let mut separators = cuts(n, runs, &mut r);
                for w in separators.windows(2) {
                    a[w[0]..w[1]].sort_unstable_by_key(|&x| $key(x));
                }
                let mut expected = a.clone();
                expected.sort_unstable_by_key(|&x| $key(x));
                merge_runs_simd(&mut a, &mut separators);
                let same = a.iter().zip(&expected).all(|(&x, &y)| $key(x) == $key(y));
                assert!(same, "{} n={} runs={}", stringify!($t), n, runs);
            }
        }
    }};
}

#[test]
fn simd_merges() {
    simd_case!(u32, |x| x as u32, |x: u32| x);
    simd_case!(i32, |x| x as i32, |x: i32| x);
    let f32_of = |x: u64| Some(f32::from_bits(x as u32)).filter(|v| !v.is_nan()).unwrap_or(1.5);
    let f64_of = |x: u64| Some(f64::from_bits(x)).filter(|v| !v.is_nan()).unwrap_or(1.5);
    simd_case!(f32, f32_of, |x: f32| float_key_32(x.to_bits()));
    simd_case!(u64, |x| x, |x: u64| x);
    simd_case!(i64, |x| x as i64, |x: i64| x);
    simd_case!(f64, f64_of, |x: f64| float_key_64(x.to_bits()));
}

/// The merge probes on their three cases: runs one after the other
/// (predictable, not dense), runs strictly alternating (periodic:
/// predictable and dense), random runs (unpredictable).
#[test]
fn merge_probes() {
    use predictable::{dense_merge, predictable_merge, predictable_pair};
    let n = 1000u32;
    let apart: Vec<u32> = (0..n).collect();
    let alternate: Vec<u32> =
        (0..n / 2).map(|i| 2 * i).chain((0..n / 2).map(|i| 2 * i + 1)).collect();
    let mut r = Rng(17);
    let mut random: Vec<u32> = (0..n).map(|_| r.next() as u32).collect();
    let mid = (n / 2) as usize;
    random[..mid].sort_unstable();
    random[mid..].sort_unstable();
    assert!(predictable_merge(&apart, mid) && !dense_merge(&apart, mid));
    assert!(predictable_merge(&alternate, mid) && dense_merge(&alternate, mid));
    assert!(!predictable_merge(&random, mid));
    assert!(predictable_pair(&apart[..mid], &apart[mid..]));
    assert!(!predictable_pair(&random[..mid], &random[mid..]));
}
