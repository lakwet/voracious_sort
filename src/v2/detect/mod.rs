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

pub(crate) mod clusters;
pub(crate) mod counting;
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
            use super::super::cache::{budget, Budget};
            use super::super::detect::counting::{domain_count, exact_count, window_count, window_count_split};
            use super::super::detect::*;
            use super::*;

            int_range!($t, $uint, $encode, $decode, $roller_min);
            int_dense!($t, $uint, $encode, $decode);
            int_few!($t, $uint, $encode, $decode);
        }
    };
}
