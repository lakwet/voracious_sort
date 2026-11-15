//! Merges of sorted runs of 32-bit keys by blocks of four in vector
//! registers: a bitonic merge network (min / max, no branch) merges the
//! next block with the four largest keys kept from the previous step, and
//! only the choice of the run to load the next block from depends on the
//! data (one choice per four keys instead of one per key). Unstable on
//! equal keys, which does not matter for primitive keys (equal keys are
//! indistinguishable). Classic idea: AA-sort (Inoue et al. 2007), Chhugani
//! et al. (VLDB 2008); NEON: arXiv 2409.03970. Own code.

/// Merge of the sorted runs `a` and `b` into `dst` (`dst.len() == a.len() +
/// b.len()`), keys of 32 bits compared unsigned.
pub(crate) fn merge_u32(a: &[u32], b: &[u32], dst: &mut [u32]) {
    assert_eq!(dst.len(), a.len() + b.len());
    #[cfg(target_arch = "aarch64")]
    {
        if a.len() >= 4 && b.len() >= 4 {
            let k = STREAMS.load(std::sync::atomic::Ordering::Relaxed);
            if k > 1 && a.len() + b.len() >= k * SPLIT_MIN {
                // SAFETY: NEON is always present on aarch64.
                unsafe { neon::merge_split(a, b, dst, k) };
                return;
            }
            // SAFETY: NEON is always present on aarch64; lengths checked.
            unsafe { neon::merge(a, b, dst) };
            return;
        }
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

#[cfg(target_arch = "aarch64")]
mod neon {
    use std::arch::aarch64::*;

    /// Bitonic merge of two sorted blocks of four: the four smallest and
    /// the four largest, each sorted.
    #[inline(always)]
    unsafe fn net(a: uint32x4_t, b: uint32x4_t) -> (uint32x4_t, uint32x4_t) {
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

    /// One block merge in progress: read positions in both runs, write
    /// position, the four largest keys of the last step.
    struct Stream {
        pa: *const u32,
        ea: *const u32,
        pb: *const u32,
        eb: *const u32,
        pd: *mut u32,
        hi: uint32x4_t,
    }

    /// Next step of `s` (both runs hold at least four more keys).
    #[inline(always)]
    unsafe fn step(s: &mut Stream) {
        let take_a = *s.pa <= *s.pb;
        let p = if take_a { s.pa } else { s.pb };
        s.pa = s.pa.add(take_a as usize * 4);
        s.pb = s.pb.add(!take_a as usize * 4);
        let (lo, h) = net(vld1q_u32(p), s.hi);
        vst1q_u32(s.pd, lo);
        s.pd = s.pd.add(4);
        s.hi = h;
    }

    /// Steps `s` can take without leaving a run.
    #[inline(always)]
    unsafe fn safe(s: &Stream) -> usize {
        (s.ea.offset_from(s.pa) as usize).min(s.eb.offset_from(s.pb) as usize) / 4
    }

    /// Ends the merge of `s`: its four kept keys and both rests.
    unsafe fn finish(s: &Stream) {
        let mut kept = [0u32; 4];
        vst1q_u32(kept.as_mut_ptr(), s.hi);
        let ra = std::slice::from_raw_parts(s.pa, s.ea.offset_from(s.pa) as usize);
        let rb = std::slice::from_raw_parts(s.pb, s.eb.offset_from(s.pb) as usize);
        let n = 4 + ra.len() + rb.len();
        let dst = std::slice::from_raw_parts_mut(s.pd, n);
        let (short, long) = if ra.len() <= rb.len() { (ra, rb) } else { (rb, ra) };
        let mut tmp = vec![0u32; 4 + short.len()];
        super::scalar(&kept, short, &mut tmp);
        super::scalar(&tmp, long, dst);
    }

    /// Starts a stream on `a`, `b` (at least four keys each) into `d`.
    unsafe fn start(a: &[u32], b: &[u32], d: *mut u32) -> Stream {
        let (lo, hi) = net(vld1q_u32(a.as_ptr()), vld1q_u32(b.as_ptr()));
        vst1q_u32(d, lo);
        Stream { pa: a.as_ptr().add(4), ea: a.as_ptr().add(a.len()), pb: b.as_ptr().add(4), eb: b.as_ptr().add(b.len()), pd: d.add(4), hi }
    }

    /// Runs streams together while they all have room, then each alone.
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

    /// The merge cut into `k` parts (2 or 4) by binary searches, the parts
    /// merged together in one loop.
    ///
    /// # Safety
    /// `dst.len() == a.len() + b.len()`.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn merge_split(a: &[u32], b: &[u32], dst: &mut [u32], k: usize) {
        let n = a.len() + b.len();
        let k = if k >= 4 { 4 } else { 2 };
        let mut cut = [(0usize, 0usize); 5];
        for p in 1..k {
            let d = p * n / k;
            let i = super::corank(a, b, d);
            cut[p] = (i, d - i);
        }
        cut[k] = (a.len(), b.len());
        let mut streams: Vec<Stream> = Vec::with_capacity(4);
        for p in 0..k {
            let (sa, sb) = cut[p];
            let (ea, eb) = cut[p + 1];
            let (pa, pb) = (&a[sa..ea], &b[sb..eb]);
            let d0 = sa + sb;
            if pa.len() >= 4 && pb.len() >= 4 {
                streams.push(start(pa, pb, dst.as_mut_ptr().add(d0)));
            } else {
                super::scalar(pa, pb, &mut dst[d0..d0 + pa.len() + pb.len()]);
            }
        }
        run_all(&mut streams);
    }

    /// # Safety
    /// `a.len() >= 4`, `b.len() >= 4`, `dst.len() == a.len() + b.len()`.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn merge(a: &[u32], b: &[u32], dst: &mut [u32]) {
        let (pa, pb, pd) = (a.as_ptr(), b.as_ptr(), dst.as_mut_ptr());
        let (na, nb) = (a.len(), b.len());
        let (lo, mut hi) = net(vld1q_u32(pa), vld1q_u32(pb));
        vst1q_u32(pd, lo);
        let (mut i, mut j, mut k) = (4usize, 4usize, 4usize);
        // Each step loads the next block from the run whose next key is
        // smaller; the four largest of the step stay in `hi`.
        while i + 4 <= na && j + 4 <= nb {
            let take_a = *pa.add(i) <= *pb.add(j);
            let p = if take_a { pa.add(i) } else { pb.add(j) };
            i += take_a as usize * 4;
            j += !take_a as usize * 4;
            let (lo, h) = net(vld1q_u32(p), hi);
            vst1q_u32(pd.add(k), lo);
            hi = h;
            k += 4;
        }
        // Tail: the four kept keys and what is left of both runs.
        let mut kept = [0u32; 4];
        vst1q_u32(kept.as_mut_ptr(), hi);
        let ra = &a[i..];
        let rb = &b[j..];
        // Merge `kept` with the shorter rest into a small block, then that
        // block with the longer rest.
        let (short, long) = if ra.len() <= rb.len() { (ra, rb) } else { (rb, ra) };
        let mut small = [0u32; 8];
        let sn = 4 + short.len().min(4);
        if short.len() <= 4 {
            super::scalar(&kept, short, &mut small[..4 + short.len()]);
            super::scalar(&small[..4 + short.len()], long, &mut dst[k..]);
        } else {
            let mut tmp = vec![0u32; 4 + short.len()];
            super::scalar(&kept, short, &mut tmp);
            super::scalar(&tmp, long, &mut dst[k..]);
        }
        let _ = sn;
    }
}
