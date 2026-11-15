//! Two-run merges in place through a buffer of the shorter run only, with
//! four chains in flight: the shorter run is moved out, then the merge is
//! cut at crossover points into merges whose two inputs both lie outside
//! their output, each done from both ends on both halves. Idea of
//! glidesort (Orson Peters, 2023: gap merges split at the crossover
//! point); the records' version is `records2::merge::merge_two_x`. Own
//! code.

/// Below this many elements on the shorter side a gap merge is finished
/// with one chain.
const SMALL: usize = 64;

/// `a` when `c`, else `b`, kept a select (see `bidir::pick`).
#[inline(always)]
fn pick<T>(c: bool, a: *const T, b: *const T) -> *const T {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    return std::hint::select_unpredictable(c, a, b);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    return (b as usize ^ ((a as usize ^ b as usize) & (c as usize).wrapping_neg())) as *const T;
}

/// Elements of `l` among the first `d` outputs of the merge of `l[..ln]`
/// and `r[..rn]`, left first on ties.
///
/// # Safety
/// Valid ranges; `d <= ln + rn`.
unsafe fn corank<T: PartialOrd>(l: *const T, ln: usize, r: *const T, rn: usize, d: usize) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(rn), d.min(ln));
    while lo < hi {
        let i = (lo + hi) / 2;
        if *l.add(i) <= *r.add(d - i - 1) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Forward one-chain merge of `l[..ln]` (outside the output) and `r[..rn]`,
/// the tail of the output `d[..ln + rn]` (`r == d + ln`): each write lands
/// at or before the next unread element of `r`.
///
/// # Safety
/// As stated.
#[inline(always)]
unsafe fn fwd_gap<T: Copy + PartialOrd>(mut l: *const T, ln: usize, mut r: *const T, rn: usize, mut d: *mut T) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        let t = *r < *l;
        *d = *pick(t, r, l);
        d = d.add(1);
        r = r.add(t as usize);
        l = l.add(!t as usize);
    }
    std::ptr::copy_nonoverlapping(l, d, le.offset_from(l) as usize);
}

/// Backward one-chain merge of `l[..ln]`, the head of the output
/// (`l == d`), and `r[..rn]` (outside the output) into `d[..ln + rn]`.
///
/// # Safety
/// As stated.
#[inline(always)]
unsafe fn bwd_gap<T: Copy + PartialOrd>(l: *const T, ln: usize, r: *const T, rn: usize, d: *mut T) {
    let (mut le, mut re, mut de) = (l.add(ln), r.add(rn), d.add(ln + rn));
    while le > l && re > r {
        let t = *re.sub(1) < *le.sub(1);
        de = de.sub(1);
        *de = *pick(t, le, re).sub(1);
        le = le.sub(t as usize);
        re = re.sub(!t as usize);
    }
    let rest = re.offset_from(r) as usize;
    std::ptr::copy_nonoverlapping(r, de.sub(rest), rest);
}

/// Forward merge of two inputs into `d`, none overlapping.
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(always)]
unsafe fn fwd<T: Copy + PartialOrd>(mut l: *const T, ln: usize, mut r: *const T, rn: usize, mut d: *mut T) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        let t = *r < *l;
        *d = *pick(t, r, l);
        d = d.add(1);
        r = r.add(t as usize);
        l = l.add(!t as usize);
    }
    let k = le.offset_from(l) as usize;
    std::ptr::copy_nonoverlapping(l, d, k);
    std::ptr::copy_nonoverlapping(r, d.add(k), re.offset_from(r) as usize);
}

