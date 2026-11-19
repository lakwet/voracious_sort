//! Bytes (u8, i8): the order check and the counting sort over the 256
//! values, written for one-byte keys (2026-11-19, against
//! `counting::domain_count`, which also serves the 16-bit keys). Only
//! structural changes, no per-CPU setting:
//!
//! - the order check reads only: a reversed array is reversed only once it
//!   is known to be reversed (`presorted` reverses on 32 spread pairs and
//!   then scans, a whole-array write lost on descending saws);
//! - the counters on the stack (4 KiB), no allocation per call;
//! - 4 tables of counters at every size (one table below 512 keys made
//!   repeated keys wait on the same counter), `u16` counters up to 65535
//!   keys (half the memory to clear and to sum);
//! - the fill skips empty blocks of 16 values with one test;
//! - at most 256 keys (no more keys than values): one-byte counters in 2
//!   tables (at most 128 keys each, no overflow) and each key placed at the
//!   start of its cell in a 256-byte buffer on the stack, then copied back:
//!   work in the keys, not in the 256 cells (idea from blazing-radix 0.1.0,
//!   `byte_scatter`, reimplemented).
//!
//! Kept from `domain_count`: one `u64` load per 8 keys, the fill of each
//! value (runs of at most 64 by blocks of 16 copies). See
//! `algo_doc/comptage_octets.md`.

use std::mem::MaybeUninit;

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

/// Values of a byte.
const VALUES: usize = 1 << u8::BITS;
/// Tables of counters: equal keys up to `TABLES - 1` apart go to different
/// counters (a counter incremented twice in a row makes the second
/// increment wait for the first).
const TABLES: usize = 4;
/// Values per block of the fill (an empty block skipped with one test).
const BLOCK: usize = 16;

/// Counting sort of bytes, keys `b ^ FLIP` (0: u8; 0x80: the bits of i8).
/// At most 2^32 - 1 keys.
#[inline]
pub(crate) fn count<const FLIP: u8>(bytes: &mut [u8]) {
    if bytes.len() <= VALUES {
        place::<FLIP>(bytes);
    } else {
        count_fill::<FLIP>(bytes);
    }
}

/// Tables of one-byte counters of `place`: each sees at most
/// `VALUES / SMALL_TABLES` keys, below 256.
const SMALL_TABLES: usize = 2;

/// At most `VALUES` keys: one-byte counters (`SMALL_TABLES` tables, keys
/// dealt in turn), the start of each cell per table (so that two
/// neighbouring keys never move the same offset), each key placed in a
/// buffer on the stack, the buffer copied back.
#[inline(never)]
fn place<const FLIP: u8>(bytes: &mut [u8]) {
    let n = bytes.len();
    debug_assert!(n <= VALUES);
    let mut tables = [[0u8; VALUES]; SMALL_TABLES];
    let (pairs, rest) = bytes.as_chunks::<SMALL_TABLES>();
    for pair in pairs {
        for (t, &x) in pair.iter().enumerate() {
            tables[t][(x ^ FLIP) as usize] += 1;
        }
    }
    for (t, &x) in rest.iter().enumerate() {
        tables[t][(x ^ FLIP) as usize] += 1;
    }
    // Start of each cell for each table: table t's keys of value v follow
    // those of the tables before it. Counts summed first (independent
    // additions), then the starts by `starts_of` (short chains).
    let mut counts = [0u16; VALUES];
    for (v, c) in counts.iter_mut().enumerate() {
        *c = tables.iter().map(|t| t[v] as u16).sum();
    }
    let mut starts = [starts_of(&counts); SMALL_TABLES];
    for t in 1..SMALL_TABLES {
        for v in 0..VALUES {
            starts[t][v] = starts[t - 1][v] + tables[t - 1][v] as u16;
        }
    }
    // Not zeroed: positions 0..n are each written once below.
    let mut buffer = [MaybeUninit::<u8>::uninit(); VALUES];
    for pair in pairs {
        for (t, &x) in pair.iter().enumerate() {
            let at = &mut starts[t][(x ^ FLIP) as usize];
            buffer[*at as usize].write(x);
            *at += 1;
        }
    }
    for (t, &x) in rest.iter().enumerate() {
        let at = &mut starts[t][(x ^ FLIP) as usize];
        buffer[*at as usize].write(x);
        *at += 1;
    }
    // SAFETY: the starts partition 0..n (counts summing to n), every key
    // written once at its own position: buffer[..n] is initialized.
    let sorted = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, n) };
    bytes.copy_from_slice(sorted);
}

