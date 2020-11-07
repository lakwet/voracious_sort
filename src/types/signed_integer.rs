#[cfg(feature = "voracious_multithread")]
use rayon::slice::ParallelSliceMut;

use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort_heu;
use super::super::sorts::msd_sort::msd_radixsort;
#[cfg(feature = "voracious_multithread")]
use super::super::sorts::peeka_sort::peeka_sort;
use super::super::sorts::rollercoaster_sort::{
    rollercoaster_sort, rollercoaster_sort_heu,
};
use super::super::sorts::utils::{get_empty_histograms, Params};
use super::super::sorts::voracious_sort::voracious_sort_heu;
use super::super::Radixable;

impl Radixable<i8> for i8 {
    type Key = i8;

    #[inline]
    fn key(&self) -> i8 { *self }
    #[inline]
    fn extract(&self, mask: u8, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> i8 { (v as u8 ^ 0x80) as i8 }
    #[inline]
    fn into_key_type(&self) -> u8 { *self as u8 ^ 0x80 }
    fn get_full_histograms(
        &self,
        arr: &mut [i8],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
        let default_mask = self.default_mask(p.radix);

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let v0 = arr.get_unchecked(i).into_key_type();
                let v1 = arr.get_unchecked(i + 1).into_key_type();
                let v2 = arr.get_unchecked(i + 2).into_key_type();
                let v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[0][(v0 & default_mask) as usize] += 1;
                histograms[0][(v1 & default_mask) as usize] += 1;
                histograms[0][(v2 & default_mask) as usize] += 1;
                histograms[0][(v3 & default_mask) as usize] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let v = arr.get_unchecked(offset + i).into_key_type();
                histograms[0][(v & default_mask) as usize] += 1;
            }
        }

        histograms
    }
    fn voracious_sort(&self, arr: &mut [i8]) {
        if arr.len() < 50 {
            arr.sort_unstable();
        } else {
            counting_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [i8]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        peeka_sort(arr, 8, 1_150_000, thread_n);
    }
}

impl Radixable<i16> for i16 {
    type Key = i16;

    #[inline]
    fn key(&self) -> i16 { *self }
    #[inline]
    fn extract(&self, mask: u16, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> i16 { (v as u16 ^ 0x8000) as i16 }
    #[inline]
    fn into_key_type(&self) -> u16 { *self as u16 ^ 0x8000 }
    fn get_full_histograms(
        &self,
        arr: &mut [i16],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u16;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = arr.get_unchecked(i).into_key_type();
                    let v1 = arr.get_unchecked(i + 1).into_key_type();
                    let v2 = arr.get_unchecked(i + 2).into_key_type();
                    let v3 = arr.get_unchecked(i + 3).into_key_type();
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        }

        histograms
    }
    fn voracious_sort(&self, arr: &mut [i16]) {
        if arr.len() <= 230 {
            arr.sort_unstable();
        } else if arr.len() <= 150_000 {
            lsd_radixsort_heu(arr, 8, 150_000);
        } else {
            counting_sort(arr, 16);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [i16]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        peeka_sort(arr, 8, 1_150_000, thread_n);
    }
}

impl Radixable<i32> for i32 {
    type Key = i32;

