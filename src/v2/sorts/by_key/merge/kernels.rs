//! Two-run merge kernels from a source to a destination (pointers), with
//! branches or selects, shared by the rounds and the two-run merges.

/// Stable merge of `src[s..m]` and `src[m..e]` into `dst[s..e]` from both
/// ends, with branches: faster when the merge is predictable (long
/// stretches from one run, or a short period).
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
#[inline(never)]
pub(crate) unsafe fn merge_into_b<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    s: usize,
    m: usize,
    e: usize,
    enc: &E,
) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let steps = (m - s).min(e - m);
    let (mut lf, mut le, mut rf, mut re, mut df, mut de) = (s, m, m, e, s, e);
    for _ in 0..steps {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
        de -= 1;
        if enc(&*src.add(re - 1)) < enc(&*src.add(le - 1)) {
            le -= 1;
            dst.add(de).write(std::ptr::read(src.add(le)));
        } else {
            re -= 1;
            dst.add(de).write(std::ptr::read(src.add(re)));
        }
    }
    while lf < le && rf < re {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
}

/// Split of the merge of `src[s..m]` and `src[m..e]` after `d` outputs:
/// the number of left elements among them (stable: left first on ties).
///
/// # Safety
/// The ranges are valid in `src`; `d <= e - s`.
pub(crate) unsafe fn corank<T, E: Fn(&T) -> u128>(
    src: *const T,
    s: usize,
    m: usize,
    e: usize,
    d: usize,
    enc: &E,
) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(e - m), d.min(m - s));
    while lo < hi {
        let i = (lo + hi) / 2;
        if enc(&*src.add(s + i)) <= enc(&*src.add(m + d - i - 1)) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Forward branchy merge of `src[lf..le]` and `src[rf..re]` into `dst[df..]`.
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
#[inline(always)]
pub(crate) unsafe fn merge_fwd_b<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    mut lf: usize,
    le: usize,
    mut rf: usize,
    re: usize,
    mut df: usize,
    enc: &E,
) {
    while lf < le && rf < re {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
}

/// `a` when `c`, else `b`, written as a mask so that LLVM keeps a
/// conditional select (an `if` there becomes a branch, mispredicted half
/// the time on random merges).
/// On x86 LLVM turns the mask back into a branch, but honours
/// `select_unpredictable` (cmov); on aarch64 it is the reverse (checked on
/// rustc 1.99).
#[inline(always)]
pub(crate) fn pick<T>(c: bool, a: *const T, b: *const T) -> *const T {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    return std::hint::select_unpredictable(c, a, b);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    return (b as usize ^ ((a as usize ^ b as usize) & (c as usize).wrapping_neg())) as *const T;
}

/// `merge_into_p` as two independent merges (the output split in half by
/// a binary search): four chains in flight.
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
#[inline(never)]
pub(crate) unsafe fn merge_into_p4<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    s: usize,
    m: usize,
    e: usize,
    enc: &E,
) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let d = (e - s) / 2;
    let i = corank(src, s, m, e, d, enc);
    let (mut alf, mut ale, mut arf, mut are) =
        (src.add(s), src.add(s + i), src.add(m), src.add(m + d - i));
    let (mut blf, mut ble, mut brf, mut bre) =
        (src.add(s + i), src.add(m), src.add(m + d - i), src.add(e));
    let (mut adf, mut ade, mut bdf, mut bde) =
        (dst.add(s), dst.add(s + d), dst.add(s + d), dst.add(e));
    let steps = i.min(d - i).min(m - s - i).min(e - m - d + i);
    for _ in 0..steps {
        let r = enc(&*arf) < enc(&*alf);
        adf.write(std::ptr::read(pick(r, arf, alf)));
        adf = adf.add(1);
        arf = arf.add(r as usize);
        alf = alf.add(!r as usize);
        let r = enc(&*brf) < enc(&*blf);
        bdf.write(std::ptr::read(pick(r, brf, blf)));
        bdf = bdf.add(1);
        brf = brf.add(r as usize);
        blf = blf.add(!r as usize);
        let l = enc(&*are.sub(1)) < enc(&*ale.sub(1));
        ade = ade.sub(1);
        ade.write(std::ptr::read(pick(l, ale, are).sub(1)));
        ale = ale.sub(l as usize);
        are = are.sub(!l as usize);
        let l = enc(&*bre.sub(1)) < enc(&*ble.sub(1));
        bde = bde.sub(1);
        bde.write(std::ptr::read(pick(l, ble, bre).sub(1)));
        ble = ble.sub(l as usize);
        bre = bre.sub(!l as usize);
    }
    let _ = (ade, bde);
    let o = |p: *const T| p.offset_from(src) as usize;
    merge_fwd_b(src, dst, o(alf), o(ale), o(arf), o(are), adf.offset_from(dst) as usize, enc);
    merge_fwd_b(src, dst, o(blf), o(ble), o(brf), o(bre), bdf.offset_from(dst) as usize, enc);
}
