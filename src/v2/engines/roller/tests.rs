//! Unit tests of `v2/engines/roller.rs` (a child module: sees its private items).

use crate::v2::testing::*;

/// Every module: `roller` sorts raw bits in the codec's order, below
/// `LEAF_KEYS` (one leaf) and beyond (MSD passes, then the leaves).
macro_rules! check {
    ($m:ident, $u:ty, $key:expr) => {{
        use super::$m as r;
        let key: fn($u) -> $u = $key;
        for_each_case::<$u>(|shape, a| {
            let mut got = a.clone();
            r::roller(&mut got);
            let what = format!("{} {} n={}", stringify!($m), shape, a.len());
            assert!(sorted_by(&got, key), "{}: not sorted", what);
            assert!(same_values(&got, &a), "{}: values changed", what);
        });
    }};
}

#[test]
fn floats() {
    check!(r32, u32, float_key_32);
    check!(r64, u64, float_key_64);
}

#[test]
fn unsigned() {
    check!(ru32, u32, |b| b);
    check!(ru64, u64, |b| b);
    check!(ru128, u128, |b| b);
}

#[test]
fn signed() {
    check!(ri32, u32, |b| b ^ 1 << 31);
    check!(ri64, u64, |b| b ^ 1 << 63);
    check!(ri128, u128, |b| b ^ 1 << 127);
}
