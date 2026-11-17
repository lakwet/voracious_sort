//! Merges of sorted runs of 32-bit keys by blocks of four in vector
//! registers: a bitonic merge network (min / max, no branch) merges the
//! next block with the four largest keys kept from the previous step, and
//! only the choice of the run to load the next block from depends on the
//! data (one choice per four keys instead of one per key). Unstable on
//! equal keys, which does not matter for primitive keys (equal keys are
//! indistinguishable). Classic idea: AA-sort (Inoue et al. 2007), Chhugani
//! et al. (VLDB 2008); NEON: arXiv 2409.03970. Own code.
//!
//! The stream code is shared (`streams!`); each architecture gives its
//! vector type, load, store and network: NEON on aarch64, SSE4.1 on x86_64
//! (`pminud` / `pmaxud`, detected at run time).

/// Whether the vector block merge is available on this CPU.
pub(crate) fn available() -> bool {
    #[cfg(target_arch = "aarch64")]
    return true;
    #[cfg(target_arch = "x86_64")]
    return std::arch::is_x86_feature_detected!("sse4.1");
    #[allow(unreachable_code)]
    false
}

/// Merge of the sorted runs `a` and `b` into `dst` (`dst.len() == a.len() +
/// b.len()`), keys of 32 bits compared as `key::<M>` (`M`: 0 u32, 1 i32, 2
/// f32 bits).
pub(crate) fn merge_u32<const M: u8>(a: &[u32], b: &[u32], dst: &mut [u32]) {
    assert_eq!(dst.len(), a.len() + b.len());
    if a.len() >= 4 && b.len() >= 4 && available() {
        let k = STREAMS;
        let k = if k > 1 && a.len() + b.len() >= k * SPLIT_MIN { k } else { 1 };
        // SAFETY: the vector unit is present (`available`); lengths checked.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            neon::merge_split::<M>(a, b, dst, k)
        };
        #[cfg(target_arch = "x86_64")]
        unsafe {
            sse::merge_split::<M>(a, b, dst, k)
        };
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        return;
    }
    scalar::<u32, M>(a, b, dst);
}

/// Keys of the vector merge: 32 or 64 bits, compared as `key::<M>`, the
/// bits as an unsigned integer of the same order (`M`: 0 unsigned as they
/// are, 1 signed with the sign bit flipped, 2 floats with the sign bit
/// flipped when positive and every bit when negative; -0 before +0: equal
/// for the caller, which orders signed zeros afterwards).
pub(crate) trait Lane: Copy + Ord {
    const ZERO: Self;
    fn key<const M: u8>(self) -> Self;
}

impl Lane for u32 {
    const ZERO: Self = 0;
    #[inline(always)]
    fn key<const M: u8>(self) -> u32 {
        match M {
            0 => self,
            1 => self ^ (1 << 31),
            _ => self ^ ((((self as i32) >> 31) as u32) | (1 << 31)),
        }
    }
}

impl Lane for u64 {
    const ZERO: Self = 0;
    #[inline(always)]
    fn key<const M: u8>(self) -> u64 {
        match M {
            0 => self,
            1 => self ^ (1 << 63),
            // f64: never runs while `SIMD_F64` is off (it lost to the f64
            // radix sort on the Snapdragon X); kept to measure on x86.
            _ => self ^ ((((self as i64) >> 63) as u64) | (1 << 63)),
        }
    }
}

/// Whether the vector merge of 64-bit keys is available on this CPU
/// (x86: `pcmpgtq`, SSE4.2).
pub(crate) fn available64() -> bool {
    #[cfg(target_arch = "aarch64")]
    return true;
    #[cfg(target_arch = "x86_64")]
    return std::arch::is_x86_feature_detected!("sse4.2");
    #[allow(unreachable_code)]
    false
}

/// `merge_u32` for 64-bit keys (mode `M` as there).
pub(crate) fn merge_u64<const M: u8>(a: &[u64], b: &[u64], dst: &mut [u64]) {
    assert_eq!(dst.len(), a.len() + b.len());
    if a.len() >= 4 && b.len() >= 4 && available64() {
        let k = STREAMS;
        let k = if k > 1 && a.len() + b.len() >= k * SPLIT_MIN { k } else { 1 };
        // SAFETY: the vector unit is present (`available64`); lengths checked.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            neon64::merge_split::<M>(a, b, dst, k)
        };
        #[cfg(target_arch = "x86_64")]
        unsafe {
            sse64::merge_split::<M>(a, b, dst, k)
        };
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        return;
    }
    scalar::<u64, M>(a, b, dst);
}

/// Independent block merges run together in one loop (the output cut by
/// binary searches): 1 (one merge), 2 or 4.
pub(crate) const STREAMS: usize = 4;
/// Shortest part per stream.
const SPLIT_MIN: usize = 256;

