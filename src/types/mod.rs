mod bool;
mod char;
pub mod custom;
mod floats;
mod signed_integer;
mod string;
mod tuple;
mod unsigned_integer;

use std::ops::{BitAnd, BitOrAssign, Shl, Shr};

use super::sorts::utils::{compute_max_level, compute_offset, Params};

pub trait Radixable<T = Self>: Sized + Copy
where
    T: Radixable,
{
    type KeyType: Copy
        + Shl<Output = Self::KeyType>
        + Shr<Output = Self::KeyType>
        + BitAnd<Output = Self::KeyType>
        + BitOrAssign
        + PartialEq
        + PartialOrd
        + Ord
        + std::fmt::Display;

    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize;
    fn into_key_type(&self) -> Self::KeyType;
    fn type_size(&self) -> usize;
    fn usize_to_keytype(&self, item: usize) -> Self::KeyType;
    fn keytype_to_usize(&self, item: Self::KeyType) -> usize;
    #[inline]
    fn default_mask(&self, radix: usize) -> Self::KeyType {
        let mut mask: usize = 0;
        for _ in 0..radix {
            mask = (mask << 1) | 1;
        }
        self.usize_to_keytype(mask)
    }
    #[inline]
    fn get_mask_and_shift(&self, p: &Params) -> (Self::KeyType, usize) {
        let mask = self.default_mask(p.radix);
        let shift: usize = p.radix * (p.max_level - p.level - 1);
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    #[inline] // default implementation, might be override
    fn get_mask_and_shift_for_partial(
        &self,
        p: &Params,
    ) -> (Self::KeyType, usize) {
        let mask = self.default_mask(p.radix);
        let bits = self.type_size();
        let shift = if p.radix * p.max_level > bits - p.offset {
            p.radix * (p.max_level - 1 - p.level)
        } else {
            bits - p.offset - p.radix * (p.level + 1)
        };
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    #[inline] // default implementation, might be override
    fn to_generic(&self, _value: usize) -> Self {
        *self
    }
    #[inline] // default implementation, might be override
    fn compute_offset(&self, arr: &mut [T], radix: usize) -> (usize, usize) {
        compute_offset(arr, radix)
    }
    #[inline] // default implementation, might be override
    fn compute_max_level(&self, offset: usize, radix: usize) -> usize {
        compute_max_level(self.type_size(), offset, radix)
    }
    fn default_key(&self) -> Self::KeyType;
    fn one(&self) -> Self::KeyType;
    #[inline]
    fn mask_for_high_bits(
        &self,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> Self::KeyType {
        let bits = self.type_size();
        let default_mask = self.default_mask(radix);
        let mut mask = self.default_key();
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << self.usize_to_keytype(radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << self.usize_to_keytype(
                        bits - offset - radix * (max_level - level),
                    );
            }
        }
        mask
    }
    fn voracious_sort(&self, arr: &mut [T]) -> ();
    fn dlsd_sort(&self, arr: &mut [T]) -> ();
}

pub trait RadixSort<T: Radixable> {
    fn voracious_sort(&mut self);
    fn dlsd_sort(&mut self);
}

impl<T> RadixSort<T> for [T]
where
    T: Radixable,
{
    fn voracious_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_sort(self);
        }
    }
    fn dlsd_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_sort(self);
        }
    }
}

impl<T> RadixSort<T> for Vec<T>
where
    T: Radixable,
{
    fn voracious_sort(&mut self) {
        self.as_mut_slice().voracious_sort();
    }
    fn dlsd_sort(&mut self) {
        self.as_mut_slice().dlsd_sort();
    }
}
