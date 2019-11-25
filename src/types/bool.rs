use super::super::sorts::boolean_sort::boolean_sort;
use super::super::types::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for bool {
    type KeyType = u8;

    #[inline]
    fn get_key(&self, _mask: u8, _shift: usize) -> usize {
        if *self {
            1
        } else {
            0
        }
    }
    #[inline]
    fn to_generic(&self, value: usize) -> Self {
        match value {
            1 => true,
            _ => false,
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u8,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u8 {
        let bits = 8;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [bool] {
    type T = bool;
    type KeyType = u8;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        1
    }
    #[inline]
    fn into_key_type(&self, v: bool) -> u8 {
        if v {
            1
        } else {
            0
        }
    }
    #[inline]
    fn from_key_type(&self, v: u8) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u8 {
        v as u8
    }
}

impl RadixSort for [bool] {
    fn voracious_sort(&mut self) {
        boolean_sort(self);
    }
    fn dlsd_sort(&mut self) {
        boolean_sort(self);
    }
}
