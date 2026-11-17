//! # Ravenous sort: radix sort for `f32` and `f64`
//!
//! Floats are sorted through their bits, encoded so that unsigned order is
//! the [`f64::total_cmp`] order (`key = bits ^ (sign ? 0xFF..FF : 0x80..00)`;
//! `-0.0 < +0.0`). The arrays are only reinterpreted, never copied.
//!
//! 1. at most 32 keys: a sorting network;
//! 2. detectors: constant array, integer range, power-of-two grid;
//! 3. small arrays: `sort_unstable` on the encoded keys;
//! 4. the radix path (`pipeline`), with few distinct values and heavy
//!    hitters as detectors; chunks sorted by the fused LSD (f32) or the DLSD
//!    (f64), the rollercoaster beyond the cache.
//!
//! NaN is not supported yet (the Verge step compares with `<`).

use super::detect::floats::{detect32, detect64, few32, few64};
use super::detect::heavy::{heavy_from, HEAVY_MIN};
use super::engines::dlsd::d64;
use super::engines::lsd::w32;
use super::engines::roller::{r32, r64};
use super::inspect::presorted::{presorted, Unsorted};
use super::pipeline::radix_path;
use super::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
use crate::v2::dev::trace::hit;

/// Up to these sizes: `sort_unstable` on the encoded keys (where it crosses
/// the radix path, median over the distributions, Snapdragon X Elite).
const SMALL_F32: usize = 350;
const SMALL_F64: usize = 700;

/// From this chunk size f64 goes to the rollercoaster instead of the DLSD:
/// L2 / 48, where the DLSD's keys and buffer take a third of the L2.
fn roller_min_f64() -> usize {
    super::cache::budget(super::cache::Budget::RollerF64) / 48
}

/// From this chunk size f32 goes to the rollercoaster instead of the fused
/// LSD: L2 / 12.
fn roller_min_f32() -> usize {
    super::cache::budget(super::cache::Budget::RollerF32) / 12
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

/// Sorts `f32` or `f64` slices in [`f64::total_cmp`] order.
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
    ($float:ty, $uint:ty, $sint:ty, $m:ident) => {
        mod $m {
            /// The floats as their raw bits.
            #[inline(always)]
            pub(super) fn bits(arr: &mut [$float]) -> &mut [$uint] {
                // SAFETY: same size and alignment, every bit pattern valid
                // for both; the slice reborrows `arr` exclusively.
                unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $uint, arr.len()) }
            }

            #[inline(always)]
            pub(super) fn encode(bits: $uint) -> $uint {
                const SIGN: $uint = 1 << (<$uint>::BITS - 1);
                bits ^ ((((bits as $sint) >> (<$uint>::BITS - 1)) as $uint) | SIGN)
            }

            #[inline(always)]
            pub(super) fn decode(key: $uint) -> $uint {
                const SIGN: $uint = 1 << (<$uint>::BITS - 1);
                key ^ ((key >> (<$uint>::BITS - 1)).wrapping_sub(1) | SIGN)
            }

            /// Small arrays: encode, sort as integers, decode.
            pub(super) fn sort_small(bits: &mut [$uint]) {
                if bits.len() <= super::MERGE_MAX {
                    super::merge_sort(bits, encode, decode);
                    return;
                }
                bits.iter_mut().for_each(|b| *b = encode(*b));
                bits.sort_unstable();
                bits.iter_mut().for_each(|b| *b = decode(*b));
            }

            /// -0.0 and +0.0 compare equal in the generic steps of the radix
            /// path (repair, Verge, merges): they come out together but
            /// mixed. Their block is found by two binary searches and
            /// rewritten, -0.0 first.
            pub(super) fn order_signed_zeros(arr: &mut [$float]) {
                let lo = arr.partition_point(|x| *x < 0.0);
                let hi = lo + arr[lo..].partition_point(|x| *x <= 0.0);
                if hi - lo < 2 {
                    return;
                }
                let negative = arr[lo..hi].iter().filter(|x| x.is_sign_negative()).count();
                arr[lo..lo + negative].fill(-0.0);
                arr[lo + negative..hi].fill(0.0);
            }

            /// Total order on the floats (for the heavy hitters detector).
            #[inline(always)]
            pub(super) fn less(x: $float, y: $float) -> bool {
                x.total_cmp(&y).is_lt()
            }

            #[inline(always)]
            pub(super) fn same(x: $float, y: $float) -> bool {
                x.to_bits() == y.to_bits()
            }
        }
    };
}

