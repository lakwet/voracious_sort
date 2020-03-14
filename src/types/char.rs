use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort_heu;
use super::super::sorts::utils::{get_empty_histograms, Params};
use super::super::{Radixable};

impl Radixable<char> for char {
    type Key = char;

    #[inline]
    fn key(&self) -> char { *self }
    #[inline]
    fn extract(&self, mask: u32, shift: usize) -> usize {
        ((*self as u32 & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> char {
        std::char::from_u32(v as u32).unwrap()
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        *self as u32
    }
    fn get_full_histograms(
        &self,
        arr: &mut [char],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p, p.max_level);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u32;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = *arr.get_unchecked(i) as u32;
                    let v1 = *arr.get_unchecked(i + 1) as u32;
                    let v2 = *arr.get_unchecked(i + 2) as u32;
                    let v3 = *arr.get_unchecked(i + 3) as u32;
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = *arr.get_unchecked(offset + i) as u32;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i) as u32;
                    let mut v1 = *arr.get_unchecked(i + 1) as u32;
                    let mut v2 = *arr.get_unchecked(i + 2) as u32;
                    let mut v3 = *arr.get_unchecked(i + 3) as u32;
                    histograms[1][(v0 & default_mask) as usize] += 1;
                    histograms[1][(v1 & default_mask) as usize] += 1;
                    histograms[1][(v2 & default_mask) as usize] += 1;
                    histograms[1][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let mut v = *arr.get_unchecked(offset + i) as u32;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 3 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i) as u32;
                    let mut v1 = *arr.get_unchecked(i + 1) as u32;
                    let mut v2 = *arr.get_unchecked(i + 2) as u32;
                    let mut v3 = *arr.get_unchecked(i + 3) as u32;
                    histograms[2][(v0 & default_mask) as usize] += 1;
                    histograms[2][(v1 & default_mask) as usize] += 1;
                    histograms[2][(v2 & default_mask) as usize] += 1;
                    histograms[2][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[1][(v0 & default_mask) as usize] += 1;
                    histograms[1][(v1 & default_mask) as usize] += 1;
                    histograms[1][(v2 & default_mask) as usize] += 1;
                    histograms[1][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let mut v = *arr.get_unchecked(offset + i) as u32;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 4 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i) as u32;
                    let mut v1 = *arr.get_unchecked(i + 1) as u32;
                    let mut v2 = *arr.get_unchecked(i + 2) as u32;
                    let mut v3 = *arr.get_unchecked(i + 3) as u32;
                    histograms[3][(v0 & default_mask) as usize] += 1;
                    histograms[3][(v1 & default_mask) as usize] += 1;
                    histograms[3][(v2 & default_mask) as usize] += 1;
                    histograms[3][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[2][(v0 & default_mask) as usize] += 1;
                    histograms[2][(v1 & default_mask) as usize] += 1;
                    histograms[2][(v2 & default_mask) as usize] += 1;
                    histograms[2][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[1][(v0 & default_mask) as usize] += 1;
                    histograms[1][(v1 & default_mask) as usize] += 1;
                    histograms[1][(v2 & default_mask) as usize] += 1;
                    histograms[1][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let mut v = *arr.get_unchecked(offset + i) as u32;
                    histograms[3][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        }

        histograms
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
