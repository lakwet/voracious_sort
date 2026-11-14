//! Records sorted through packed integers `(key - min) << index_bits |
//! index` (the index on log2(n) bits, 33 from 4.29G records), in a u64 when
//! they fit, else a u128: sorted by `devour_sort2`, equal keys keep their
//! index order (stable).

use super::super::integers::devour_sort2;
use super::permute::permute;

/// The packed (key, index) sort, then the permutation applied in place by
/// following its cycles: an element already at its place is not touched,
/// the others are moved once (each write lands on the slot just read). No
/// buffer of elements. Returns false (nothing moved) when key and index do
/// not fit in a u128.
pub(super) fn sort_in_place<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, lo: u128, key_bits: u32) -> bool {
    let n = arr.len();
    let index_bits = usize::BITS - n.saturating_sub(1).leading_zeros();
    if key_bits + index_bits <= 64 {
        let mut packed: Vec<u64> = arr.iter().enumerate().map(|(i, x)| (((enc(x) - lo) << index_bits) as u64) | i as u64).collect();
        devour_sort2(&mut packed);
        let mask = (1u64 << index_bits).wrapping_sub(1);
        permute(arr, &mut packed, |p| (p & mask) as usize, |p, i| *p = (*p & !mask) | i as u64);
        true
    } else if key_bits + index_bits <= 128 {
        let mut packed: Vec<u128> = arr.iter().enumerate().map(|(i, x)| ((enc(x) - lo) << index_bits) | i as u128).collect();
        devour_sort2(&mut packed);
        let mask = (1u128 << index_bits).wrapping_sub(1);
        permute(arr, &mut packed, |p| (p & mask) as usize, |p, i| *p = (*p & !mask) | i as u128);
        true
    } else {
        false
    }
}

/// Moves the `n` elements at `src` to `dst` in key order (stable): the
/// (key - lo, index) pairs packed in one integer and sorted. Returns false
/// (nothing moved) when key and index do not fit in a u128.
///
/// # Safety
/// `src` and `dst` point to `n` elements, do not overlap; the elements of
/// `src` are moved out (bitwise) and must be treated as moved by the caller.
pub(super) unsafe fn sort_into<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, n: usize, enc: &E, lo: u128, key_bits: u32) -> bool {
    let index_bits = usize::BITS - n.saturating_sub(1).leading_zeros();
    if key_bits + index_bits <= 64 {
        let mut packed: Vec<u64> = (0..n).map(|i| (((enc(&*src.add(i)) - lo) << index_bits) as u64) | i as u64).collect();
        devour_sort2(&mut packed);
        let mask = (1u64 << index_bits).wrapping_sub(1);
        for (j, &p) in packed.iter().enumerate() {
            dst.add(j).write(std::ptr::read(src.add((p & mask) as usize)));
        }
        true
    } else if key_bits + index_bits <= 128 {
        let mut packed: Vec<u128> = (0..n).map(|i| ((enc(&*src.add(i)) - lo) << index_bits) | i as u128).collect();
        devour_sort2(&mut packed);
        let mask = (1u128 << index_bits).wrapping_sub(1);
        for (j, &p) in packed.iter().enumerate() {
            dst.add(j).write(std::ptr::read(src.add((p & mask) as usize)));
        }
        true
    } else {
        false
    }
}
