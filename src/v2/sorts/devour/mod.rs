//! # Devour sort: radix sort for integers (and `char`)
//!
//! Keys are sorted as unsigned integers (signed ones with their sign bit
//! flipped):
//!
//! 1. at most 32 keys: a sorting network; up to 64: blocks and a merge;
//! 2. already sorted, reversed or constant arrays (one scan);
//! 3. 8-bit keys: counted over their 256 values;
//! 4. small arrays: `sort_unstable`;
//! 5. the radix path (`decide::path`), with the integer detectors (ranges,
//!    plateaus, few values, heavy hitters, dense ranges); chunks sorted by
//!    the fused LSD (8 to 32 bits) or the DLSD (64 bits), the rollercoaster
//!    beyond the cache (32 bits: only on ordered keys).
//!
//! 128-bit keys: see `wide128`.

pub(crate) mod keys;
pub(crate) mod wide128;

use std::mem::MaybeUninit;

use crate::v2::decide::chunks::{self, Engine};
use crate::v2::decide::path::radix_path;
use crate::v2::decide::shape::clustered;
use crate::v2::engines::counting::domain_count;
use crate::v2::engines::dlsd::{di64, du64};
use crate::v2::engines::lsd::{wi16, wi32, wi64, wi8, wu16, wu32, wu64, wu8};
use crate::v2::engines::roller::{ri32, ri64, ru32, ru64};
use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
use crate::v2::inspect::presorted::presorted;
use crate::v2::special::heavy::{heavy_sorted_by, may_be_heavy, HEAVY_MIN};
use crate::v2::special::plateaus::plateaus;
use crate::v2::special::{spread, DENSE_MIN, SAMPLE};
use keys::*;

mod sealed {
    pub trait Sealed {}
    macro_rules! sealed {
        ($($t:ty),*) => { $(impl Sealed for $t {})* };
    }
    sealed!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char);
}

/// Integer types sortable by [`devour_sort2`].
pub trait DevourSort2: sealed::Sealed + Copy + Ord {
    #[doc(hidden)]
    fn devour_sort2_impl(arr: &mut [Self]);
}

/// Sorts slices of `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`,
/// `i32`, `i64`, `i128`, `isize` or `char`.
///
/// ```text
/// use voracious_radix_sort::devour_sort2;
///
/// let mut v = vec![5i64, -3, 12, 0, -40];
/// devour_sort2(&mut v);
/// assert_eq!(v, vec![-40, -3, 0, 5, 12]);
/// ```
#[inline]
pub fn devour_sort2<T: DevourSort2>(arr: &mut [T]) {
    T::devour_sort2_impl(arr);
}

/// What the integer sort needs of a key type: its key map, its small-array
/// limit, its detectors and its chunk engine.
trait IntSort: DevourSort2 + BitsOf {
    /// Up to this size: `sort_unstable` (where it crosses the radix path).
    const SMALL: usize;
    fn encode(bits: Self::Bits) -> Self::Bits;
    fn decode(key: Self::Bits) -> Self::Bits;
    fn range(arr: &mut [Self], sample: &[Self; SAMPLE]) -> bool;
    fn few(arr: &mut [Self], sample: &[Self; SAMPLE]) -> bool;
    fn dense(arr: &mut [Self], sorted_sample: &[Self; SAMPLE]) -> bool;
    fn sort_chunk(bits: &mut [Self::Bits], buffer: &mut Vec<MaybeUninit<Self::Bits>>);
}

/// Arrays beyond the sorting network.
#[inline(never)]
fn sort_integers<T: IntSort>(arr: &mut [T]) {
    let n = arr.len();
    chain! {
        "merge_small" if n <= MERGE_MAX => do { merge_sort(bits_of(arr), T::encode, T::decode) };
    }
    let known = match presorted(arr) {
        Ok(()) => {
            trace!("presorted");
            return;
        },
        Err(known) => known,
    };
    chain! {
        // 8-bit keys: counting over the 256 values beats any detection.
        "count8" if T::Bits::MAX_U64 == u8::MAX as u64 && n >= 256 && n <= u32::MAX as usize
            => do { domain_count(bits_of(arr), T::encode, T::decode) };
        "std_small" if n <= T::SMALL => do { arr.sort_unstable() };
    }
    let buffer = std::cell::RefCell::new(Vec::new());
    let sort_chunk = |chunk: &mut [T]| T::sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
    radix_path(arr, known, true, &detect::<T>, &sort_chunk);
}

/// The integer detectors, on one spread sample (sorted once, only when the
/// detectors that need it in order are reached). `equal_pairs`: equal pairs
/// among the probes of the near-sorted repair.
fn detect<T: IntSort>(arr: &mut [T], equal_pairs: usize) -> bool {
    let mut sample = spread(arr);
    chain! {
        return true;
        "range" => try T::range(arr, &sample);
        "plateaus" => try plateaus(arr, equal_pairs);
        "few" => try T::few(arr, &sample);
    }
    // Heavy keys and dense ranges: the sample in order.
    if arr.len() < HEAVY_MIN || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y)) {
        return false;
    }
    sample.sort_unstable();
    chain! {
        return true;
        "heavy" => try heavy_sorted_by(
            arr,
            &sample,
            HEAVY_MIN,
            &|r: &mut [T]| devour_sort2(r),
            |x, y| x == y,
            |x, y| x < y,
        );
        "dense" if arr.len() >= DENSE_MIN => try T::dense(arr, &sample);
    }
    false
}

/// 8 to 32 bits: the fused LSD or the rollercoaster (`decide::chunks`).
macro_rules! lsd_chunks {
    ($uint:ty, $lsd:ident, $roller:expr, $rule:path) => {
        #[inline(always)]
        fn sort_chunk(bits: &mut [$uint], _: &mut Vec<MaybeUninit<$uint>>) {
            if $rule(bits) == Engine::Roller {
                trace!("roller");
                $roller(bits);
            } else {
                trace!("lsd");
                $lsd::lsd(bits);
            }
        }
    };
}

