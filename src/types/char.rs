use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort_heu;
use super::utils::offset_from_bits;
use super::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for char {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        ((*self as u32 & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> char {
        std::char::from_u32(v as u32).unwrap()
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u32,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u32 {
        let bits = 21;
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

impl RadixableForContainer for [char] {
    type T = char;
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self.iter().map(|v| *v as u32).max().unwrap();
        offset_from_bits(buf, radix, 21, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        21
    }
    #[inline]
    fn into_key_type(&self, v: char) -> u32 {
        v as u32
    }
    #[inline]
    fn from_key_type(&self, v: u32) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u32 {
        v as u32
    }
}

impl RadixSort for [char] {
    fn voracious_sort(&mut self) {
        if self.len() <= 256 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort_heu(self, 8, 11_000);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 256 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}
