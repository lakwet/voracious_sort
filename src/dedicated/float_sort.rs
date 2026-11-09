//! # Float sort
//!
//! Dedicated radix sort for `f32` and `f64`.
//!
//! Floats are sorted through their bits, with the monotone encoding
//! `key = bits ^ (sign ? 0xFF..FF : 0x80..00)`: comparing the encoded keys as
//! unsigned integers gives the [`f32::total_cmp`] / [`f64::total_cmp`] order
//! (`-0.0 < +0.0`, NaNs at both ends according to their sign).
//!
//! Steps:
//! 1. low-cardinality detectors (integer range, power-of-two grid, few
//!    values), see `float_detect`;
//! 2. small arrays: encoded keys sorted with `sort_unstable`;
//! 3. Verge sort pre-processing (already sorted parts are not radix sorted),
//!    then on each unsorted chunk the fused LSD of `float_lsd` (the encoding
//!    is fused into the first scatter pass, the decoding into the last one),
//!    or for very large f64 chunks rollercoaster on the encoded keys.
//!
//! The arrays are only reinterpreted (`f32` <-> `u32`, `f64` <-> `u64`), never
//! copied, so bit patterns are preserved.
//!
//! The Verge pre-processing compares floats with `<`: arrays containing NaN
//! are not supported yet (it may panic), as for the generic float sorts.

use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::sorts::rollercoaster_sort::rollercoaster_sort;
use super::float_detect::{detect32, detect64};
use super::float_lsd::{w32, w64};
#[cfg(test)]
use super::float_lsd::Digits;

/// Below these sizes, the whole array is sorted with `sort_unstable` on the
/// encoded keys.
const SMALL_F32: usize = 600;
const SMALL_F64: usize = 3_000;
/// From this chunk size f64 uses rollercoaster instead of the fused LSD.
const ROLLER_F64: usize = 2_000_000;

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// Float types sortable by [`float_sort`].
pub trait FloatSort: sealed::Sealed + Copy + PartialOrd {
    #[doc(hidden)]
    fn float_sort_impl(arr: &mut [Self]);
}

/// # Float sort
///
/// Sorts `f32` or `f64` slices in [`f64::total_cmp`] order, with a dedicated
/// radix sort (the float to integer conversion is fused into the first and
/// last radix passes).
///
/// ```
/// use voracious_radix_sort::float_sort;
///
/// let mut v = vec![1.5f64, -0.5, 3.0, -2.25];
/// float_sort(&mut v);
/// assert_eq!(v, vec![-2.25, -0.5, 1.5, 3.0]);
/// ```
pub fn float_sort<T: FloatSort>(arr: &mut [T]) {
    T::float_sort_impl(arr);
}

macro_rules! float_width {
    ($float:ty, $uint:ty, $sint:ty, $bits:ident, $encode:ident, $decode:ident, $small:ident) => {
        /// Reinterprets the floats as their raw bits.
        #[inline(always)]
        fn $bits(arr: &mut [$float]) -> &mut [$uint] {
            // SAFETY: the float and the unsigned integer have the same size
            // and alignment, every bit pattern is valid for both, and the
            // returned slice reborrows `arr` exclusively for its lifetime.
            unsafe {
                std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $uint, arr.len())
            }
        }

        #[inline(always)]
        fn $encode(bits: $uint) -> $uint {
            const SIGN: $uint = 1 << (<$uint>::BITS - 1);
            bits ^ ((((bits as $sint) >> (<$uint>::BITS - 1)) as $uint) | SIGN)
        }

        #[inline(always)]
        fn $decode(key: $uint) -> $uint {
            const SIGN: $uint = 1 << (<$uint>::BITS - 1);
            key ^ ((key >> (<$uint>::BITS - 1)).wrapping_sub(1) | SIGN)
        }

        /// Small arrays: encode, `sort_unstable` on integers, decode.
        fn $small(bits: &mut [$uint]) {
            bits.iter_mut().for_each(|b| *b = $encode(*b));
            bits.sort_unstable();
            bits.iter_mut().for_each(|b| *b = $decode(*b));
        }
    };
}

float_width!(f32, u32, i32, bits32, encode32, decode32, small32);
float_width!(f64, u64, i64, bits64, encode64, decode64, small64);

impl FloatSort for f32 {
    fn float_sort_impl(arr: &mut [f32]) {
        if detect32(arr) {
            return;
        }
        if arr.len() <= SMALL_F32 {
            small32(bits32(arr));
            return;
        }
        float_sort_f32_radix(arr);
    }
}

impl FloatSort for f64 {
    fn float_sort_impl(arr: &mut [f64]) {
        if detect64(arr) {
            return;
        }
        if arr.len() <= SMALL_F64 {
            small64(bits64(arr));
            return;
        }
        float_sort_f64_radix(arr);
    }
}

/// f32 radix path: Verge pre-processing, then the fused LSD on each chunk.
pub(crate) fn float_sort_f32_radix(arr: &mut [f32]) {
    let mut separators =
        verge_sort_preprocessing(arr, 8, &|chunk: &mut [f32], _| w32::lsd(bits32(chunk)));
    k_way_merge(arr, &mut separators);
}

/// f32 radix path with a forced digit layout (tests).
#[cfg(test)]
pub(crate) fn float_sort_f32_radix_mode(arr: &mut [f32], mode: Digits) {
    let mut separators =
        verge_sort_preprocessing(arr, 8, &|chunk: &mut [f32], _| w32::lsd_mode(bits32(chunk), mode));
    k_way_merge(arr, &mut separators);
}

/// f64 radix path: Verge pre-processing, then on each chunk the fused LSD,
/// or rollercoaster on the encoded keys for very large chunks.
pub(crate) fn float_sort_f64_radix(arr: &mut [f64]) {
    let mut separators = verge_sort_preprocessing(arr, 8, &|chunk: &mut [f64], _| {
        if chunk.len() < ROLLER_F64 {
            w64::lsd(bits64(chunk));
        } else {
            roller64(chunk);
        }
    });
    k_way_merge(arr, &mut separators);
}

/// f64 radix path with a forced digit layout, fused LSD only (tests).
#[cfg(test)]
pub(crate) fn float_sort_f64_radix_mode(arr: &mut [f64], mode: Digits) {
    let mut separators =
        verge_sort_preprocessing(arr, 8, &|chunk: &mut [f64], _| w64::lsd_mode(bits64(chunk), mode));
    k_way_merge(arr, &mut separators);
}

/// Rollercoaster on the encoded keys (one encode and one decode pass).
fn roller64(chunk: &mut [f64]) {
    let keys = bits64(chunk);
    keys.iter_mut().for_each(|b| *b = encode64(*b));
    rollercoaster_sort(keys, 8);
    keys.iter_mut().for_each(|b| *b = decode64(*b));
}
