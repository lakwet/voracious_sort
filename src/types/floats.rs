#[cfg(feature = "voracious_multithread")]
use rayon::slice::ParallelSliceMut;

use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
#[cfg(feature = "voracious_multithread")]
use super::super::sorts::peeka_sort::peeka_sort;
use super::super::sorts::rollercoaster_sort::rollercoaster_sort;
use super::super::sorts::utils::{get_empty_histograms, Params};
use super::super::Radixable;

impl Radixable<f32> for f32 {
    type Key = f32;

    #[inline]
    fn key(&self) -> f32 { *self }
    #[inline]
    fn extract(&self, mask: u32, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        let submask = 0x8000_0000;
        let casted = (*self).to_bits();

        if casted & submask == submask {
            casted ^ 0xFFFF_FFFF
        } else {
            casted ^ submask
        }
    }
    fn get_full_histograms(
        &self,
        arr: &mut [f32],
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
    fn voracious_sort(&self, arr: &mut [f32]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 2_900_000 {
            lsd_radixsort(arr, 8);
        } else {
            rollercoaster_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [f32]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() < 1_000_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let chunk_size = if arr.len() < 10_000_000 {
                200_000
            } else if arr.len() < 100_000_000 {
                300_000
            } else if arr.len() < 300_000_000 {
                600_000
            } else {
                700_000
            };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}

impl Radixable<f64> for f64 {
    type Key = f64;

    #[inline]
    fn key(&self) -> f64 { *self }
    #[inline]
    fn extract(&self, mask: u64, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        let submask = 0x8000_0000_0000_0000;
        let casted = (*self).to_bits();

        if casted & submask == submask {
            casted ^ 0xFFFF_FFFF_FFFF_FFFF
        } else {
            casted ^ submask
        }
    }
    fn get_full_histograms(
        &self,
        arr: &mut [f64],
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
    fn voracious_sort(&self, arr: &mut [f64]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap())
        } else if arr.len() < 800 {
            dlsd_radixsort(arr, 8);
        } else {
            rollercoaster_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [f64]) {
        self.voracious_sort(arr);
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() < 800_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let chunk_size = if arr.len() < 1_000_000 {
                100_000
            } else if arr.len() < 5_000_000 {
                200_000
            } else if arr.len() < 20_000_000 {
                500_000
            } else if arr.len() < 500_000_000 {
                400_000
            } else {
                500_000
            };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}
