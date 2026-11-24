//! Unit tests of `v2/sorts/devour/wide128.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Narrowing and the few-values count of 128-bit keys: sorted when they
/// answer `true`, untouched otherwise. Few values with none over a
/// quarter of the sample (else left to the heavy hitters) are taken.
macro_rules! check {
    ($t:ty) => {
        for_each_case::<u128>(|shape, raw| {
            let a: Vec<$t> = raw.iter().map(|&x| x as $t).collect();
            if a.len() < 2 {
                return;
            }
            let what = format!("{} {} n={}", stringify!($t), shape, a.len());
            let mut got = a.clone();
            if narrow(&mut got) {
                assert!(sorted_by(&got, |x| x) && same_values(&got, &a), "narrow {}", what);
            } else {
                assert!(got == a, "narrow {}: gave up but changed", what);
            }
            let mut got = a.clone();
            if few(&mut got, None) {
                assert!(sorted_by(&got, |x| x) && same_values(&got, &a), "few {}", what);
            } else {
                assert!(got == a, "few {}: gave up but changed", what);
            }
        });
        let mut r = Rng(67);
        // From 8192 keys the sample (n / 32, at most 256) holds twice the values.
        for &n in [8192usize, 100_000].iter() {
            let a: Vec<$t> = (0..n).map(|_| ((r.next() % 100) as $t) << 70).collect();
            let mut got = a.clone();
            assert!(few(&mut got, None), "few {} n={}: 100 values missed", stringify!($t), n);
            assert!(sorted_by(&got, |x| x) && same_values(&got, &a));
        }
    };
}

#[test]
fn narrow_and_few() {
    check!(u128);
    check!(i128);
}
