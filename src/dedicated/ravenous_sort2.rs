//! # Ravenous sort 2 (algorithmic work copy of `ravenous_sort`, see the journal)
//!
//! Dedicated radix sort for `f32` and `f64`.
//!
//! Floats are sorted through their bits, with the monotone encoding
//! `key = bits ^ (sign ? 0xFF..FF : 0x80..00)`: comparing the encoded keys as
//! unsigned integers gives the [`f32::total_cmp`] / [`f64::total_cmp`] order
//! (`-0.0 < +0.0`, NaNs at both ends according to their sign).
//!
//! Steps:
//! 0. at most 32 keys: a sorting network (`small_sort`);
//! 1. low-cardinality detectors (integer range, power-of-two grid), see
//!    `float_detect`;
//! 2. small arrays: encoded keys sorted with `sort_unstable` (blocks of 32
//!    and a merge up to 64 keys);
//! 3. repair of nearly sorted arrays (a few misplaced elements set aside,
//!    sorted and inserted back), see `near_sorted`; then few distinct
//!    values (at most 256, counted in a hash table), see `float_detect`;
//! 4. Verge sort pre-processing (already sorted parts are not radix sorted;
//!    the faster `verge_sort_fast` and `k_way_merge_fast` variants),
//!    then each unsorted chunk is radix sorted: f32 with the fused LSD of
//!    `float_lsd` (the encoding is fused into the first scatter pass, the
//!    decoding into the last one), f64 with the DLSD of `float_dlsd`; from
//!    a size derived from the L2 size with the float rollercoaster of
//!    `float_roller` (in-place MSD passes, then the DLSD on cache-sized
//!    buckets);
//! 5. when merging the segments would cost more than radix sorting the
//!    whole array (several segments with unpredictable interleavings), the
//!    whole array is radix sorted instead.
//!
//! The arrays are only reinterpreted (`f32` <-> `u32`, `f64` <-> `u64`), never
//! copied, so bit patterns are preserved.
//!
//! The Verge pre-processing compares floats with `<`: arrays containing NaN
//! are not supported yet (it may panic), as for the generic float sorts.

use super::super::algo::k_way_merge_fast::predictable_merge;
use super::super::algo::k_way_merge_fast2::k_way_merge_fast2;
use super::super::algo::near_sorted::repair_near_sorted;
use super::super::algo::verge_sort_fast::verge_sort_preprocessing_fast;
use super::devour_sort2::heavy_by;
use super::float_detect::{detect32, detect64, few32, few64};
use super::float_dlsd::dlsd64_with;
use super::float_lsd::w32;
#[cfg(test)]
use super::float_lsd::w64;
use super::float_roller::{roller32, roller64};
use super::small_sort::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
#[cfg(test)]
use super::float_lsd::Digits;

/// Below these sizes, the whole array is sorted with `sort_unstable` on the
/// encoded keys: where the median over the distributions of radix path /
/// small path crosses 1 (f32 against the fused LSD: 1.04 at 300, 0.94 at
/// 400; f64 against the DLSD: 1.05 at 600, 0.96 at 800, 0.89 at 1000,
/// Snapdragon X Elite).
const SMALL_F32: usize = 350;
const SMALL_F64: usize = 700;
/// From this chunk size (keys) f64 is sorted by the float rollercoaster
/// (in-place MSD pass, then the DLSD on leaves of 2^15 keys) instead of the
/// DLSD alone: L2 / 48, where the DLSD's keys and scratch buffer (16 bytes
/// per key) take a third of the L2 and its prefix needs 3 passes. Measured
/// on Snapdragon X Elite (12 MiB L2: 256k keys), rollercoaster / DLSD over
/// 36 distributions: 1.09 at 128k, 1.00 at 256k, 0.95 at 512k, 0.91 at 1M.
pub(crate) fn roller_f64() -> usize {
    super::cache_info::l2_bytes() / 48
}

