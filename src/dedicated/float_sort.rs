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
//! 3. repair of nearly sorted arrays (a few misplaced elements set aside,
//!    sorted and inserted back), see `near_sorted`;
//! 4. Verge sort pre-processing (already sorted parts are not radix sorted;
//!    the faster `verge_sort_fast` and `k_way_merge_fast` variants),
//!    then each unsorted chunk is radix sorted: f32 with the fused LSD of
//!    `float_lsd` (the encoding is fused into the first scatter pass, the
//!    decoding into the last one); f64 with the DLSD of `float_dlsd`, or
//!    from `ROLLER_F64` keys the float rollercoaster of `float_roller`
//!    (in-place MSD passes, then the DLSD on cache-sized buckets);
//! 5. when merging the segments would cost more than radix sorting the
//!    whole array (several segments with unpredictable interleavings), the
//!    whole array is radix sorted instead.
//!
//! The arrays are only reinterpreted (`f32` <-> `u32`, `f64` <-> `u64`), never
//! copied, so bit patterns are preserved.
//!
//! The Verge pre-processing compares floats with `<`: arrays containing NaN
//! are not supported yet (it may panic), as for the generic float sorts.

use super::super::algo::k_way_merge_fast::{k_way_merge_fast, predictable_merge};
use super::super::algo::near_sorted::repair_near_sorted;
use super::super::algo::verge_sort_fast::verge_sort_preprocessing_fast;
use super::float_detect::{detect32, detect64};
use super::float_dlsd::dlsd64_with;
use super::float_lsd::w32;
#[cfg(test)]
use super::float_lsd::w64;
use super::float_roller::roller64;
#[cfg(test)]
use super::float_lsd::Digits;

/// Below these sizes, the whole array is sorted with `sort_unstable` on the
/// encoded keys: where the median over the distributions of radix path /
/// small path crosses 1 (f32 against the fused LSD; f64 against the DLSD:
/// 1.13 at 600, 1.00 at 800, 0.94 at 1000, Snapdragon X Elite).
const SMALL_F32: usize = 600;
const SMALL_F64: usize = 800;
/// From this chunk size f64 is sorted by the float rollercoaster instead of
/// the DLSD alone: about where the DLSD's keys and scratch buffer (16 bytes
/// per key) outgrow the L2 and its prefix needs a third pass. Measured
/// crossover between 1M (DLSD 2.63 vs 2.68 ns, geometric mean over 36
/// distributions) and 1.25M (3.33 vs 2.98) on Snapdragon X Elite (12 MiB
/// L2); machine dependent, to derive from the L2 size.
const ROLLER_F64: usize = 1 << 20;

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
        #[cfg(test)]
        let small = match F64_SMALL_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
            0 => SMALL_F64,
            v => v,
        };
        #[cfg(not(test))]
        let small = SMALL_F64;
        if arr.len() <= small {
            small64(bits64(arr));
            return;
        }
        float_sort_f64_radix(arr);
    }
}

/// Radix path shared by f32 and f64: repair of nearly sorted arrays, else
/// the Verge pre-processing records the unsorted chunks; when merging the
/// segments would cost more than sorting the whole array (several segments
/// with unpredictable interleavings), `sort` is applied to the whole array,
/// else to each chunk and the segments are merged.
fn radix_path<T: Copy + PartialOrd>(arr: &mut [T], sort: &dyn Fn(&mut [T])) {
    if repair_near_sorted(arr) {
        return;
    }
    let base = arr.as_ptr() as usize;
    let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [T], _| {
        let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<T>();
        chunks.borrow_mut().push((start, chunk.len()));
    });
    let chunks = chunks.into_inner();
    if merge_rule_enabled() && unpredictable_merges(arr, &separators, &chunks) {
        sort(arr);
        return;
    }
    for &(start, len) in chunks.iter() {
        sort(&mut arr[start..start + len]);
    }
    k_way_merge_fast(arr, &mut separators);
}

/// f32 radix path (see `radix_path`), the fused LSD sorting the chunks.
pub(crate) fn float_sort_f32_radix(arr: &mut [f32]) {
    radix_path(arr, &|s: &mut [f32]| w32::lsd(bits32(s)));
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

/// f64 radix path (see `radix_path`): the DLSD sorts the chunks (one scratch
/// buffer for all of them), the float rollercoaster the large ones.
pub(crate) fn float_sort_f64_radix(arr: &mut [f64]) {
    let scratch = std::cell::RefCell::new(Vec::new());
    radix_path(arr, &|s: &mut [f64]| {
        #[cfg(test)]
        match F64_CHUNK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            1 => return dlsd64_with(bits64(s), &mut scratch.borrow_mut()),
            2 => return w64::lsd(bits64(s)),
            3 => return roller64(bits64(s)),
            _ => {}
        }
        if s.len() >= ROLLER_F64 {
            roller64(bits64(s));
        } else {
            dlsd64_with(bits64(s), &mut scratch.borrow_mut());
        }
    });
}

/// Tests / benchmarks: f64 small-path limit override (0: `SMALL_F64`; 1:
/// radix path from 2 keys).
#[cfg(test)]
pub(crate) static F64_SMALL_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Tests / benchmarks: f64 chunk sort forced (0 default, 1 DLSD, 2 fused
/// LSD, 3 rollercoaster).
#[cfg(test)]
pub(crate) static F64_CHUNK_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// f64 radix path with a forced digit layout, fused LSD only (tests).
#[cfg(test)]
pub(crate) fn float_sort_f64_radix_mode(arr: &mut [f64], mode: Digits) {
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f64], _| {
        w64::lsd_mode(bits64(chunk), mode)
    });
    k_way_merge_fast(arr, &mut separators);
}

