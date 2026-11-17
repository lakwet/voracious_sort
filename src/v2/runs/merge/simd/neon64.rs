//! The 64-bit block merge streams on NEON (aarch64): four keys in two
//! registers.

use std::arch::aarch64::*;

type E = u64;
/// Four keys in two registers.
type V = (uint64x2_t, uint64x2_t);

#[inline(always)]
unsafe fn to_key<const M: u8>(v: uint64x2_t) -> uint64x2_t {
    let sign = vdupq_n_u64(1 << 63);
    match M {
        0 => v,
        1 => veorq_u64(v, sign),
        _ => veorq_u64(
            v,
            vorrq_u64(vreinterpretq_u64_s64(vshrq_n_s64::<63>(vreinterpretq_s64_u64(v))), sign),
        ),
    }
}

#[inline(always)]
unsafe fn from_key<const M: u8>(v: uint64x2_t) -> uint64x2_t {
    let sign = vdupq_n_u64(1 << 63);
    match M {
        0 => v,
        1 => veorq_u64(v, sign),
        _ => {
            let not = veorq_u64(v, vdupq_n_u64(u64::MAX));
            veorq_u64(
                v,
                vorrq_u64(
                    vreinterpretq_u64_s64(vshrq_n_s64::<63>(vreinterpretq_s64_u64(not))),
                    sign,
                ),
            )
        },
    }
}

#[inline(always)]
unsafe fn ld<const M: u8>(p: *const u64) -> V {
    (to_key::<M>(vld1q_u64(p)), to_key::<M>(vld1q_u64(p.add(2))))
}

#[inline(always)]
unsafe fn st<const M: u8>(p: *mut u64, v: V) {
    vst1q_u64(p, from_key::<M>(v.0));
    vst1q_u64(p.add(2), from_key::<M>(v.1));
}

#[inline(always)]
fn pick(c: bool, a: *const u64, b: *const u64) -> *const u64 {
    if c {
        a
    } else {
        b
    }
}

/// Compare-exchange lane by lane (no 64-bit min / max in NEON).
#[inline(always)]
unsafe fn cx(a: uint64x2_t, b: uint64x2_t) -> (uint64x2_t, uint64x2_t) {
    let gt = vcgtq_u64(a, b);
    (vbslq_u64(gt, b, a), vbslq_u64(gt, a, b))
}

/// Bitonic merge of two sorted blocks of four (two registers each).
#[inline(always)]
unsafe fn net(a: V, b: V) -> (V, V) {
    // b reversed: (b3 b2) (b1 b0).
    let (r0, r1) = (vextq_u64::<1>(b.1, b.1), vextq_u64::<1>(b.0, b.0));
    let (l0, h0) = cx(a.0, r0);
    let (l1, h1) = cx(a.1, r1);
    // Distance 2 within each half.
    let (l0, l1) = cx(l0, l1);
    let (h0, h1) = cx(h0, h1);
    // Distance 1: the two lanes of each register.
    let fin = |x0: uint64x2_t, x1: uint64x2_t| {
        let (mn, mx) = cx(vzip1q_u64(x0, x1), vzip2q_u64(x0, x1));
        (vzip1q_u64(mn, mx), vzip2q_u64(mn, mx))
    };
    (fin(l0, l1), fin(h0, h1))
}

streams!("neon");