/// From this chunk size (keys) f32 is sorted by the float rollercoaster
/// instead of the fused LSD (on 32-bit keys the DLSD alone does not beat the
/// LSD: its prefix needs as many passes): L2 / 12. Measured on Snapdragon X
/// Elite (12 MiB L2: 1M keys), rollercoaster / LSD over 31 distributions:
/// 1.13 at 512k, 1.00 at 1M, 0.73 from 2M to 8M.
pub(crate) fn roller_f32() -> usize {
    super::cache_info::l2_bytes() / 12
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// Float types sortable by [`ravenous_sort2`].
pub trait RavenousSort2: sealed::Sealed + Copy + PartialOrd {
    #[doc(hidden)]
    fn ravenous_sort2_impl(arr: &mut [Self]);
}

/// # Ravenous sort
///
/// Sorts `f32` or `f64` slices in [`f64::total_cmp`] order, with a dedicated
/// radix sort (the float to integer conversion is fused into the first and
/// last radix passes).
///
/// ```text
/// use voracious_radix_sort::ravenous_sort2;
///
/// let mut v = vec![1.5f64, -0.5, 3.0, -2.25];
/// ravenous_sort2(&mut v);
/// assert_eq!(v, vec![-2.25, -0.5, 1.5, 3.0]);
/// ```
#[inline]
pub fn ravenous_sort2<T: RavenousSort2>(arr: &mut [T]) {
    T::ravenous_sort2_impl(arr);
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
            if bits.len() <= MERGE_MAX {
                merge_sort(bits, $encode, $decode);
                return;
            }
            bits.iter_mut().for_each(|b| *b = $encode(*b));
            bits.sort_unstable();
            bits.iter_mut().for_each(|b| *b = $decode(*b));
        }
    };
}

float_width!(f32, u32, i32, bits32, encode32, decode32, small32);
float_width!(f64, u64, i64, bits64, encode64, decode64, small64);

impl RavenousSort2 for f32 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f32]) {
        // Shortest arrays first: no other step pays off there.
        if arr.len() <= NETWORK_MAX {
            network_sort(bits32(arr), encode32, decode32);
            return;
        }
        ravenous_sort2_f32_rest(arr);
    }
}

/// f32 arrays longer than `NETWORK_MAX`: detectors, small path, radix path.
#[inline(never)]
fn ravenous_sort2_f32_rest(arr: &mut [f32]) {
    if detect32(arr) {
        return;
    }
    #[cfg(test)]
    let small = match F32_SMALL_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => SMALL_F32,
        v => v,
    };
    #[cfg(not(test))]
    let small = SMALL_F32;
    if arr.len() <= small {
        small32(bits32(arr));
        return;
    }
    ravenous_sort2_f32_radix(arr);
}

impl RavenousSort2 for f64 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f64]) {
        // Shortest arrays first: no other step pays off there.
        if arr.len() <= NETWORK_MAX {
            network_sort(bits64(arr), encode64, decode64);
            return;
        }
        ravenous_sort2_f64_rest(arr);
    }
}

/// f64 arrays longer than `NETWORK_MAX`: detectors, small path, radix path.
#[inline(never)]
fn ravenous_sort2_f64_rest(arr: &mut [f64]) {
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
    ravenous_sort2_f64_radix(arr);
}

/// Radix path shared by f32 and f64: repair of nearly sorted arrays, else
/// the Verge pre-processing records the unsorted chunks; when merging the
/// segments would cost more than sorting the whole array (several segments
/// with unpredictable interleavings), `sort` is applied to the whole array,
/// else to each chunk and the segments are merged.
pub(crate) fn radix_path<T: Copy + PartialOrd>(arr: &mut [T], few: &dyn Fn(&mut [T]) -> bool, sort: &dyn Fn(&mut [T])) {
    if repair_near_sorted(arr) || few(arr) {
        return;
    }
    if skip_verge(arr) {
        sort(arr);
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
    k_way_merge_fast2(arr, &mut separators);
}

/// Whether the Verge pre-processing can be skipped: it only keeps
/// monotone runs (non-decreasing or non-increasing) of at least `n /
/// log2(n)` keys, and without one it leaves the array untouched and sorts
/// it whole. Such a run contains a point of a grid of step half that length
/// followed by `RUN_PROBE` monotone pairs (the run is longer than the step
/// plus the window); if no grid point has them, there is no such run and
/// the result is the same without Verge. Random keys fail each probe within
/// a couple of pairs: about 2 log2(n) probes instead of Verge's
/// explorations and allocations (0.45 ns per key at 1k u64).
pub(crate) fn skip_verge<T: Copy + PartialOrd>(arr: &[T]) -> bool {
    #[cfg(test)]
    match VERGE_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => return false,
        2 => {
            let min = VERGE_MIN.load(std::sync::atomic::Ordering::Relaxed);
            if arr.len() < min {
                return true;
            }
        }
        _ => {}
    }
    const RUN_PROBE: usize = 8;
    let n = arr.len();
    let big_enough = super::super::algo::verge_sort_heuristic::compute_big_enough_run(n);
    let step = big_enough / 2;
    if step <= RUN_PROBE {
        return false;
    }
    let mut p = 0;
    while p + RUN_PROBE < n {
        let w = &arr[p..=p + RUN_PROBE];
        let asc = w.windows(2).all(|x| x[0] <= x[1]);
        let desc = w.windows(2).all(|x| x[0] >= x[1]);
        if asc || desc {
            return false;
        }
        p += step;
    }
    true
}

