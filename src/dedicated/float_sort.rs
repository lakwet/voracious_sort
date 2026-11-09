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
//! 3. f32: repair of nearly sorted arrays (a few misplaced elements set
//!    aside, sorted and inserted back), see `near_sorted`;
//! 4. Verge sort pre-processing (already sorted parts are not radix sorted;
//!    for f32 the faster `verge_sort_fast` and `k_way_merge_fast` variants),
//!    then on each unsorted chunk the fused LSD of `float_lsd` (the encoding
//!    is fused into the first scatter pass, the decoding into the last one),
//!    or for very large f64 chunks rollercoaster on the encoded keys;
//! 5. f32: when merging the segments would cost more than radix sorting the
//!    whole array (several segments with unpredictable interleavings), the
//!    whole array is radix sorted instead.
//!
//! The arrays are only reinterpreted (`f32` <-> `u32`, `f64` <-> `u64`), never
//! copied, so bit patterns are preserved.
//!
//! The Verge pre-processing compares floats with `<`: arrays containing NaN
//! are not supported yet (it may panic), as for the generic float sorts.

use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::k_way_merge_fast::{k_way_merge_fast, predictable_merge};
use super::super::algo::near_sorted::repair_near_sorted;
use super::super::algo::verge_sort_fast::verge_sort_preprocessing_fast;
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

/// f32 radix path: repair of nearly sorted arrays, else Verge
/// pre-processing, then the fused LSD on each chunk.
pub(crate) fn float_sort_f32_radix(arr: &mut [f32]) {
    if repair_near_sorted(arr) {
        return;
    }
    float_sort_f32_radix_norepair(arr);
}

/// f32 radix path without the near-sorted repair.
pub(crate) fn float_sort_f32_radix_norepair(arr: &mut [f32]) {
    let base = arr.as_ptr() as usize;
    let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    // The Verge pre-processing only records the unsorted chunks: whether to
    // sort them depends on the merge decision below.
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f32], _| {
        let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<f32>();
        chunks.borrow_mut().push((start, chunk.len()));
    });
    let chunks = chunks.into_inner();
    if merge_rule_enabled() && unpredictable_merges(arr, &separators, &chunks) {
        w32::lsd(bits32(arr));
        return;
    }
    for &(start, len) in chunks.iter() {
        w32::lsd(bits32(&mut arr[start..start + len]));
    }
    k_way_merge_fast(arr, &mut separators);
}

/// Tests / benchmarks: always merge the Verge segments (A/B comparisons).
#[cfg(test)]
pub(crate) static NO_MERGE_RULE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn merge_rule_enabled() -> bool {
    #[cfg(test)]
    {
        !NO_MERGE_RULE.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Whether merging the Verge segments would cost more than radix sorting
/// the whole array: at least 3 segments (2 merge rounds or more) and most
/// first-round merges between two runs unpredictable for a branchy merge. A
/// merge round with an irregular interleaving (e.g. concatenated sorted
/// sources with random values) costs about as much as a whole radix sort,
/// while a predictable one (long stretches, runs holding the same values)
/// costs much less. Chunks are not sorted yet, so only run-run pairs are
/// sampled.
fn unpredictable_merges<T: Copy + PartialOrd>(
    arr: &[T],
    separators: &[usize],
    chunks: &[(usize, usize)],
) -> bool {
    if separators.len() < 4 {
        return false;
    }
    let is_chunk = |s: usize| chunks.iter().any(|&(c, _)| c == s);
    let (mut sampled, mut unpredictable) = (0, 0);
    for w in separators.windows(3).step_by(2) {
        let (s, m, e) = (w[0], w[1], w[2]);
        if is_chunk(s) || is_chunk(m) || arr[m - 1] <= arr[m] {
            continue;
        }
        sampled += 1;
        if !predictable_merge(&arr[s..e], m - s) {
            unpredictable += 1;
        }
    }
    sampled > 0 && 2 * unpredictable > sampled
}

/// f32 radix path with a forced digit layout (tests).
#[cfg(test)]
pub(crate) fn float_sort_f32_radix_mode(arr: &mut [f32], mode: Digits) {
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f32], _| {
        w32::lsd_mode(bits32(chunk), mode)
    });
    k_way_merge_fast(arr, &mut separators);
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
