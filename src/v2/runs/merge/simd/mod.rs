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
    /// Zero of the lane type.
    const ZERO: Self;
    /// The key compared in mode `M` (0 unsigned, 1 signed, 2 float bits).
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
/// `V` holding four keys and the always-inlined `ld`, `st`, `pick` (select
/// of the next block) and `net` (merge network of two sorted blocks of four:
/// the four smallest and the four largest, each sorted), and the target
/// feature of the entry point.
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
            Stream {
                pa: pa.add(4),
                ea: pa.add(a.len()),
                pb: pb.add(4),
                eb: pb.add(b.len()),
                pd: d.add(4),
                hi,
            }
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
                    },
                    [s0, s1] => {
                        for _ in 0..m {
                            step::<M>(s0);
                            step::<M>(s1);
                        }
                    },
                    _ => {
                        for s in streams.iter_mut() {
                            for _ in 0..m {
                                step::<M>(s);
                            }
                        }
                    },
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
            let k = if k >= 4 {
                4
            } else if k >= 2 {
                2
            } else {
                1
            };
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
            let streams =
                std::slice::from_raw_parts_mut(streams.as_mut_ptr() as *mut Stream, count);
            run_all::<M>(streams);
        }
    };
}

#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(target_arch = "x86_64")]
mod sse;

#[cfg(target_arch = "aarch64")]
mod neon64;

#[cfg(target_arch = "x86_64")]
mod sse64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    /// Two sorted runs of every length pair up to 40 and a few long ones,
    /// merged in each mode (0 unsigned, 1 signed, 2 float bits) against
    /// the merged order of the same keys.
    macro_rules! check {
        ($u:ty, $merge:expr, $key:expr) => {{
            let key: fn($u) -> $u = $key;
            let mut r = Rng(19);
            let lens: Vec<usize> = (0..40).chain([255, 1000, 20_000].iter().copied()).collect();
            for &la in lens.iter() {
                for &lb in [0usize, 1, 3, 4, 5, 17, 39, 1000].iter() {
                    let mut a: Vec<$u> = (0..la).map(|_| r.next() as $u).collect();
                    let mut b: Vec<$u> = (0..lb).map(|_| r.next() as $u).collect();
                    a.sort_unstable_by_key(|&x| key(x));
                    b.sort_unstable_by_key(|&x| key(x));
                    let mut dst = vec![0 as $u; la + lb];
                    $merge(&a, &b, &mut dst);
                    let mut expected: Vec<$u> = a.iter().chain(&b).copied().collect();
                    expected.sort_unstable_by_key(|&x| key(x));
                    let same = dst.iter().zip(&expected).all(|(&x, &y)| key(x) == key(y));
                    assert!(same, "{} {} {}+{}", stringify!($merge), stringify!($key), la, lb);
                }
            }
        }};
    }

    #[test]
    fn merges() {
        check!(u32, merge_u32::<0>, |x| x);
        check!(u32, merge_u32::<1>, |x| x ^ 1 << 31);
        check!(u32, merge_u32::<2>, float_key_32);
        check!(u64, merge_u64::<0>, |x| x);
        check!(u64, merge_u64::<1>, |x| x ^ 1 << 63);
        check!(u64, merge_u64::<2>, float_key_64);
    }
}
