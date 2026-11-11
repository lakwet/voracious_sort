//! # Devour sort
//!
//! Dedicated radix sort for `u32`, `u64`, `i32` and `i64`, built on the
//! engines of [`ravenous_sort`](super::ravenous_sort) with an integer key map
//! (identity for unsigned integers, flipped sign bit for signed ones):
//!
//! 0. at most 32 keys: a sorting network (`small_sort`); up to 64 keys:
//!    blocks of 32 and a merge;
//! 1. constant array check;
//! 2. small arrays: `sort_unstable`;
//! 3. radix path (shared with `ravenous_sort`): repair of nearly sorted
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
use super::float_lsd::{wi16, wi8, wu16, wu8};
use super::float_roller::{ri32, ri64, ru32, ru64};
use super::cache_info::l2_bytes;
use super::ravenous_sort::radix_path;
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
const SMALL_U16: usize = 150;
const SMALL_I16: usize = 150;
const SMALL_U8: usize = 150;
const SMALL_I8: usize = 150;
/// From this size the few-distinct-values counting is tried.
const FEW_MIN: usize = 129;
/// Narrow-range counting: at most this many integers spanned...
const RANGE_MAX: usize = 1 << 16;
/// ...and at most one counter per this many keys: the counting costs the
/// zeroing, the scan and the writes of the whole table on top of the keys.
/// Measured: u32 Normale(0, 2^13) at 10k keys with 4 counters per key, 8.0
/// ns per key against 2.5 for a radix sort; u16 random at 100k keys (1.5
/// keys per counter) 21 % slower than the byte LSD, at 200k (3 keys per
/// counter) 24 % faster; u32 neutral between 1 and 4.
const RANGE_KEYS_PER_COUNTER: usize = 2;
/// Sample keys checked before the full min / max pass.
const RANGE_SAMPLE: usize = 64;
/// Wide-range counting (spans beyond `RANGE_MAX`, counters within half
/// the L2): at least this many keys per counter. Measured on char over its
/// 21 bits (1.1M counters, Snapdragon X Elite): count / LSD 1.64 at 2M keys
/// (1.8 per counter), 1.11 at 3M (2.7), 0.59 at 6M (5.5), 0.50 at 10M.
const WIDE_KEYS_PER_COUNTER: usize = 4;
/// Offset sort: bytes saved at least (span against varying bytes).
const OFFSET_BYTES: u32 = 1;
/// Shortest array tried for a dense range with outliers.
const DENSE_MIN: usize = 4096;
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
    impl Sealed for u8 {}
    impl Sealed for i8 {}
    impl Sealed for u16 {}
    impl Sealed for i16 {}
    impl Sealed for char {}
    impl Sealed for usize {}
    impl Sealed for isize {}
}

/// Integer types sortable by [`devour_sort`].
pub trait DevourSort: sealed::Sealed + Copy + Ord {
    #[doc(hidden)]
    fn devour_sort_impl(arr: &mut [Self]);
}

/// # Devour sort
///
/// Sorts slices of `u8`, `u16`, `u32`, `u64`, `usize`, `i8`, `i16`, `i32`,
/// `i64`, `isize` or `char` (unstable; for primitive integers stability
/// makes no difference).
///
/// ```
/// use voracious_radix_sort::devour_sort;
///
/// let mut v = vec![5i64, -3, 12, 0, -40];
/// devour_sort(&mut v);
/// assert_eq!(v, vec![-40, -3, 0, 5, 12]);
/// ```
#[inline]
pub fn devour_sort<T: DevourSort>(arr: &mut [T]) {
    T::devour_sort_impl(arr);
}

