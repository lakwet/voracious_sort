//! Bytes (u8, i8): the order check and the counting sort over the 256
//! values, written for one-byte keys (2026-11-18, against
//! `counting::domain_count`):
//!
//! - the order check reads only: a reversed array is reversed only once it
//!   is known to be reversed (`presorted` reverses on 32 spread pairs and
//!   then scans, a whole-array write lost on descending saws);
//! - 4 tables of counters on the stack from the smallest size (one table
//!   below 512 keys made repeated keys wait on the same counter);
//! - the fill skips empty blocks of 16 values and writes short buckets with
//!   one fixed 16- or 64-byte store that the higher buckets overwrite (idea
//!   from blazing-radix 0.1.0, reimplemented);
//! - byte loads (`WORD_LOADS`: one `u64` per 8 keys and shifts).

use std::convert::TryInto;

use crate::v2::runs::verge::{explore_forward_asc, explore_forward_desc};

/// Whether every key equals `first`: blocks of 64 compared without early
/// exit inside a block (vectorized), an exit between blocks.
#[inline]
fn all_equal<T: Copy + PartialEq>(arr: &[T], first: T) -> bool {
    let (blocks, rest) = arr.as_chunks::<64>();
    for block in blocks {
        if !block.iter().fold(true, |all, &v| all & (v == first)) {
            return false;
        }
    }
    rest.iter().all(|&v| v == first)
}

/// Whether `arr` is sorted, reversed (then reversed in place) or constant;
/// nothing written otherwise.
#[inline]
pub(crate) fn sorted_or_reversed<T: Copy + PartialOrd>(arr: &mut [T]) -> bool {
    let n = arr.len();
    let (first, last) = (arr[0], arr[n - 1]);
    if first == last && first == arr[n / 2] && all_equal(arr, first) {
        return true;
    }
    if first <= last {
        return explore_forward_asc(arr, 0) == n;
    }
    if explore_forward_desc(arr, 0) == n {
        arr.reverse();
        return true;
    }
    false
}

/// Variant bits of `count_with` (rule under test): word loads (one `u64`
/// per 8 keys, shifts) instead of byte loads; the fill of
/// `counting::domain_count` (a branch per value, blocks of 16 copies).
pub(crate) const WORD_LOADS: u8 = 1;
pub(crate) const OLD_FILL: u8 = 4;

/// Counting sort of bytes, keys `b ^ FLIP` (0: u8; 0x80: the bits of i8).
/// At most 2^32 - 1 keys.
#[inline]
pub(crate) fn count<const FLIP: u8>(bytes: &mut [u8]) { count_with::<FLIP>(bytes, 0) }

/// `count` with the variant bits `v` (`WORD_LOADS`, `OLD_FILL`).
#[inline]
pub(crate) fn count_with<const FLIP: u8>(bytes: &mut [u8], v: u8) {
    match (v & WORD_LOADS != 0, v & OLD_FILL != 0) {
        (false, false) => count_impl::<FLIP, false, false>(bytes),
        (true, false) => count_impl::<FLIP, true, false>(bytes),
        (false, true) => count_impl::<FLIP, false, true>(bytes),
        (true, true) => count_impl::<FLIP, true, true>(bytes),
    }
}

#[inline(always)]
fn count_impl<const FLIP: u8, const WORD: bool, const OLD: bool>(bytes: &mut [u8]) {
    let n = bytes.len();
    debug_assert!(n as u64 <= u32::MAX as u64);
    // 4 tables: equal keys 1 to 3 apart go to different counters.
    let mut lanes = [[0u32; 256]; 4];
    let (blocks, rest) = bytes.as_chunks::<8>();
    if WORD {
        for b in blocks {
            let w = u64::from_le_bytes(*b);
            for i in 0..8 {
                lanes[i % 4][(((w >> (8 * i)) as u8) ^ FLIP) as usize] += 1;
            }
        }
    } else {
        for b in blocks {
            for (i, &x) in b.iter().enumerate() {
                lanes[i % 4][(x ^ FLIP) as usize] += 1;
            }
        }
    }
    for &x in rest {
        lanes[0][(x ^ FLIP) as usize] += 1;
    }
    let [mut counts, l1, l2, l3] = lanes;
    for v in 0..256 {
        counts[v] += l1[v] + l2[v] + l3[v];
    }
    if OLD {
        fill_by_value::<FLIP>(bytes, &counts);
    } else {
        fill_by_blocks::<FLIP>(bytes, &counts);
    }
}

