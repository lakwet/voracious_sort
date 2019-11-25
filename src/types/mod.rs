mod bool;
mod char;
pub mod custom;
mod floats;
mod signed_integer;
mod string;
mod tuple;
mod unsigned_integer;
pub mod utils;

use std::ops::{BitAnd, Shl, Shr};

use super::sorts::utils::Params;

pub trait Radixable: Sized + Copy {
    type KeyType;

    fn get_key(&self, mask: Self::KeyType, shift: usize) -> usize;

    fn to_generic(&self, _value: usize) -> Self {
        *self
    }

    fn mask_for_high_bits(
        &self,
        default: Self::KeyType,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> Self::KeyType;
}

pub trait RadixableForContainer {
    type T: Radixable;
    type KeyType: Copy
        + Shl<Output = Self::KeyType>
        + Shr<Output = Self::KeyType>
        + BitAnd<Output = Self::KeyType>
        + PartialEq
        + PartialOrd
        + std::fmt::Display;

    fn compute_offset(&self, _radix: usize) -> (usize, usize);
    fn element_bit_size(&self) -> usize;
    fn into_key_type(&self, v: Self::T) -> Self::KeyType;
    fn from_key_type(&self, v: Self::KeyType) -> usize;
    fn usize_into_key_type(&self, v: usize) -> Self::KeyType;

    #[inline]
    fn compute_max_level(&self, offset: usize, radix: usize) -> usize {
        let rest = self.element_bit_size() - offset;
        if rest % radix != 0 {
            (rest / radix) + 1
        } else {
            rest / radix
        }
    }
    #[inline]
    fn get_default_mask(&self, p: &Params) -> Self::KeyType {
        let mut mask: usize = 1;
        for _ in 0..(p.radix - 1) {
            mask = (mask << 1) | 1;
        }
        self.usize_into_key_type(mask)
    }
    #[inline]
    fn get_mask_and_shift(&self, p: &Params) -> (Self::KeyType, usize) {
        let mask = self.get_default_mask(p);
        let shift: usize = p.radix * (p.max_level - p.level - 1);
        let mask = mask << self.usize_into_key_type(shift);

        (mask, shift)
    }
    #[inline]
    fn get_mask_and_shift_for_partial(
        &self,
        p: &Params,
    ) -> (Self::KeyType, usize) {
        let mask = self.get_default_mask(p);
        let bits = self.element_bit_size();
        let shift = if p.radix * p.max_level > bits - p.offset {
            p.radix * (p.max_level - 1 - p.level)
        } else {
            bits - p.offset - p.radix * (p.level + 1)
        };
        let mask = mask << self.usize_into_key_type(shift);

        (mask, shift)
    }
}

pub trait RadixSort {
    fn voracious_sort(&mut self);
    fn dlsd_sort(&mut self);
}

impl<T> RadixSort for Vec<T>
where
    T: Radixable,
    [T]: RadixSort + RadixableForContainer,
{
    fn voracious_sort(&mut self) {
        self.as_mut_slice().voracious_sort();
    }
    fn dlsd_sort(&mut self) {
        self.as_mut_slice().dlsd_sort();
    }
}