macro_rules! int_width {
    ($t:ty, $uint:ty, $encode:expr, $decode:expr, $small:expr, $roller_min:path, $chunk:expr, $alt_chunk:expr, $roller:path, $rest:ident, $few:ident, $range:ident, $dense:ident) => {
        impl DevourSort for $t {
            #[inline]
            fn devour_sort_impl(arr: &mut [$t]) {
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
            if arr[n / 2] == first && arr[n - 1] == first && all_equal(arr, first) {
                return;
            }
            // 8-bit keys from 256 on: counting over the 256 values is
            // cheaper than any other detection; only a fully sorted array
            // (either direction) is checked first, at the cost of 32 pairs
            // when it is not (sorted u8: 0.46 to 0.04 ns per key; long runs
            // of equal keys serialise the counters).
            if <$uint>::BITS == 8 && n >= 256 && n <= u32::MAX as usize {
                if sorted_either(arr) {
                    return;
                }
                domain_count(bits_of(arr), $encode, $decode);
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
            radix_path(arr, &|a: &mut [$t]| $range(a) || plateaus(a) || $few(a) || $dense(a), &|s: &mut [$t]| {
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
        /// at most one counter per `RANGE_KEYS_PER_COUNTER` keys are sorted by
        /// counting (one counting pass, then the values written in order).
        /// A spread sample rejects wide ranges first. A sample range narrow
        /// against the limit (ratings, small counters) is counted in one
        /// pass in a window around it (`window_count`); a wider one gets
        /// the exact min / max first, then the count. Measured (Snapdragon
        /// X Elite, against min / max then a 1-lane count): u32 0..100 0.54
        /// to 0.35 ns per key at 1M, Alt16 0.60 to 0.38; one wide outlier
        /// late in narrow keys costs a wasted counting pass (0.58 to 0.75).
        fn $range(values: &mut [$t]) -> bool {
            let n = values.len();
            if n < FEW_MIN {
                return false;
            }
            let limit = RANGE_MAX.min(n / RANGE_KEYS_PER_COUNTER) as u64;
            // Whole domain within the limit (8- and 16-bit keys): direct count.
            let domain = n <= u32::MAX as usize && (<$uint>::MAX as u64) < limit;
            // From 8 keys per domain counter on, the whole table is cheap
            // against the keys: counted directly.
            if domain && n as u64 >= (<$uint>::MAX as u64).saturating_add(1).saturating_mul(8) {
                domain_count(bits_of(values), $encode, $decode);
                return true;
            }
            let (mut lo, mut hi) = (values[0], values[0]);
            // Same sample, keys rotated by half the domain (top encoded bit
            // flipped): a range wrapping around the end of the domain
            // (small values and values just below the maximum, e.g. signed
            // offsets stored unsigned) becomes contiguous.
            let rot = |v: $t| $encode(v as $uint) ^ ((1 as $uint) << (<$uint>::BITS - 1));
            let (mut rlo, mut rhi) = (values[0], values[0]);
            for i in 0..RANGE_SAMPLE as u64 {
                let pos = ((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32;
                let v = values[pos as usize];
                lo = lo.min(v);
                hi = hi.max(v);
                if rot(v) < rot(rlo) {
                    rlo = v;
                }
                if rot(v) > rot(rhi) {
                    rhi = v;
                }
            }
            if domain {
                // Very narrow sample range (in either order): a window of
                // at most 1/32 of the domain table, the whole domain when
                // keys fall outside it (u16 at 131k-300k: 16 values 0.96
                // to 0.48 ns per key, 0..255 0.70 to 0.44, values at both
                // ends 0.67 to 0.37-0.44).
                let span = (hi as i128 - lo as i128) as u64 + 1;
                let rspan = (rot(rhi) - rot(rlo)) as u64 + 1;
                let tiny = limit / 64;
                if span <= tiny && window_count(bits_of(values), lo as $uint, hi as $uint, limit as usize, $encode, $decode) {
                    return true;
                }
                if span > tiny && rspan <= tiny && wrapped_count(values, rlo, rhi, limit as usize) {
                    return true;
                }
                domain_count(bits_of(values), $encode, $decode);
                return true;
            }
            if (hi as i128 - lo as i128) as u64 >= limit {
                let rspan = (rot(rhi) - rot(rlo)) as u64 + 1;
                if n <= u32::MAX as usize && 2 * rspan < limit {
                    return wrapped_count(values, rlo, rhi, limit as usize);
                }
                // Values at both ends of the domain, wider than a window (u64
                // at 1k-1M: 6.7-9.0 to 3.0-4.0 ns per key; u32 3.1 to 2.1).
                if <$uint>::BITS >= 32 && n < $roller_min() && offset_sort(values, lo, hi, rlo, rhi) {
                    return true;
                }
                // Wide range, counted all the same when its counters fit in
                // half the L2 and enough keys share each counter: one read
                // and one write against the radix passes (char over its 21
                // bits at 10M keys: 3.5 to 1.1 ns per key; u64 sawtooth at 3M-10M:
                // 5.4 to 0.9).
                if <$uint>::BITS >= 32 && n <= u32::MAX as usize {
                    let wide = ((l2_bytes() / 8).min(n / WIDE_KEYS_PER_COUNTER)) as u128;
                    if (hi as i128 - lo as i128) as u128 + 1 <= wide {
                        let (lo, hi) = values.iter().fold((lo, hi), |(l, h), &v| (l.min(v), h.max(v)));
                        let span = (hi as i128 - lo as i128) as u128 + 1;
                        if span <= wide {
                            exact_count(bits_of(values), $encode(lo as $uint), span as usize, $encode, $decode);
                            return true;
                        }
                    }
                }
                return false;
            }

            /// Keys at both ends of the domain (small values and values just
            /// below the maximum, wider than a counting window): rotated by
            /// half the domain and shifted to `key - min`, they span fewer
            /// bytes than vary among them; sorted on the remaining bytes,
            /// rotated back into true order and shifted back. From the
            /// sample first, then checked on the exact min / max (one pass,
            /// nothing written when the gain is not there). Only below the
            /// rollercoaster size: its MSD pass already splits the two ends.
            /// Without rotation (a narrow range across a byte boundary) the
            /// DLSD already sizes its digits from a sample: 10-25 % slower.
            fn offset_sort(values: &mut [$t], lo: $t, hi: $t, rlo: $t, rhi: $t) -> bool {
                const TOP: $uint = (1 as $uint) << (<$uint>::BITS - 1);
                let bytes = |x: $uint| (<$uint>::BITS - x.leading_zeros() + 7) / 8;
                let rot = |v: $t| $encode(v as $uint) ^ TOP;
                let (elo, ehi) = ($encode(lo as $uint), $encode(hi as $uint));
                let rspan = rot(rhi).wrapping_sub(rot(rlo));
                if rspan >= ehi - elo || bytes(rspan) + OFFSET_BYTES > bytes(elo ^ ehi) {
                    return false;
                }
                let keys = bits_of(values);
                // Exact min / max of the rotated keys, and the bytes that vary
                // among the keys as they are (what the radix sort would see).
                let e0 = $encode(keys[0]);
                let (mut kmin, mut kmax, mut diff) = (<$uint>::MAX, 0 as $uint, 0 as $uint);
                for &k in keys.iter() {
                    let e = $encode(k);
                    diff |= e ^ e0;
                    kmin = kmin.min(e ^ TOP);
                    kmax = kmax.max(e ^ TOP);
                }
                if bytes(kmax - kmin) + OFFSET_BYTES > bytes(diff) {
                    return false;
                }
                for k in keys.iter_mut() {
                    *k = ($encode(*k) ^ TOP) - kmin;
                }
                <$uint as DevourSort>::devour_sort_impl(keys);
                // Rotated order puts the keys of the top half of the encoded
                // domain first; true order puts them last.
                let split = keys.partition_point(|&k| k + kmin < TOP);
                keys.rotate_left(split);
                for k in keys.iter_mut() {
                    *k = $decode((*k + kmin) ^ TOP);
                }
                true
            }

            /// Window count on keys rotated by half the domain, written back
            /// from the rotated value `TOP` on (the keys of the bottom half
            /// of the encoded domain first, their true order).
            fn wrapped_count(values: &mut [$t], rlo: $t, rhi: $t, limit: usize) -> bool {
                const TOP: $uint = (1 as $uint) << (<$uint>::BITS - 1);
                fn renc(k: $uint) -> $uint {
                    $encode(k) ^ TOP
                }
                fn rdec(k: $uint) -> $uint {
                    $decode(k ^ TOP)
                }
                window_count_split(bits_of(values), rlo as $uint, rhi as $uint, limit, renc, rdec, TOP as u64)
            }

            let small = n <= u32::MAX as usize;
            // Sample range narrow against the limit: count in a window
            // around it, in one pass.
            if small && 2 * (hi as i128 - lo as i128 + 1) < limit as i128 {
                return window_count(bits_of(values), lo as $uint, hi as $uint, limit as usize, $encode, $decode);
            }
            let (lo, hi) = values.iter().fold((lo, hi), |(l, h), &v| (l.min(v), h.max(v)));
            let span = (hi as i128 - lo as i128) as u64 + 1;
            if span > limit {
                return false;
            }
            if small {
                exact_count(bits_of(values), $encode(lo as $uint), span as usize, $encode, $decode);
                return true;
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

        /// Most keys in a narrow range, a minority spread (text with a
        /// few rare characters, sparse data, sensors with outliers): when
        /// 3 / 4 of the sample fit in a window of half the counting
        /// limit, a 3-way partition (below / inside / above), the inside
        /// counted, the two minorities sorted recursively: nothing to
        /// merge. Tried after the few-values count (cheaper when it
        /// applies). Gives up (array permuted, unsorted) when less than
        /// half the keys fall inside. Mixed text (85 % ASCII, 10 % Latin,
        /// 3 % CJK, 2 % emoji): 3.0-4.2 to 1.1-2.4 ns per key.
        fn $dense(values: &mut [$t]) -> bool {
            let n = values.len();
            if n < DENSE_MIN || n > u32::MAX as usize {
                return false;
            }
            let limit = RANGE_MAX.min(n / RANGE_KEYS_PER_COUNTER) as u64;
            let mut sample = [values[0]; RANGE_SAMPLE];
            for i in 0..RANGE_SAMPLE as u64 {
                let pos = ((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32;
                sample[i as usize] = values[pos as usize];
            }
            sample.sort_unstable();
            let w = limit as i128 / 2;
            let (mut best, mut bi, mut bj, mut j) = (0, 0, 0, 0);
            for i in 0..RANGE_SAMPLE {
                j = j.max(i);
                while j + 1 < RANGE_SAMPLE && (sample[j + 1] as i128 - sample[i] as i128) < w {
                    j += 1;
                }
                if j + 1 - i > best {
                    best = j + 1 - i;
                    bi = i;
                    bj = j;
                }
            }
            if 4 * best < 3 * RANGE_SAMPLE {
                return false;
            }
            let (slo, shi) = (sample[bi] as i128, sample[bj] as i128);
            let width = (2 * (shi - slo + 1)).max(256).min(limit as i128);
            let lo = (slo - (width - (shi - slo + 1)) / 2).max(<$t>::MIN as i128);
            let hi = (lo + width - 1).min(<$t>::MAX as i128);
            // The inside keys expected from the sample must fill the window
            // as densely as a narrow range (`RANGE_KEYS_PER_COUNTER`):
            // otherwise counting them costs more than a radix sort (u16
            // Gaussian at 20k-30k keys, 85 % of them within n / 2 values:
            // 2.1 against 1.3 ns per key).
            if !exp(8) && (best * n) < RANGE_KEYS_PER_COUNTER * (hi - lo + 1) as usize * RANGE_SAMPLE {
                return false;
            }
            let (lo, hi) = (lo as $t, hi as $t);
            // 3-way partition, [0, a) below, [a, i) inside, [b, n) above
            // (branches: the inside keys are the predicted majority; a
            // branch-free split into a buffer was slower, 1.3 to 2.2 ns
            // per key on mixed text at 1M).
            let (mut a, mut i, mut b) = (0, 0, n);
            while i < b {
                let v = values[i];
                if v < lo {
                    values.swap(a, i);
                    a += 1;
                    i += 1;
                } else if v > hi {
                    b -= 1;
                    values.swap(i, b);
                } else {
                    i += 1;
                }
            }
            if 2 * (b - a) < n {
                return false;
            }
            exact_count(bits_of(&mut values[a..b]), $encode(lo as $uint), (hi as i128 - lo as i128 + 1) as usize, $encode, $decode);
            <$t as DevourSort>::devour_sort_impl(&mut values[..a]);
            <$t as DevourSort>::devour_sort_impl(&mut values[b..]);
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

/// Tests / benchmarks: experiment bits (A/B in one process).
#[cfg(test)]
pub(crate) static INT_EXP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(always)]
fn exp(bit: usize) -> bool {
    #[cfg(test)]
    {
        INT_EXP.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
    }
    #[cfg(not(test))]
    {
        let _ = bit;
        false
    }
}

/// Long runs of equal keys (values held for a while, repeated blocks): when
/// at least 31 of 32 spread adjacent pairs are equal, one scan lists the
/// runs (giving up beyond n / 32 of them, nothing written), the (value,
/// length) pairs are sorted and the array rewritten: one read and one write
/// instead of a sort of n keys.
fn plateaus<T: Copy + Ord>(arr: &mut [T]) -> bool {
    let n = arr.len();
    if n < 1024 {
        return false;
    }
    let step = (n - 1) / 32;
    let equal = (0..32).filter(|&k| arr[k * step] == arr[k * step + 1]).count();
    if equal < 31 {
        return false;
    }
    let max = n / 32;
    let mut runs: Vec<(T, usize)> = Vec::with_capacity(max.min(4096));
    let mut start = 0;
    while start < n {
        let v = arr[start];
        let mut end = start + 1;
        // Blocks of 16 compared without exit inside (vectorized).
        while end + 16 <= n && arr[end..end + 16].iter().fold(true, |acc, &x| acc & (x == v)) {
            end += 16;
        }
        while end < n && arr[end] == v {
            end += 1;
        }
        if runs.len() == max {
            return false;
        }
        runs.push((v, end - start));
        start = end;
    }
    runs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut pos = 0;
    for &(v, len) in runs.iter() {
        arr[pos..pos + len].fill(v);
        pos += len;
    }
    true
}

/// Whether every key equals `first`: blocks of 64 compared without early
/// exit inside a block (vectorized), an exit between blocks.
#[inline]
fn all_equal<T: Copy + PartialEq>(arr: &[T], first: T) -> bool {
    let mut blocks = arr.chunks_exact(64);
    for b in &mut blocks {
        if !b.iter().fold(true, |acc, &v| acc & (v == first)) {
            return false;
        }
    }
    blocks.remainder().iter().all(|&v| v == first)
}

/// Narrow-range counting in one pass over the keys, without a min / max
/// pass: the keys are counted in a window around the sample range (twice
/// its width, at least 256 and at most `limit` integers). A key outside the
/// window stops the count: the exact range of what is left (min / max of
/// the remaining keys, first / last used counter for the counted ones)
/// either exceeds `limit` (returns `false`, nothing written) or gives the
/// new window, the counts moved to it and the count resumed. Small windows
/// count on 4 lanes (repeated keys would otherwise wait on the previous
/// increment of the same counter). At most 2^32 - 1 keys.
#[inline]
fn window_count<U: WindowKey, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(keys: &mut [U], lo: U, hi: U, limit: usize, encode: E, decode: D) -> bool {
    window_count_split(keys, lo, hi, limit, encode, decode, 0)
}

/// `window_count` writing the encoded values from `split` on first, then
/// the ones below it (keys counted rotated by half the domain: written back
/// in their true order, no rotation of the array afterwards).
#[inline]
fn window_count_split<U: WindowKey, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(keys: &mut [U], lo: U, hi: U, limit: usize, encode: E, decode: D, split: u64) -> bool {
    let n = keys.len();
    let (lo, hi) = (encode(lo).to_u64(), encode(hi).to_u64());
    let mut width = ((hi - lo + 1) as usize * 2).max(256).min(limit);
    // Centre the window on the sample range, inside the key domain.
    let pad = (width as u64 - (hi - lo + 1)) / 2;
    let mut base = lo.saturating_sub(pad).min(U::MAX_U64 - (width as u64 - 1));
    let mut lanes = lanes_for(width, n);
    let mut counts = vec![0u32; width * lanes];
    let mut pos = 0;
    loop {
        let missed = count_window(&keys[pos..], encode, base, width, lanes, &mut counts);
        // Lane 0 receives the sums of the lanes.
        if lanes == 4 {
            let (c0, rest) = counts.split_at_mut(width);
            for l in 0..3 {
                for (a, b) in c0.iter_mut().zip(rest[l * width..(l + 1) * width].iter()) {
                    *a += *b;
                }
            }
        }
        let at = match missed {
            Some(at) => at,
            None => break,
        };
        pos += at;
        let (mut new_lo, mut new_hi) = (u64::MAX, 0u64);
        if let Some(first) = counts[..width].iter().position(|&c| c != 0) {
            let last = counts[..width].iter().rposition(|&c| c != 0).unwrap();
            new_lo = base + first as u64;
            new_hi = base + last as u64;
        }
        for &k in keys[pos..].iter() {
            let k = encode(k).to_u64();
            new_lo = new_lo.min(k);
            new_hi = new_hi.max(k);
        }
        if new_hi - new_lo >= limit as u64 {
            return false;
        }
        let new_width = (new_hi - new_lo + 1) as usize;
        let new_lanes = lanes_for(new_width, n - pos);
        let mut new_counts = vec![0u32; new_width * new_lanes];
        for (d, &c) in counts[..width].iter().enumerate() {
            if c != 0 {
                new_counts[(base + d as u64 - new_lo) as usize] = c;
            }
        }
        counts = new_counts;
        base = new_lo;
        width = new_width;
        lanes = new_lanes;
    }
    let cut = (split.saturating_sub(base) as usize).min(width);
    let mut start = 0;
    for d in (cut..width).chain(0..cut) {
        let c = counts[d];
        if c != 0 {
            let c = c as usize;
            fill_run(keys, start, c, decode(U::from_u64(base + d as u64)));
            start += c;
        }
    }
    debug_assert_eq!(start, n);
    true
}

/// Counter lanes of a count table of `width` counters for `n` keys: 4 lanes
/// (repeated values would otherwise wait on the previous increment of their
/// counter) when the 4 tables fit in an eighth of the L2 (12 MiB L2: up to
/// ~98k counters; 256 KiB: 2048) and hold at most one counter per 2 keys;
/// else 1 lane (keys spread over many counters rarely repeat, and 4 copies
/// would leave the cache of small-L2 cores). Measured on 32- and 64-bit
/// keys: a fixed 4096-counter cap was 1-2 % slower at 3M-10M (12 MiB L2).
#[inline]
fn lanes_for(width: usize, n: usize) -> usize {
    if 4 * width * 4 <= l2_bytes() / 8 && 2 * width <= n {
        4
    } else {
        1
    }
}

/// Writes `c` copies of `v` from `start`: runs of at most 64 by fixed blocks
/// of 16 copies (no call, no length-dependent loop for the many short runs
/// of a wide table), the copies past `start + c` being overwritten by the
/// following runs, written later in order; longer runs and runs near the
/// end of the slice by a plain fill (wide stores).
#[inline(always)]
fn fill_run<U: Copy>(keys: &mut [U], start: usize, c: usize, v: U) {
    let end = start + c;
    if c <= 64 && end + 16 <= keys.len() {
        let block = [v; 16];
        let mut p = start;
        while p < end {
            keys[p..p + 16].copy_from_slice(&block);
            p += 16;
        }
    } else {
        keys[start..end].fill(v);
    }
}

/// Counting sort over the whole key domain (`2^bits` encoded values), used
/// when the domain is not wider than the narrow-range limit (8- and 16-bit
/// keys): no sample, no min / max, no bound check. At most 2^32 - 1 keys.
#[inline]
fn domain_count<U: WindowKey, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(keys: &mut [U], encode: E, decode: D) {
    let width = (U::MAX_U64 + 1) as usize;
    // Wide domains (16-bit keys): 4 lanes only when keys repeat nearby. One
    // lane keeps a quarter of the table in cache (u16 at 1M-10M, against 4
    // lanes: uniform 0.87 to 0.70 ns per key, Gaussian 0.58 to 0.37,
    // sawtooth 0.47 to 0.34; Zipf keeps its 4 lanes).
    let four = lanes_for(width, keys.len()) == 4 && (width <= 256 || repeats_near(keys));
    // Mixed index only for keys sharing their low byte (the case it fixes;
    // 3-6 % faster without it otherwise).
    let mix = U::MAX_U64 > 255 && shared_low_byte(keys);
    match (mix, four) {
        (true, true) => domain_count_with::<U, E, D, true, true>(keys, encode, decode),
        (true, false) => domain_count_with::<U, E, D, true, false>(keys, encode, decode),
        (false, true) => domain_count_with::<U, E, D, false, true>(keys, encode, decode),
        (false, false) => domain_count_with::<U, E, D, false, false>(keys, encode, decode),
    }
}

/// Whether 16 spread keys share their low byte (aligned values: their
/// counters would alias in the cache without the mixed index).
#[inline]
fn shared_low_byte<U: WindowKey>(keys: &[U]) -> bool {
    let n = keys.len();
    let first = keys[0].to_u64() & 0xFF;
    (1..16).all(|i| keys[i * (n / 16)].to_u64() & 0xFF == first)
}

/// Whether keys repeat within a few positions (on 256 spread positions, at
/// least 1 in 16 equal to one of the next 3 keys): the increments of one
/// counter would then wait on each other, and 4 lanes of counters pay off;
/// otherwise one lane (a quarter of the table, kept closer in cache).
#[inline]
fn repeats_near<U: WindowKey>(keys: &[U]) -> bool {
    let n = keys.len();
    if n < 1024 {
        return true;
    }
    let mut hits = 0;
    for i in 0..256u64 {
        let p = (((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * (n - 4) as u64) >> 32) as usize;
        let k = keys[p].to_u64();
        hits += (k == keys[p + 1].to_u64() || k == keys[p + 2].to_u64() || k == keys[p + 3].to_u64()) as usize;
    }
    hits >= 16
}

/// Whether `arr` is sorted in either direction, a descending one being
/// reversed: the direction from 32 spread adjacent pairs (a monotone array
/// has none against it, so any mix gives up at once; with all pairs equal
/// the two ends decide), then one scan, stopping at the first pair against
/// it.
#[inline]
fn sorted_either<T: Copy + PartialOrd>(arr: &mut [T]) -> bool {
    use super::super::algo::verge_sort_fast::{explore_forward_asc, explore_forward_desc};
    let n = arr.len();
    let step = (n - 1) / 32;
    let (mut asc, mut desc) = (false, false);
    for k in 0..32 {
        let i = k * step;
        asc |= arr[i] < arr[i + 1];
        desc |= arr[i] > arr[i + 1];
    }
    if asc && desc {
        return false;
    }
    if desc || (!asc && arr[0] > arr[n - 1]) {
        if explore_forward_desc(arr, 0) == n {
            arr.reverse();
            return true;
        }
        return false;
    }
    explore_forward_asc(arr, 0) == n
}

#[inline(always)]
fn domain_count_with<U: WindowKey, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy, const MIX: bool, const FOUR: bool>(keys: &mut [U], encode: E, decode: D) {
    let n = keys.len();
    let width = (U::MAX_U64 + 1) as usize;
    let lanes = if FOUR { 4 } else { 1 };
    let mut counts = vec![0u32; width * lanes];
    // Wide domains (16-bit keys): counter of key k at k ^ (k >> 8) (low
    // byte mixed with the high byte, a bijection within each block of 256
    // counters). Keys sharing their low byte (multiples of 256) otherwise
    // hit counters 1 KiB apart, all in the same few L1 sets (u16 multiples
    // of 256 at 1M-10M: 4.3 to 0.35 ns per key).
    let mix = MIX;
    let idx = |k: U| {
        let e = encode(k).to_u64() as usize;
        if mix { e ^ ((e >> 8) & 0xFF) } else { e }
    };
    let mut quads = keys.chunks_exact(4);
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(width);
        let (c1, rest) = rest.split_at_mut(width);
        let (c2, c3) = rest.split_at_mut(width);
        if std::mem::size_of::<U>() == 1 {
            // Bytes: one 64-bit load per 8 keys (u8 / i8 counting 3 to 6 %
            // faster than one load per key).
            // SAFETY: `U` is a 1-byte integer here.
            let bytes = unsafe { std::slice::from_raw_parts(keys.as_ptr() as *const u8, n) };
            let mut words = bytes.chunks_exact(8);
            let at = |w: u64, i: u32| idx(U::from_u64((w >> (8 * i)) & 0xFF));
            for c in &mut words {
                let w = u64::from_le_bytes(std::convert::TryInto::try_into(c).unwrap());
                // SAFETY: an encoded key is below the domain size `width`.
                unsafe {
                    *c0.get_unchecked_mut(at(w, 0)) += 1;
                    *c1.get_unchecked_mut(at(w, 1)) += 1;
                    *c2.get_unchecked_mut(at(w, 2)) += 1;
                    *c3.get_unchecked_mut(at(w, 3)) += 1;
                    *c0.get_unchecked_mut(at(w, 4)) += 1;
                    *c1.get_unchecked_mut(at(w, 5)) += 1;
                    *c2.get_unchecked_mut(at(w, 6)) += 1;
                    *c3.get_unchecked_mut(at(w, 7)) += 1;
                }
            }
            let done = n - words.remainder().len();
            for &k in keys[done..].iter() {
                c0[idx(k)] += 1;
            }
            quads = keys[..0].chunks_exact(4);
        }
        for q in &mut quads {
            // SAFETY: an encoded key is below the domain size `width`.
            unsafe {
                *c0.get_unchecked_mut(idx(q[0])) += 1;
                *c1.get_unchecked_mut(idx(q[1])) += 1;
                *c2.get_unchecked_mut(idx(q[2])) += 1;
                *c3.get_unchecked_mut(idx(q[3])) += 1;
            }
        }
        for &k in quads.remainder() {
            c0[idx(k)] += 1;
        }
        for l in 1..4 {
            let (c0, rest) = counts.split_at_mut(width);
            for (a, b) in c0.iter_mut().zip(rest[(l - 1) * width..l * width].iter()) {
                *a += *b;
            }
        }
    } else {
        for q in &mut quads {
            // SAFETY: as above.
            unsafe {
                *counts.get_unchecked_mut(idx(q[0])) += 1;
                *counts.get_unchecked_mut(idx(q[1])) += 1;
                *counts.get_unchecked_mut(idx(q[2])) += 1;
                *counts.get_unchecked_mut(idx(q[3])) += 1;
            }
        }
        for &k in quads.remainder() {
            counts[idx(k)] += 1;
        }
    }
    let mut start = 0;
    for d in 0..width {
        let c = counts[if mix { d ^ ((d >> 8) & 0xFF) } else { d }];
        if c != 0 {
            fill_run(keys, start, c as usize, decode(U::from_u64(d as u64)));
            start += c as usize;
        }
    }
    debug_assert_eq!(start, n);
}

/// Counting sort of keys spanning `width` integers from `base` (encoded),
/// at most 2^32 - 1 keys; 4 lanes of counters when the table is small
/// against the keys.
#[inline]
fn exact_count<U: WindowKey, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(keys: &mut [U], base: U, width: usize, encode: E, decode: D) {
    let n = keys.len();
    let base = base.to_u64();
    let lanes = lanes_for(width, n);
    let mut counts = vec![0u32; width * lanes];
    let missed = count_window(keys, encode, base, width, lanes, &mut counts);
    debug_assert!(missed.is_none());
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(width);
        for l in 0..3 {
            for (a, b) in c0.iter_mut().zip(rest[l * width..(l + 1) * width].iter()) {
                *a += *b;
            }
        }
    }
    let mut start = 0;
    for (d, &c) in counts[..width].iter().enumerate() {
        if c != 0 {
            let c = c as usize;
            fill_run(keys, start, c, decode(U::from_u64(base + d as u64)));
            start += c;
        }
    }
    debug_assert_eq!(start, n);
}

/// Counts `keys` in `counts` (`lanes` tables of `width` counters for the
/// keys `base..base + width`); stops at the first group of keys holding one
/// outside the window, uncounted, and returns its position.
#[inline(always)]
fn count_window<U: WindowKey, E: Fn(U) -> U + Copy>(keys: &[U], encode: E, base: u64, width: usize, lanes: usize, counts: &mut [u32]) -> Option<usize> {
    let off = |k: U| encode(k).to_u64().wrapping_sub(base) as usize;
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(width);
        let (c1, rest) = rest.split_at_mut(width);
        let (c2, c3) = rest.split_at_mut(width);
        let mut quads = keys.chunks_exact(4);
        let mut i = 0;
        for q in &mut quads {
            let (d0, d1, d2, d3) = (off(q[0]), off(q[1]), off(q[2]), off(q[3]));
            if d0.max(d1).max(d2.max(d3)) >= width {
                return Some(i);
            }
            // SAFETY: each offset is below width.
            unsafe {
                *c0.get_unchecked_mut(d0) += 1;
                *c1.get_unchecked_mut(d1) += 1;
                *c2.get_unchecked_mut(d2) += 1;
                *c3.get_unchecked_mut(d3) += 1;
            }
            i += 4;
        }
        for &k in quads.remainder() {
            let d = off(k);
            if d >= width {
                return Some(i);
            }
            c0[d] += 1;
            i += 1;
        }
    } else {
        for (i, &k) in keys.iter().enumerate() {
            let d = off(k);
            if d >= width {
                return Some(i);
            }
            // SAFETY: d < width.
            unsafe { *counts.get_unchecked_mut(d) += 1 };
        }
    }
    None
}

/// Unsigned key bits seen through u64 (window counting).
trait WindowKey: Copy {
    const MAX_U64: u64;
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
}

impl WindowKey for u8 {
    const MAX_U64: u64 = u8::MAX as u64;
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v as u8
    }
}

impl WindowKey for u16 {
    const MAX_U64: u64 = u16::MAX as u64;
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v as u16
    }
}

impl WindowKey for u32 {
    const MAX_U64: u64 = u32::MAX as u64;
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v as u32
    }
}

impl WindowKey for u64 {
    const MAX_U64: u64 = u64::MAX;
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self
    }
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v
    }
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
impl BitsOf for u8 {
    type Bits = u8;
}
impl BitsOf for i8 {
    type Bits = u8;
}
impl BitsOf for u16 {
    type Bits = u16;
}
impl BitsOf for i16 {
    type Bits = u16;
}

#[inline(always)]
fn id8(k: u8) -> u8 {
    k
}
#[inline(always)]
fn flip8(k: u8) -> u8 {
    k ^ (1 << 7)
}
#[inline(always)]
fn id16(k: u16) -> u16 {
    k
}
#[inline(always)]
fn flip16(k: u16) -> u16 {
    k ^ (1 << 15)
}

/// 8- and 16-bit keys never take the rollercoaster.
fn no_roller() -> usize {
    usize::MAX
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

int_width!(u32, u32, id32, id32, SMALL_U32, roller_int32, |b: &mut [u32], _s: &mut Vec<std::mem::MaybeUninit<u32>>| wu32::lsd(b), du32::dlsd_with, ru32::roller, rest_u32, few_u32, range_u32, dense_u32);
int_width!(i32, u32, flip32, flip32, SMALL_I32, roller_int32, |b: &mut [u32], _s: &mut Vec<std::mem::MaybeUninit<u32>>| wi32::lsd(b), di32::dlsd_with, ri32::roller, rest_i32, few_i32, range_i32, dense_i32);
int_width!(u64, u64, id64, id64, SMALL_U64, roller_int64, du64::dlsd_with, |b: &mut [u64], _s: &mut Vec<std::mem::MaybeUninit<u64>>| wu64::lsd(b), ru64::roller, rest_u64, few_u64, range_u64, dense_u64);
int_width!(i64, u64, flip64, flip64, SMALL_I64, roller_int64, di64::dlsd_with, |b: &mut [u64], _s: &mut Vec<std::mem::MaybeUninit<u64>>| wi64::lsd(b), ri64::roller, rest_i64, few_i64, range_i64, dense_i64);
// 8- and 16-bit keys: from 2^16 keys the narrow-range counting always
// applies (the whole domain is at most 2^16 integers); below, the fused LSD
// on bytes (one or two passes).
int_width!(u16, u16, id16, id16, SMALL_U16, no_roller, |b: &mut [u16], _s: &mut Vec<std::mem::MaybeUninit<u16>>| wu16::lsd(b), |b: &mut [u16], _s: &mut Vec<std::mem::MaybeUninit<u16>>| wu16::lsd(b), wu16::lsd, rest_u16, few_u16, range_u16, dense_u16);
int_width!(i16, u16, flip16, flip16, SMALL_I16, no_roller, |b: &mut [u16], _s: &mut Vec<std::mem::MaybeUninit<u16>>| wi16::lsd(b), |b: &mut [u16], _s: &mut Vec<std::mem::MaybeUninit<u16>>| wi16::lsd(b), wi16::lsd, rest_i16, few_i16, range_i16, dense_i16);
int_width!(u8, u8, id8, id8, SMALL_U8, no_roller, |b: &mut [u8], _s: &mut Vec<std::mem::MaybeUninit<u8>>| wu8::lsd(b), |b: &mut [u8], _s: &mut Vec<std::mem::MaybeUninit<u8>>| wu8::lsd(b), wu8::lsd, rest_u8, few_u8, range_u8, dense_u8);
int_width!(i8, u8, flip8, flip8, SMALL_I8, no_roller, |b: &mut [u8], _s: &mut Vec<std::mem::MaybeUninit<u8>>| wi8::lsd(b), |b: &mut [u8], _s: &mut Vec<std::mem::MaybeUninit<u8>>| wi8::lsd(b), wi8::lsd, rest_i8, few_i8, range_i8, dense_i8);

/// `char`: sorted as its code point (`u32`; the order of `char` is the
/// order of the code points, and sorting permutes valid values only).
impl DevourSort for char {
    #[inline]
    fn devour_sort_impl(arr: &mut [char]) {
        // SAFETY: `char` has the size and alignment of `u32` and every
        // `char` is a valid `u32`; the slice holds the same values once
        // sorted, all valid `char`s.
        let codes = unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut u32, arr.len()) };
        u32::devour_sort_impl(codes);
    }
}

/// `usize` / `isize`: sorted as the unsigned / signed integer of the same
/// width (same size, alignment and representation).
macro_rules! pointer_width {
    ($t:ty, $as:ty) => {
        impl DevourSort for $t {
            #[inline]
            fn devour_sort_impl(arr: &mut [$t]) {
                const _: () = assert!(std::mem::size_of::<$t>() == std::mem::size_of::<$as>());
                // SAFETY: same size, alignment and value representation.
                let same = unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $as, arr.len()) };
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
