//! Gap merges of two balanced runs (the shorter one moved out, the merge
//! writing into the gap it leaves, from both ends): `merge_two_x`.

use std::mem::MaybeUninit;

use super::*;

/// Two-run merges through the shorter run in the buffer with several chains
/// in flight (`merge_two`, pipe organs): the merge is cut at crossover
/// points into merges whose two inputs both lie outside their output, each
/// done from both ends on both halves (four chains), instead of one chain
/// from the side the buffered run came from. Idea of glidesort (Orson
/// Peters, 2023: gap merges split at the crossover point); own code.
/// Below this many elements on the shorter side a gap merge is finished
/// with one chain.
pub(crate) const X2_SMALL: usize = 64;

/// Number of elements of `l[..ln]` among the first `d` of the stable merge
/// of `l[..ln]` and `r[..rn]` (left first on ties).
///
/// # Safety
/// `l`, `r` valid for `ln`, `rn` elements; `d <= ln + rn`.
pub(crate) unsafe fn corank2<T, E: Fn(&T) -> u128>(
    l: *const T,
    ln: usize,
    r: *const T,
    rn: usize,
    d: usize,
    enc: &E,
) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(rn), d.min(ln));
    while lo < hi {
        let i = (lo + hi) / 2;
        if enc(&*l.add(i)) <= enc(&*r.add(d - i - 1)) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Forward one-chain merge of `l[..ln]` (outside the output) and `r[..rn]`,
/// which is the tail of the output `d[..ln + rn]` (`r == d + ln`), left
/// first on ties. Each write lands at or before the next unread element
/// of `r`.
///
/// # Safety
/// As stated.
#[inline(always)]
pub(crate) unsafe fn merge_fwd_gap<T, E: Fn(&T) -> u128>(
    mut l: *const T,
    ln: usize,
    mut r: *const T,
    rn: usize,
    mut d: *mut T,
    enc: &E,
) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        let t = enc(&*r) < enc(&*l);
        d.write(std::ptr::read(pick(t, r, l)));
        d = d.add(1);
        r = r.add(t as usize);
        l = l.add(!t as usize);
    }
    // What is left of `l` fills the slots before the unread rest of `r`,
    // which is already in place.
    std::ptr::copy_nonoverlapping(l, d, le.offset_from(l) as usize);
}

/// Backward one-chain merge of `l[..ln]`, the head of the output
/// (`l == d`), and `r[..rn]` (outside the output) into `d[..ln + rn]`,
/// right last on ties.
///
/// # Safety
/// As stated.
#[inline(always)]
pub(crate) unsafe fn merge_bwd_gap<T, E: Fn(&T) -> u128>(
    l: *const T,
    ln: usize,
    r: *const T,
    rn: usize,
    d: *mut T,
    enc: &E,
) {
    let (mut le, mut re, mut de) = (l.add(ln), r.add(rn), d.add(ln + rn));
    while le > l && re > r {
        let t = enc(&*re.sub(1)) < enc(&*le.sub(1));
        de = de.sub(1);
        de.write(std::ptr::read(pick(t, le, re).sub(1)));
        le = le.sub(t as usize);
        re = re.sub(!t as usize);
    }
    // What is left of `r` fills the slots after the unread rest of `l`.
    let rest = re.offset_from(r) as usize;
    std::ptr::copy_nonoverlapping(r, de.sub(rest), rest);
}

/// Forward branchy merge of two inputs into `d`, none overlapping.
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(always)]
pub(crate) unsafe fn merge_fwd_b_ptr<T, E: Fn(&T) -> u128>(
    mut l: *const T,
    ln: usize,
    mut r: *const T,
    rn: usize,
    mut d: *mut T,
    enc: &E,
) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        if enc(&*r) < enc(&*l) {
            d.write(std::ptr::read(r));
            r = r.add(1);
        } else {
            d.write(std::ptr::read(l));
            l = l.add(1);
        }
        d = d.add(1);
    }
    let k = le.offset_from(l) as usize;
    std::ptr::copy_nonoverlapping(l, d, k);
    std::ptr::copy_nonoverlapping(r, d.add(k), re.offset_from(r) as usize);
}

