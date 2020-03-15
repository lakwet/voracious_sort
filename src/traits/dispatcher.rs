use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::msd_stable_sort::msd_stable_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::super::{RadixKey, Radixable};

pub trait Dispatcher<T: Radixable<K>, K: RadixKey> {
    fn voracious_sort(&self, arr: &mut [T]);
    fn voracious_stable_sort(&self, arr: &mut [T]);
}

impl<T: Radixable<bool>> Dispatcher<T, bool> for bool {
    fn voracious_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
    }
}

impl<T: Radixable<char>> Dispatcher<T, char> for char {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 256 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 256 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
}

impl<T: Radixable<f32>> Dispatcher<T, f32> for f32 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        if arr.len() <= 300 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
}

impl<T: Radixable<f64>> Dispatcher<T, f64> for f64 {
    fn voracious_sort(&self, arr: &mut [T]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
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
}

impl<T: Radixable<i32>> Dispatcher<T, i32> for i32 {
    fn voracious_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
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
}

impl<T: Radixable<i128>> Dispatcher<T, i128> for i128 {
    fn voracious_sort(&self, arr: &mut [T]) {
        voracious_sort(arr, 8);
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
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
}

impl<T: Radixable<u32>> Dispatcher<T, u32> for u32 {
    fn voracious_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        lsd_radixsort(arr, 8);
    }
}

impl<T: Radixable<u64>> Dispatcher<T, u64> for u64 {
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
}

impl<T: Radixable<u128>> Dispatcher<T, u128> for u128 {
    fn voracious_sort(&self, arr: &mut [T]) {
        voracious_sort(arr, 8);
    }
    fn voracious_stable_sort(&self, arr: &mut [T]) {
        msd_stable_radixsort(arr, 8);
    }
}
