use super::super::sorts::msd_string_sort::msd_string_radixsort;
use super::super::sorts::utils::Params;
use super::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for &str {
    type KeyType = u8;

    #[inline]
    fn get_key(&self, _mask: u8, shift: usize) -> usize {
        if shift >= self.len() {
            0 as usize
        } else {
            self.as_bytes()[shift] as usize
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        _default_mask: u8,
        _radix: usize,
        _offset: usize,
        _max_level: usize,
    ) -> u8 {
        0xFF // dummy
    }
}

impl<'a> RadixableForContainer for [&'a str] {
    type T = &'a str;
    type KeyType = u8;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0) // dummy
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        self.len()
    }
    #[inline]
    fn into_key_type(&self, v: &'a str) -> u8 {
        v.as_bytes()[0]
    }
    #[inline]
    fn from_key_type(&self, v: u8) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u8 {
        v as u8
    }
    // overrided function
    #[inline]
    fn compute_max_level(&self, _offset: usize, _radix: usize) -> usize {
        self.iter().map(|s| s.len()).max().unwrap()
    }
    // overrided function
    #[inline]
    fn get_mask_and_shift(&self, p: &Params) -> (Self::KeyType, usize) {
        (0, p.level)
    }
}

impl RadixSort for [&str] {
    fn voracious_sort(&mut self) {
        msd_string_radixsort(self);
    }
    fn dlsd_sort(&mut self) {
        msd_string_radixsort(self);
    }
}
