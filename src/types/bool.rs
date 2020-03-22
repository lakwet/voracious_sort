use super::super::sorts::boolean_sort::boolean_sort;
use super::super::Radixable;

impl Radixable<bool> for bool {
    type Key = bool;

    #[inline]
    fn key(&self) -> bool { *self }
    #[inline]
    fn to_generic(&self, value: usize) -> Self { value == 1 }
    fn voracious_sort(&self, arr: &mut [bool]) { boolean_sort(arr); }
    fn voracious_stable_sort(&self, arr: &mut [bool]) { boolean_sort(arr); }
    fn voracious_mt_sort(&self, arr: &mut [bool], _thread_n: usize) {
        boolean_sort(arr);
    }
}
