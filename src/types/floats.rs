use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::Radixable;

impl Radixable for f32 {
    type KeyType = u32;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        unsafe {
            let submask = 0x8000_0000;
            let casted = std::mem::transmute::<f32, u32>(*self);

            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF
            } else {
                casted ^ submask
            }
        }
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
    fn voracious_sort(&self, arr: &mut [f32]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [f32]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for f64 {
    type KeyType = u64;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(*self);

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
    fn voracious_sort(&self, arr: &mut [f64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [f64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}
