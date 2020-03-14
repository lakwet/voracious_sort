use super::super::sorts::boolean_sort::boolean_sort;
use super::super::{Radixable};

impl Radixable<bool> for bool {
    type Key = bool;

    #[inline]
    fn key(&self) -> bool { *self }
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
    fn voracious_sort(&self, arr: &mut [bool]) {
        boolean_sort(arr);
    }
    fn dlsd_sort(&self, arr: &mut [bool]) {
        boolean_sort(arr);
    }
}
