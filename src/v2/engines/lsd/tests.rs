//! Unit tests of `v2/engines/lsd/mod.rs` (a child module: sees its private items).

use std::mem::MaybeUninit;

use crate::v2::testing::*;

/// Every module: `lsd` sorts raw bits in the codec's order, the codec
/// round-trips, and `lsd_encoded` sorts encoded keys into the same
/// result (twice with the same buffer: a reused buffer is not trusted).
macro_rules! check {
    ($m:ident, $u:ty, $key:expr) => {{
        use super::$m as w;
        let key: fn($u) -> $u = $key;
        for_each_case::<$u>(|shape, a| {
            let mut got = a.clone();
            w::lsd(&mut got);
            let what = format!("{} {} n={}", stringify!($m), shape, a.len());
            assert!(sorted_by(&got, key), "{}: not sorted", what);
            assert!(same_values(&got, &a), "{}: values changed", what);
            let mut buffer: Vec<MaybeUninit<$u>> = Vec::new();
            for _ in 0..2 {
                let mut keys: Vec<$u> = a.iter().map(|&b| w::encode_key(b)).collect();
                assert!(keys.iter().zip(&a).all(|(&k, &b)| w::decode_key(k) == b));
                w::lsd_encoded(&mut keys, &mut buffer);
                assert!(keys == got, "{}: lsd_encoded differs", what);
            }
        });
    }};
}

#[test]
fn floats() {
    check!(w32, u32, float_key_32);
    check!(w64, u64, float_key_64);
}

#[test]
fn unsigned() {
    check!(wu8, u8, |b| b);
    check!(wu16, u16, |b| b);
    check!(wu32, u32, |b| b);
    check!(wu64, u64, |b| b);
    check!(wu128, u128, |b| b);
}

#[test]
fn signed() {
    check!(wi8, u8, |b| b ^ 1 << 7);
    check!(wi16, u16, |b| b ^ 1 << 15);
    check!(wi32, u32, |b| b ^ 1 << 31);
    check!(wi64, u64, |b| b ^ 1 << 63);
    check!(wi128, u128, |b| b ^ 1 << 127);
}
