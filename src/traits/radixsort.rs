use super::super::{RadixKey, Radixable};

pub trait RadixSort<T: Radixable<K>, K: RadixKey> {
    fn voracious_sort(&mut self);
    fn voracious_stable_sort(&mut self);
    fn voracious_mt_sort(&mut self, thread_n: usize);
}

impl<T: Radixable<K>, K: RadixKey> RadixSort<T, K> for [T] {
    fn voracious_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_sort(self);
        }
    }
    fn voracious_stable_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_stable_sort(self);
        }
    }
    fn voracious_mt_sort(&mut self, thread_n: usize) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_mt_sort(self, thread_n);
        }
    }
}

impl<T: Radixable<K>, K: RadixKey> RadixSort<T, K> for Vec<T> {
    fn voracious_sort(&mut self) { self.as_mut_slice().voracious_sort(); }
    fn voracious_stable_sort(&mut self) {
        self.as_mut_slice().voracious_sort();
    }
    fn voracious_mt_sort(&mut self, thread_n: usize) {
        self.as_mut_slice().voracious_mt_sort(thread_n);
    }
}
