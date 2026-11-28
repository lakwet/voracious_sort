//! Counting sorts of integer keys: over the whole domain (8- and 16-bit
//! keys), over an exact range, or in a window that follows the keys in one
//! pass. Counters in 4 lanes when the table is small against the keys
//! (repeated values would otherwise wait on the same counter). At most
//! 2^32 - 1 keys (`u32` counters).

use crate::v2::cache::{budget, Budget};
use crate::v2::devour::keys::Key;

/// Trials of the 32-bit chain copy (its mask; never set for the library).
#[allow(dead_code)]
#[inline(always)]
fn trial(bit: u64) -> bool {
    #[cfg(feature = "ab")]
    {
        crate::v2::devour::ab_copy32::SKIP.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
    }
    #[cfg(not(feature = "ab"))]
    {
        let _ = bit;
        false
    }
}

/// The 4 keys of a quad, 32-bit keys read by two 64-bit loads (trial
/// 1 << 61: the loop is bound by its loads, 4 key loads + 4 counter loads
/// per quad, 6 this way).
#[inline(always)]
fn quad<U: Key>(q: &[U; 4]) -> [U; 4] {
    if std::mem::size_of::<U>() == 4 && trial(1 << 61) {
        // SAFETY: [U; 4] of 4-byte keys is 16 bytes, read unaligned.
        let w: [u64; 2] = unsafe { std::ptr::read_unaligned(q.as_ptr() as *const [u64; 2]) };
        [
            U::from_u64(w[0] & 0xFFFF_FFFF),
            U::from_u64(w[0] >> 32),
            U::from_u64(w[1] & 0xFFFF_FFFF),
            U::from_u64(w[1] >> 32),
        ]
    } else {
        *q
    }
}

/// Whether the 4-equal-keys shortcut pays (trial 1 << 62, as the 16-bit
/// domain count): at least half of 64 spread quads made of one value.
#[inline]
fn runs_pay<U: Key>(keys: &[U]) -> bool {
    if !trial(1 << 62) {
        return false;
    }
    let (quads, _) = keys.as_chunks::<4>();
    if quads.len() < 64 {
        return false;
    }
    let step = quads.len() / 64;
    (0..64).filter(|&i| {
        eq4(&quads[i * step])
    })
    .count()
        >= 32
}

/// The 4 keys of a quad equal.
#[inline(always)]
fn eq4<U: Key>(q: &[U; 4]) -> bool {
    let a = q[0].to_u64();
    a == q[1].to_u64() && a == q[2].to_u64() && a == q[3].to_u64()
}

/// Counters held on the stack (trial 1 << 59): tables of 32 768 to 65 536
/// (128 to 256 KiB), new pages at each call on the heap (smaller ones come
/// from the allocator's reused memory).
const STACK_COUNTERS: usize = 1 << 16;

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
pub(crate) fn window_count<U: Key, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(
    keys: &mut [U],
    lo: U,
    hi: U,
    limit: usize,
    encode: E,
    decode: D,
) -> bool {
    window_count_split(keys, lo, hi, limit, encode, decode, 0)
}

