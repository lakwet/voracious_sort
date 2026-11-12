//! Sorting elements (structs) by a key, stably.
//!
//! Each key is encoded once into an unsigned integer with the same order
//! (`SortKey`), then packed with its element's index into one integer:
//! `(key - min) << index_bits | index`, where `index_bits` covers `n - 1`
//! (33 bits from 4.29 billion elements) and only the bits of `max - min`
//! are kept from the key. The packed integers (u64 when key and index fit
//! in 64 bits, else u128) are sorted by `devour_sort2` with all its steps
//! (presorted arrays, few values, heavy hitters, narrowing, Verge...); equal
//! keys keep their index order, so the sort is stable. One gather pass then
//! puts the elements in that order (skipped when it is the identity).
//! Keys too wide for a u128 with their index fall back to `sort_by`.

use super::devour_sort2::devour_sort2;
use std::mem::MaybeUninit;

/// A key encoded into an unsigned integer with the same order.
pub trait SortKey: Copy {
    fn encode(self) -> u128;
}

macro_rules! key_unsigned {
    ($($t:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                self as u128
            }
        }
    )*};
}
key_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! key_signed {
    ($($t:ty => $u:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                ((self as $u) ^ (1 << (<$u>::BITS - 1))) as u128
            }
        }
    )*};
}
key_signed!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128, isize => usize);

impl SortKey for char {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u32 as u128
    }
}

impl SortKey for bool {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u128
    }
}

// Floats in total order (`total_cmp`): -0.0 before +0.0, NaN not handled.
impl SortKey for f32 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i32) >> 31) as u32) | (1 << 31))) as u128
    }
}

impl SortKey for f64 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i64) >> 63) as u64) | (1 << 63))) as u128
    }
}

/// Sorts `arr` by `key`, stably.
pub fn devour_sort_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let (mut lo, mut hi) = (u128::MAX, 0u128);
    for x in arr.iter() {
        let k = key(x).encode();
        lo = lo.min(k);
        hi = hi.max(k);
    }
    let index_bits = usize::BITS - (n - 1).leading_zeros();
    let key_bits = u128::BITS - (hi - lo).leading_zeros();
    if key_bits + index_bits <= 64 {
        let mut packed: Vec<u64> = arr.iter().enumerate().map(|(i, x)| (((key(x).encode() - lo) << index_bits) as u64) | i as u64).collect();
        devour_sort2(&mut packed);
        let mask = (1u64 << index_bits) - 1;
        gather(arr, |j| (packed[j] & mask) as usize);
    } else if key_bits + index_bits <= 128 {
        let mut packed: Vec<u128> = arr.iter().enumerate().map(|(i, x)| ((key(x).encode() - lo) << index_bits) | i as u128).collect();
        devour_sort2(&mut packed);
        let mask = (1u128 << index_bits) - 1;
        gather(arr, |j| (packed[j] & mask) as usize);
    } else {
        arr.sort_by(|a, b| key(a).encode().cmp(&key(b).encode()));
    }
}

/// Puts `arr[index(j)]` at position `j` for every `j` (`index` is a
/// permutation), through a buffer; nothing moved when it is the identity.
fn gather<T>(arr: &mut [T], index: impl Fn(usize) -> usize) {
    let n = arr.len();
    if (0..n).all(|j| index(j) == j) {
        return;
    }
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: `index` is a permutation of 0..n: every element is read
    // exactly once into the buffer, then the buffer is copied back over the
    // whole array. Elements are moved bitwise, none is dropped or
    // duplicated; the buffer (MaybeUninit) does not drop them. No code that
    // can panic runs between the reads and the copy back.
    unsafe {
        let src = arr.as_ptr();
        let dst = buf.as_mut_ptr() as *mut T;
        for j in 0..n {
            dst.add(j).write(std::ptr::read(src.add(index(j))));
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}