/// 64 bits: the DLSD, the LSD on tight clusters (the cluster rule reads the
/// DLSD's own sorted sample) or the rollercoaster (`decide::chunks`).
macro_rules! dlsd_chunks {
    ($dlsd:ident, $lsd:ident, $roller:ident) => {
        #[inline(always)]
        fn sort_chunk(bits: &mut [u64], buffer: &mut Vec<MaybeUninit<u64>>) {
            match chunks::int64(bits.len()) {
                Engine::Roller => {
                    trace!("roller");
                    $roller::roller(bits);
                },
                Engine::DlsdOrLsdOnClusters => {
                    if $dlsd::dlsd_unless(bits, buffer, |sample: &[u64]| clustered(sample)) {
                        trace!("dlsd");
                    } else {
                        trace!("lsd_clustered");
                        $lsd::lsd(bits);
                    }
                },
                _ => {
                    trace!("dlsd");
                    $dlsd::dlsd_with(bits, buffer);
                },
            }
        }
    };
}

macro_rules! int_type {
    ($t:ty, $uint:ty, $m:ident, $encode:ident, $decode:ident, $small:expr, $roller_min:path, $chunks:ident!($($engines:tt)*)) => {
        int_detectors!($m, $t, $uint, $encode, $decode, $roller_min);

        impl IntSort for $t {
            const SMALL: usize = $small;
            #[inline(always)]
            fn encode(bits: $uint) -> $uint {
                $encode(bits)
            }
            #[inline(always)]
            fn decode(key: $uint) -> $uint {
                $decode(key)
            }
            #[inline(always)]
            fn range(arr: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
                $m::range(arr, sample)
            }
            #[inline(always)]
            fn few(arr: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
                $m::few(arr, sample)
            }
            #[inline(always)]
            fn dense(arr: &mut [$t], sorted_sample: &[$t; SAMPLE]) -> bool {
                $m::dense(arr, sorted_sample)
            }
            $chunks!($($engines)*);
        }

        impl DevourSort2 for $t {
            #[inline]
            fn devour_sort2_impl(arr: &mut [$t]) {
                if arr.len() <= NETWORK_MAX {
                    trace!("network");
                    network_sort(bits_of(arr), $encode, $decode);
                } else {
                    sort_integers(arr);
                }
            }
        }
    };
}

// Small-array limits: in-process A/B, Snapdragon X Elite (much lower than
// for floats: the integer detectors often apply).
int_type!(
    u32,
    u32,
    detect_u32,
    id32,
    id32,
    200,
    chunks::roller_min_32,
    lsd_chunks!(u32, wu32, ru32::roller, chunks::int32)
);
int_type!(
    i32,
    u32,
    detect_i32,
    flip32,
    flip32,
    150,
    chunks::roller_min_32,
    lsd_chunks!(u32, wi32, ri32::roller, chunks::int32)
);
int_type!(
    u64,
    u64,
    detect_u64,
    id64,
    id64,
    150,
    chunks::roller_min_64,
    dlsd_chunks!(du64, wu64, ru64)
);
int_type!(
    i64,
    u64,
    detect_i64,
    flip64,
    flip64,
    150,
    chunks::roller_min_64,
    dlsd_chunks!(di64, wi64, ri64)
);
// 8 and 16 bits: from 2^16 keys the narrow-range counting always applies;
// below, the fused LSD on bytes.
int_type!(
    u16,
    u16,
    detect_u16,
    id16,
    id16,
    150,
    chunks::no_roller,
    lsd_chunks!(u16, wu16, wu16::lsd, chunks::narrow)
);
int_type!(
    i16,
    u16,
    detect_i16,
    flip16,
    flip16,
    150,
    chunks::no_roller,
    lsd_chunks!(u16, wi16, wi16::lsd, chunks::narrow)
);
int_type!(
    u8,
    u8,
    detect_u8,
    id8,
    id8,
    150,
    chunks::no_roller,
    lsd_chunks!(u8, wu8, wu8::lsd, chunks::narrow)
);
int_type!(
    i8,
    u8,
    detect_i8,
    flip8,
    flip8,
    150,
    chunks::no_roller,
    lsd_chunks!(u8, wi8, wi8::lsd, chunks::narrow)
);

/// `char`: sorted as its code point (the order of `char`; sorting permutes
/// valid values only).
impl DevourSort2 for char {
    #[inline]
    fn devour_sort2_impl(arr: &mut [char]) {
        // SAFETY: `char` has the size and alignment of `u32`, every `char`
        // is a valid `u32`, and the sorted slice holds the same values.
        let codes =
            unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut u32, arr.len()) };
        u32::devour_sort2_impl(codes);
    }
}

/// `usize` / `isize`: sorted as the integer of the same width.
macro_rules! pointer_width {
    ($t:ty, $as:ty) => {
        impl DevourSort2 for $t {
            #[inline]
            fn devour_sort2_impl(arr: &mut [$t]) {
                const _: () = assert!(std::mem::size_of::<$t>() == std::mem::size_of::<$as>());
                // SAFETY: same size, alignment and value representation.
                let same = unsafe {
                    std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $as, arr.len())
                };
                <$as as DevourSort2>::devour_sort2_impl(same);
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

impl DevourSort2 for u128 {
    #[inline]
    fn devour_sort2_impl(arr: &mut [u128]) {
        wide128::sort128(arr);
    }
}

impl DevourSort2 for i128 {
    #[inline]
    fn devour_sort2_impl(arr: &mut [i128]) {
        wide128::sort128(arr);
    }
}