/// Merge of `l[..ln]` and `r[..rn]` into `d`, neither input overlapping the
/// output: split in half by a binary search, both halves from both ends
/// (four chains).
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(never)]
pub(crate) unsafe fn merge_ext4<T, E: Fn(&T) -> u128>(
    l: *const T,
    ln: usize,
    r: *const T,
    rn: usize,
    d: *mut T,
    enc: &E,
) {
    if ln == 0 || rn == 0 || enc(&*l.add(ln - 1)) <= enc(&*r) {
        std::ptr::copy_nonoverlapping(l, d, ln);
        std::ptr::copy_nonoverlapping(r, d.add(ln), rn);
        return;
    }
    let h = (ln + rn) / 2;
    let i = corank2(l, ln, r, rn, h, enc);
    let (mut alf, mut ale, mut arf, mut are) = (l, l.add(i), r, r.add(h - i));
    let (mut blf, mut ble, mut brf, mut bre) = (l.add(i), l.add(ln), r.add(h - i), r.add(rn));
    let (mut adf, mut ade, mut bdf, mut bde) = (d, d.add(h), d.add(h), d.add(ln + rn));
    let steps = i.min(h - i).min(ln - i).min(rn - (h - i));
    for _ in 0..steps {
        let t = enc(&*arf) < enc(&*alf);
        adf.write(std::ptr::read(pick(t, arf, alf)));
        adf = adf.add(1);
        arf = arf.add(t as usize);
        alf = alf.add(!t as usize);
        let t = enc(&*brf) < enc(&*blf);
        bdf.write(std::ptr::read(pick(t, brf, blf)));
        bdf = bdf.add(1);
        brf = brf.add(t as usize);
        blf = blf.add(!t as usize);
        let t = enc(&*are.sub(1)) < enc(&*ale.sub(1));
        ade = ade.sub(1);
        ade.write(std::ptr::read(pick(t, ale, are).sub(1)));
        ale = ale.sub(t as usize);
        are = are.sub(!t as usize);
        let t = enc(&*bre.sub(1)) < enc(&*ble.sub(1));
        bde = bde.sub(1);
        bde.write(std::ptr::read(pick(t, ble, bre).sub(1)));
        ble = ble.sub(t as usize);
        bre = bre.sub(!t as usize);
    }
    let _ = (ade, bde);
    let len = |a: *const T, b: *const T| b.offset_from(a) as usize;
    merge_fwd_b_ptr(alf, len(alf, ale), arf, len(arf, are), adf, enc);
    merge_fwd_b_ptr(blf, len(blf, ble), brf, len(brf, bre), bdf, enc);
}

/// Left gap merge: `l[..ln]` (outside the array) and `r[..rn]`, which lies
/// at `d + ln`, merged into `d[..ln + rn]`. Cut at the crossover point of
/// the first `ln` outputs: `l[..i]` and `r[..ln - i]` merged into `d[..ln]`
/// (both inputs outside it), then the same with the rest, which has the
/// same shape (`l[i..]` outside, `r[ln - i..]` after a gap of `ln - i`).
///
/// # Safety
/// `l` does not overlap `d[..ln + rn]`; `r == d + ln`.
pub(crate) unsafe fn gap_left<T, E: Fn(&T) -> u128>(
    mut l: *const T,
    mut ln: usize,
    mut r: *mut T,
    mut rn: usize,
    mut d: *mut T,
    enc: &E,
) {
    while ln > 0 {
        if rn == 0 {
            std::ptr::copy_nonoverlapping(l, d, ln);
            return;
        }
        if ln.min(rn) < X2_SMALL {
            merge_fwd_gap(l, ln, r, rn, d, enc);
            return;
        }
        let i = corank2(l, ln, r, rn, ln, enc);
        let j = ln - i;
        merge_ext4(l, i, r, j, d, enc);
        l = l.add(i);
        d = d.add(ln);
        r = r.add(j);
        rn -= j;
        ln = j;
    }
}

/// Right gap merge: `l[..ln]`, which lies at the head of the output `l`,
/// and `r[..rn]` (outside the array) merged into `l[..ln + rn]`, cut from
/// the end (mirror of `gap_left`).
///
/// # Safety
/// `r` does not overlap `l[..ln + rn]`.
pub(crate) unsafe fn gap_right<T, E: Fn(&T) -> u128>(
    l: *mut T,
    mut ln: usize,
    r: *const T,
    mut rn: usize,
    enc: &E,
) {
    while rn > 0 {
        if ln == 0 {
            std::ptr::copy_nonoverlapping(r, l, rn);
            return;
        }
        if ln.min(rn) < X2_SMALL {
            merge_bwd_gap(l, ln, r, rn, l, enc);
            return;
        }
        // Left elements among the last `rn` outputs, right ones the rest.
        let a = ln - corank2(l, ln, r, rn, ln, enc);
        let b = rn - a;
        merge_ext4(l.add(ln - a), a, r.add(rn - b), b, l.add(ln), enc);
        ln -= a;
        rn = a;
    }
}

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer of at least the
/// shorter run.
pub(crate) fn merge_two_x<T, E: Fn(&T) -> u128>(
    arr: &mut [T],
    m: usize,
    enc: &E,
    buf: &mut Vec<MaybeUninit<T>>,
) {
    let n = arr.len();
    if m == 0 || m == n || enc(&arr[m - 1]) <= enc(&arr[m]) {
        return;
    }
    // Keys already in place at both ends stay.
    let (top, bottom) = (enc(&arr[m - 1]), enc(&arr[m]));
    let s = arr[..m].partition_point(|x| enc(x) <= bottom);
    let e = m + arr[m..].partition_point(|x| enc(x) < top);
    let a = &mut arr[s..e];
    let (m, n) = (m - s, e - s);
    let p = a.as_mut_ptr();
    let b = buf.as_mut_ptr() as *mut T;
    // SAFETY: the shorter run is moved to the buffer; the gap merges then
    // fill the array, each write on a slot already read (see `gap_left`,
    // `gap_right`); every element moved once more.
    unsafe {
        if m <= n - m {
            debug_assert!(buf.capacity() >= m);
            std::ptr::copy_nonoverlapping(p, b, m);
            gap_left(b, m, p.add(m), n - m, p, enc);
        } else {
            debug_assert!(buf.capacity() >= n - m);
            std::ptr::copy_nonoverlapping(p.add(m), b, n - m);
            gap_right(p, m, b, n - m, enc);
        }
    }
}
