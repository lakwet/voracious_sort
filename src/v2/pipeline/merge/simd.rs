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
/// b.len()`), keys of 32 bits compared unsigned.
pub(crate) fn merge_u32(a: &[u32], b: &[u32], dst: &mut [u32]) {
    assert_eq!(dst.len(), a.len() + b.len());
    if a.len() >= 4 && b.len() >= 4 && available() {
        let k = STREAMS.load(std::sync::atomic::Ordering::Relaxed);
        let k = if k > 1 && a.len() + b.len() >= k * SPLIT_MIN { k } else { 1 };
        // SAFETY: the vector unit is present (`available`); lengths checked.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            neon::merge_split(a, b, dst, k)
        };
        #[cfg(target_arch = "x86_64")]
        unsafe {
            sse::merge_split(a, b, dst, k)
        };
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        return;
    }
    scalar(a, b, dst);
}

/// Independent block merges run together in one loop (the output cut by
/// binary searches): 1 (one merge), 2 or 4.
pub(crate) static STREAMS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(4);
/// Shortest part per stream.
const SPLIT_MIN: usize = 256;

/// Number of elements of `a` among the first `d` of the merge of `a` and
/// `b` (left first on ties).
fn corank(a: &[u32], b: &[u32], d: usize) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(b.len()), d.min(a.len()));
    while lo < hi {
        let i = (lo + hi) / 2;
        if a[i] <= b[d - i - 1] {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Scalar merge (tails and other architectures).
fn scalar(a: &[u32], b: &[u32], dst: &mut [u32]) {
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        let t = b[j] < a[i];
        dst[k] = if t { b[j] } else { a[i] };
        j += t as usize;
        i += !t as usize;
        k += 1;
    }
    dst[k..k + a.len() - i].copy_from_slice(&a[i..]);
    k += a.len() - i;
    dst[k..].copy_from_slice(&b[j..]);
}

/// The block merge streams, given in scope a vector type `V` and the
/// always-inlined `ld`, `st`, `pick` (select of the next block) and `net` (merge network of two sorted blocks
/// of four: the four smallest and the four largest, each sorted), and the
/// target feature of the entry point.
#[allow(unused_macros)]
macro_rules! streams {
    ($feature:literal) => {
        /// One block merge in progress: read positions in both runs, write
        /// position, the four largest keys of the last step.
        struct Stream {
            pa: *const u32,
            ea: *const u32,
            pb: *const u32,
            eb: *const u32,
            pd: *mut u32,
            hi: V,
        }

        /// Next step of `s` (both runs hold at least four more keys): the
        /// next block comes from the run whose next key is smaller.
        #[inline(always)]
        unsafe fn step(s: &mut Stream) {
            let take_a = *s.pa <= *s.pb;
            let p = pick(take_a, s.pa, s.pb);
            s.pa = s.pa.add(take_a as usize * 4);
            s.pb = s.pb.add(!take_a as usize * 4);
            let (lo, h) = net(ld(p), s.hi);
            st(s.pd, lo);
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
        unsafe fn finish(s: &Stream) {
            let mut kept = [0u32; 4];
            st(kept.as_mut_ptr(), s.hi);
            let ra = std::slice::from_raw_parts(s.pa, s.ea.offset_from(s.pa) as usize);
            let rb = std::slice::from_raw_parts(s.pb, s.eb.offset_from(s.pb) as usize);
            let dst = std::slice::from_raw_parts_mut(s.pd, 4 + ra.len() + rb.len());
            // `kept` merged with the shorter rest, then with the longer one.
            let (short, long) = if ra.len() <= rb.len() { (ra, rb) } else { (rb, ra) };
            if short.len() <= 4 {
                let mut small = [0u32; 8];
                let small = &mut small[..4 + short.len()];
                super::scalar(&kept, short, small);
                super::scalar(small, long, dst);
            } else {
                let mut tmp = vec![0u32; 4 + short.len()];
                super::scalar(&kept, short, &mut tmp);
                super::scalar(&tmp, long, dst);
            }
        }

        /// Starts a stream on `a`, `b` (at least four keys each) into `d`.
        #[inline(always)]
        unsafe fn start(a: &[u32], b: &[u32], d: *mut u32) -> Stream {
            let (lo, hi) = net(ld(a.as_ptr()), ld(b.as_ptr()));
            st(d, lo);
            let (pa, pb) = (a.as_ptr(), b.as_ptr());
            Stream { pa: pa.add(4), ea: pa.add(a.len()), pb: pb.add(4), eb: pb.add(b.len()), pd: d.add(4), hi }
        }

        /// Runs the streams together while they all have room, then each
        /// alone.
        #[inline(always)]
        unsafe fn run_all(streams: &mut [Stream]) {
            loop {
                let m = streams.iter().map(|s| safe(s)).min().unwrap_or(0);
                if m == 0 {
                    break;
                }
                match streams {
                    [s0, s1, s2, s3] => {
                        for _ in 0..m {
                            step(s0);
                            step(s1);
                            step(s2);
                            step(s3);
                        }
                    }
                    [s0, s1] => {
                        for _ in 0..m {
                            step(s0);
                            step(s1);
                        }
                    }
                    _ => {
                        for s in streams.iter_mut() {
                            for _ in 0..m {
                                step(s);
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
                        step(s);
                    }
                }
                finish(s);
            }
        }

        /// The merge cut into `k` parts (1, 2 or 4) by binary searches, the
        /// parts merged together in one loop.
        ///
        /// # Safety
        /// The target feature is present, `a.len() >= 4`, `b.len() >= 4`,
        /// `dst.len() == a.len() + b.len()`.
        #[target_feature(enable = $feature)]
        pub(super) unsafe fn merge_split(a: &[u32], b: &[u32], dst: &mut [u32], k: usize) {
            let n = a.len() + b.len();
            let k = if k >= 4 { 4 } else if k >= 2 { 2 } else { 1 };
            let mut cut = [(0usize, 0usize); 5];
            for p in 1..k {
                let d = p * n / k;
                let i = super::corank(a, b, d);
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
                    streams[count].write(start(pa, pb, dst.as_mut_ptr().add(d0)));
                    count += 1;
                } else {
                    super::scalar(pa, pb, &mut dst[d0..d0 + pa.len() + pb.len()]);
                }
            }
            // SAFETY: the first `count` streams are written (`Stream` has no
            // drop).
            let streams = std::slice::from_raw_parts_mut(streams.as_mut_ptr() as *mut Stream, count);
            run_all(streams);
        }
    };
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use std::arch::aarch64::*;

    type V = uint32x4_t;

    #[inline(always)]
    unsafe fn ld(p: *const u32) -> V {
        vld1q_u32(p)
    }

    #[inline(always)]
    unsafe fn st(p: *mut u32, v: V) {
        vst1q_u32(p, v)
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

    type V = __m128i;

    #[inline(always)]
    unsafe fn ld(p: *const u32) -> V {
        _mm_loadu_si128(p as *const V)
    }

    #[inline(always)]
    unsafe fn st(p: *mut u32, v: V) {
        _mm_storeu_si128(p as *mut V, v)
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
