use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::Radixable;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub struct Custom {
    min: u32,
    max: u32,
}

impl Custom {
    #![allow(dead_code)]
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
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        self.max - self.min
    }
    #[inline]
    fn type_size(&self) -> usize {
        32
    }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> u32 {
        item as u32
    }
    #[inline]
    fn keytype_to_usize(&self, item: u32) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [Custom]) {
        lsd_radixsort(arr, 8);
    }
    fn dlsd_sort(&self, arr: &mut [Custom]) {
        dlsd_radixsort(arr, 8);
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
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        self.value as u32 ^ 0x8000_0000
    }
    #[inline]
    fn type_size(&self) -> usize {
        32
    }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> u32 {
        item as u32
    }
    #[inline]
    fn keytype_to_usize(&self, item: u32) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [MyStruct]) {
        lsd_radixsort(arr, 8);
    }
    fn dlsd_sort(&self, arr: &mut [MyStruct]) {
        dlsd_radixsort(arr, 8);
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
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(self.rate);

            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF_FFFF_FFFF
            } else {
                casted ^ submask
            }
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        64
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u64 {
        item as u64
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u64) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [StructWithF64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [StructWithF64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}
