//! The 32-bit block merge streams on SSE4.1 (x86_64).

use std::arch::x86_64::*;

type E = u32;
type V = __m128i;

/// `Lane::key` on four lanes.
#[inline(always)]
unsafe fn to_key<const M: u8>(v: V) -> V {
    let sign = _mm_set1_epi32(i32::MIN);
    match M {
        0 => v,
        1 => _mm_xor_si128(v, sign),
        _ => _mm_xor_si128(v, _mm_or_si128(_mm_srai_epi32::<31>(v), sign)),
    }
}

/// Inverse of `to_key`.
#[inline(always)]
unsafe fn from_key<const M: u8>(v: V) -> V {
    let sign = _mm_set1_epi32(i32::MIN);
    match M {
        0 => v,
        1 => _mm_xor_si128(v, sign),
        _ => _mm_xor_si128(
            v,
            _mm_or_si128(_mm_srai_epi32::<31>(_mm_xor_si128(v, _mm_set1_epi32(-1))), sign),
        ),
    }
}

#[inline(always)]
unsafe fn ld<const M: u8>(p: *const u32) -> V { to_key::<M>(_mm_loadu_si128(p as *const V)) }

#[inline(always)]
unsafe fn st<const M: u8>(p: *mut u32, v: V) { _mm_storeu_si128(p as *mut V, from_key::<M>(v)) }

/// A plain `if` became a branch here, mispredicted on random runs: written
/// as a `cmov`.
#[inline(always)]
fn pick(c: bool, a: *const u32, b: *const u32) -> *const u32 {
    std::hint::select_unpredictable(c, a, b)
}

/// Same network as the NEON one, lane moves by shuffles and blends.
#[inline(always)]
unsafe fn net(a: V, b: V) -> (V, V) {
    // b reversed: a ascending then b descending is bitonic.
    let b = _mm_shuffle_epi32::<0x1B>(b);
    let lo = _mm_min_epu32(a, b);
    let hi = _mm_max_epu32(a, b);
    // Distance 2: (lo0 lo1 hi0 hi1) against (lo2 lo3 hi2 hi3).
    let x = _mm_unpacklo_epi64(lo, hi);
    let y = _mm_unpackhi_epi64(lo, hi);
    let m = _mm_min_epu32(x, y);
    let n = _mm_max_epu32(x, y);
    // Distance 1: (m0 n0 m2 n2) against (m1 n1 m3 n3).
    let x = _mm_blend_epi16::<0xCC>(m, _mm_slli_si128::<4>(n));
    let y = _mm_blend_epi16::<0xCC>(_mm_srli_si128::<4>(m), n);
    let mn = _mm_min_epu32(x, y);
    let mx = _mm_max_epu32(x, y);
    (_mm_unpacklo_epi32(mn, mx), _mm_unpackhi_epi32(mn, mx))
}

streams!("sse4.1");
