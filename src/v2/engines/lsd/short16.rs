//! LSD of short 16-bit arrays (up to `SHORT_MAX` keys): two byte passes
//! through a buffer (on the stack up to `STACK_MAX` keys), both histograms
//! in one read. The bits are
//! sorted as keys whose order is that of `bits ^ F` (0 for u16, the sign bit
//! for i16).
//!
//! The fixed cost is what matters at these sizes: one counting table per
//! byte unless neighbours share their high byte (`near16`: smooth or repeated
//! keys, where one table would wait on its own counter), counted in place
//! (tables moved out of a branch get copied). Ideas from blazing-radix (MIT
//! License, Copyright (c) 2026 Max Tikhomirov): 4 keys read before their
//! stores, 4 keys of one digit taking their slots in one update; from 4096
//! keys, 8 keys sharing a byte adding 8 at once.

use std::mem::MaybeUninit;

/// The largest array sorted here (counts below 2^32 all the same; the
/// 16-bit chain counts the whole domain beyond).
pub(crate) const SHORT_MAX: usize = 1 << 16;

/// Up to this size the buffer on the stack (11 KiB), beyond it on the heap.
pub(crate) const STACK_MAX: usize = 5504;

/// From this size the count by blocks of 8 keys on two tables.
const SMOOTH_MIN: usize = 4096;

/// Sorts the bits `a` (at most `SHORT_MAX`) in the order of `bits ^ F`.
#[inline(never)]
pub(crate) fn lsd_short<const F: u16>(a: &mut [u16]) {
    debug_assert!(a.len() <= SHORT_MAX);
    if a.len() < 16 {
        a.sort_unstable_by_key(|&k| k ^ F);
    } else if a.len() >= SMOOTH_MIN {
        lsd::<F, 3>(a)
    } else if near16(a) {
        lsd::<F, 2>(a)
    } else {
        lsd::<F, 1>(a)
    }
}

/// At least 3 of the first 16 neighbours share their high byte.
#[inline(always)]
fn near16(a: &[u16]) -> bool {
    let h = a.first_chunk::<16>().unwrap();
    (0..15).filter(|&i| h[i] >> 8 == h[i + 1] >> 8).count() >= 3
}

/// `L`: 1 one table per byte; 2 keys in turn on two tables; 3 two tables,
/// 8 keys sharing a byte added at once.
fn lsd<const F: u16, const L: usize>(a: &mut [u16]) {
    let n = a.len();
    let ord = |k: u16| k ^ F;
    let mut lo = [0u32; 256];
    let mut hi = [0u32; 256];
    if L == 1 {
        let (quads, rest) = a.as_chunks::<4>();
        for q in quads {
            for &k in q {
                lo[(k & 255) as usize] += 1;
                hi[(ord(k) >> 8) as usize] += 1;
            }
        }
        for &k in rest {
            lo[(k & 255) as usize] += 1;
            hi[(ord(k) >> 8) as usize] += 1;
        }
    } else {
        let mut l1 = [0u32; 256];
        let mut h1 = [0u32; 256];
        if L == 3 {
            let (octs, rest) = a.as_chunks::<8>();
            for o in octs {
                let l0 = o[0] & 255;
                if o.iter().all(|&k| k & 255 == l0) {
                    lo[l0 as usize] += 8;
                } else {
                    for i in 0..4 {
                        lo[(o[2 * i] & 255) as usize] += 1;
                        l1[(o[2 * i + 1] & 255) as usize] += 1;
                    }
                }
                let h0 = ord(o[0]) >> 8;
                if o.iter().all(|&k| ord(k) >> 8 == h0) {
                    hi[h0 as usize] += 8;
                } else {
                    for i in 0..4 {
                        hi[(ord(o[2 * i]) >> 8) as usize] += 1;
                        h1[(ord(o[2 * i + 1]) >> 8) as usize] += 1;
                    }
                }
            }
            for &k in rest {
                lo[(k & 255) as usize] += 1;
                hi[(ord(k) >> 8) as usize] += 1;
            }
        } else {
            let (pairs, rest) = a.as_chunks::<2>();
            for p in pairs {
                lo[(p[0] & 255) as usize] += 1;
                hi[(ord(p[0]) >> 8) as usize] += 1;
                l1[(p[1] & 255) as usize] += 1;
                h1[(ord(p[1]) >> 8) as usize] += 1;
            }
            for &k in rest {
                lo[(k & 255) as usize] += 1;
                hi[(ord(k) >> 8) as usize] += 1;
            }
        }
        for d in 0..256 {
            lo[d] += l1[d];
            hi[d] += h1[d];
        }
    }
    let lo_varies = lo[(a[0] & 255) as usize] as usize != n;
    let hi_varies = hi[(ord(a[0]) >> 8) as usize] as usize != n;
    match (lo_varies, hi_varies) {
        (false, false) => return,
        (true, false) => return put_byte::<F, 0>(a, &lo),
        (false, true) => return put_byte::<F, 8>(a, &hi),
        _ => {},
    }
    // Both prefix sums in one loop: two independent chains.
    let (mut sl, mut sh) = (0u32, 0u32);
    for d in 0..256 {
        let (x, y) = (lo[d], hi[d]);
        lo[d] = sl;
        hi[d] = sh;
        sl += x;
        sh += y;
    }
    if n <= STACK_MAX {
        let mut buf = [MaybeUninit::<u16>::uninit(); STACK_MAX];
        // SAFETY: `buf` holds STACK_MAX >= n slots.
        unsafe { passes::<F>(a, buf.as_mut_ptr() as *mut u16, &mut lo, &mut hi) }
    } else {
        let mut buf: Vec<MaybeUninit<u16>> = Vec::with_capacity(n);
        // SAFETY: `buf` holds n slots.
        unsafe { passes::<F>(a, buf.as_mut_ptr() as *mut u16, &mut lo, &mut hi) }
    }
}

