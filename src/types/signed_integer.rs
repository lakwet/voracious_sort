use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort_heu;
use super::utils::offset_from_bits;
use super::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for i8 {
    type KeyType = u8;

    #[inline]
    fn get_key(&self, mask: u8, shift: usize) -> usize {
        let flip = 0x80;
        (((*self as u8 ^ flip) & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> i8 {
        (v as u8 ^ 0x80) as i8
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

impl Radixable for i16 {
    type KeyType = u16;

    #[inline]
    fn get_key(&self, mask: u16, shift: usize) -> usize {
        let flip = 0x8000;
        (((*self as u16 ^ flip) & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> i16 {
        (v as u16 ^ 0x8000) as i16
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u16,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u16 {
        let bits = 16;
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

impl Radixable for i32 {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        let flip = 0x8000_0000;
        (((*self as u32 ^ flip) & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> i32 {
        (v as u32 ^ 0x8000_0000) as i32
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u32,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u32 {
        let bits = 32;
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

impl Radixable for i64 {
    type KeyType = u64;

    #[inline]
    fn get_key(&self, mask: u64, shift: usize) -> usize {
        let flip = 0x8000_0000_0000_0000;
        (((*self as u64 ^ flip) & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> i64 {
        (v as u64 ^ 0x8000_0000) as i64
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u64,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u64 {
        let bits = 64;
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

impl Radixable for i128 {
    type KeyType = u128;

    #[inline]
    fn get_key(&self, mask: u128, shift: usize) -> usize {
        let flip = 0x8000_0000_0000_0000_0000_0000_0000_0000;
        (((*self as u128 ^ flip) & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> i128 {
        (v as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000) as i128
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u128,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u128 {
        let bits = 128;
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

// we assume radix == 8
impl RadixableForContainer for [i8] {
    type T = i8;
    type KeyType = u8;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self.iter().map(|v| *v as u8 ^ 0x80).max().unwrap();
        offset_from_bits(buf, radix, 8, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        8
    }
    #[inline]
    fn into_key_type(&self, v: i8) -> u8 {
        v as u8 ^ 0x80
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

impl RadixableForContainer for [i16] {
    type T = i16;
    type KeyType = u16;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self.iter().map(|v| *v as u16 ^ 0x8000).max().unwrap();
        offset_from_bits(buf, radix, 16, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        16
    }
    #[inline]
    fn into_key_type(&self, v: i16) -> u16 {
        v as u16 ^ 0x8000
    }
    #[inline]
    fn from_key_type(&self, v: u16) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u16 {
        v as u16
    }
}

impl RadixableForContainer for [i32] {
    type T = i32;
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self.iter().map(|v| *v as u32 ^ 0x8000_0000).max().unwrap();
        offset_from_bits(buf, radix, 32, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        32
    }
    #[inline]
    fn into_key_type(&self, v: i32) -> u32 {
        v as u32 ^ 0x8000_0000
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

impl RadixableForContainer for [i64] {
    type T = i64;
    type KeyType = u64;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self
            .iter()
            .map(|v| *v as u64 ^ 0x8000_0000_0000_0000)
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 64, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        64
    }
    #[inline]
    fn into_key_type(&self, v: i64) -> u64 {
        v as u64 ^ 0x8000_0000_0000_0000
    }
    #[inline]
    fn from_key_type(&self, v: u64) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u64 {
        v as u64
    }
}

impl RadixableForContainer for [i128] {
    type T = i128;
    type KeyType = u128;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self
            .iter()
            .map(|v| *v as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000)
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 128, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        128
    }
    #[inline]
    fn into_key_type(&self, v: i128) -> u128 {
        v as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000
    }
    #[inline]
    fn from_key_type(&self, v: u128) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u128 {
        v as u128
    }
}

impl RadixSort for [i8] {
    fn voracious_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 8);
        } else {
            counting_sort(self, 8);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 8);
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}

impl RadixSort for [i16] {
    fn voracious_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable();
        } else if self.len() <= 300_000 {
            lsd_radixsort(self, 8);
        } else {
            counting_sort(self, 16);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable();
        } else {
            dlsd_radixsort(self, 16);
        }
    }
}

impl RadixSort for [i32] {
    fn voracious_sort(&mut self) {
        lsd_radixsort_heu(self, 8, 200_000);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}

impl RadixSort for [i64] {
    fn voracious_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable();
        } else if self.len() <= 8000 {
            msd_radixsort(self, 8);
        } else if self.len() <= 100_000 {
            lsd_radixsort_heu(self, 8, 200_000);
        } else {
            voracious_sort_heu(self, 8, 200_000);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable();
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}

impl RadixSort for [i128] {
    fn voracious_sort(&mut self) {
        voracious_sort_heu(self, 8, 200_000);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}
