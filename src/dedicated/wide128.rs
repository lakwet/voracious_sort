//! 128-bit integers (u128, i128).
//!
//! The shared steps first (sorted or reversed arrays, near-sorted repair,
//! heavy hitters, Verge and merges), then the keys:
//! - narrowed when they fit in fewer bits: one read gives the min, the max
//!   and the bits that differ; when `(key - min)` without its common low
//!   bits fits in 64 (or 32) bits, the narrowed keys are written over the
//!   start of the same array (each write lands on keys already read: no
//!   extra memory), sorted by the 64-bit (or 32-bit) sort with all its
//!   detectors, and widened back from the end. Nanosecond timestamps, small
//!   values stored in 128 bits, fixed-point amounts, narrow i128 ranges.
//! - else sorted by the DLSD on 128-bit keys: a stable LSD on the top bits
//!   of `key - min` only (about log2 n + 8 of them, most keys are told
//!   apart there), then the small groups sharing that prefix finished; the
//!   rollercoaster above the cache.

use super::devour_sort2::{devour_sort2, heavy_by, presorted, DevourSort2};
use super::float_dlsd::{di128, du128};
use super::float_roller::{ri128, ru128};
use super::ravenous_sort2::radix_path;

/// Up to this size: `sort_unstable`.
const SMALL: usize = 64;

/// A 128-bit key: its bits and their encoding (unsigned order = key order).
pub(crate) trait Key128: Copy + Ord {
    fn enc(self) -> u128;
    fn dec(key: u128) -> Self;
    /// DLSD, rollercoaster on the raw bits.
    fn dlsd(bits: &mut [u128]);
    fn roller(bits: &mut [u128]);
}

impl Key128 for u128 {
    #[inline(always)]
    fn enc(self) -> u128 {
        self
    }
    #[inline(always)]
    fn dec(key: u128) -> Self {
        key
    }
    fn dlsd(bits: &mut [u128]) {
        du128::dlsd(bits)
    }
    fn roller(bits: &mut [u128]) {
        ru128::roller(bits)
    }
}

impl Key128 for i128 {
    #[inline(always)]
    fn enc(self) -> u128 {
        (self as u128) ^ (1 << 127)
    }
    #[inline(always)]
    fn dec(key: u128) -> Self {
        (key ^ (1 << 127)) as i128
    }
    fn dlsd(bits: &mut [u128]) {
        di128::dlsd(bits)
    }
    fn roller(bits: &mut [u128]) {
        ri128::roller(bits)
    }
}

#[inline(always)]
fn bits_of<T: Key128>(arr: &mut [T]) -> &mut [u128] {
    // SAFETY: u128 and i128 have the same size and alignment as `T` (the
    // only implementors), every bit pattern is valid.
    unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut u128, arr.len()) }
}

pub(crate) fn sort128<T: Key128 + DevourSort2>(arr: &mut [T]) {
    let n = arr.len();
    if n <= SMALL {
        arr.sort_unstable();
        return;
    }
    let (prefix, probes) = match presorted(arr) {
        Ok(()) => return,
        Err(found) => found,
    };
    if narrow(arr) {
        return;
    }
    radix_path(
        arr,
        prefix,
        probes,
        true,
        &|a: &mut [T], _| few(a) || heavy_by(a, &|r: &mut [T]| devour_sort2(r), |x, y| x == y, |x, y| x < y),
        &|chunk: &mut [T]| {
            if chunk.len() >= roller_min() {
                T::roller(bits_of(chunk));
            } else {
                T::dlsd(bits_of(chunk));
            }
        },
    );
}

/// From this size the rollercoaster (in-place MSD passes, then the DLSD on
/// leaves in cache) instead of the DLSD: L2 / 24 (keys of 16 bytes, as the
/// 64-bit threshold L2 / 12).
fn roller_min() -> usize {
    super::cache_info::l2_bytes() / 24
}

/// Narrowing (see the module doc). Returns whether the keys were sorted.
fn narrow<T: Key128>(arr: &mut [T]) -> bool {
    let first = arr[0].enc();
    let (mut lo, mut hi, mut diff) = (u128::MAX, 0u128, 0u128);
    for &k in arr.iter() {
        let e = k.enc();
        lo = lo.min(e);
        hi = hi.max(e);
        diff |= e ^ first;
    }
    if diff == 0 {
        return true;
    }
    // Low bits equal in every key are equal in `key - min` too: zeros.
    let tz = diff.trailing_zeros();
    let span = (hi - lo) >> tz;
    if span <= u32::MAX as u128 {
        narrow_to::<T, u32>(arr, lo, tz);
        return true;
    }
    if span <= u64::MAX as u128 {
        narrow_to::<T, u64>(arr, lo, tz);
        return true;
    }
    false
}

