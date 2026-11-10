//! # Int sort
//!
//! Dedicated radix sort for `u32`, `u64`, `i32` and `i64`, built on the
//! engines of [`float_sort`](super::float_sort) with an integer key map
//! (identity for unsigned integers, flipped sign bit for signed ones):
//!
//! 0. at most 32 keys: a sorting network (`small_sort`); up to 64 keys:
//!    blocks of 32 and a merge;
//! 1. constant array check;
//! 2. small arrays: `sort_unstable`;
//! 3. radix path (shared with `float_sort`): repair of nearly sorted
//!    arrays; narrow range (at most 2^16 integers, sorted by counting);
//!    few distinct values (at most 256, counted in a hash table);
//!    Verge pre-processing and merge-vs-radix rule; the chunks are sorted
//!    by the fused LSD (32-bit keys) or the DLSD (64-bit keys), and from a
//!    size derived from the L2 size by the rollercoaster (in-place MSD
//!    passes, then the DLSD on cache-sized buckets).
//!
//! Thresholds start from those measured for floats of the same width.

#[cfg_attr(not(test), allow(unused_imports))]
use super::float_dlsd::{di32, di64, du32, du64};
#[cfg_attr(not(test), allow(unused_imports))]
use super::float_lsd::{wi32, wi64, wu32, wu64};
use super::float_roller::{ri32, ri64, ru32, ru64};
use super::cache_info::l2_bytes;
use super::float_sort::radix_path;
use super::small_sort::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};

/// Up to these sizes, `sort_unstable`: where the median over the
/// distributions of radix path / small path crosses 1 (in-process A/B,
/// Snapdragon X Elite: u32 1.27 at 150, 1.00 at 200; i32 0.81 at 150; u64
/// 1.01 at 150; i64 0.96 at 150). Much lower than for floats: the integer
/// detectors (narrow range, few values) often apply.
const SMALL_U32: usize = 200;
const SMALL_I32: usize = 150;
const SMALL_U64: usize = 150;
const SMALL_I64: usize = 150;
/// From this size the few-distinct-values counting is tried.
const FEW_MIN: usize = 129;
/// Narrow-range counting: at most this many integers spanned...
const RANGE_MAX: usize = 1 << 16;
/// ...and at most this many per key of the array: the counting costs the
/// zeroing and the scan of the whole count table, on top of the keys. With
/// 4 per key, u32 Normale(0, 2^13) at 10k keys (span ~35k) took 8.0 ns per
/// key against 2.5 for a radix sort.
const RANGE_PER_KEY: usize = 1;
/// Sample keys checked before the full min / max pass.
const RANGE_SAMPLE: usize = 64;
/// Most distinct values counted, and the hash table size (at most half full).
const FEW_MAX: usize = 256;
const FEW_TABLE_BITS: u32 = 9;
const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;

mod sealed {
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
}

/// Integer types sortable by [`int_sort`].
pub trait IntSort: sealed::Sealed + Copy + Ord {
    #[doc(hidden)]
    fn int_sort_impl(arr: &mut [Self]);
}

/// # Int sort
///
/// Sorts `u32`, `u64`, `i32` or `i64` slices (unstable; for primitive
/// integers stability makes no difference).
///
/// ```
/// use voracious_radix_sort::int_sort;
///
/// let mut v = vec![5i64, -3, 12, 0, -40];
/// int_sort(&mut v);
/// assert_eq!(v, vec![-40, -3, 0, 5, 12]);
/// ```
#[inline]
pub fn int_sort<T: IntSort>(arr: &mut [T]) {
    T::int_sort_impl(arr);
}