/// Number of elements of `a` among the first `d` of the merge of `a` and
/// `b` (left first on ties).
fn corank<E: Lane, const M: u8>(a: &[E], b: &[E], d: usize) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(b.len()), d.min(a.len()));
    while lo < hi {
        let i = (lo + hi) / 2;
        if a[i].key::<M>() <= b[d - i - 1].key::<M>() {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Scalar merge (tails and other architectures).
fn scalar<E: Lane, const M: u8>(a: &[E], b: &[E], dst: &mut [E]) {
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        let t = b[j].key::<M>() < a[i].key::<M>();
        dst[k] = if t { b[j] } else { a[i] };
        j += t as usize;
        i += !t as usize;
        k += 1;
    }
    dst[k..k + a.len() - i].copy_from_slice(&a[i..]);
    k += a.len() - i;
    dst[k..].copy_from_slice(&b[j..]);
}

/// The block merge streams, given in scope the key type `E`, a vector type
/// `V` holding four keys and the
/// always-inlined `ld`, `st`, `pick` (select of the next block) and `net` (merge network of two sorted blocks
/// of four: the four smallest and the four largest, each sorted), and the
/// target feature of the entry point.
#[allow(unused_macros)]
macro_rules! streams {
    ($feature:literal) => {
        /// One block merge in progress: read positions in both runs, write
        /// position, the four largest keys of the last step.
        struct Stream {
            pa: *const E,
            ea: *const E,
            pb: *const E,
            eb: *const E,
            pd: *mut E,
            hi: V,
        }

        /// Next step of `s` (both runs hold at least four more keys): the
        /// next block comes from the run whose next key is smaller.
        #[inline(always)]
        unsafe fn step<const M: u8>(s: &mut Stream) {
            let take_a = super::Lane::key::<M>(*s.pa) <= super::Lane::key::<M>(*s.pb);
            let p = pick(take_a, s.pa, s.pb);
            s.pa = s.pa.add(take_a as usize * 4);
            s.pb = s.pb.add(!take_a as usize * 4);
            let (lo, h) = net(ld::<M>(p), s.hi);
            st::<M>(s.pd, lo);
            s.pd = s.pd.add(4);
            s.hi = h;
        }

        /// Steps `s` can take without leaving a run.
        #[inline(always)]
        unsafe fn safe(s: &Stream) -> usize {
            (s.ea.offset_from(s.pa) as usize).min(s.eb.offset_from(s.pb) as usize) / 4
        }

        /// Ends the merge of `s`: its four kept keys and both rests.
        #[inline(always)]
        unsafe fn finish<const M: u8>(s: &Stream) {
            let mut kept = [<E as super::Lane>::ZERO; 4];
            st::<M>(kept.as_mut_ptr(), s.hi);
            let ra = std::slice::from_raw_parts(s.pa, s.ea.offset_from(s.pa) as usize);
            let rb = std::slice::from_raw_parts(s.pb, s.eb.offset_from(s.pb) as usize);
            let dst = std::slice::from_raw_parts_mut(s.pd, 4 + ra.len() + rb.len());
            // `kept` merged with the shorter rest, then with the longer one.
            let (short, long) = if ra.len() <= rb.len() { (ra, rb) } else { (rb, ra) };
            if short.len() <= 4 {
                let mut small = [<E as super::Lane>::ZERO; 8];
                let small = &mut small[..4 + short.len()];
                super::scalar::<E, M>(&kept, short, small);
                super::scalar::<E, M>(small, long, dst);
            } else {
                let mut tmp = vec![<E as super::Lane>::ZERO; 4 + short.len()];
                super::scalar::<E, M>(&kept, short, &mut tmp);
                super::scalar::<E, M>(&tmp, long, dst);
            }
        }

        /// Starts a stream on `a`, `b` (at least four keys each) into `d`.
        #[inline(always)]
        unsafe fn start<const M: u8>(a: &[E], b: &[E], d: *mut E) -> Stream {
            let (lo, hi) = net(ld::<M>(a.as_ptr()), ld::<M>(b.as_ptr()));
            st::<M>(d, lo);
            let (pa, pb) = (a.as_ptr(), b.as_ptr());
            Stream { pa: pa.add(4), ea: pa.add(a.len()), pb: pb.add(4), eb: pb.add(b.len()), pd: d.add(4), hi }
        }

        /// Runs the streams together while they all have room, then each
        /// alone.
        #[inline(always)]
        unsafe fn run_all<const M: u8>(streams: &mut [Stream]) {
            loop {
                let m = streams.iter().map(|s| safe(s)).min().unwrap_or(0);
                if m == 0 {
                    break;
                }
                match streams {
                    [s0, s1, s2, s3] => {
                        for _ in 0..m {
                            step::<M>(s0);
                            step::<M>(s1);
                            step::<M>(s2);
                            step::<M>(s3);
                        }
                    }
                    [s0, s1] => {
                        for _ in 0..m {
                            step::<M>(s0);
                            step::<M>(s1);
                        }
                    }
                    _ => {
                        for s in streams.iter_mut() {
                            for _ in 0..m {
                                step::<M>(s);
                            }
                        }
                    }
                }
            }
            for s in streams.iter_mut() {
                loop {
                    let m = safe(s);
                    if m == 0 {
                        break;
                    }
                    for _ in 0..m {
                        step::<M>(s);
                    }
                }
                finish::<M>(s);
            }
        }

        /// The merge cut into `k` parts (1, 2 or 4) by binary searches, the
        /// parts merged together in one loop.
        ///
        /// # Safety
        /// The target feature is present, `a.len() >= 4`, `b.len() >= 4`,
        /// `dst.len() == a.len() + b.len()`.
        #[target_feature(enable = $feature)]
        pub(super) unsafe fn merge_split<const M: u8>(a: &[E], b: &[E], dst: &mut [E], k: usize) {
            let n = a.len() + b.len();
            let k = if k >= 4 { 4 } else if k >= 2 { 2 } else { 1 };
            let mut cut = [(0usize, 0usize); 5];
            for p in 1..k {
                let d = p * n / k;
                let i = super::corank::<E, M>(a, b, d);
                cut[p] = (i, d - i);
            }
            cut[k] = (a.len(), b.len());
            let mut streams: [std::mem::MaybeUninit<Stream>; 4] =
                [const { std::mem::MaybeUninit::uninit() }; 4];
            let mut count = 0;
            for p in 0..k {
                let (sa, sb) = cut[p];
                let (ea, eb) = cut[p + 1];
                let (pa, pb) = (&a[sa..ea], &b[sb..eb]);
                let d0 = sa + sb;
                if pa.len() >= 4 && pb.len() >= 4 {
                    streams[count].write(start::<M>(pa, pb, dst.as_mut_ptr().add(d0)));
                    count += 1;
                } else {
                    super::scalar::<E, M>(pa, pb, &mut dst[d0..d0 + pa.len() + pb.len()]);
                }
            }
            // SAFETY: the first `count` streams are written (`Stream` has no
            // drop).
            let streams = std::slice::from_raw_parts_mut(streams.as_mut_ptr() as *mut Stream, count);
            run_all::<M>(streams);
        }
    };
}

#[cfg(target_arch = "aarch64")]
mod neon {
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
            _ => veorq_u32(v, vorrq_u32(vreinterpretq_u32_s32(vshrq_n_s32::<31>(vreinterpretq_s32_u32(v))), sign)),
        }
    }

    /// Inverse of `to_key`.
    #[inline(always)]
    unsafe fn from_key<const M: u8>(v: V) -> V {
        let sign = vdupq_n_u32(1 << 31);
        match M {
            0 => v,
            1 => veorq_u32(v, sign),
            _ => veorq_u32(v, vorrq_u32(vreinterpretq_u32_s32(vshrq_n_s32::<31>(vreinterpretq_s32_u32(vmvnq_u32(v)))), sign)),
        }
    }

    #[inline(always)]
    unsafe fn ld<const M: u8>(p: *const u32) -> V {
        to_key::<M>(vld1q_u32(p))
    }

    #[inline(always)]
    unsafe fn st<const M: u8>(p: *mut u32, v: V) {
        vst1q_u32(p, from_key::<M>(v))
    }

    /// A plain select: LLVM makes it a `csel` here.
    #[inline(always)]
    fn pick(c: bool, a: *const u32, b: *const u32) -> *const u32 {
        if c { a } else { b }
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
}