/// A narrower unsigned key.
trait Narrow: Copy + DevourSort2 {
    fn from128(v: u128) -> Self;
    fn to128(self) -> u128;
}

impl Narrow for u32 {
    #[inline(always)]
    fn from128(v: u128) -> Self {
        v as u32
    }
    #[inline(always)]
    fn to128(self) -> u128 {
        self as u128
    }
}

impl Narrow for u64 {
    #[inline(always)]
    fn from128(v: u128) -> Self {
        v as u64
    }
    #[inline(always)]
    fn to128(self) -> u128 {
        self as u128
    }
}

fn narrow_to<T: Key128, U: Narrow>(arr: &mut [T], lo: u128, tz: u32) {
    let n = arr.len();
    let wide = arr.as_mut_ptr();
    let small = wide as *mut U;
    // SAFETY: the narrow key `i` is written at bytes [i s, i s + s) with
    // s <= 8, below the start (16 i) of the wide key `i`, read first:
    // only keys already read are overwritten. The narrow slice covers the
    // first `n s` bytes of the array, borrowed exclusively.
    unsafe {
        for i in 0..n {
            let k = wide.add(i).read();
            small.add(i).write(U::from128((k.enc() - lo) >> tz));
        }
        devour_sort2(std::slice::from_raw_parts_mut(small, n));
        // Widened from the end: the wide key `i` covers bytes [16 i, 16 i +
        // 16), above the narrow keys still to read (indices < i), except
        // for i = 0, read before it is written.
        for i in (0..n).rev() {
            let v = small.add(i).read();
            wide.add(i).write(T::dec(lo + (v.to128() << tz)));
        }
    }
}

/// Most distinct values counted, and the hash table size (at most half full).
const FEW_MAX: usize = 256;
const FEW_TABLE_BITS: u32 = 9;
const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;
/// From this size the few-values counting is tried.
const FEW_MIN: usize = 129;

/// Few distinct values (at most `FEW_MAX`): a spread sample gives up as soon
/// as half of it is distinct; else one pass counts every value in an
/// open-addressing hash table, giving up beyond `FEW_MAX` values with
/// nothing written; the values are sorted and the array rewritten (same
/// scheme as the other integer widths).
fn few<T: Key128>(arr: &mut [T]) -> bool {
    let n = arr.len();
    if n < FEW_MIN {
        return false;
    }
    let hash = |v: T| {
        let e = v.enc();
        (((e as u64) ^ ((e >> 64) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - FEW_TABLE_BITS)) as usize
    };
    let mut keys = [arr[0]; FEW_TABLE];
    let mut used = [false; FEW_TABLE];
    let s = (n / 32).clamp(16, 256);
    let mut distinct = 0;
    for i in 0..s as u64 {
        let v = arr[((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64 >> 32) as usize];
        let mut h = hash(v);
        while used[h] && keys[h] != v {
            h = (h + 1) & (FEW_TABLE - 1);
        }
        if !used[h] {
            used[h] = true;
            keys[h] = v;
            distinct += 1;
            if distinct > s / 2 {
                return false;
            }
        }
    }
    let mut keys = [arr[0]; FEW_TABLE];
    let mut used = [false; FEW_TABLE];
    let mut counts = [0usize; FEW_TABLE];
    let mut distinct = 0;
    for &v in arr.iter() {
        let mut h = hash(v);
        while used[h] && keys[h] != v {
            h = (h + 1) & (FEW_TABLE - 1);
        }
        if !used[h] {
            if distinct == FEW_MAX {
                return false;
            }
            used[h] = true;
            keys[h] = v;
            distinct += 1;
        }
        counts[h] += 1;
    }
    let mut found: Vec<(T, usize)> = (0..FEW_TABLE).filter(|&h| used[h]).map(|h| (keys[h], counts[h])).collect();
    found.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut start = 0;
    for (v, c) in found {
        arr[start..start + c].fill(v);
        start += c;
    }
    true
}