/// Tests / benchmarks: Verge skipping (0 default; 1 never skipped, as
/// before; 2 default plus skipped below `VERGE_MIN` keys).
#[cfg(test)]
pub(crate) static VERGE_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static VERGE_MIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// f32 radix path (see `radix_path`), the fused LSD sorting the chunks.
pub(crate) fn ravenous_sort2_f32_radix(arr: &mut [f32]) {
    radix_path(arr, &|a: &mut [f32]| few32(a) || heavy_by(a, &|r: &mut [f32]| ravenous_sort2(r), |x: f32, y: f32| x.to_bits() == y.to_bits(), |x: f32, y: f32| x.total_cmp(&y).is_lt()), &|s: &mut [f32]| {
        #[cfg(test)]
        match F32_CHUNK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            2 => return w32::lsd(bits32(s)),
            3 => return roller32(bits32(s)),
            _ => {}
        }
        if s.len() >= roller_f32() {
            roller32(bits32(s));
        } else {
            w32::lsd(bits32(s));
        }
    });
}

/// Tests / benchmarks: f32 chunk sort forced (0 default, 2 fused LSD, 3
/// rollercoaster).
#[cfg(test)]
pub(crate) static F32_CHUNK_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Tests / benchmarks: previous merge rule for keys of at most 16 bits.
#[cfg(test)]
pub(crate) static NARROW_MERGE_OLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    // Keys of at most 16 bits: the radix sort is 2 byte passes or one
    // count, cheaper than a single unpredictable merge round, so 2 segments
    // (1 round) are enough to check (u16 pipe organ 1k-100k: 3.0 to 1.5-2.0
    // ns per key).
    #[cfg(test)]
    let narrow = std::mem::size_of::<T>() <= 2 && !NARROW_MERGE_OLD.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
    let narrow = std::mem::size_of::<T>() <= 2;
    if separators.len() < if narrow { 3 } else { 4 } {
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
pub(crate) fn ravenous_sort2_f32_radix_mode(arr: &mut [f32], mode: Digits) {
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f32], _| {
        w32::lsd_mode(bits32(chunk), mode)
    });
    k_way_merge_fast2(arr, &mut separators);
}

/// f64 radix path (see `radix_path`): the DLSD sorts the chunks (one scratch
/// buffer for all of them), the float rollercoaster the large ones.
pub(crate) fn ravenous_sort2_f64_radix(arr: &mut [f64]) {
    let scratch = std::cell::RefCell::new(Vec::new());
    radix_path(arr, &|a: &mut [f64]| few64(a) || heavy_by(a, &|r: &mut [f64]| ravenous_sort2(r), |x: f64, y: f64| x.to_bits() == y.to_bits(), |x: f64, y: f64| x.total_cmp(&y).is_lt()), &|s: &mut [f64]| {
        #[cfg(test)]
        match F64_CHUNK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            1 => return dlsd64_with(bits64(s), &mut scratch.borrow_mut()),
            2 => return w64::lsd(bits64(s)),
            3 => return roller64(bits64(s)),
            _ => {}
        }
        if s.len() >= roller_f64() {
            roller64(bits64(s));
        } else {
            dlsd64_with(bits64(s), &mut scratch.borrow_mut());
        }
    });
}

/// Tests / benchmarks: f32 small-path limit override (0: `SMALL_F32`; 1:
/// radix path from 2 keys).
#[cfg(test)]
pub(crate) static F32_SMALL_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

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
pub(crate) fn ravenous_sort2_f64_radix_mode(arr: &mut [f64], mode: Digits) {
    let mut separators = verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f64], _| {
        w64::lsd_mode(bits64(chunk), mode)
    });
    k_way_merge_fast2(arr, &mut separators);
}

