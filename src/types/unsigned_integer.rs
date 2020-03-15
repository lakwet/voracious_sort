use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::utils::{get_empty_histograms, Params};
use super::super::sorts::voracious_sort::voracious_sort_heu;
use super::super::Radixable;

impl Radixable<u8> for u8 {
    type Key = u8;

    #[inline]
    fn key(&self) -> u8 {
        *self
    }
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
    fn get_full_histograms(
        &self,
        arr: &mut [u8],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p, p.max_level);
        let default_mask = self.default_mask(p.radix);

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let v0 = arr.get_unchecked(i);
                let v1 = arr.get_unchecked(i + 1);
                let v2 = arr.get_unchecked(i + 2);
                let v3 = arr.get_unchecked(i + 3);
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let v = arr.get_unchecked(offset + i);
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }

        histograms
    }
    fn voracious_sort(&self, arr: &mut [u8]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            counting_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [u8]) {
        self.voracious_sort(arr);
    }
}

impl Radixable<u16> for u16 {
    type Key = u16;

    #[inline]
    fn key(&self) -> u16 {
        *self
    }
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
    fn get_full_histograms(
        &self,
        arr: &mut [u16],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p, p.max_level);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u16;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = arr.get_unchecked(i);
                    let v1 = arr.get_unchecked(i + 1);
                    let v2 = arr.get_unchecked(i + 2);
                    let v3 = arr.get_unchecked(i + 3);
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i);
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        }

        histograms
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
    fn voracious_stable_sort(&self, arr: &mut [u16]) {
        self.voracious_sort(arr);
    }
}

impl Radixable<u32> for u32 {
    type Key = u32;

    #[inline]
    fn key(&self) -> u32 {
        *self
    }
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
    fn get_full_histograms(
        &self,
        arr: &mut [u32],
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
                    let v0 = arr.get_unchecked(i);
                    let v1 = arr.get_unchecked(i + 1);
                    let v2 = arr.get_unchecked(i + 2);
                    let v3 = arr.get_unchecked(i + 3);
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i);
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 3 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
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
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
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
    fn voracious_sort(&self, arr: &mut [u32]) {
        lsd_radixsort_heu(arr, 8, 200_000);
    }
    fn voracious_stable_sort(&self, arr: &mut [u32]) {
        self.voracious_sort(arr);
    }
}

impl Radixable<u64> for u64 {
    type Key = u64;

    #[inline]
    fn key(&self) -> u64 {
        *self
    }
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
    fn get_full_histograms(
        &self,
        arr: &mut [u64],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p, p.max_level);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u64;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = arr.get_unchecked(i);
                    let v1 = arr.get_unchecked(i + 1);
                    let v2 = arr.get_unchecked(i + 2);
                    let v3 = arr.get_unchecked(i + 3);
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i);
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 3 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
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
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[3][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 5 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
                    histograms[4][(v0 & default_mask) as usize] += 1;
                    histograms[4][(v1 & default_mask) as usize] += 1;
                    histograms[4][(v2 & default_mask) as usize] += 1;
                    histograms[4][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[4][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[3][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 6 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
                    histograms[5][(v0 & default_mask) as usize] += 1;
                    histograms[5][(v1 & default_mask) as usize] += 1;
                    histograms[5][(v2 & default_mask) as usize] += 1;
                    histograms[5][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[4][(v0 & default_mask) as usize] += 1;
                    histograms[4][(v1 & default_mask) as usize] += 1;
                    histograms[4][(v2 & default_mask) as usize] += 1;
                    histograms[4][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[5][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[4][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[3][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 7 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
                    histograms[6][(v0 & default_mask) as usize] += 1;
                    histograms[6][(v1 & default_mask) as usize] += 1;
                    histograms[6][(v2 & default_mask) as usize] += 1;
                    histograms[6][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[5][(v0 & default_mask) as usize] += 1;
                    histograms[5][(v1 & default_mask) as usize] += 1;
                    histograms[5][(v2 & default_mask) as usize] += 1;
                    histograms[5][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[4][(v0 & default_mask) as usize] += 1;
                    histograms[4][(v1 & default_mask) as usize] += 1;
                    histograms[4][(v2 & default_mask) as usize] += 1;
                    histograms[4][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[6][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[5][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[4][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[3][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[2][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 8 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = *arr.get_unchecked(i);
                    let mut v1 = *arr.get_unchecked(i + 1);
                    let mut v2 = *arr.get_unchecked(i + 2);
                    let mut v3 = *arr.get_unchecked(i + 3);
                    histograms[7][(v0 & default_mask) as usize] += 1;
                    histograms[7][(v1 & default_mask) as usize] += 1;
                    histograms[7][(v2 & default_mask) as usize] += 1;
                    histograms[7][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[6][(v0 & default_mask) as usize] += 1;
                    histograms[6][(v1 & default_mask) as usize] += 1;
                    histograms[6][(v2 & default_mask) as usize] += 1;
                    histograms[6][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[5][(v0 & default_mask) as usize] += 1;
                    histograms[5][(v1 & default_mask) as usize] += 1;
                    histograms[5][(v2 & default_mask) as usize] += 1;
                    histograms[5][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
                    histograms[4][(v0 & default_mask) as usize] += 1;
                    histograms[4][(v1 & default_mask) as usize] += 1;
                    histograms[4][(v2 & default_mask) as usize] += 1;
                    histograms[4][(v3 & default_mask) as usize] += 1;
                    v0 >>= shift;
                    v1 >>= shift;
                    v2 >>= shift;
                    v3 >>= shift;
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
                    let mut v = *arr.get_unchecked(offset + i);
                    histograms[7][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[6][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[5][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[4][(v & default_mask) as usize] += 1;
                    v >>= shift;
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
    fn voracious_stable_sort(&self, arr: &mut [u64]) {
        self.voracious_sort(arr);
    }
}

impl Radixable<u128> for u128 {
    type Key = u128;

    #[inline]
    fn key(&self) -> u128 {
        *self
    }
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
    fn default_key(&self) -> u128 {
        0
    }
    #[inline]
    fn one(&self) -> u128 {
        1
    }
    fn voracious_sort(&self, arr: &mut [u128]) {
        voracious_sort_heu(arr, 8, 200_000);
    }
    fn voracious_stable_sort(&self, arr: &mut [u128]) {
        self.voracious_sort(arr);
    }
}
