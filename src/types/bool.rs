use super::super::sorts::boolean_sort::boolean_sort;
use super::super::types::Radixable;

impl Radixable for bool {
    type KeyType = u8;

    #[inline] // overrided function
    fn extract(&self, _mask: u8, _shift: usize) -> usize {
        if *self {
            1
        } else {
            0
        }
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> bool {
        value == 1
    }
    #[inline]
    fn into_key_type(&self) -> u8 {
        if *self {
            1
        } else {
            0
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        1
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
    fn voracious_sort(&self, arr: &mut [bool]) {
        boolean_sort(arr);
    }
    fn dlsd_sort(&self, arr: &mut [bool]) {
        boolean_sort(arr);
    }
}
