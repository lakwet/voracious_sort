use super::super::{RadixKey, Radixable};

pub trait RadixSort<T: Radixable<K>, K: RadixKey> {
    fn voracious_sort(&mut self);
    fn voracious_stable_sort(&mut self);
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&mut self, thread_n: usize);
}

impl<T: Radixable<K>, K: RadixKey> RadixSort<T, K> for [T] {
    fn voracious_sort(&mut self) {
        if let Some(rest) = crate::traits::nan::set_nan_aside(self).filter(|r| !r.is_empty()) {
            let _checked = crate::traits::nan::Checked::enter();
            let dummy = rest[0];
            dummy.voracious_sort(rest);
        }
    }
    fn voracious_stable_sort(&mut self) {
        if let Some(rest) = crate::traits::nan::set_nan_aside(self).filter(|r| !r.is_empty()) {
            let _checked = crate::traits::nan::Checked::enter();
            let dummy = rest[0];
            dummy.voracious_stable_sort(rest);
        }
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&mut self, thread_n: usize) {
        if let Some(rest) = crate::traits::nan::set_nan_aside(self).filter(|r| !r.is_empty()) {
            let _checked = crate::traits::nan::Checked::enter();
            let dummy = rest[0];
            dummy.voracious_mt_sort(rest, thread_n);
        }
    }
}

impl<T: Radixable<K>, K: RadixKey> RadixSort<T, K> for Vec<T> {
    fn voracious_sort(&mut self) { self.as_mut_slice().voracious_sort(); }
    fn voracious_stable_sort(&mut self) {
        self.as_mut_slice().voracious_stable_sort();
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&mut self, thread_n: usize) {
        self.as_mut_slice().voracious_mt_sort(thread_n);
    }
}