/// The fill of `counting::domain_count`: a branch per value, runs of at
/// most 64 by blocks of 16 copies.
#[inline(always)]
fn fill_by_value<const FLIP: u8>(bytes: &mut [u8], counts: &[u32; 256]) {
    let n = bytes.len();
    let mut start = 0;
    for (d, &c) in counts.iter().enumerate() {
        let c = c as usize;
        if c != 0 {
            let v = (d as u8) ^ FLIP;
            let end = start + c;
            if c <= 64 && end + 16 <= n {
                let mut p = start;
                while p < end {
                    bytes[p..p + 16].copy_from_slice(&[v; 16]);
                    p += 16;
                }
            } else {
                bytes[start..end].fill(v);
            }
            start = end;
        }
    }
    debug_assert_eq!(start, n);
}

/// Fill by blocks of 16 values: an empty block skipped with one branch; in
/// the others, a bucket of at most 16 (or 64) keys written with one fixed
/// 16-byte (64-byte) store whose excess the higher buckets overwrite.
#[inline(always)]
fn fill_by_blocks<const FLIP: u8>(bytes: &mut [u8], counts: &[u32; 256]) {
    let n = bytes.len();
    let mut start = 0;
    for (block, c16) in counts.as_chunks::<16>().0.iter().enumerate() {
        if c16.iter().fold(0, |any, &c| any | c) == 0 {
            continue;
        }
        for (k, &c) in c16.iter().enumerate() {
            let v = ((block * 16 + k) as u8) ^ FLIP;
            let c = c as usize;
            // The bytes past `start + c` belong to higher buckets, written
            // after this one.
            if c <= 16 && start + 16 <= n {
                let dst: &mut [u8; 16] = (&mut bytes[start..start + 16]).try_into().unwrap();
                *dst = [v; 16];
            } else if c <= 64 && start + 64 <= n {
                let dst: &mut [u8; 64] = (&mut bytes[start..start + 64]).try_into().unwrap();
                *dst = [v; 64];
            } else {
                bytes[start..start + c].fill(v);
            }
            start += c;
        }
    }
    debug_assert_eq!(start, n);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    #[test]
    fn counting_and_order() {
        for_each_case::<u8>(|shape, raw| {
            if raw.is_empty() {
                return;
            }
            let mut expected = raw.to_vec();
            expected.sort_unstable();
            for v in 0..8 {
                let mut got = raw.to_vec();
                count_with::<0>(&mut got, v);
                assert!(got == expected, "u8 {} n={} variant {}", shape, raw.len(), v);
            }
            // i8 through its bits.
            let signed: Vec<i8> = raw.iter().map(|&x| x as i8).collect();
            let mut bits: Vec<u8> = signed.iter().map(|&x| x as u8).collect();
            count::<0x80>(&mut bits);
            let got: Vec<i8> = bits.iter().map(|&x| x as i8).collect();
            let mut expected = signed.clone();
            expected.sort_unstable();
            assert!(got == expected, "i8 {} n={}", shape, raw.len());
            // The order check: true exactly on sorted, reversed, constant.
            let mut a = raw.to_vec();
            let sorted = a.windows(2).all(|p| p[0] <= p[1]);
            let reversed = a.windows(2).all(|p| p[0] >= p[1]);
            assert_eq!(sorted_or_reversed(&mut a), sorted || reversed, "{}", shape);
            if sorted || reversed {
                assert!(a.windows(2).all(|p| p[0] <= p[1]), "{}", shape);
            } else {
                assert_eq!(a, raw.to_vec(), "{}", shape);
            }
        });
    }
}
