//! NaN in the float sorts: set aside before sorting.
//!
//! The radix passes order floats by their key (`into_key_type`), a total
//! order from -NaN to +NaN. The comparison parts of the sorts (small-array
//! fallbacks, Verge, k-way merges) use `PartialOrd`, which answers nothing
//! on a NaN: they panicked. One vectorised pass finds whether some element
//! compares to nothing and whether the array is in order; NaNs are moved to
//! the ends (negative ones first, positive ones last, each group by its
//! bits, stably) and only the rest is sorted. Arrays without NaN are sorted
//! exactly as before; arrays in order are left at once. Float keys only, and
//! once per sort (the sorts called inside skip the pass).

use std::cell::Cell;
use std::cmp::Ordering;

use super::super::{RadixKey, Radixable};

thread_local! {
    /// Set while a sort runs on keys already checked on this thread: the
    /// sorts it calls (the dispatcher's choice, recursive calls) do not read
    /// the array again.
    static CHECKED: Cell<bool> = const { Cell::new(false) };
}

/// Marks the keys as checked on this thread until dropped.
pub(crate) struct Checked(bool);

impl Checked {
    #[inline]
    pub(crate) fn enter() -> Self { Checked(CHECKED.with(|c| c.replace(true))) }
}

impl Drop for Checked {
    #[inline]
    fn drop(&mut self) { CHECKED.with(|c| c.set(self.0)); }
}

/// One read: whether some element compares to nothing, not even itself (a
/// NaN key), and whether the array is already in order (`<=` fails on a
/// NaN: in order means no NaN).
#[inline]
fn scan<T: PartialOrd>(arr: &[T]) -> (bool, bool) {
    // No early exit: both tests over every pair, vectorised.
    let (nan, sorted) = arr
        .iter()
        .zip(arr.iter().skip(1))
        .fold((false, true), |(nan, sorted), (a, b)| (nan | a.partial_cmp(a).is_none(), sorted & (a <= b)));
    let nan = nan | arr.last().map_or(false, |x| x.partial_cmp(x).is_none());
    (nan, sorted && !nan)
}

/// Order of the radix keys (integers: always comparable).
fn by_radix_key<T: Radixable<K>, K: RadixKey>(a: &T, b: &T) -> Ordering {
    let (x, y) = (a.into_key_type(), b.into_key_type());
    if x < y {
        Ordering::Less
    } else if x > y {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// What is left to sort: `None` when the array is already in order, else
/// the array with its NaNs moved to the ends in key order (stably) and the
/// rest returned (the whole array when there is no NaN).
///
/// Integer keys (anything but `f32` / `f64`) are returned at once: they
/// cannot be NaN and pay nothing.
#[inline]
pub(crate) fn set_nan_aside<T: Radixable<K>, K: RadixKey>(arr: &mut [T]) -> Option<&mut [T]> {
    if !matches!(std::any::type_name::<K>(), "f32" | "f64") || CHECKED.with(|c| c.get()) {
        return Some(arr);
    }
    let (nan, sorted) = scan(arr);
    if sorted {
        None
    } else if nan {
        Some(split_nan(arr))
    } else {
        Some(arr)
    }
}

#[cold]
#[inline(never)]
fn split_nan<T: Radixable<K>, K: RadixKey>(arr: &mut [T]) -> &mut [T] {
    let x = arr[0];
    // The key flips the sign bit of positive floats and every bit of
    // negative ones: a negative NaN has the top bit of its key clear.
    let top = x.usize_to_keytype(1) << x.usize_to_keytype(x.type_size() - 1);
    let zero = x.usize_to_keytype(0);
    let (mut neg, mut mid, mut pos) = (Vec::new(), Vec::with_capacity(arr.len()), Vec::new());
    for &e in arr.iter() {
        if e.partial_cmp(&e).is_some() {
            mid.push(e);
        } else if e.into_key_type() & top == zero {
            neg.push(e);
        } else {
            pos.push(e);
        }
    }
    neg.sort_by(by_radix_key);
    pos.sort_by(by_radix_key);
    let (a, b) = (neg.len(), neg.len() + mid.len());
    arr[..a].copy_from_slice(&neg);
    arr[a..b].copy_from_slice(&mid);
    arr[b..].copy_from_slice(&pos);
    &mut arr[a..b]
}
