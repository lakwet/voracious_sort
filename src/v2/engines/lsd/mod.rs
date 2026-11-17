//! Fused LSD radix sort: the key encoding is fused into the first scatter
//! pass and the decoding into the last one. Ideas from blazing-radix (MIT
//! License, Copyright (c) 2026 Max Tikhomirov): all histograms in one read on
//! several lanes; `u32` counters up to `u32::MAX` keys; 11-bit digits when
//! they save a pass; digits equal in every key skipped; 4 keys read before
//! being written, branch-free positions on the top digit; the scratch buffer
//! left uninitialized (fully written by the first pass).
//!
//! One module per key width and codec (`w32`, `wu64`, `wi16`...), made by
//! the `fused_lsd!` macro from the parts in `histogram.rs`, `scatter.rs` and
//! `passes.rs`.

#[macro_use]
mod histogram;
#[macro_use]
mod scatter;
#[macro_use]
mod passes;

pub(crate) use super::counter::Counter;
use std::mem::MaybeUninit;

/// Up to this size: `sort_unstable` on the encoded keys.
const LSD_MIN: usize = 128;
/// Below this size the byte histograms take one lane on the stack.
const SHORT_MAX: usize = 4096;
/// From this size 11-bit digits may be used.
const WIDE_MIN: usize = 8192;

/// Up to this size 11-bit digits may be used on 32-bit keys: while keys and
/// buffer fit in half the L2 (beyond, their 2048 write streams leave the
/// cache; journal §54).
fn wide_max_32() -> usize {
    crate::v2::cache::budget(crate::v2::cache::Budget::Lsd11) / 2 / (2 * 4)
}

macro_rules! fused_lsd {
    ($uint:ty, $sint:ty, $name:ident, $wide_bits:expr, $codec:ident) => {
        pub(crate) mod $name {
            use super::{wide_max_32, Counter, MaybeUninit, LSD_MIN, SHORT_MAX, WIDE_MIN};

            const BITS: u32 = <$uint>::BITS;
            /// Varying bits (on a sample) from which 11-bit digits pay off.
            const WIDE_BITS: u32 = $wide_bits;

            key_codec!($codec, $uint, $sint);
            lsd_histograms!($uint);
            lsd_scatters!($uint);
            lsd_passes!($uint);
        }
    };
}

// Floats (total order), unsigned and signed integers.
fused_lsd!(u32, i32, w32, 20, float);
fused_lsd!(u64, i64, w64, 44, float);
fused_lsd!(u32, i32, wu32, 20, unsigned);
fused_lsd!(u64, i64, wu64, 44, unsigned);
fused_lsd!(u32, i32, wi32, 20, signed);
fused_lsd!(u64, i64, wi64, 44, signed);
fused_lsd!(u128, i128, wu128, 88, unsigned);
fused_lsd!(u128, i128, wi128, 88, signed);
// 8- and 16-bit integers (devour_sort): byte digits only (never wide).
fused_lsd!(u16, i16, wu16, 17, unsigned);
fused_lsd!(u16, i16, wi16, 17, signed);
fused_lsd!(u8, i8, wu8, 9, unsigned);
fused_lsd!(u8, i8, wi8, 9, signed);

#[cfg(test)]
mod tests {
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
}
