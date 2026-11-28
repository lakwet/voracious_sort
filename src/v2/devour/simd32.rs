//! 32-bit keys sorted by comparison networks in
//! SIMD registers, up to `SIMD_MAX` keys: AVX2 on x86-64 (detected at run
//! time, 8 lanes), NEON on AArch64 (always there, 4 lanes: 16 registers per
//! block, 4-input column networks, 4 x 4 transposes; blocks merged 8 keys
//! at a time). Keys are compared as unsigned after an XOR (`flip`: the sign bit
//! for signed keys), padded with `u32::MAX` to a multiple of 64.
//!
//! - a block of 64 keys: 8 registers, each lane column sorted by an
//!   8-input network (min / max between registers), the 8 x 8 matrix
//!   transposed (8 sorted registers), then bitonic merges of 2, 4, 8
//!   registers;
//! - blocks merged pairwise, 8 keys at a time: the larger half of a
//!   two-register bitonic merge kept, the next 8 keys loaded from the run
//!   with the smaller head.
//!
//! Idea of turbosort (MIT, register networks then merges for its small
//! arrays); the code is ours.

/// Largest array of the hooks below the sort_unstable size and in
/// `short32`.
pub(crate) const SIMD_MAX: usize = 512;
/// Largest array sorted here (the buffers).
const BUF: usize = 4096;

/// Whether this CPU runs the AVX2 networks.
#[inline]
pub(crate) fn available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(target_arch = "aarch64")]
    {
        true
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        false
    }
}

/// Up to 128 keys: loaded into registers, their range checked there (min /
/// max of the registers, the max an upper bound); below `narrow` values,
/// nothing written and `Some((lo, hi))` (encoded keys) returned for a
/// count; else sorted in place and `None`. Callers check `available()`.
pub(crate) fn sort_or_range(bits: &mut [u32], flip: u32, narrow: usize) -> Option<(u32, u32)> {
    debug_assert!(bits.len() <= SIMD_MAX);
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: the caller checked that AVX2 is available.
        unsafe { avx2::sort_or_range(bits, flip, narrow) }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is part of AArch64.
        unsafe { neon::sort_or_range(bits, flip, narrow) }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = (flip, narrow);
        unreachable!("no SIMD networks here")
    }
}

/// Sorts `bits` (at most `SIMD_MAX` keys) compared as `k ^ flip`.
/// Callers check `available()` first.
pub(crate) fn sort(bits: &mut [u32], flip: u32) {
    debug_assert!(bits.len() <= BUF);
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: the caller checked that AVX2 is available.
        unsafe { avx2::sort(bits, flip) }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is part of AArch64.
        unsafe { neon::sort(bits, flip) }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = flip;
        unreachable!("no SIMD networks here")
    }
}

#[cfg(target_arch = "x86_64")]
mod avx2 {
    use super::BUF;
    use std::arch::x86_64::*;

    type V = __m256i;

    #[target_feature(enable = "avx2")]
    #[inline]
    fn rev(v: V) -> V { _mm256_permutevar8x32_epi32(v, _mm256_setr_epi32(7, 6, 5, 4, 3, 2, 1, 0)) }