/// Values per block of `starts_of`.
const PREFIX_BLOCK: usize = 16;

/// Start of each cell (sum of the counts before it), in blocks of
/// `PREFIX_BLOCK` values: the block totals first, then each block from its
/// own start, so that the additions form short independent chains instead
/// of one chain over all the values.
#[inline(always)]
fn starts_of(counts: &[u16; VALUES]) -> [u16; VALUES] {
    let blocks = counts.as_chunks::<PREFIX_BLOCK>().0;
    let mut block_start = [0u16; VALUES / PREFIX_BLOCK];
    let mut next = 0u16;
    for (start, block) in block_start.iter_mut().zip(blocks) {
        *start = next;
        next += block.iter().sum::<u16>();
    }
    let mut starts = [0u16; VALUES];
    let outs = starts.as_chunks_mut::<PREFIX_BLOCK>().0;
    for ((out, block), &first) in outs.iter_mut().zip(blocks).zip(&block_start) {
        let mut at = first;
        for (o, &c) in out.iter_mut().zip(block) {
            *o = at;
            at += c;
        }
    }
    starts
}

/// More keys than values: `TABLES` tables of counters, as narrow as `n`
/// allows (`u16` up to 65535 keys: half the memory to clear and to sum),
/// then each value written again as many times as it was counted.
#[inline(never)]
fn count_fill<const FLIP: u8>(bytes: &mut [u8]) {
    if bytes.len() <= u16::MAX as usize {
        count_fill_with::<FLIP, u16>(bytes);
    } else {
        count_fill_with::<FLIP, u32>(bytes);
    }
}

#[inline(always)]
fn count_fill_with<const FLIP: u8, C>(bytes: &mut [u8])
where
    C: Copy + Default + PartialEq + From<u8> + Into<u64>,
    C: std::ops::AddAssign + std::ops::BitOr<Output = C>,
{
    let n = bytes.len();
    debug_assert!(n as u64 <= u32::MAX as u64);
    let one = C::from(1);
    let mut tables = [[C::default(); VALUES]; TABLES];
    let (words, rest) = bytes.as_chunks::<8>();
    for w in words {
        let w = u64::from_le_bytes(*w);
        for i in 0..8 {
            tables[i % TABLES][(((w >> (8 * i)) as u8) ^ FLIP) as usize] += one;
        }
    }
    for &x in rest {
        tables[0][(x ^ FLIP) as usize] += one;
    }
    let (counts, others) = tables.split_first_mut().unwrap();
    for t in others.iter() {
        for (c, &o) in counts.iter_mut().zip(t.iter()) {
            *c += o;
        }
    }
    let mut start = 0;
    for (block, cb) in counts.as_chunks::<BLOCK>().0.iter().enumerate() {
        if cb.iter().fold(C::default(), |any, &c| any | c) == C::default() {
            continue;
        }
        for (k, &c) in cb.iter().enumerate() {
            let c = c.into() as usize;
            if c != 0 {
                fill_run(bytes, start, c, ((block * BLOCK + k) as u8) ^ FLIP);
                start += c;
            }
        }
    }
    debug_assert_eq!(start, n);
}

/// Writes `c` copies of `v` from `start` (the fill of `domain_count`): runs
/// of at most 64 by blocks of 16 copies, the copies past `start + c`
/// overwritten by the following runs; longer runs and runs near the end by
/// `fill`.
#[inline(always)]
fn fill_run(keys: &mut [u8], start: usize, c: usize, v: u8) {
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
            let mut got = raw.to_vec();
            count::<0>(&mut got);
            assert!(got == expected, "u8 {} n={}", shape, raw.len());
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
