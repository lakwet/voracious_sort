//! The 32-bit block merge streams on NEON (aarch64).

use std::arch::aarch64::*;

type E = u32;
type V = uint32x4_t;

/// `Lane::key` on four lanes.
#[inline(always)]
unsafe fn to_key<const M: u8>(v: V) -> V {
    let sign = vdupq_n_u32(1 << 31);
    match M {
        0 => v,
        1 => veorq_u32(v, sign),
        _ => veorq_u32(
            v,
            vorrq_u32(vreinterpretq_u32_s32(vshrq_n_s32::<31>(vreinterpretq_s32_u32(v))), sign),
        ),
    }
}

/// Inverse of `to_key`.
#[inline(always)]
unsafe fn from_key<const M: u8>(v: V) -> V {
    let sign = vdupq_n_u32(1 << 31);
    match M {
        0 => v,
        1 => veorq_u32(v, sign),
        _ => veorq_u32(
            v,
            vorrq_u32(
                vreinterpretq_u32_s32(vshrq_n_s32::<31>(vreinterpretq_s32_u32(vmvnq_u32(v)))),
                sign,
            ),
        ),
    }
}

#[inline(always)]
unsafe fn ld<const M: u8>(p: *const u32) -> V { to_key::<M>(vld1q_u32(p)) }

#[inline(always)]
unsafe fn st<const M: u8>(p: *mut u32, v: V) { vst1q_u32(p, from_key::<M>(v)) }

/// A plain select: LLVM makes it a `csel` here.
#[inline(always)]
fn pick(c: bool, a: *const u32, b: *const u32) -> *const u32 {
    if c {
        a
    } else {
        b
    }
}

/// Bitonic merge of two sorted blocks of four: the four smallest and
/// the four largest, each sorted.
#[inline(always)]
unsafe fn net(a: V, b: V) -> (V, V) {
    // b reversed: a ascending then b descending is bitonic.
    let r = vrev64q_u32(b);
    let b = vextq_u32::<2>(r, r);
    let lo = vminq_u32(a, b);
    let hi = vmaxq_u32(a, b);
    // Distance 2 within each half.
    let x = vreinterpretq_u32_u64(vzip1q_u64(vreinterpretq_u64_u32(lo), vreinterpretq_u64_u32(hi)));
    let y = vreinterpretq_u32_u64(vzip2q_u64(vreinterpretq_u64_u32(lo), vreinterpretq_u64_u32(hi)));
    let m = vminq_u32(x, y);
    let n = vmaxq_u32(x, y);
    // Distance 1.
    let x = vtrn1q_u32(m, n);
    let y = vtrn2q_u32(m, n);
    let mn = vminq_u32(x, y);
    let mx = vmaxq_u32(x, y);
    (vzip1q_u32(mn, mx), vzip2q_u32(mn, mx))
}

streams!("neon");