    /// A bitonic register sorted ascending: compare-exchange at lane
    /// distances 4, 2, 1.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn clean(v: V) -> V {
        let x = _mm256_permute2x128_si256(v, v, 1);
        let v = _mm256_blend_epi32(_mm256_min_epu32(v, x), _mm256_max_epu32(v, x), 0b1111_0000);
        let x = _mm256_shuffle_epi32(v, 0b01_00_11_10);
        let v = _mm256_blend_epi32(_mm256_min_epu32(v, x), _mm256_max_epu32(v, x), 0b1100_1100);
        let x = _mm256_shuffle_epi32(v, 0b10_11_00_01);
        _mm256_blend_epi32(_mm256_min_epu32(v, x), _mm256_max_epu32(v, x), 0b1010_1010)
    }

    /// Two ascending runs of `N / 2` registers (the halves of `r`) merged
    /// into one: the second half reversed (the whole is bitonic), the
    /// half-cleaners at register distances N/2 .. 1, then each register.
    /// `N` is a constant: the loops unroll, `r` stays in registers.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn merge<const N: usize>(mut r: [V; N]) -> [V; N] {
        let h = N / 2;
        for i in 0..h / 2 {
            let (a, b) = (r[h + i], r[N - 1 - i]);
            r[h + i] = rev(b);
            r[N - 1 - i] = rev(a);
        }
        if h % 2 == 1 {
            r[h + h / 2] = rev(r[h + h / 2]);
        }
        let mut d = h;
        while d > 0 {
            for i in 0..N {
                if i & d == 0 {
                    let (a, b) = (r[i], r[i + d]);
                    r[i] = _mm256_min_epu32(a, b);
                    r[i + d] = _mm256_max_epu32(a, b);
                }
            }
            d /= 2;
        }
        for v in r.iter_mut() {
            *v = clean(*v);
        }
        r
    }

    /// Merges every group of `N` registers of `r` (runs of `N / 2`).
    #[target_feature(enable = "avx2")]
    #[inline]
    fn level<const R: usize, const N: usize>(r: &mut [V; R]) {
        for g in 0..R / N {
            let q: [V; N] = std::array::from_fn(|i| r[g * N + i]);
            let q = merge::<N>(q);
            for i in 0..N {
                r[g * N + i] = q[i];
            }
        }
    }

    /// 8 x 8 transpose of 32-bit lanes.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn transpose(r: &mut [V; 8]) {
        let t0 = _mm256_unpacklo_epi32(r[0], r[1]);
        let t1 = _mm256_unpackhi_epi32(r[0], r[1]);
        let t2 = _mm256_unpacklo_epi32(r[2], r[3]);
        let t3 = _mm256_unpackhi_epi32(r[2], r[3]);
        let t4 = _mm256_unpacklo_epi32(r[4], r[5]);
        let t5 = _mm256_unpackhi_epi32(r[4], r[5]);
        let t6 = _mm256_unpacklo_epi32(r[6], r[7]);
        let t7 = _mm256_unpackhi_epi32(r[6], r[7]);
        let u0 = _mm256_unpacklo_epi64(t0, t2);
        let u1 = _mm256_unpackhi_epi64(t0, t2);
        let u2 = _mm256_unpacklo_epi64(t1, t3);
        let u3 = _mm256_unpackhi_epi64(t1, t3);
        let u4 = _mm256_unpacklo_epi64(t4, t6);
        let u5 = _mm256_unpackhi_epi64(t4, t6);
        let u6 = _mm256_unpacklo_epi64(t5, t7);
        let u7 = _mm256_unpackhi_epi64(t5, t7);
        r[0] = _mm256_permute2x128_si256(u0, u4, 0x20);
        r[1] = _mm256_permute2x128_si256(u1, u5, 0x20);
        r[2] = _mm256_permute2x128_si256(u2, u6, 0x20);
        r[3] = _mm256_permute2x128_si256(u3, u7, 0x20);
        r[4] = _mm256_permute2x128_si256(u0, u4, 0x31);
        r[5] = _mm256_permute2x128_si256(u1, u5, 0x31);
        r[6] = _mm256_permute2x128_si256(u2, u6, 0x31);
        r[7] = _mm256_permute2x128_si256(u3, u7, 0x31);
    }

    /// Sorts `R` registers (8 or 16): the lane columns of each group of 8
    /// sorted by Batcher's 8-input network (19 comparators), transposed,
    /// then merged.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn sort_regs<const R: usize>(r: &mut [V; R]) {
        for g in 0..R / 8 {
            let mut q: [V; 8] = std::array::from_fn(|i| r[8 * g + i]);
            macro_rules! c {
                ($($a:literal $b:literal),*) => {$(
                    let (x, y) = (q[$a], q[$b]);
                    q[$a] = _mm256_min_epu32(x, y);
                    q[$b] = _mm256_max_epu32(x, y);
                )*};
            }
            c!(0 1, 2 3, 4 5, 6 7, 0 2, 1 3, 4 6, 5 7, 1 2, 5 6,
               0 4, 1 5, 2 6, 3 7, 2 4, 3 5, 1 2, 3 4, 5 6);
            transpose(&mut q);
            for i in 0..8 {
                r[8 * g + i] = q[i];
            }
        }
        level::<R, 2>(r);
        level::<R, 4>(r);
        level::<R, 8>(r);
        if R == 16 {
            level::<R, 16>(r);
        }
    }

    /// The keys `k ^ flip` of up to 8 values at `p` (`m` of them), the
    /// lanes past them `u32::MAX`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn load(p: *const u32, m: usize, f: V) -> V {
        if m >= 8 {
            return _mm256_xor_si256(unsafe { _mm256_loadu_si256(p as *const V) }, f);
        }
        let lanes = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
        let mask = _mm256_cmpgt_epi32(_mm256_set1_epi32(m as i32), lanes);
        // A masked load reads only the lanes of the mask.
        let x = unsafe { _mm256_maskload_epi32(p as *const i32, mask) };
        _mm256_or_si256(_mm256_xor_si256(x, f), _mm256_andnot_si256(mask, _mm256_set1_epi32(-1)))
    }

    /// The first `m` lanes of `v ^ flip` written to `p` (a short tail
    /// through a copy on the stack: no masked store, which delays the
    /// loads that read it soon after).
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn store(p: *mut u32, m: usize, v: V, f: V) {
        let v = _mm256_xor_si256(v, f);
        if m >= 8 {
            unsafe { _mm256_storeu_si256(p as *mut V, v) };
            return;
        }
        let mut t = [0u32; 8];
        unsafe {
            _mm256_storeu_si256(t.as_mut_ptr() as *mut V, v);
            std::ptr::copy_nonoverlapping(t.as_ptr(), p, m);
        }
    }

    /// Sorts the `n` keys at `p` (at most `8 * R`) in `R` registers, read
    /// and written in place.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn sort_direct<const R: usize>(p: *mut u32, n: usize, f: V) {
        let mut r: [V; R] =
            std::array::from_fn(|i| unsafe { load(p.add(8 * i), n.saturating_sub(8 * i), f) });
        sort_regs::<R>(&mut r);
        for i in 0..R {
            if 8 * i < n {
                unsafe { store(p.add(8 * i), n - 8 * i, r[i], f) };
            }
        }
    }

    /// `sort_direct` with the range check of `sort_or_range`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn direct_or_range<const R: usize>(p: *mut u32, n: usize, f: V, narrow: usize) -> Option<(u32, u32)> {
        let mut r: [V; R] =
            std::array::from_fn(|i| unsafe { load(p.add(8 * i), n.saturating_sub(8 * i), f) });
        let mut mn = r[0];
        let mut mx = _mm256_setzero_si256();
        for i in 0..R {
            if 8 * i < n {
                mn = _mm256_min_epu32(mn, r[i]);
                // Padding lanes (the maximum) left out of the max: the
                // lanes of a short last register read as its first key.
                let v = if n - 8 * i >= 8 {
                    r[i]
                } else {
                    let lanes = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
                    let real = _mm256_cmpgt_epi32(_mm256_set1_epi32((n - 8 * i) as i32), lanes);
                    _mm256_blendv_epi8(_mm256_permutevar8x32_epi32(r[i], _mm256_setzero_si256()), r[i], real)
                };
                mx = _mm256_max_epu32(mx, v);
            }
        }
        let hmin = |v: V| {
            let v = _mm256_min_epu32(v, _mm256_permute2x128_si256(v, v, 1));
            let v = _mm256_min_epu32(v, _mm256_shuffle_epi32(v, 0b01_00_11_10));
            let v = _mm256_min_epu32(v, _mm256_shuffle_epi32(v, 0b10_11_00_01));
            _mm256_cvtsi256_si32(v) as u32
        };
        let hmax = |v: V| {
            let v = _mm256_max_epu32(v, _mm256_permute2x128_si256(v, v, 1));
            let v = _mm256_max_epu32(v, _mm256_shuffle_epi32(v, 0b01_00_11_10));
            let v = _mm256_max_epu32(v, _mm256_shuffle_epi32(v, 0b10_11_00_01));
            _mm256_cvtsi256_si32(v) as u32
        };
        let (lo, hi) = (hmin(mn), hmax(mx));
        if ((hi - lo) as usize) < narrow {
            return Some((lo, hi));
        }
        sort_regs::<R>(&mut r);
        for i in 0..R {
            if 8 * i < n {
                unsafe { store(p.add(8 * i), n - 8 * i, r[i], f) };
            }
        }
        None
    }

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn sort_or_range(bits: &mut [u32], flip: u32, narrow: usize) -> Option<(u32, u32)> {
        let n = bits.len();
        let f = _mm256_set1_epi32(flip as i32);
        let p = bits.as_mut_ptr();
        if n <= 64 {
            return unsafe { direct_or_range::<8>(p, n, f, narrow) };
        }
        if n <= 128 {
            return unsafe { direct_or_range::<16>(p, n, f, narrow) };
        }
        // Beyond 128 keys: one vector min / max pass first.
        let (mut mn, mut mx) = (_mm256_set1_epi32(-1), _mm256_setzero_si256());
        let mut i = 0;
        while i + 8 <= n {
            let v = _mm256_xor_si256(unsafe { _mm256_loadu_si256(p.add(i) as *const V) }, f);
            mn = _mm256_min_epu32(mn, v);
            mx = _mm256_max_epu32(mx, v);
            i += 8;
        }
        let mut lanes_mn = [0u32; 8];
        let mut lanes_mx = [0u32; 8];
        unsafe {
            _mm256_storeu_si256(lanes_mn.as_mut_ptr() as *mut V, mn);
            _mm256_storeu_si256(lanes_mx.as_mut_ptr() as *mut V, mx);
        }
        let (mut lo, mut hi) = (*lanes_mn.iter().min().unwrap(), *lanes_mx.iter().max().unwrap());
        for &k in &bits[i..] {
            let e = k ^ flip;
            lo = lo.min(e);
            hi = hi.max(e);
        }
        if ((hi - lo) as usize) < narrow {
            return Some((lo, hi));
        }
        unsafe { sort(bits, flip) };
        None
    }

    /// Merges the runs `a` and `b` (lengths non-zero multiples of 8) into
    /// `out`: `hi` holds the 8 largest keys loaded so far, each step merges
    /// it with the next 8 keys of the run with the smaller head and writes
    /// the lower 8.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn merge_runs(a: &[u32], b: &[u32], out: *mut u32) {
        let ld = |s: &[u32], i: usize| unsafe { _mm256_loadu_si256(s.as_ptr().add(i) as *const V) };
        let (mut i, mut j, mut o) = (8, 8, 0);
        let mut lo = ld(a, 0);
        let mut hi = ld(b, 0);
        loop {
            let q = merge::<2>([lo, hi]);
            hi = q[1];
            unsafe { _mm256_storeu_si256(out.add(o) as *mut V, q[0]) };
            o += 8;
            let take_a = if i < a.len() {
                j == b.len() || a[i] <= b[j]
            } else if j < b.len() {
                false
            } else {
                break;
            };
            if take_a {
                lo = ld(a, i);
                i += 8;
            } else {
                lo = ld(b, j);
                j += 8;
            }
        }
        unsafe { _mm256_storeu_si256(out.add(o) as *mut V, hi) };
    }

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn sort(bits: &mut [u32], flip: u32) {
        let n = bits.len();
        let f = _mm256_set1_epi32(flip as i32);
        let p = bits.as_mut_ptr();
        if n <= 64 {
            unsafe { sort_direct::<8>(p, n, f) };
            return;
        }
        if n <= 128 {
            unsafe { sort_direct::<16>(p, n, f) };
            return;
        }
        // Blocks of 128 keys sorted into a buffer (padded with the
        // maximum), then merged pairwise, ping-pong between two buffers.
        let mut xb = std::mem::MaybeUninit::<[u32; BUF]>::uninit();
        let mut yb = std::mem::MaybeUninit::<[u32; BUF]>::uninit();
        let (mut px, mut py) = (xb.as_mut_ptr() as *mut u32, yb.as_mut_ptr() as *mut u32);
        // SAFETY: each block writes its slots of the source (the last one
        // 64 when the rest fits: less padding); each merge level writes all
        // `padded` slots of its destination.
        unsafe {
            let mut b = 0;
            while b < n {
                if n - b <= 64 {
                    let mut r: [V; 8] = std::array::from_fn(|i| {
                        load(p.add(b + 8 * i), n.saturating_sub(b + 8 * i), f)
                    });
                    sort_regs::<8>(&mut r);
                    for i in 0..8 {
                        _mm256_storeu_si256(px.add(b + 8 * i) as *mut V, r[i]);
                    }
                    b += 64;
                } else {
                    let mut r: [V; 16] = std::array::from_fn(|i| {
                        load(p.add(b + 8 * i), n.saturating_sub(b + 8 * i), f)
                    });
                    sort_regs::<16>(&mut r);
                    for i in 0..16 {
                        _mm256_storeu_si256(px.add(b + 8 * i) as *mut V, r[i]);
                    }
                    b += 128;
                }
            }
            // The last run merged only up to the multiple of 8 above its
            // keys, its padding (the maximum) sorted to its end (cold Ryzen
            // against the best rival, 150-450 keys: 6-27 more cells won).
            let padded = n.next_multiple_of(8);
            let mut w = 128;
            while w < padded {
                let src = std::slice::from_raw_parts(px, padded);
                let mut s = 0;
                while s < padded {
                    let mid = (s + w).min(padded);
                    let end = (s + 2 * w).min(padded);
                    if mid == end {
                        std::ptr::copy_nonoverlapping(px.add(s), py.add(s), end - s);
                    } else {
                        merge_runs(&src[s..mid], &src[mid..end], py.add(s));
                    }
                    s = end;
                }
                std::mem::swap(&mut px, &mut py);
                w *= 2;
            }
            for i in (0..n).step_by(8) {
                let v = _mm256_loadu_si256(px.add(i) as *const V);
                store(p.add(i), n - i, v, f);
            }
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::BUF;
    use std::arch::aarch64::*;

    type V = uint32x4_t;

    // SAFETY (all intrinsics of this module): NEON is part of AArch64.

    #[inline(always)]
    fn rev(v: V) -> V {
        unsafe {
            let x = vrev64q_u32(v);
            vextq_u32(x, x, 2)
        }
    }

    /// A bitonic register sorted ascending: lane distances 2, 1.
    #[inline(always)]
    fn clean(v: V) -> V {
        unsafe {
            let x = vextq_u32(v, v, 2);
            let (l, h) = (vminq_u32(v, x), vmaxq_u32(v, x));
            let v = vcombine_u32(vget_low_u32(l), vget_high_u32(h));
            let x = vrev64q_u32(v);
            let (l, h) = (vminq_u32(v, x), vmaxq_u32(v, x));
            vtrn1q_u32(l, h)
        }
    }

    /// Two ascending runs of `N / 2` registers (the halves of `r`) merged
    /// into one (`N` constant: unrolled, kept in registers).
    #[inline(always)]
    fn merge<const N: usize>(mut r: [V; N]) -> [V; N] {
        let h = N / 2;
        for i in 0..h / 2 {
            let (a, b) = (r[h + i], r[N - 1 - i]);
            r[h + i] = rev(b);
            r[N - 1 - i] = rev(a);
        }
        if h % 2 == 1 {
            r[h + h / 2] = rev(r[h + h / 2]);
        }
        let mut d = h;
        while d > 0 {
            for i in 0..N {
                if i & d == 0 {
                    let (a, b) = (r[i], r[i + d]);
                    unsafe {
                        r[i] = vminq_u32(a, b);
                        r[i + d] = vmaxq_u32(a, b);
                    }
                }
            }
            d /= 2;
        }
        for v in r.iter_mut() {
            *v = clean(*v);
        }
        r
    }

    /// Merges every group of `N` registers of `r` (runs of `N / 2`).
    #[inline(always)]
    fn level<const R: usize, const N: usize>(r: &mut [V; R]) {
        for g in 0..R / N {
            let q: [V; N] = std::array::from_fn(|i| r[g * N + i]);
            let q = merge::<N>(q);
            for i in 0..N {
                r[g * N + i] = q[i];
            }
        }
    }

    /// 4 x 4 transpose.
    #[inline(always)]
    fn transpose(r: &mut [V; 4]) {
        unsafe {
            let t0 = vtrn1q_u32(r[0], r[1]);
            let t1 = vtrn2q_u32(r[0], r[1]);
            let t2 = vtrn1q_u32(r[2], r[3]);
            let t3 = vtrn2q_u32(r[2], r[3]);
            let q = |a: V| vreinterpretq_u64_u32(a);
            r[0] = vreinterpretq_u32_u64(vtrn1q_u64(q(t0), q(t2)));
            r[1] = vreinterpretq_u32_u64(vtrn1q_u64(q(t1), q(t3)));
            r[2] = vreinterpretq_u32_u64(vtrn2q_u64(q(t0), q(t2)));
            r[3] = vreinterpretq_u32_u64(vtrn2q_u64(q(t1), q(t3)));
        }
    }

    /// Sorts `R` registers (16 or 32): the lane columns of each group of 4
    /// sorted (5 comparators), transposed, then merged.
    #[inline(always)]
    fn sort_regs<const R: usize>(r: &mut [V; R]) {
        for g in 0..R / 4 {
            let mut q: [V; 4] = std::array::from_fn(|i| r[4 * g + i]);
            for &(a, b) in [(0, 1), (2, 3), (0, 2), (1, 3), (1, 2)].iter() {
                let (x, y) = (q[a], q[b]);
                unsafe {
                    q[a] = vminq_u32(x, y);
                    q[b] = vmaxq_u32(x, y);
                }
            }
            transpose(&mut q);
            for i in 0..4 {
                r[4 * g + i] = q[i];
            }
        }
        level::<R, 2>(r);
        level::<R, 4>(r);
        level::<R, 8>(r);
        level::<R, 16>(r);
        if R == 32 {
            level::<R, 32>(r);
        }
    }

    /// The keys `k ^ flip` of up to 4 values at `p` (`m` of them), the
    /// lanes past them `u32::MAX`.
    #[inline(always)]
    unsafe fn load(p: *const u32, m: usize, f: V) -> V {
        unsafe {
            if m >= 4 {
                return veorq_u32(vld1q_u32(p), f);
            }
            let mut t = [u32::MAX; 4];
            for i in 0..m {
                t[i] = p.add(i).read() ^ vgetq_lane_u32(f, 0);
            }
            vld1q_u32(t.as_ptr())
        }
    }

    /// The first `m` lanes of `v ^ flip` written to `p`.
    #[inline(always)]
    unsafe fn store(p: *mut u32, m: usize, v: V, f: V) {
        unsafe {
            let v = veorq_u32(v, f);
            if m >= 4 {
                vst1q_u32(p, v);
                return;
            }
            let mut t = [0u32; 4];
            vst1q_u32(t.as_mut_ptr(), v);
            std::ptr::copy_nonoverlapping(t.as_ptr(), p, m);
        }
    }

    /// Sorts the `n` keys at `p` (at most `4 * R`) in `R` registers.
    #[inline(always)]
    unsafe fn sort_direct<const R: usize>(p: *mut u32, n: usize, f: V) {
        let mut r: [V; R] =
            std::array::from_fn(|i| unsafe { load(p.add(4 * i), n.saturating_sub(4 * i), f) });
        sort_regs::<R>(&mut r);
        for i in 0..R {
            if 4 * i < n {
                unsafe { store(p.add(4 * i), n - 4 * i, r[i], f) };
            }
        }
    }

    /// `sort_direct` with the range check of `sort_or_range`.
    #[inline(always)]
    unsafe fn direct_or_range<const R: usize>(p: *mut u32, n: usize, f: V, narrow: usize) -> Option<(u32, u32)> {
        let mut r: [V; R] =
            std::array::from_fn(|i| unsafe { load(p.add(4 * i), n.saturating_sub(4 * i), f) });
        let (mut lo, mut hi) = (u32::MAX, 0u32);
        unsafe {
            for i in 0..R {
                if 4 * i < n {
                    lo = lo.min(vminvq_u32(r[i]));
                    if n - 4 * i >= 4 {
                        hi = hi.max(vmaxvq_u32(r[i]));
                    } else {
                        for j in 4 * i..n {
                            hi = hi.max(p.add(j).read() ^ vgetq_lane_u32(f, 0));
                        }
                    }
                }
            }
        }
        if ((hi - lo) as usize) < narrow {
            return Some((lo, hi));
        }
        sort_regs::<R>(&mut r);
        for i in 0..R {
            if 4 * i < n {
                unsafe { store(p.add(4 * i), n - 4 * i, r[i], f) };
            }
        }
        None
    }

    pub(super) unsafe fn sort_or_range(bits: &mut [u32], flip: u32, narrow: usize) -> Option<(u32, u32)> {
        let n = bits.len();
        let f = unsafe { vdupq_n_u32(flip) };
        let p = bits.as_mut_ptr();
        if n <= 64 {
            unsafe { direct_or_range::<16>(p, n, f, narrow) }
        } else {
            unsafe { direct_or_range::<32>(p, n, f, narrow) }
        }
    }

    /// Merges the runs `a` and `b` (lengths non-zero multiples of 8) into
    /// `out`, 8 keys (two registers) per step.
    #[inline(always)]
    unsafe fn merge_runs(a: &[u32], b: &[u32], out: *mut u32) {
        let ld = |s: &[u32], i: usize| unsafe {
            [vld1q_u32(s.as_ptr().add(i)), vld1q_u32(s.as_ptr().add(i + 4))]
        };
        let (mut i, mut j, mut o) = (8, 8, 0);
        let mut lo = ld(a, 0);
        let mut hi = ld(b, 0);
        loop {
            let q = merge::<4>([lo[0], lo[1], hi[0], hi[1]]);
            hi = [q[2], q[3]];
            unsafe {
                vst1q_u32(out.add(o), q[0]);
                vst1q_u32(out.add(o + 4), q[1]);
            }
            o += 8;
            let take_a = if i < a.len() {
                j == b.len() || a[i] <= b[j]
            } else if j < b.len() {
                false
            } else {
                break;
            };
            if take_a {
                lo = ld(a, i);
                i += 8;
            } else {
                lo = ld(b, j);
                j += 8;
            }
        }
        unsafe {
            vst1q_u32(out.add(o), hi[0]);
            vst1q_u32(out.add(o + 4), hi[1]);
        }
    }

    pub(super) unsafe fn sort(bits: &mut [u32], flip: u32) {
        let n = bits.len();
        let f = unsafe { vdupq_n_u32(flip) };
        let p = bits.as_mut_ptr();
        if n <= 64 {
            unsafe { sort_direct::<16>(p, n, f) };
            return;
        }
        if n <= 128 {
            unsafe { sort_direct::<32>(p, n, f) };
            return;
        }
        let mut xb = std::mem::MaybeUninit::<[u32; BUF]>::uninit();
        let mut yb = std::mem::MaybeUninit::<[u32; BUF]>::uninit();
        let (mut px, mut py) = (xb.as_mut_ptr() as *mut u32, yb.as_mut_ptr() as *mut u32);
        // SAFETY: as the AVX2 version.
        unsafe {
            let mut b = 0;
            while b < n {
                if n - b <= 64 {
                    let mut r: [V; 16] = std::array::from_fn(|i| {
                        load(p.add(b + 4 * i), n.saturating_sub(b + 4 * i), f)
                    });
                    sort_regs::<16>(&mut r);
                    for i in 0..16 {
                        vst1q_u32(px.add(b + 4 * i), r[i]);
                    }
                    b += 64;
                } else {
                    let mut r: [V; 32] = std::array::from_fn(|i| {
                        load(p.add(b + 4 * i), n.saturating_sub(b + 4 * i), f)
                    });
                    sort_regs::<32>(&mut r);
                    for i in 0..32 {
                        vst1q_u32(px.add(b + 4 * i), r[i]);
                    }
                    b += 128;
                }
            }
            let padded = b;
            let mut w = 128;
            while w < padded {
                let src = std::slice::from_raw_parts(px, padded);
                let mut s = 0;
                while s < padded {
                    let mid = (s + w).min(padded);
                    let end = (s + 2 * w).min(padded);
                    if mid == end {
                        std::ptr::copy_nonoverlapping(px.add(s), py.add(s), end - s);
                    } else {
                        merge_runs(&src[s..mid], &src[mid..end], py.add(s));
                    }
                    s = end;
                }
                std::mem::swap(&mut px, &mut py);
                w *= 2;
            }
            for i in (0..n).step_by(4) {
                let v = vld1q_u32(px.add(i));
                store(p.add(i), n - i, v, f);
            }
        }
    }
}