float_width!(f32, u32, i32, f32_bits);
float_width!(f64, u64, i64, f64_bits);

impl RavenousSort2 for f32 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f32]) {
        if arr.len() <= NETWORK_MAX {
            trace!("network");
            network_sort(f32_bits::bits(arr), f32_bits::encode, f32_bits::decode);
        } else {
            sort_f32(arr);
        }
    }
}

impl RavenousSort2 for f64 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f64]) {
        if arr.len() <= NETWORK_MAX {
            trace!("network");
            network_sort(f64_bits::bits(arr), f64_bits::encode, f64_bits::decode);
        } else {
            sort_f64(arr);
        }
    }
}

/// From this size the presorted check and the repair of nearly sorted
/// arrays come before the float detectors (f32 timestamps, a few distinct
/// values nearly in order: 0.67 to 0.15-0.27 ns per key at 10k-3M). Below,
/// the detectors first (1k: sparse 2.8 against 3.6).
const PRESORTED_FIRST_MIN: usize = 4096;

/// f32 arrays beyond the network: detectors, small path, radix path.
#[inline(never)]
fn sort_f32(arr: &mut [f32]) {
    let presorted_first = arr.len() >= PRESORTED_FIRST_MIN;
    if !presorted_first {
        if hit(detect32(arr), "float_detect") {
            return;
        }
        if arr.len() <= SMALL_F32 {
            trace!("std_small");
            f32_bits::sort_small(f32_bits::bits(arr));
            return;
        }
    }
    // As for integers: the presorted check first, then the repair of nearly
    // sorted arrays, and only then the detectors (in `detect`).
    let known = if presorted_first {
        match presorted(arr) {
            Ok(()) => {
                trace!("presorted");
                f32_bits::order_signed_zeros(arr);
                return;
            }
            Err(known) => known,
        }
    } else {
        Unsorted::default()
    };
    let detect = |a: &mut [f32], _| (presorted_first && hit(detect32(a), "float_detect")) || hit(few32(a), "few") || hit(heavy_from(a, HEAVY_MIN, &|r: &mut [f32]| ravenous_sort2(r), f32_bits::same, f32_bits::less), "heavy");
    let sort_chunk = |chunk: &mut [f32]| {
        if chunk.len() >= roller_min_f32() {
            trace!("roller");
            r32::roller(f32_bits::bits(chunk));
        } else {
            trace!("lsd");
            w32::lsd(f32_bits::bits(chunk));
        }
    };
    radix_path(arr, known, false, &detect, &sort_chunk);
    f32_bits::order_signed_zeros(arr);
}

/// f64 arrays beyond the network: detectors, small path, radix path (one
/// DLSD buffer for all the chunks).
#[inline(never)]
fn sort_f64(arr: &mut [f64]) {
    let presorted_first = arr.len() >= PRESORTED_FIRST_MIN;
    if !presorted_first {
        if hit(detect64(arr), "float_detect") {
            return;
        }
        if arr.len() <= SMALL_F64 {
            trace!("std_small");
            f64_bits::sort_small(f64_bits::bits(arr));
            return;
        }
    }
    // As for integers: the presorted check first, then the repair of nearly
    // sorted arrays, and only then the detectors (in `detect`).
    let known = if presorted_first {
        match presorted(arr) {
            Ok(()) => {
                trace!("presorted");
                f64_bits::order_signed_zeros(arr);
                return;
            }
            Err(known) => known,
        }
    } else {
        Unsorted::default()
    };
    let buffer = std::cell::RefCell::new(Vec::new());
    let detect = |a: &mut [f64], _| (presorted_first && hit(detect64(a), "float_detect")) || hit(few64(a), "few") || hit(heavy_from(a, HEAVY_MIN, &|r: &mut [f64]| ravenous_sort2(r), f64_bits::same, f64_bits::less), "heavy");
    let sort_chunk = |chunk: &mut [f64]| {
        if chunk.len() >= roller_min_f64() {
            trace!("roller");
            r64::roller(f64_bits::bits(chunk));
        } else {
            trace!("dlsd");
            d64::dlsd_with(f64_bits::bits(chunk), &mut buffer.borrow_mut());
        }
    };
    radix_path(arr, known, false, &detect, &sort_chunk);
    f64_bits::order_signed_zeros(arr);
}