macro_rules! int_width {
    ($t:ty, $uint:ty, $encode:expr, $decode:expr, $small:expr, $roller_min:path, $chunk:expr, $alt_chunk:expr, $roller:path, $rest:ident, $few:ident, $range:ident) => {
        impl IntSort for $t {
            #[inline]
            fn int_sort_impl(arr: &mut [$t]) {
                if arr.len() <= NETWORK_MAX {
                    network_sort(bits_of(arr), $encode, $decode);
                    return;
                }
                $rest(arr);
            }
        }

        /// Arrays longer than `NETWORK_MAX`.
        #[inline(never)]
        fn $rest(arr: &mut [$t]) {
            let n = arr.len();
            if n <= MERGE_MAX {
                merge_sort(bits_of(arr), $encode, $decode);
                return;
            }
            // Constant array: first, middle and last keys equal, then all.
            let first = arr[0];
            if arr[n / 2] == first && arr[n - 1] == first && arr.iter().all(|&v| v == first) {
                return;
            }
            #[cfg(test)]
            let small = match INT_SMALL_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
                0 => $small,
                v => v,
            };
            #[cfg(not(test))]
            let small = $small;
            if n <= small {
                arr.sort_unstable();
                return;
            }
            let scratch = std::cell::RefCell::new(Vec::new());
            radix_path(arr, &|a: &mut [$t]| $range(a) || $few(a), &|s: &mut [$t]| {
                #[cfg(test)]
                match INT_CHUNK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                    1 => return $chunk(bits_of(s), &mut scratch.borrow_mut()),
                    2 => return $roller(bits_of(s)),
                    3 => return $alt_chunk(bits_of(s), &mut scratch.borrow_mut()),
                    _ => {}
                }
                if s.len() >= $roller_min() {
                    $roller(bits_of(s));
                } else {
                    $chunk(bits_of(s), &mut scratch.borrow_mut());
                }
            });
        }

        /// Narrow range: values spanning at most `RANGE_MAX` integers and
        /// at most `RANGE_PER_KEY` times the array length are sorted by
        /// counting (one counting pass, then the values written in order).
        /// A spread sample rejects wide ranges before the full min / max
        /// pass, which only narrow-looking arrays pay.
        fn $range(values: &mut [$t]) -> bool {
            let n = values.len();
            if n < FEW_MIN {
                return false;
            }
            let limit = RANGE_MAX.min(RANGE_PER_KEY * n) as u64;
            let (mut lo, mut hi) = (values[0], values[0]);
            for i in 0..RANGE_SAMPLE as u64 {
                let pos = ((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32;
                let v = values[pos as usize];
                lo = lo.min(v);
                hi = hi.max(v);
            }
            if (hi as i128 - lo as i128) as u64 >= limit {
                return false;
            }
            let (lo, hi) = values.iter().fold((lo, hi), |(l, h), &v| (l.min(v), h.max(v)));
            let span = (hi as i128 - lo as i128) as u64 + 1;
            if span > limit {
                return false;
            }
            let mut counts = vec![0u32; span as usize];
            // Counts of at most 2^31 keys per block (u32 counters).
            let mut total = vec![0usize; 0];
            let big = n > u32::MAX as usize;
            if big {
                total = vec![0usize; span as usize];
            }
            for block in values.chunks(if big { 1 << 31 } else { usize::MAX }) {
                for &v in block {
                    // SAFETY: lo <= v <= hi, so the offset is below span.
                    unsafe { *counts.get_unchecked_mut((v as i128 - lo as i128) as usize) += 1 };
                }
                if big {
                    for (t, c) in total.iter_mut().zip(counts.iter_mut()) {
                        *t += *c as usize;
                        *c = 0;
                    }
                }
            }
            let mut start = 0;
            for d in 0..span as usize {
                let c = if big { total[d] } else { counts[d] as usize };
                if c != 0 {
                    values[start..start + c].fill((lo as i128 + d as i128) as $t);
                    start += c;
                }
            }
            debug_assert_eq!(start, n);
            true
        }

        /// Few distinct values (at most `FEW_MAX`): a spread sample gives up
        /// as soon as half of it is distinct; else one pass counts every
        /// value in an open-addressing hash table (4 lanes of counters),
        /// giving up beyond `FEW_MAX` values with nothing written; the
        /// distinct values are sorted and the array rewritten (same scheme
        /// as `float_detect::few`).
        fn $few(values: &mut [$t]) -> bool {
            let n = values.len();
            if n < FEW_MIN {
                return false;
            }
            let hash = |v: $t| (((v as $uint as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) >> (64 - FEW_TABLE_BITS)) as usize;
            let s = (n / 32).clamp(16, 256);
            let mut keys = [0 as $t; FEW_TABLE];
            let mut used = [false; FEW_TABLE];
            let mut distinct = 0;
            for i in 0..s as u64 {
                let pos = ((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32;
                let v = values[pos as usize];
                let mut h = hash(v);
                while used[h] && keys[h] != v {
                    h = (h + 1) & (FEW_TABLE - 1);
                }
                if !used[h] {
                    used[h] = true;
                    keys[h] = v;
                    distinct += 1;
                    if distinct > s / 2 {
                        return false;
                    }
                }
            }
            let mut keys = [0 as $t; FEW_TABLE];
            let mut used = [false; FEW_TABLE];
            let mut counts = [[0usize; FEW_TABLE]; 4];
            let mut distinct = 0;
            let mut find = |v: $t, distinct: &mut usize| -> Option<usize> {
                let mut h = hash(v);
                while used[h] && keys[h] != v {
                    h = (h + 1) & (FEW_TABLE - 1);
                }
                if !used[h] {
                    if *distinct == FEW_MAX {
                        return None;
                    }
                    used[h] = true;
                    keys[h] = v;
                    *distinct += 1;
                }
                Some(h)
            };
            let mut quads = values.chunks_exact(4);
            for q in &mut quads {
                for lane in 0..4 {
                    match find(q[lane], &mut distinct) {
                        Some(h) => counts[lane][h] += 1,
                        None => return false,
                    }
                }
            }
            for &v in quads.remainder() {
                match find(v, &mut distinct) {
                    Some(h) => counts[0][h] += 1,
                    None => return false,
                }
            }
            let mut found: Vec<($t, usize)> = (0..FEW_TABLE)
                .filter(|&h| used[h])
                .map(|h| (keys[h], counts[0][h] + counts[1][h] + counts[2][h] + counts[3][h]))
                .collect();
            found.sort_unstable_by_key(|&(v, _)| v);
            let mut start = 0;
            for (v, c) in found {
                values[start..start + c].fill(v);
                start += c;
            }
            debug_assert_eq!(start, n);
            true
        }
    };
}

/// Tests / benchmarks: small-path limit override (0: default; 1: radix
/// path from 65 keys).
#[cfg(test)]
pub(crate) static INT_SMALL_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Tests / benchmarks: chunk sort forced (0 default, 1 default engine (LSD
/// for 32-bit keys, DLSD for 64-bit), 2 rollercoaster, 3 alternative
/// engine (DLSD for 32-bit keys, fused LSD for 64-bit)).
#[cfg(test)]
pub(crate) static INT_CHUNK_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// From this chunk size (keys) 32-bit integers are sorted by the
/// rollercoaster instead of the fused LSD: L2 (12M keys with 12 MiB), for
/// its memory (n instead of 2n) on very large arrays. Measured on
/// Snapdragon X Elite, rollercoaster / LSD (geometric mean over the
/// generators): u32 1.14 at 250k, 1.05 at 1M, 0.94 at 4M, 1.03 at 10M;
/// i32 1.19, 1.01, 0.98, 1.02; on uniform i32 at 10M the LSD is faster
/// (5.6 vs 7.5 ns per key).
fn roller_int32() -> usize {
    l2_bytes()
}

/// From this chunk size 64-bit integers are sorted by the rollercoaster
/// instead of the DLSD: L2 / 12 (1M keys with 12 MiB). Rollercoaster /
/// DLSD: u64 1.07 at 250k, 1.00 at 1M, 0.82 at 4M, 0.84 at 10M; i64 1.05,
/// 1.00, 0.91, 0.93.
fn roller_int64() -> usize {
    l2_bytes() / 12
}

/// Reinterprets integers as their unsigned bits (same size and alignment,
/// every bit pattern valid for both).
#[inline(always)]
fn bits_of<T: BitsOf>(arr: &mut [T]) -> &mut [T::Bits] {
    // SAFETY: `BitsOf` is only implemented for integer pairs of the same
    // size and alignment; the slice reborrows `arr` exclusively.
    unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut T::Bits, arr.len()) }
}

trait BitsOf {
    type Bits;
}
impl BitsOf for u32 {
    type Bits = u32;
}
impl BitsOf for u64 {
    type Bits = u64;
}
impl BitsOf for i32 {
    type Bits = u32;
}
impl BitsOf for i64 {
    type Bits = u64;
}

#[inline(always)]
fn id32(k: u32) -> u32 {
    k
}
#[inline(always)]
fn id64(k: u64) -> u64 {
    k
}
#[inline(always)]
fn flip32(k: u32) -> u32 {
    k ^ (1 << 31)
}
#[inline(always)]
fn flip64(k: u64) -> u64 {
    k ^ (1 << 63)
}

int_width!(u32, u32, id32, id32, SMALL_U32, roller_int32, |b: &mut [u32], _s: &mut Vec<std::mem::MaybeUninit<u32>>| wu32::lsd(b), du32::dlsd_with, ru32::roller, rest_u32, few_u32, range_u32);
int_width!(i32, u32, flip32, flip32, SMALL_I32, roller_int32, |b: &mut [u32], _s: &mut Vec<std::mem::MaybeUninit<u32>>| wi32::lsd(b), di32::dlsd_with, ri32::roller, rest_i32, few_i32, range_i32);
int_width!(u64, u64, id64, id64, SMALL_U64, roller_int64, du64::dlsd_with, |b: &mut [u64], _s: &mut Vec<std::mem::MaybeUninit<u64>>| wu64::lsd(b), ru64::roller, rest_u64, few_u64, range_u64);
int_width!(i64, u64, flip64, flip64, SMALL_I64, roller_int64, di64::dlsd_with, |b: &mut [u64], _s: &mut Vec<std::mem::MaybeUninit<u64>>| wi64::lsd(b), ri64::roller, rest_i64, few_i64, range_i64);
