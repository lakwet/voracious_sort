//! The 64-bit block merge streams on SSE4.2 (x86_64): four keys in two
//! registers.

use std::arch::x86_64::*;

type E = u64;
/// Four keys in two registers, held in signed order (`pcmpgtq` is
/// signed): the key of `Lane::key` with the sign bit flipped.
type V = (__m128i, __m128i);

#[inline(always)]
unsafe fn neg_mask(v: __m128i) -> __m128i {
    _mm_cmpgt_epi64(_mm_setzero_si128(), v)
}

/// Bits to the signed-order key (its own inverse for M 0 and 1).
#[inline(always)]
unsafe fn to_key<const M: u8>(v: __m128i) -> __m128i {
    match M {
        0 => _mm_xor_si128(v, _mm_set1_epi64x(i64::MIN)),
        1 => v,
        // Negative floats: every bit but the sign flipped.
        _ => _mm_xor_si128(v, _mm_and_si128(neg_mask(v), _mm_set1_epi64x(i64::MAX))),
    }
}

#[inline(always)]
unsafe fn ld<const M: u8>(p: *const u64) -> V {
    let p = p as *const __m128i;
    (to_key::<M>(_mm_loadu_si128(p)), to_key::<M>(_mm_loadu_si128(p.add(1))))
}

#[inline(always)]
unsafe fn st<const M: u8>(p: *mut u64, v: V) {
    let p = p as *mut __m128i;
    _mm_storeu_si128(p, to_key::<M>(v.0));
    _mm_storeu_si128(p.add(1), to_key::<M>(v.1));
}

#[inline(always)]
fn pick(c: bool, a: *const u64, b: *const u64) -> *const u64 {
    std::hint::select_unpredictable(c, a, b)
}

#[inline(always)]
unsafe fn cx(a: __m128i, b: __m128i) -> (__m128i, __m128i) {
    let gt = _mm_cmpgt_epi64(a, b);
    (_mm_blendv_epi8(a, b, gt), _mm_blendv_epi8(b, a, gt))
}

/// Same network as `neon64`.
#[inline(always)]
unsafe fn net(a: V, b: V) -> (V, V) {
    let (r0, r1) = (_mm_shuffle_epi32::<0x4E>(b.1), _mm_shuffle_epi32::<0x4E>(b.0));
    let (l0, h0) = cx(a.0, r0);
    let (l1, h1) = cx(a.1, r1);
    let (l0, l1) = cx(l0, l1);
    let (h0, h1) = cx(h0, h1);
    let fin = |x0: __m128i, x1: __m128i| {
        let (mn, mx) = cx(_mm_unpacklo_epi64(x0, x1), _mm_unpackhi_epi64(x0, x1));
        (_mm_unpacklo_epi64(mn, mx), _mm_unpackhi_epi64(mn, mx))
    };
    (fin(l0, l1), fin(h0, h1))
}

streams!("sse4.2");
