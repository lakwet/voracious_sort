//! Unit tests of `v2/engines/counting.rs` (a child module: sees its private items).

use super::*;
use crate::v2::devour::keys::{flip16, flip32, flip64, flip8, id16, id32, id64, id8};
use crate::v2::testing::*;

/// The detectors count from `FEW_MIN` keys on: shorter arrays never
/// reach these sorts.
const MIN: usize = crate::v2::special::FEW_MIN;

/// `domain_count` on 8- and 16-bit keys, unsigned and signed codecs.
macro_rules! domain {
    ($u:ty, $enc:ident) => {
        for_each_case::<$u>(|shape, a| {
            if a.len() < MIN {
                return;
            }
            let mut got = a.clone();
            domain_count(&mut got, $enc, $enc);
            let what =
                format!("domain {} {} {} n={}", stringify!($u), stringify!($enc), shape, a.len());
            assert!(sorted_by(&got, $enc), "{}: not sorted", what);
            assert!(same_values(&got, &a), "{}: values changed", what);
        })
    };
}

#[test]
fn domain() {
    domain!(u8, id8);
    domain!(u8, flip8);
    domain!(u16, id16);
    domain!(u16, flip16);
}

/// `exact_count` over the exact range of the keys (narrow ones), and
/// `window_count` from a sample range: sorted when it returns `true`,
/// untouched otherwise; narrow keys always counted.
macro_rules! ranges {
    ($u:ty, $enc:ident) => {
        for_each_case::<$u>(|shape, a| {
            if a.len() < MIN {
                return;
            }
            let what = format!("{} {} {} n={}", stringify!($u), stringify!($enc), shape, a.len());
            let lo = a.iter().map(|&k| $enc(k)).min().unwrap();
            let hi = a.iter().map(|&k| $enc(k)).max().unwrap();
            let width = (hi - lo) as u128 + 1;
            if width <= 1 << 16 {
                let mut got = a.clone();
                exact_count(&mut got, lo, width as usize, $enc, $enc);
                assert!(sorted_by(&got, $enc), "exact {}: not sorted", what);
                assert!(same_values(&got, &a), "exact {}: values changed", what);
            }
            // The window starts on the range of the first and middle keys,
            // when narrow against the limit (the callers' condition).
            let (x, y) = (a[0], a[a.len() / 2]);
            let (s, t) = if $enc(x) <= $enc(y) { (x, y) } else { (y, x) };
            if 2 * (($enc(t) - $enc(s)) as u128 + 1) >= 1 << 16 {
                return;
            }
            let mut got = a.clone();
            if window_count(&mut got, s, t, 1 << 16, $enc, $enc) {
                assert!(sorted_by(&got, $enc), "window {}: not sorted", what);
                assert!(same_values(&got, &a), "window {}: values changed", what);
            } else {
                assert!(got == a, "window {}: gave up but changed", what);
                assert!(width > 1 << 16, "window {}: narrow keys not counted", what);
            }
        })
    };
}

#[test]
fn exact_and_window() {
    ranges!(u32, id32);
    ranges!(u32, flip32);
    ranges!(u64, id64);
    ranges!(u64, flip64);
}
