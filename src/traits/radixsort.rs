use super::super::{RadixKey, Radixable};

pub trait RadixSort<T, K>
where
    T: Radixable<K>,
    K: RadixKey,
{
    fn voracious_sort(&mut self);
    fn voracious_stable_sort(&mut self);
}

impl<T, K> RadixSort<T, K> for [T]
where
    T: Radixable<K>,
    K: RadixKey,
{
    fn voracious_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_sort(self);
        }
    }
    fn voracious_stable_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.voracious_sort(self);
        }
    }
}

impl<T, K> RadixSort<T, K> for Vec<T>
where
    T: Radixable<K>,
    K: RadixKey,
{
    fn voracious_sort(&mut self) {
        self.as_mut_slice().voracious_sort();
    }
    fn voracious_stable_sort(&mut self) {
        self.as_mut_slice().voracious_sort();
    }
}
