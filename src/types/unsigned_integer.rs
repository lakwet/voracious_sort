use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort_heu;
use super::Radixable;

impl Radixable for u8 {
    type KeyType = u8;

    #[inline] // default implementation, might be override
    fn extract(&self, mask: u8, shift: usize) -> usize {
        ((*self & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> u8 {
        v as u8
    }
    #[inline]
    fn into_key_type(&self) -> u8 {
        *self
    }
    #[inline]
    fn type_size(&self) -> usize {
        8
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u8 {
        item as u8
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u8) -> usize {
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
    fn voracious_sort(&self, arr: &mut [u8]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            counting_sort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [u8]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for u16 {
    type KeyType = u16;

    #[inline] // default implementation, might be override
    fn extract(&self, mask: u16, shift: usize) -> usize {
        ((*self & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> u16 {
        v as u16
    }
    #[inline]
    fn into_key_type(&self) -> u16 {
        *self
    }
    #[inline]
    fn type_size(&self) -> usize {
        16
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u16 {
        item as u16
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u16) -> usize {
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
    fn voracious_sort(&self, arr: &mut [u16]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 300_000 {
            lsd_radixsort(arr, 8);
        } else {
            counting_sort(arr, 16);
        }
    }
    fn dlsd_sort(&self, arr: &mut [u16]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for u32 {
    type KeyType = u32;

    #[inline] // default implementation, might be override
    fn extract(&self, mask: u32, shift: usize) -> usize {
        ((*self & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> u32 {
        v as u32
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        *self
    }
    #[inline]
    fn type_size(&self) -> usize {
        32
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u32 {
        item as u32
    }
    #[inline(always)]
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
    fn voracious_sort(&self, arr: &mut [u32]) {
        lsd_radixsort_heu(arr, 8, 200_000);
    }
    fn dlsd_sort(&self, arr: &mut [u32]) {
        dlsd_radixsort(arr, 8);
    }
}

impl Radixable for u64 {
    type KeyType = u64;

    #[inline] // default implementation, might be override
    fn extract(&self, mask: u64, shift: usize) -> usize {
        ((*self & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> u64 {
        v as u64
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        *self
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
    fn voracious_sort(&self, arr: &mut [u64]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort_heu(arr, 8, 200_000);
        } else {
            voracious_sort_heu(arr, 8, 200_000);
        }
    }
    fn dlsd_sort(&self, arr: &mut [u64]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for u128 {
    type KeyType = u128;

    #[inline] // default implementation, might be override
    fn extract(&self, mask: u128, shift: usize) -> usize {
        ((*self & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> u128 {
        v as u128
    }
    #[inline]
    fn into_key_type(&self) -> u128 {
        *self
    }
    #[inline]
    fn type_size(&self) -> usize {
        128
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u128 {
        item as u128
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u128) -> usize {
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
    fn voracious_sort(&self, arr: &mut [u128]) {
        voracious_sort_heu(arr, 8, 200_000);
    }
    fn dlsd_sort(&self, arr: &mut [u128]) {
        dlsd_radixsort(arr, 8);
    }
}
