use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::utils::offset_from_bits;
use super::{RadixSort, Radixable, RadixableForContainer};
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub struct Custom {
    min: u32,
    max: u32,
}

impl Custom {
    pub fn new(min: u32, max: u32) -> Custom {
        Custom { min, max }
    }
}

impl PartialOrd for Custom {
    fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
        (self.max - self.min).partial_cmp(&(other.max - other.min))
    }
}

impl PartialEq for Custom {
    fn eq(&self, other: &Self) -> bool {
        self.max - self.min == other.max - other.min
    }
}

impl Radixable for Custom {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        (((self.max - self.min) & mask) >> shift) as usize
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

impl RadixableForContainer for [Custom] {
    type T = Custom;
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self.iter().map(|v| v.max - v.min).max().unwrap();
        offset_from_bits(buf, radix, 32, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        32
    }
    #[inline]
    fn into_key_type(&self, v: Custom) -> u32 {
        v.max - v.min
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

impl RadixSort for [Custom] {
    fn voracious_sort(&mut self) {
        lsd_radixsort(self, 8);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MyStruct {
    pub value: i32,
    pub _rank: u8,
}

impl PartialOrd for MyStruct {
    fn partial_cmp(&self, other: &MyStruct) -> Option<Ordering> {
        self.value.partial_cmp(&(other.value))
    }
}

impl PartialEq for MyStruct {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Radixable for MyStruct {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        (((self.value as u32 ^ 0x8000_0000) & mask) >> shift) as usize
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

impl RadixableForContainer for [MyStruct] {
    type T = MyStruct;
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self
            .iter()
            .map(|v| v.value as u32 ^ 0x8000_0000)
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 32, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        32
    }
    #[inline]
    fn into_key_type(&self, v: MyStruct) -> u32 {
        v.value as u32 ^ 0x8000_0000
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

impl RadixSort for [MyStruct] {
    fn voracious_sort(&mut self) {
        voracious_sort(self, 8);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructWithF64 {
    pub rate: f64,
}

impl PartialOrd for StructWithF64 {
    fn partial_cmp(&self, other: &StructWithF64) -> Option<Ordering> {
        self.rate.partial_cmp(&(other.rate))
    }
}

impl PartialEq for StructWithF64 {
    fn eq(&self, other: &Self) -> bool {
        self.rate == other.rate
    }
}

impl Radixable for StructWithF64 {
    type KeyType = u64;

    #[inline]
    fn get_key(&self, mask: u64, shift: usize) -> usize {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(self.rate);
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

impl RadixableForContainer for [StructWithF64] {
    type T = StructWithF64;
    type KeyType = u64;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        // because first bit are never all zeros.
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        64
    }
    #[inline]
    fn into_key_type(&self, v: StructWithF64) -> u64 {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(v.rate);
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

impl RadixSort for [StructWithF64] {
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
