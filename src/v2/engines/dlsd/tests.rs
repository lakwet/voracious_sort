//! Unit tests of `v2/engines/dlsd/mod.rs` (a child module: sees its private items).

use std::mem::MaybeUninit;

use crate::v2::testing::*;

/// Every module: `dlsd` sorts raw bits in the codec's order;
/// `dlsd_encoded` gives the same result from encoded keys (scratch
/// reused); `dlsd_unless` leaves the array untouched when its sample is
/// rejected and sorts it otherwise.
macro_rules! check {
    ($m:ident, $w:ident, $u:ty, $key:expr) => {{
        use super::super::lsd::$w as w;
        use super::$m as d;
        let key: fn($u) -> $u = $key;
        let mut scratch: Vec<MaybeUninit<$u>> = Vec::new();
        for_each_case::<$u>(|shape, a| {
            let what = format!("{} {} n={}", stringify!($m), shape, a.len());
            let mut got = a.clone();
            d::dlsd(&mut got);
            assert!(sorted_by(&got, key), "{}: not sorted", what);
            assert!(same_values(&got, &a), "{}: values changed", what);
            let mut keys: Vec<$u> = a.iter().map(|&b| w::encode_key(b)).collect();
            d::dlsd_encoded(&mut keys, &mut scratch);
            assert!(keys == got, "{}: dlsd_encoded differs", what);
            let mut kept = a.clone();
            if !d::dlsd_unless(&mut kept, &mut scratch, |_| true) {
                assert!(kept == a, "{}: rejected but changed", what);
            }
            let mut sorted = a.clone();
            assert!(d::dlsd_unless(&mut sorted, &mut scratch, |_| false));
            assert!(sorted == got, "{}: dlsd_unless differs", what);
        });
    }};
}

#[test]
fn floats() {
    check!(d32, w32, u32, float_key_32);
    check!(d64, w64, u64, float_key_64);
}

#[test]
fn unsigned() {
    check!(du32, wu32, u32, |b| b);
    check!(du64, wu64, u64, |b| b);
    check!(du128, wu128, u128, |b| b);
}

#[test]
fn signed() {
    check!(di32, wi32, u32, |b| b ^ 1 << 31);
    check!(di64, wi64, u64, |b| b ^ 1 << 63);
    check!(di128, wi128, u128, |b| b ^ 1 << 127);
}
