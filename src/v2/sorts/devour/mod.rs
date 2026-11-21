//! # Devour sort: radix sort for integers (and `char`; `bool`: count and fills)
//!
//! The public entry and its trait; each type sorts through its own decision
//! chain in `types` (`types::u32`, `types::i64`...). Shared here: the key
//! encodings (`keys`) and the 128-bit helpers (`wide128`).

pub(crate) mod keys;
pub(crate) mod wide128;

mod sealed {
    pub trait Sealed {}
    macro_rules! sealed {
        ($($t:ty),*) => { $(impl Sealed for $t {})* };
    }
    sealed!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char, bool);
}

/// Integer types, `char` and `bool`, sortable by [`devour_sort`].
pub trait DevourSort: sealed::Sealed + Copy + Ord {
    #[doc(hidden)]
    /// Sorts `arr` by the chain of the type.
    fn devour_sort_impl(arr: &mut [Self]);
}

/// Sorts slices of `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`,
/// `i32`, `i64`, `i128`, `isize`, `char` or `bool`.
///
/// ```
/// use voracious_radix_sort::devour_sort;
///
/// let mut v = vec![5i64, -3, 12, 0, -40];
/// devour_sort(&mut v);
/// assert_eq!(v, vec![-40, -3, 0, 5, 12]);
/// ```
#[inline]
pub fn devour_sort<T: DevourSort>(arr: &mut [T]) { T::devour_sort_impl(arr); }

/// Each type: its own chain (`types`).
macro_rules! own_chain {
    ($($t:ident),*) => {$(
        impl DevourSort for $t {
            #[inline]
            fn devour_sort_impl(arr: &mut [$t]) {
                crate::v2::types::$t::sort(arr);
            }
        }
    )*};
}
own_chain!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, char, bool);

/// `usize` / `isize`: sorted as the integer of the same width.
macro_rules! pointer_width {
    ($t:ty, $as:ty) => {
        impl DevourSort for $t {
            #[inline]
            fn devour_sort_impl(arr: &mut [$t]) {
                const _: () = assert!(std::mem::size_of::<$t>() == std::mem::size_of::<$as>());
                // SAFETY: same size, alignment and value representation.
                let same = unsafe {
                    std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $as, arr.len())
                };
                <$as as DevourSort>::devour_sort_impl(same);
            }
        }
    };
}

#[cfg(target_pointer_width = "64")]
pointer_width!(usize, u64);
#[cfg(target_pointer_width = "64")]
pointer_width!(isize, i64);
#[cfg(target_pointer_width = "32")]
pointer_width!(usize, u32);
#[cfg(target_pointer_width = "32")]
pointer_width!(isize, i32);
#[cfg(target_pointer_width = "16")]
pointer_width!(usize, u16);
#[cfg(target_pointer_width = "16")]
pointer_width!(isize, i16);
