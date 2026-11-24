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

/// The short 16-bit LSD: every shape and size up to `SHORT_MAX`, in both
/// orders (u16 bits, i16 bits with the sign bit flipped), and smooth keys
/// (the two-table count) every 7 sizes from 16 to `STACK_MAX` (the stack
/// buffer), then a few sizes up to `SHORT_MAX` (the heap one).
#[test]
fn short16() {
    use super::short16::{lsd_short, SHORT_MAX, STACK_MAX};
    let check = |a: &[u16], what: &str| {
        for f in [0u16, 0x8000] {
            let mut got = a.to_vec();
            if f == 0 {
                lsd_short::<0>(&mut got);
            } else {
                lsd_short::<0x8000>(&mut got);
            }
            let mut want = a.to_vec();
            want.sort_unstable_by_key(|&k| k ^ f);
            assert!(got == want, "short16 {} F={:#x} n={}", what, f, a.len());
        }
    };
    for_each_case::<u16>(|shape, a| {
        if a.len() <= SHORT_MAX {
            check(&a, shape);
        }
    });
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    for n in (16..STACK_MAX).step_by(7).chain([STACK_MAX, STACK_MAX + 1, 20_000, SHORT_MAX]) {
        let mut x = 0u16;
        let a: Vec<u16> = (0..n)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                x = x.wrapping_add((seed % 64) as u16);
                x
            })
            .collect();
        check(&a, "smooth");
    }
}
