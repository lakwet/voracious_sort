use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort_heu;
use super::Radixable;

impl Radixable for char {
    type KeyType = u32;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> char {
        std::char::from_u32(v as u32).unwrap()
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        *self as u32
    }
    #[inline]
    fn type_size(&self) -> usize {
        21
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
    fn voracious_sort(&self, arr: &mut [char]) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort_heu(arr, 8, 11_000);
        }
    }
    fn dlsd_sort(&self, arr: &mut [char]) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}
