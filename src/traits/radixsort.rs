use super::super::{Radixable, RadixKey};

pub trait RadixSort<T, K>
where
    T: Radixable<K>,
    K: RadixKey,
{
    fn voracious_sort(&mut self);
    fn dlsd_sort(&mut self);
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
    fn dlsd_sort(&mut self) {
        if !self.is_empty() {
            let dummy = self[0];
            dummy.dlsd_sort(self);
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
    fn dlsd_sort(&mut self) {
        self.as_mut_slice().dlsd_sort();
    }
}