    #[inline]
    fn key(&self) -> i32 { *self }
    #[inline]
    fn extract(&self, mask: u32, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> i32 { (v as u32 ^ 0x8000_0000) as i32 }
    #[inline]
    fn is_i32(&self) -> bool { true }
    #[inline]
    fn into_key_type(&self) -> u32 { *self as u32 ^ 0x8000_0000 }
    fn get_full_histograms(
        &self,
        arr: &mut [i32],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u32;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = arr.get_unchecked(i).into_key_type();
                    let v1 = arr.get_unchecked(i + 1).into_key_type();
                    let v2 = arr.get_unchecked(i + 2).into_key_type();
                    let v3 = arr.get_unchecked(i + 3).into_key_type();
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 3 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
    fn voracious_sort(&self, arr: &mut [i32]) {
        if arr.len() < 900 {
            arr.sort_unstable();
        } else if arr.len() < 2_000_000 {
            lsd_radixsort_heu(arr, 8, 100_000);
        } else {
            rollercoaster_sort_heu(arr, 8, 100_000);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [i32]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() < 3_500_000 {
            arr.par_sort_unstable();
        } else {
            let chunk_size = if arr.len() < 20_000_000 {
                400_000
            } else if arr.len() < 30_000_000 {
                500_000
            } else if arr.len() < 70_000_000 {
                650_000
            } else if arr.len() < 400_000_000 {
                700_000
            } else if arr.len() < 800_000_000 {
                800_000
            } else {
                900_000
            };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}

impl Radixable<i64> for i64 {
    type Key = i64;

    #[inline]
    fn key(&self) -> i64 { *self }
    #[inline]
    fn extract(&self, mask: u64, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> i64 {
        (v as u64 ^ 0x8000_0000_0000_0000) as i64
    }
    #[inline]
    fn is_i64(&self) -> bool { true }
    #[inline]
    fn into_key_type(&self) -> u64 { *self as u64 ^ 0x8000_0000_0000_0000 }
    fn get_full_histograms(
        &self,
        arr: &mut [i64],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
        let default_mask = self.default_mask(p.radix);
        let shift = p.radix as u64;

        let quotient = arr.len() / 4;
        let remainder = arr.len() % 4;
        let offset = quotient * 4;

        if p.max_level == 1 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let v0 = arr.get_unchecked(i).into_key_type();
                    let v1 = arr.get_unchecked(i + 1).into_key_type();
                    let v2 = arr.get_unchecked(i + 2).into_key_type();
                    let v3 = arr.get_unchecked(i + 3).into_key_type();
                    histograms[0][(v0 & default_mask) as usize] += 1;
                    histograms[0][(v1 & default_mask) as usize] += 1;
                    histograms[0][(v2 & default_mask) as usize] += 1;
                    histograms[0][(v3 & default_mask) as usize] += 1;
                }
            }
            for i in 0..remainder {
                unsafe {
                    let v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 2 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
                    histograms[1][(v & default_mask) as usize] += 1;
                    v >>= shift;
                    histograms[0][(v & default_mask) as usize] += 1;
                }
            }
        } else if p.max_level == 3 {
            for q in 0..quotient {
                unsafe {
                    let i = q * 4;
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
                    let mut v0 = arr.get_unchecked(i).into_key_type();
                    let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                    let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                    let mut v3 = arr.get_unchecked(i + 3).into_key_type();
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
                    let mut v = arr.get_unchecked(offset + i).into_key_type();
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
    fn voracious_sort(&self, arr: &mut [i64]) {
        if arr.len() <= 7000 {
            arr.sort_unstable();
        } else {
            rollercoaster_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [i64]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() < 10_000_000 {
            arr.par_sort_unstable();
        } else {
            let chunk_size =
                if arr.len() < 60_000_000 { 500_000 } else { 700_000 };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}

impl Radixable<i128> for i128 {
    type Key = i128;

    #[inline]
    fn key(&self) -> i128 { *self }
    #[inline]
    fn extract(&self, mask: u128, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, v: usize) -> i128 {
        (v as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000) as i128
    }
    #[inline]
    fn into_key_type(&self) -> u128 {
        *self as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u128 { item as u128 }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u128) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> u128 { 0 }
    #[inline]
    fn one(&self) -> u128 { 1 }
    #[inline]
    fn is_i128(&self) -> bool { true }
    fn voracious_sort(&self, arr: &mut [i128]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            dlsd_radixsort(arr, 8);
        } else if arr.len() <= 1_000_000 {
            voracious_sort_heu(arr, 8, 200_000);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [i128]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        peeka_sort(arr, 8, 650_000, thread_n);
    }
}
