use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for f32 {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        unsafe {
            let submask = 0x8000_0000;
            let casted = std::mem::transmute::<f32, u32>(*self);
            let v = if casted & submask == submask {
                casted ^ 0xFFFF_FFFF
            } else {
                casted ^ submask
            };

            ((v & mask) >> shift) as usize
        }
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

impl Radixable for f64 {
    type KeyType = u64;

    #[inline]
    fn get_key(&self, mask: u64, shift: usize) -> usize {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(*self);
            let v = if casted & submask == submask {
                casted ^ 0xFFFF_FFFF_FFFF_FFFF
            } else {
                casted ^ submask
            };

            ((v & mask) >> shift) as usize
        }
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

impl RadixableForContainer for [f32] {
    type T = f32;
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        32
    }
    #[inline]
    fn into_key_type(&self, v: f32) -> u32 {
        unsafe {
            let submask = 0x8000_0000;
            let casted = std::mem::transmute::<f32, u32>(v);
            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF
            } else {
                casted ^ submask
            }
        }
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

impl RadixableForContainer for [f64] {
    type T = f64;
    type KeyType = u64;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        64
    }
    #[inline]
    fn into_key_type(&self, v: f64) -> u64 {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(v);
            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF_FFFF_FFFF
            } else {
                casted ^ submask
            }
        }
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

impl RadixSort for [f32] {
    fn voracious_sort(&mut self) {
        if self.len() <= 300 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(self, 8);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 300 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}

impl RadixSort for [f64] {
    fn voracious_sort(&mut self) {
        if self.len() <= 500 {
            voracious_sort(self, 8);
        } else {
            lsd_radixsort(self, 8);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 500 {
            voracious_sort(self, 8);
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}