#[cfg(target_arch = "x86_64")]
mod sse {
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
            _ => _mm_xor_si128(v, _mm_or_si128(_mm_srai_epi32::<31>(_mm_xor_si128(v, _mm_set1_epi32(-1))), sign)),
        }
    }

    #[inline(always)]
    unsafe fn ld<const M: u8>(p: *const u32) -> V {
        to_key::<M>(_mm_loadu_si128(p as *const V))
    }

    #[inline(always)]
    unsafe fn st<const M: u8>(p: *mut u32, v: V) {
        _mm_storeu_si128(p as *mut V, from_key::<M>(v))
    }

    /// A plain `if` became a branch here (mispredicted on random runs, the
    /// streams no help: 1.0 ns per key on the Ryzen 3950X); `cmov`.
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
}

#[cfg(target_arch = "aarch64")]
mod neon64 {
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
            _ => veorq_u64(v, vorrq_u64(vreinterpretq_u64_s64(vshrq_n_s64::<63>(vreinterpretq_s64_u64(v))), sign)),
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
                veorq_u64(v, vorrq_u64(vreinterpretq_u64_s64(vshrq_n_s64::<63>(vreinterpretq_s64_u64(not))), sign))
            }
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
        if c { a } else { b }
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
}

#[cfg(target_arch = "x86_64")]
mod sse64 {
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
}
