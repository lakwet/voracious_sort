use rayon::slice::ParallelSliceMut;

use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::lsd_stable_sort::lsd_stable_radixsort;
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::msd_stable_sort::msd_stable_radixsort;
use super::super::sorts::peeka_sort::peeka_sort;
use super::super::sorts::rollercoaster_sort::rollercoaster_sort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::super::{RadixKey, Radixable};

pub trait Dispatcher<T: Radixable<K>, K: RadixKey> {
    fn voracious_sort(&self, arr: &mut [T]);
    fn voracious_stable_sort(&self, arr: &mut [T]);
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize);
}

impl<T: Radixable<bool>> Dispatcher<T, bool> for bool {
    fn voracious_sort(&self, arr: &mut [T]) {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 128 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() < 100_000 {
            lsd_stable_radixsort(arr, 1);
        } else {
            msd_stable_radixsort(arr, 1);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() < 1_000_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 1, 75_000, thread_n);
        }
    }
}

impl<T: Radixable<char>> Dispatcher<T, char> for char {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 400 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap())
        } else if arr.len() <= 9_000 {
            lsd_radixsort(arr, 7);
        } else {
            lsd_radixsort(arr, 11);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 170 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap())
        } else if arr.len() <= 9_000 {
            lsd_stable_radixsort(arr, 7);
        } else {
            lsd_stable_radixsort(arr, 11);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 900_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let chunk_size = if arr.len() < 5_000_000 {
                100_000
            } else if arr.len() < 900_000_000 {
                200_000
            } else {
                300_000
            };
            peeka_sort(arr, 7, chunk_size, thread_n);
        }
    }
}

impl<T: Radixable<f32>> Dispatcher<T, f32> for f32 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 400 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() < 500_000 {
            lsd_radixsort(arr, 8);
        } else {
            rollercoaster_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 800_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let chunk_size = if arr.len() < 1_000_000 {
                100_000
            } else if arr.len() < 2_000_000 {
                150_000
            } else if arr.len() < 10_000_000 {
                200_000
            } else if arr.len() < 600_000_000 {
                150_000
            } else {
                200_000
            };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}

impl<T: Radixable<f64>> Dispatcher<T, f64> for f64 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 800 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            rollercoaster_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() < 350 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() < 100_000 {
            msd_stable_radixsort(arr, 8);
        } else if arr.len() < 3_000_000 {
            lsd_stable_radixsort(arr, 8);
        } else {
            msd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 800_000 {
            arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let chunk_size = if arr.len() < 1_000_000 {
                75_000
            } else if arr.len() < 7_000_000 {
                100_000
            } else if arr.len() < 10_000_000 {
                150_000
            } else if arr.len() < 300_000_000 {
                200_000
            } else {
                300_000
            };
            peeka_sort(arr, 8, chunk_size, thread_n);
        }
    }
}

impl<T: Radixable<i8>> Dispatcher<T, i8> for i8 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_stable_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "8")]
impl<T: Radixable<isize>> Dispatcher<T, isize> for isize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_stable_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<i16>> Dispatcher<T, i16> for i16 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "16")]
impl<T: Radixable<isize>> Dispatcher<T, isize> for isize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<i32>> Dispatcher<T, i32> for i32 {
    fn voracious_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "32")]
impl<T: Radixable<isize>> Dispatcher<T, isize> for isize {
    fn voracious_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<i64>> Dispatcher<T, i64> for i64 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            voracious_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_stable_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            msd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl<T: Radixable<isize>> Dispatcher<T, isize> for isize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            voracious_sort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_stable_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            msd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

impl<T: Radixable<i128>> Dispatcher<T, i128> for i128 {
    fn voracious_sort(&self, arr: &mut [T]) { voracious_sort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "128")]
impl<T: Radixable<isize>> Dispatcher<T, isize> for isize {
    fn voracious_sort(&self, arr: &mut [T]) { voracious_sort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

impl<T: Radixable<u8>> Dispatcher<T, u8> for u8 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_stable_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "8")]
impl<T: Radixable<usize>> Dispatcher<T, usize> for usize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            msd_stable_radixsort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<u16>> Dispatcher<T, u16> for u16 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "16")]
impl<T: Radixable<usize>> Dispatcher<T, usize> for usize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<u32>> Dispatcher<T, u32> for u32 {
    fn voracious_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "32")]
impl<T: Radixable<usize>> Dispatcher<T, usize> for usize {
    fn voracious_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) { lsd_radixsort(arr, 8); }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 1_150_000, thread_n);
        }
    }
}

impl<T: Radixable<u64>> Dispatcher<T, u64> for u64 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_stable_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            msd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl<T: Radixable<usize>> Dispatcher<T, usize> for usize {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 200 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_stable_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort(arr, 8);
        } else {
            msd_stable_radixsort(arr, 8);
        }
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

impl<T: Radixable<u128>> Dispatcher<T, u128> for u128 {
    fn voracious_sort(&self, arr: &mut [T]) { voracious_sort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}

#[cfg(target_pointer_width = "128")]
impl<T: Radixable<usize>> Dispatcher<T, usize> for usize {
    fn voracious_sort(&self, arr: &mut [T]) { voracious_sort(arr, 8); }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
    }
    fn voracious_mt_sort(&self, arr: &mut [T], thread_n: usize) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            peeka_sort(arr, 8, 650_000, thread_n);
        }
    }
}
