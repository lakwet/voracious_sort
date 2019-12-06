use super::super::sorts::msd_string_sort::msd_string_radixsort;
use super::super::sorts::utils::Params;
use super::Radixable;

impl Radixable for &str {
    type KeyType = u8;

    #[inline] // overrided function
    fn extract(&self, _mask: u8, shift: usize) -> usize {
        if shift >= self.len() {
            0 as usize
        } else {
            self.as_bytes()[shift] as usize
        }
    }
    #[inline]
    fn into_key_type(&self) -> u8 {
        0 // dummy value
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
    #[inline] // overrided function
    fn get_mask_and_shift(&self, p: &Params) -> (u8, usize) {
        (0, p.level)
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [&str],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [&str]) {
        let max_level = arr.iter().map(|item| item.len()).max().unwrap();
        msd_string_radixsort(arr, max_level);
    }
    fn dlsd_sort(&self, arr: &mut [&str]) {
        let max_level = arr.iter().map(|item| item.len()).max().unwrap();
        msd_string_radixsort(arr, max_level);
    }
}