/// `window_count` writing the encoded values from `split` on first, then
/// the ones below it (keys counted rotated by half the domain: written back
/// in their true order, no rotation of the array afterwards).
#[inline]
pub(crate) fn window_count_split<U: Key, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(
    keys: &mut [U],
    lo: U,
    hi: U,
    limit: usize,
    encode: E,
    decode: D,
    split: u64,
) -> bool {
    let n = keys.len();
    let (lo, hi) = (encode(lo).to_u64(), encode(hi).to_u64());
    let mut width = ((hi - lo + 1) as usize * 2).max(256).min(limit);
    // Centre the window on the sample range, inside the key domain.
    let pad = (width as u64 - (hi - lo + 1)) / 2;
    let mut base = lo.saturating_sub(pad).min(U::MAX_U64 - (width as u64 - 1));
    let mut lanes = lanes_for(width, n);
    let mut stack;
    let mut heap: Vec<u32>;
    let mut counts: &mut [u32] = if trial(1 << 59) && (STACK_COUNTERS / 2..=STACK_COUNTERS).contains(&(width * lanes)) {
        stack = [0u32; STACK_COUNTERS];
        &mut stack[..width * lanes]
    } else {
        heap = vec![0u32; width * lanes];
        &mut heap
    };
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
        heap = new_counts;
        counts = &mut heap;
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

/// Widest table counted on 4 lanes; beyond, one lane, which keeps the
/// table in cache (measured on u32 from 100k to 10M keys; 4096 was too
/// small).
const LANES_MAX_WIDTH: usize = 8192;

/// Counter lanes of a count table of `width` counters for `n` keys: 4 lanes
/// (repeated values would otherwise wait on the previous increment of their
/// counter) when the 4 tables fit in an eighth of the L2 (12 MiB L2: up to
/// ~98k counters; 256 KiB: 2048) and hold at most one counter per 2 keys;
/// else 1 lane (keys spread over many counters rarely repeat, and 4 copies
/// would leave the cache of small-L2 cores). Measured on 32- and 64-bit
/// keys: a fixed 4096-counter cap was 1-2 % slower at 3M-10M (12 MiB L2).
#[inline]
fn lanes_for(width: usize, n: usize) -> usize {
    if width <= LANES_MAX_WIDTH && 4 * width * 4 <= budget(Budget::CountLanes) / 8 && 2 * width <= n
    {
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
pub(crate) fn domain_count<U: Key, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(
    keys: &mut [U],
    encode: E,
    decode: D,
) {
    let width = (U::MAX_U64 + 1) as usize;
    // Wide domains (16-bit keys): 4 lanes only when keys repeat nearby; one
    // lane keeps a quarter of the table in cache.
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
fn shared_low_byte<U: Key>(keys: &[U]) -> bool {
    let n = keys.len();
    let first = keys[0].to_u64() & 0xFF;
    (1..16).all(|i| keys[i * (n / 16)].to_u64() & 0xFF == first)
}

/// Whether keys repeat within a few positions (on 256 spread positions, at
/// least 1 in 16 equal to one of the next 3 keys): the increments of one
/// counter would then wait on each other, and 4 lanes of counters pay off;
/// otherwise one lane (a quarter of the table, kept closer in cache).
#[inline]
fn repeats_near<U: Key>(keys: &[U]) -> bool {
    let n = keys.len();
    if n < 1024 {
        return true;
    }
    let mut hits = 0;
    for i in 0..256u64 {
        let p = (((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * (n - 4) as u64) >> 32) as usize;
        let k = keys[p].to_u64();
        hits += (k == keys[p + 1].to_u64()
            || k == keys[p + 2].to_u64()
            || k == keys[p + 3].to_u64()) as usize;
    }
    hits >= 16
}

#[inline(always)]
/// `domain_count` with the index mixing and the lanes fixed.
fn domain_count_with<
    U: Key,
    E: Fn(U) -> U + Copy,
    D: Fn(U) -> U + Copy,
    const MIX: bool,
    const FOUR: bool,
>(
    keys: &mut [U],
    encode: E,
    decode: D,
) {
    let n = keys.len();
    let width = (U::MAX_U64 + 1) as usize;
    let lanes = if FOUR { 4 } else { 1 };
    let mut counts = vec![0u32; width * lanes];
    // Wide domains (16-bit keys), `MIX`: counter of key k at k ^ (k >> 8)
    // (a bijection within each block of 256 counters). Keys sharing their
    // low byte would otherwise hit counters 1 KiB apart, all in the same few
    // L1 sets (ten times slower on multiples of 256).
    let idx = |k: U| {
        let e = encode(k).to_u64() as usize;
        if MIX {
            e ^ ((e >> 8) & 0xFF)
        } else {
            e
        }
    };
    let (mut quads, mut tail) = keys.as_chunks::<4>();
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(width);
        let (c1, rest) = rest.split_at_mut(width);
        let (c2, c3) = rest.split_at_mut(width);
        if std::mem::size_of::<U>() == 1 {
            // Bytes: one 64-bit load per 8 keys (u8 / i8 counting 3 to 6 %
            // faster than one load per key).
            // SAFETY: `U` is a 1-byte integer here.
            let bytes = unsafe { std::slice::from_raw_parts(keys.as_ptr() as *const u8, n) };
            let (words, rest) = bytes.as_chunks::<8>();
            let at = |w: u64, i: u32| idx(U::from_u64((w >> (8 * i)) & 0xFF));
            for c in words {
                let w = u64::from_le_bytes(*c);
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
            let done = n - rest.len();
            for &k in keys[done..].iter() {
                c0[idx(k)] += 1;
            }
            (quads, tail) = (&[], &[]);
        }
        for q in quads {
            // SAFETY: an encoded key is below the domain size `width`.
            unsafe {
                *c0.get_unchecked_mut(idx(q[0])) += 1;
                *c1.get_unchecked_mut(idx(q[1])) += 1;
                *c2.get_unchecked_mut(idx(q[2])) += 1;
                *c3.get_unchecked_mut(idx(q[3])) += 1;
            }
        }
        for &k in tail {
            c0[idx(k)] += 1;
        }
        for l in 1..4 {
            let (c0, rest) = counts.split_at_mut(width);
            for (a, b) in c0.iter_mut().zip(rest[(l - 1) * width..l * width].iter()) {
                *a += *b;
            }
        }
    } else {
        for q in quads {
            // SAFETY: as above.
            unsafe {
                *counts.get_unchecked_mut(idx(q[0])) += 1;
                *counts.get_unchecked_mut(idx(q[1])) += 1;
                *counts.get_unchecked_mut(idx(q[2])) += 1;
                *counts.get_unchecked_mut(idx(q[3])) += 1;
            }
        }
        for &k in tail {
            counts[idx(k)] += 1;
        }
    }
    let mut start = 0;
    for d in 0..width {
        let c = counts[if MIX { d ^ ((d >> 8) & 0xFF) } else { d }];
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
pub(crate) fn exact_count<U: Key, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(
    keys: &mut [U],
    base: U,
    width: usize,
    encode: E,
    decode: D,
) {
    let n = keys.len();
    let base = base.to_u64();
    let lanes = lanes_for(width, n);
    // Tables of a power of two: offsets masked, no bound check.
    let size = width.next_power_of_two();
    let mut stack;
    let mut heap: Vec<u32>;
    let counts: &mut [u32] = if trial(1 << 59) && (STACK_COUNTERS / 2..=STACK_COUNTERS).contains(&(size * lanes)) {
        stack = [0u32; STACK_COUNTERS];
        &mut stack[..size * lanes]
    } else {
        heap = vec![0u32; size * lanes];
        &mut heap
    };
    count_masked(keys, encode, base, size, lanes, counts);
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(size);
        for l in 0..3 {
            for (a, b) in c0[..width].iter_mut().zip(&rest[l * size..l * size + width]) {
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

/// Counts `keys`, all within `base..base + size` (`size` a power of two),
/// in `lanes` tables of `size` counters: offsets masked, no bound check.
#[inline(always)]
fn count_masked<U: Key, E: Fn(U) -> U + Copy>(
    keys: &[U],
    encode: E,
    base: u64,
    size: usize,
    lanes: usize,
    counts: &mut [u32],
) {
    let mask = size - 1;
    let off = |k: U| encode(k).to_u64().wrapping_sub(base) as usize & mask;
    let (quads, tail) = keys.as_chunks::<4>();
    let run4 = runs_pay(keys);
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(size);
        let (c1, rest) = rest.split_at_mut(size);
        let (c2, c3) = rest.split_at_mut(size);
        let (c0, c1, c2, c3) = (&mut c0[..size], &mut c1[..size], &mut c2[..size], &mut c3[..size]);
        for q in quads {
            let q = quad(q);
            if run4 && eq4(&q) {
                c0[off(q[0])] += 4;
                continue;
            }
            // SAFETY: a masked offset is below `size`, the length of each table.
            unsafe {
                *c0.get_unchecked_mut(off(q[0])) += 1;
                *c1.get_unchecked_mut(off(q[1])) += 1;
                *c2.get_unchecked_mut(off(q[2])) += 1;
                *c3.get_unchecked_mut(off(q[3])) += 1;
            }
        }
        for &k in tail {
            c0[off(k)] += 1;
        }
    } else {
        let c = &mut counts[..size];
        for q in quads {
            let q = quad(q);
            if run4 && eq4(&q) {
                c[off(q[0])] += 4;
                continue;
            }
            // SAFETY: as above.
            unsafe {
                *c.get_unchecked_mut(off(q[0])) += 1;
                *c.get_unchecked_mut(off(q[1])) += 1;
                *c.get_unchecked_mut(off(q[2])) += 1;
                *c.get_unchecked_mut(off(q[3])) += 1;
            }
        }
        for &k in tail {
            c[off(k)] += 1;
        }
    }
}

/// Counts `keys` in `counts` (`lanes` tables of `width` counters for the
/// keys `base..base + width`); stops at the first group of keys holding one
/// outside the window, uncounted, and returns its position.
#[inline(always)]
fn count_window<U: Key, E: Fn(U) -> U + Copy>(
    keys: &[U],
    encode: E,
    base: u64,
    width: usize,
    lanes: usize,
    counts: &mut [u32],
) -> Option<usize> {
    let off = |k: U| encode(k).to_u64().wrapping_sub(base) as usize;
    let run4 = runs_pay(keys);
    if lanes == 4 {
        let (c0, rest) = counts.split_at_mut(width);
        let (c1, rest) = rest.split_at_mut(width);
        let (c2, c3) = rest.split_at_mut(width);
        let (quads, tail) = keys.as_chunks::<4>();
        let mut i = 0;
        for q in quads {
            let q = quad(q);
            let (d0, d1, d2, d3) = (off(q[0]), off(q[1]), off(q[2]), off(q[3]));
            if d0.max(d1).max(d2.max(d3)) >= width {
                return Some(i);
            }
            if run4 && d0 == d1 && d1 == d2 && d2 == d3 {
                c0[d0] += 4;
                i += 4;
                continue;
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
        for &k in tail {
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

#[cfg(test)] mod tests;