/// Merge of `l[..ln]` and `r[..rn]` into `d`, neither input overlapping the
/// output: split in half by a binary search, both halves from both ends.
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(never)]
unsafe fn ext4<T: Copy + PartialOrd>(l: *const T, ln: usize, r: *const T, rn: usize, d: *mut T) {
    if ln == 0 || rn == 0 || *l.add(ln - 1) <= *r {
        std::ptr::copy_nonoverlapping(l, d, ln);
        std::ptr::copy_nonoverlapping(r, d.add(ln), rn);
        return;
    }
    let h = (ln + rn) / 2;
    let i = corank(l, ln, r, rn, h);
    let (mut alf, mut ale, mut arf, mut are) = (l, l.add(i), r, r.add(h - i));
    let (mut blf, mut ble, mut brf, mut bre) = (l.add(i), l.add(ln), r.add(h - i), r.add(rn));
    let (mut adf, mut ade, mut bdf, mut bde) = (d, d.add(h), d.add(h), d.add(ln + rn));
    let steps = i.min(h - i).min(ln - i).min(rn - (h - i));
    for _ in 0..steps {
        let t = *arf < *alf;
        *adf = *pick(t, arf, alf);
        adf = adf.add(1);
        arf = arf.add(t as usize);
        alf = alf.add(!t as usize);
        let t = *brf < *blf;
        *bdf = *pick(t, brf, blf);
        bdf = bdf.add(1);
        brf = brf.add(t as usize);
        blf = blf.add(!t as usize);
        let t = *are.sub(1) < *ale.sub(1);
        ade = ade.sub(1);
        *ade = *pick(t, ale, are).sub(1);
        ale = ale.sub(t as usize);
        are = are.sub(!t as usize);
        let t = *bre.sub(1) < *ble.sub(1);
        bde = bde.sub(1);
        *bde = *pick(t, ble, bre).sub(1);
        ble = ble.sub(t as usize);
        bre = bre.sub(!t as usize);
    }
    let _ = (ade, bde);
    let len = |a: *const T, b: *const T| b.offset_from(a) as usize;
    fwd(alf, len(alf, ale), arf, len(arf, are), adf);
    fwd(blf, len(blf, ble), brf, len(brf, bre), bdf);
}

/// Left gap merge: `l[..ln]` (outside the array) and `r[..rn]`, which lies
/// at `d + ln`, merged into `d[..ln + rn]`, cut at the crossover point of
/// the first `ln` outputs (the rest has the same shape).
///
/// # Safety
/// `l` does not overlap `d[..ln + rn]`; `r == d + ln`.
unsafe fn gap_left<T: Copy + PartialOrd>(mut l: *const T, mut ln: usize, mut r: *const T, mut rn: usize, mut d: *mut T) {
    while ln > 0 {
        if rn == 0 {
            std::ptr::copy_nonoverlapping(l, d, ln);
            return;
        }
        if ln.min(rn) < SMALL {
            fwd_gap(l, ln, r, rn, d);
            return;
        }
        let i = corank(l, ln, r, rn, ln);
        let j = ln - i;
        ext4(l, i, r, j, d);
        l = l.add(i);
        d = d.add(ln);
        r = r.add(j);
        rn -= j;
        ln = j;
    }
}

/// Right gap merge: `l[..ln]`, the head of the output `l`, and `r[..rn]`
/// (outside the array) merged into `l[..ln + rn]`, cut from the end.
///
/// # Safety
/// `r` does not overlap `l[..ln + rn]`.
unsafe fn gap_right<T: Copy + PartialOrd>(l: *mut T, mut ln: usize, r: *const T, mut rn: usize) {
    while rn > 0 {
        if ln == 0 {
            std::ptr::copy_nonoverlapping(r, l, rn);
            return;
        }
        if ln.min(rn) < SMALL {
            bwd_gap(l, ln, r, rn, l);
            return;
        }
        let a = ln - corank(l, ln, r, rn, ln);
        let b = rn - a;
        ext4(l.add(ln - a), a, r.add(rn - b), b, l.add(ln));
        ln -= a;
        rn = a;
    }
}

/// Stable merge of the sorted runs `arr[..m]` and `arr[m..]` (each element
/// trimmed at the ends by the caller), the shorter run moved to `buf`.
pub(super) fn merge_two_gap<T: Copy + PartialOrd>(arr: &mut [T], m: usize, buf: &mut Vec<T>) {
    let n = arr.len();
    let short = m.min(n - m);
    if buf.len() < short {
        buf.resize(short, arr[0]);
    }
    let p = arr.as_mut_ptr();
    let b = buf.as_mut_ptr();
    // SAFETY: the shorter run is copied to the buffer; the gap merges then
    // fill the array, each write on a slot already read (`gap_left`,
    // `gap_right`). `T: Copy`: nothing to drop.
    unsafe {
        if m <= n - m {
            std::ptr::copy_nonoverlapping(p, b, m);
            gap_left(b, m, p.add(m), n - m, p);
        } else {
            std::ptr::copy_nonoverlapping(p.add(m), b, n - m);
            gap_right(p, m, b, n - m);
        }
    }
}