/// Both scatters through `tmp`.
/// SAFETY: `tmp` holds at least `a.len()` slots; `lo` and `hi` are the
/// prefix sums of the two bytes over `a`.
#[inline(always)]
unsafe fn passes<const F: u16>(
    a: &mut [u16],
    tmp: *mut u16,
    lo: &mut [u32; 256],
    hi: &mut [u32; 256],
) {
    let n = a.len();
    let src = a.as_mut_ptr();
    // SAFETY: each pass writes the n keys to the n slots given by the
    // prefix sums of its digit (a permutation of 0..n), all written by the
    // first pass before the second reads them.
    unsafe {
        pass::<F, 0>(src, tmp, n, lo);
        pass::<F, 8>(tmp, src, n, hi);
    }
}

/// One byte varies (shift `S`), its counts `t`: each count is one value
/// (the other byte that of `a[0]`), written in order.
pub(crate) fn put_byte<const F: u16, const S: u32>(a: &mut [u16], t: &[u32; 256]) {
    let rest = (a[0] ^ F) & !(255 << S);
    let mut at = 0;
    for (d, &c) in t.iter().enumerate() {
        let m = c as usize;
        put(a, at, m, (rest | ((d as u16) << S)) ^ F);
        at += m;
    }
}

/// `m` copies of `v` from `at`, by blocks of 16 while 16 slots remain (a
/// block may spill over the next values' slots, written after; an empty
/// value costs one store and no branch taken).
#[inline(always)]
pub(crate) fn put(a: &mut [u16], at: usize, m: usize, v: u16) {
    const W: usize = 16;
    let n = a.len();
    let p = a.as_mut_ptr();
    let end = at + m;
    let mut i = at;
    // SAFETY: every store stays below `n`.
    unsafe {
        loop {
            if i + W <= n {
                (p.add(i) as *mut [u16; W]).write_unaligned([v; W]);
                i += W;
                if i >= end {
                    break;
                }
            } else {
                while i < end {
                    *p.add(i) = v;
                    i += 1;
                }
                break;
            }
        }
    }
}

/// The scatter of the byte at shift `S` from `src` to `dst`, by four keys
/// (four of one digit in one update).
/// SAFETY: `src` and `dst` hold `n` keys, `t` the prefix sums of that byte
/// over them.
#[inline(always)]
unsafe fn pass<const F: u16, const S: u32>(
    src: *const u16,
    dst: *mut u16,
    n: usize,
    t: &mut [u32; 256],
) {
    let dig = |k: u16| (((k ^ F) >> S) & 255) as usize;
    let mut i = 0;
    // SAFETY: the caller's.
    unsafe {
        while i + 4 <= n {
            let (k0, k1, k2, k3) = (*src.add(i), *src.add(i + 1), *src.add(i + 2), *src.add(i + 3));
            let (d0, d1, d2, d3) = (dig(k0), dig(k1), dig(k2), dig(k3));
            let (p0, p1, p2, p3);
            if d0 == d1 && d1 == d2 && d2 == d3 {
                p0 = t[d0] as usize;
                (p1, p2, p3) = (p0 + 1, p0 + 2, p0 + 3);
                t[d0] += 4;
            } else {
                p0 = t[d0] as usize;
                t[d0] += 1;
                p1 = t[d1] as usize;
                t[d1] += 1;
                p2 = t[d2] as usize;
                t[d2] += 1;
                p3 = t[d3] as usize;
                t[d3] += 1;
            }
            *dst.add(p0) = k0;
            *dst.add(p1) = k1;
            *dst.add(p2) = k2;
            *dst.add(p3) = k3;
            i += 4;
        }
        while i < n {
            let k = *src.add(i);
            let d = &mut t[dig(k)];
            *dst.add(*d as usize) = k;
            *d += 1;
            i += 1;
        }
    }
}
