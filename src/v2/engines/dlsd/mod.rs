//! Diverting LSD (DLSD), for arrays in the cache (or the leaves of the
//! rollercoaster): a stable LSD sorts only the top bits of `key - min`
//! (about log2(n) + a few bits, enough to tell most keys apart, see
//! `prefix.rs`); one sequential pass then finishes the groups sharing that
//! prefix (insertion sort, or the DLSD again on large groups) and decodes the
//! keys. When the prefix holds every varying bit, the LSD is exact and the
//! decoding is fused into its last pass. Same engineering as the fused LSD.
//!
//! One module per key width and codec (`d64`, `du32`, `di128`...), made by
//! `dlsd_width!` from `sort.rs`, `prefix.rs` and `kernels.rs`.

#[macro_use]
mod kernels;
#[cfg(feature = "ab")]
#[macro_use]
mod sort_ab;
#[macro_use]
mod prefix;
#[macro_use]
mod sort;

use std::mem::MaybeUninit;

/// Up to this size: `sort_unstable` on the encoded keys.
const SMALL: usize = 64;
/// Up to this size on 128-bit keys: `sort_unstable`.
const SMALL_128: usize = 256;
/// Widest digit (4096 buckets: their write streams still fit in L1/L2).
const MAX_DIGIT_BITS: u32 = 12;
/// Sample for the prefix length: n / `SAMPLE_DIV` keys, at least
/// `SAMPLE_MIN` (a sample proportional to n keeps seeing pairs of keys of
/// the same tight cluster however many clusters there are).
const SAMPLE_DIV: usize = 256;
const SAMPLE_MIN: usize = 16;
/// Histogram counters kept on the stack (16 KiB); more go to the heap.
const STACK_COUNTERS: usize = 4096;
/// Sample keys kept on the stack.
const STACK_SAMPLE: usize = 256;
/// Moves per key allowed to the finishing insertion sort before the groups
/// of equal prefix are finished one by one.
const INSERTION_BUDGET: usize = 2;
/// Groups of equal prefix up to this size are finished by insertion sort.
const GROUP_INSERTION: usize = 16;

/// Floats: the bucket starts are checked for cache aliasing from L2 x 4 / 3
/// bytes of keys (below, keys and buffer mostly stay in L2, where staging
/// costs more than it saves).
fn alias_min_bytes() -> usize {
    crate::v2::cache::budget(crate::v2::cache::Budget::AliasFloat) / 3 * 4
}

/// Integers: from L2 / 32 bytes (integer sequences give regular bucket sizes
/// whose aliased streams cost even inside L2).
fn alias_min_bytes_int() -> usize {
    crate::v2::cache::budget(crate::v2::cache::Budget::AliasInt) / 32
}

/// Keys narrowed to 64 or 32 bits when their range fits (128-bit keys only).
pub(crate) trait WideNarrow: Copy {
    /// `keys` encoded (raw bits if `raw`), `min` and `range` of the encoded
    /// keys. Returns whether the keys were sorted (and decoded).
    fn narrow(
        keys: &mut [Self],
        min: Self,
        range: Self,
        raw: bool,
        enc: fn(Self) -> Self,
        dec: fn(Self) -> Self,
    ) -> bool;
}

impl WideNarrow for u32 {
    #[inline(always)]
    fn narrow(
        _: &mut [Self],
        _: Self,
        _: Self,
        _: bool,
        _: fn(Self) -> Self,
        _: fn(Self) -> Self,
    ) -> bool {
        false
    }
}

impl WideNarrow for u64 {
    #[inline(always)]
    fn narrow(
        _: &mut [Self],
        _: Self,
        _: Self,
        _: bool,
        _: fn(Self) -> Self,
        _: fn(Self) -> Self,
    ) -> bool {
        false
    }
}

impl WideNarrow for u128 {
    #[inline]
    fn narrow(
        keys: &mut [Self],
        min: Self,
        range: Self,
        raw: bool,
        enc: fn(Self) -> Self,
        dec: fn(Self) -> Self,
    ) -> bool {
        if range > u64::MAX as u128 {
            return false;
        }
        if raw {
            crate::v2::devour::wide128::narrow_keys(keys, min, range, enc, dec);
        } else {
            crate::v2::devour::wide128::narrow_keys(keys, min, range, |k| k, dec);
        }
        true
    }
}

macro_rules! dlsd_width {
    ($uint:ty, $w:ident, $m:ident, $alias_min:path) => {
        pub(crate) mod $m {
            use super::super::lsd::$w as w;
            use super::*;

            /// Keys per staging slot (one 64-byte cache line).
            const WB: usize = 64 / std::mem::size_of::<$uint>();

            dlsd_sort!($uint, $alias_min);
            dlsd_prefix!($uint);
            dlsd_kernels!($uint);
        }
    };
}

dlsd_width!(u64, w64, d64, alias_min_bytes);
dlsd_width!(u32, w32, d32, alias_min_bytes);
dlsd_width!(u64, wu64, du64, alias_min_bytes_int);
dlsd_width!(u32, wu32, du32, alias_min_bytes_int);
dlsd_width!(u64, wi64, di64, alias_min_bytes_int);
dlsd_width!(u32, wi32, di32, alias_min_bytes_int);
dlsd_width!(u128, wu128, du128, alias_min_bytes_int);
dlsd_width!(u128, wi128, di128, alias_min_bytes_int);

#[cfg(test)] mod tests;

/// Dev only (feature `ab`): the DLSD with its scratch on the stack
/// (`sort_ab.rs`), for the A/B copy of the u32 / i32 chains.
#[cfg(feature = "ab")]
macro_rules! dlsd_width_ab {
    ($uint:ty, $w:ident, $m:ident, $alias_min:path, $smin:expr) => {
        pub(crate) mod $m {
            use super::super::lsd::$w as w;
            use super::*;

            /// Smallest sample for the prefix length (shadows the engine's).
            const SAMPLE_MIN: usize = $smin;
            /// Counters kept on the stack (32 KiB; shadows the engine's).
            const STACK_COUNTERS: usize = 8192;

            /// Keys per staging slot (one 64-byte cache line).
            const WB: usize = 64 / std::mem::size_of::<$uint>();

            dlsd_sort_ab!($uint, $alias_min);
            dlsd_prefix!($uint);
            dlsd_kernels!($uint);
        }
    };
}

#[cfg(feature = "ab")]
dlsd_width_ab!(u32, wu32, su32, alias_min_bytes_int, 16);
#[cfg(feature = "ab")]
dlsd_width_ab!(u32, wi32, si32, alias_min_bytes_int, 16);
#[cfg(feature = "ab")]
dlsd_width_ab!(u32, wu32, mu32, alias_min_bytes_int, 64);
#[cfg(feature = "ab")]
dlsd_width_ab!(u32, wi32, mi32, alias_min_bytes_int, 64);
