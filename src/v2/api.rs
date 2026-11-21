//! The public trait of 2.0: `voracious_sort` and its variants as methods of
//! slices (and of `Vec` through `Deref`).

use crate::v2::sorts::by_key::{devour_sort_by_key, devour_sort_unstable_by_key, SortKey};

mod sealed {
    pub trait Sealed {}
    macro_rules! sealed {
        ($($t:ty),*) => { $(impl Sealed for $t {})* };
    }
    sealed!(bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
}

/// The element types `voracious_sort` sorts directly: `bool`, `char`, every
/// integer type and `f32` / `f64` (floats in [`f64::total_cmp`] order).
/// Other types: [`RadixSort::voracious_sort_by_key`].
pub trait Sortable: sealed::Sealed + Copy {
    #[doc(hidden)]
    fn sort_impl(arr: &mut [Self]);

    #[doc(hidden)]
    #[cfg(feature = "voracious_multithread")]
    fn mt_sort_impl(arr: &mut [Self], thread_n: usize);
}

macro_rules! sortable {
    ($sort:path, $($t:ty),*) => {$(
        impl Sortable for $t {
            #[inline]
            fn sort_impl(arr: &mut [$t]) { $sort(arr) }

            #[cfg(feature = "voracious_multithread")]
            fn mt_sort_impl(arr: &mut [$t], thread_n: usize) {
                crate::v1::RadixSort::voracious_mt_sort(arr, thread_n)
            }
        }
    )*};
}
sortable!(
    crate::v2::sorts::devour::devour_sort,
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize
);
sortable!(crate::v2::sorts::ravenous::ravenous_sort, f32, f64);

/// Radix sorts of a slice.
///
/// ```
/// use voracious_radix_sort::RadixSort;
///
/// let mut v = vec![5u32, 3, 12, 0, 40];
/// v.voracious_sort();
/// assert_eq!(v, vec![0, 3, 5, 12, 40]);
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct Event { time: i64, id: u32 }
/// let mut events = vec![Event { time: 7, id: 1 }, Event { time: -2, id: 2 }];
/// events.voracious_sort_by_key(|e| e.time);
/// assert_eq!(events[0].id, 2);
/// ```
pub trait RadixSort<T> {
    /// Sorts the slice (`bool`, `char`, integers, `f32`, `f64`). For these
    /// types equal elements are indistinguishable, so the sort is also
    /// stable; floats are put in [`f64::total_cmp`] order.
    fn voracious_sort(&mut self)
    where
        T: Sortable;

    /// The same as [`RadixSort::voracious_sort`] (kept from 1.x).
    fn voracious_stable_sort(&mut self)
    where
        T: Sortable;

    /// Sorts the slice on `thread_n` threads (the 1.x multithread sort,
    /// peeka sort, not reworked yet in 2.0).
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&mut self, thread_n: usize)
    where
        T: Sortable;

    /// Sorts the elements by `key`, stably: elements with equal keys keep
    /// their order. Keys: `bool`, `char`, integers, `f32`, `f64` (or any
    /// type implementing [`SortKey`]).
    fn voracious_sort_by_key<K: SortKey, F: Fn(&T) -> K>(&mut self, key: F);

    /// Sorts the elements by `key`; elements with equal keys may come out
    /// in any order (faster on some inputs).
    fn voracious_sort_unstable_by_key<K: SortKey, F: Fn(&T) -> K>(&mut self, key: F);
}

impl<T> RadixSort<T> for [T] {
    #[inline]
    fn voracious_sort(&mut self)
    where
        T: Sortable,
    {
        T::sort_impl(self)
    }

    #[inline]
    fn voracious_stable_sort(&mut self)
    where
        T: Sortable,
    {
        T::sort_impl(self)
    }

    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&mut self, thread_n: usize)
    where
        T: Sortable,
    {
        T::mt_sort_impl(self, thread_n)
    }

    #[inline]
    fn voracious_sort_by_key<K: SortKey, F: Fn(&T) -> K>(&mut self, key: F) {
        devour_sort_by_key(self, key)
    }

    #[inline]
    fn voracious_sort_unstable_by_key<K: SortKey, F: Fn(&T) -> K>(&mut self, key: F) {
        devour_sort_unstable_by_key(self, key)
    }
}
