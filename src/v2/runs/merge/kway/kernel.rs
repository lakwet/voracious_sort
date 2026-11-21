//! Two-run merge kernels on raw pointers, the engineering of voracious v2
//! (`src/v2/runs/merge/`: `bidir.rs`, `gap.rs`, `mod.rs::merge_out`).
//! Stability: `a` is the run of left origin, `b` of right origin; on ties
//! `a` comes first, wherever the runs lie in memory.
//!
//! ```text
//! merge_out   a, b, out disjoint: already ordered → copies; head / tail blocks
//!             already placed copied; balanced: predictable → branchy two-ended,
//!             else 4 chains; skewed: galloping stretches
//! hole_fwd    one run already at the END of out: cut so that the first part
//!             fills exactly the hole (disjoint: 4 chains), then again
//! hole_bwd    mirror: the run in place at the START of out, from the end
//! ```

use core::ptr;

/// One piece of a hole merge (inputs disjoint from the output); `pred`: the
/// decision of the whole merge, if taken.
#[inline]
unsafe fn piece<T: Copy + PartialOrd>(
    a: *const T,
    la: usize,
    b: *const T,
    lb: usize,
    d: *mut T,
    pred: Option<bool>,
) {
    merge_out_p(a, la, b, lb, d, pred)
}

/// The decision of a whole hole merge, taken once (deciding per piece
/// costs 64 steps each time: 5-6 % at 10k, measured).
unsafe fn pred_once<T: PartialOrd>(a: *const T, la: usize, b: *const T, lb: usize) -> Option<bool> {
    Some(predictable(std::slice::from_raw_parts(a, la), std::slice::from_raw_parts(b, lb)))
}

/// Moves counter of kwaymerge: nothing here.
#[inline(always)]
pub fn count(_: usize) {}

/// `a` when `c`, else `b`, kept a select (`cmov`) by
/// `select_unpredictable`. Used by the x86 steps only.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
fn pick<T>(c: bool, a: *const T, b: *const T) -> *const T {
    std::hint::select_unpredictable(c, a, b)
}

/// An empty `asm!` in a branch: the compiler keeps the branch instead of a
/// conditional move (v2 `bidir::keep_branch`).
#[inline(always)]
fn keep_branch() {
    #[cfg(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    unsafe {
        std::arch::asm!("", options(nomem, nostack, preserves_flags));
    }
}

/// Stable co-rank: the first `k` outputs of merge(a, b) are a[..i] and
/// b[..k - i]; returns `i`.
#[inline]
pub unsafe fn corank<T: PartialOrd>(
    a: *const T,
    la: usize,
    b: *const T,
    lb: usize,
    k: usize,
) -> usize {
    let (mut lo, mut hi) = (k.saturating_sub(lb), k.min(la));
    while lo < hi {
        let i = lo + (hi - lo) / 2;
        if *a.add(i) <= *b.add(k - i - 1) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Whether a branchy merge of `a` and `b` is predictable, from its first 64
/// steps: few switches, or a choice sequence periodic with a period of at
/// most 16 (v2 `predictable::predictable_pair`).
pub fn predictable<T: PartialOrd>(a: &[T], b: &[T]) -> bool {
    let (mut i, mut j, mut bits, mut steps) = (0, 0, 0u64, 0);
    while steps < 64 && i < a.len() && j < b.len() {
        let take_left = a[i] <= b[j];
        bits |= u64::from(take_left) << steps;
        i += take_left as usize;
        j += !take_left as usize;
        steps += 1;
    }
    if steps < 64 || (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}

/// One branchless step from the front: the lower of `*r`, `*l` (`l` on
/// ties) written at `d`, its run and `d` advanced. One form per
/// architecture, the faster one measured on each: on x86 the pointer is
/// chosen and `l` advanced as `l - t + 1` (one operation less than
/// `l + !t`; idea seen in glidesort); on aarch64 the value is chosen among
/// the two loaded and `l` advanced as `l + !t`.
#[inline(always)]
unsafe fn front<T: Copy + PartialOrd>(l: &mut *const T, r: &mut *const T, d: &mut *mut T) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        let t = **r < **l;
        **d = *pick(t, *r, *l);
        *r = r.add(t as usize);
        *l = l.wrapping_sub(t as usize).wrapping_add(1);
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        let (x, y) = (**r, **l);
        let t = x < y;
        **d = std::hint::select_unpredictable(t, x, y);
        *r = r.add(t as usize);
        *l = l.add(!t as usize);
    }
    *d = d.add(1);
}

/// One step from the back (`le`, `re`, `de`: one past the ends): the higher
/// of the last two (`re`'s on ties) written before `de`.
#[inline(always)]
unsafe fn back<T: Copy + PartialOrd>(le: &mut *const T, re: &mut *const T, de: &mut *mut T) {
    *de = de.sub(1);
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        let t = *re.sub(1) < *le.sub(1);
        **de = *pick(t, *le, *re).sub(1);
        *le = le.sub(t as usize);
        *re = re.wrapping_add(t as usize).wrapping_sub(1);
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        let (x, y) = (*re.sub(1), *le.sub(1));
        let t = x < y;
        **de = std::hint::select_unpredictable(t, y, x);
        *le = le.sub(t as usize);
        *re = re.sub(!t as usize);
    }
}

/// One chain forward, disjoint (or the output behind `r`): the rest of both
/// runs copied after.
#[inline(always)]
unsafe fn fwd<T: Copy + PartialOrd>(
    mut l: *const T,
    ln: usize,
    mut r: *const T,
    rn: usize,
    mut d: *mut T,
) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        front(&mut l, &mut r, &mut d);
    }
    let k = le.offset_from(l) as usize;
    ptr::copy(l, d, k);
    ptr::copy(r, d.add(k), re.offset_from(r) as usize);
}

/// One chain backward (the output may end on `l`'s place): the rest copied.
#[inline(always)]
unsafe fn bwd<T: Copy + PartialOrd>(l: *const T, ln: usize, r: *const T, rn: usize, d: *mut T) {
    let (mut le, mut re, mut de) = (l.add(ln), r.add(rn), d.add(ln + rn));
    while le > l && re > r {
        back(&mut le, &mut re, &mut de);
    }
    let (kl, kr) = (le.offset_from(l) as usize, re.offset_from(r) as usize);
    ptr::copy(r, d.add(kl), kr);
    ptr::copy(l, d, kl);
}

/// Two-ended merge (v2 `bidir::merge_bidir`): `min(la, lb)` steps from both
/// ends without any bound check, then the middle forward. `BRANCHY`: real
/// branches, for predictable interleavings.
#[inline(always)]
unsafe fn bidir<T: Copy + PartialOrd, const BRANCHY: bool>(
    a: *const T,
    la: usize,
    b: *const T,
    lb: usize,
    d: *mut T,
) {
    let steps = la.min(lb);
    let (mut lf, mut le, mut rf, mut re) = (a, a.add(la), b, b.add(lb));
    let (mut df, mut de) = (d, d.add(la + lb));
    for _ in 0..steps {
        if !BRANCHY {
            front(&mut lf, &mut rf, &mut df);
            back(&mut le, &mut re, &mut de);
            continue;
        }
        if *rf < *lf {
            *df = *rf;
            rf = rf.add(1);
            keep_branch();
        } else {
            *df = *lf;
            lf = lf.add(1);
        }
        df = df.add(1);
        de = de.sub(1);
        if *re.sub(1) < *le.sub(1) {
            le = le.sub(1);
            *de = *le;
            keep_branch();
        } else {
            re = re.sub(1);
            *de = *re;
        }
    }
    fwd(lf, le.offset_from(lf) as usize, rf, re.offset_from(rf) as usize, df);
}

/// Shortest run for one more round from both ends.
const ROUND_MIN: usize = 8;

/// The rest of a merge (`d` exactly as long as both runs): rounds of
/// `min(la, lb)` unchecked steps from both ends (each round leaves about a
/// third when the runs are 1:2), then one checked chain for the end.
#[inline(always)]
unsafe fn rounds<T: Copy + PartialOrd>(
    mut a: *const T,
    mut la: usize,
    mut b: *const T,
    mut lb: usize,
    mut d: *mut T,
) {
    while la.min(lb) >= ROUND_MIN {
        let steps = la.min(lb);
        let (mut lf, mut le, mut rf, mut re) = (a, a.add(la), b, b.add(lb));
        let (mut df, mut de) = (d, d.add(la + lb));
        for _ in 0..steps {
            front(&mut lf, &mut rf, &mut df);
            back(&mut le, &mut re, &mut de);
        }
        a = lf;
        la = le.offset_from(lf) as usize;
        b = rf;
        lb = re.offset_from(rf) as usize;
        d = df;
    }
    fwd(a, la, b, lb, d);
}

/// Four chains (v2 `gap::ext4`): cut in two halves by co-rank, each merged
/// from both ends, the four steps interleaved; the bound of the unchecked
/// phase is the shortest of the four sub-runs, the rests by `rounds`.
/// Steps: `front`, `back` (per architecture). Inputs disjoint from `d`.
#[inline(never)]
pub unsafe fn four<T: Copy + PartialOrd>(
    l: *const T,
    ln: usize,
    r: *const T,
    rn: usize,
    d: *mut T,
) {
    count(ln + rn);
    if ln == 0 || rn == 0 || *l.add(ln - 1) <= *r {
        ptr::copy_nonoverlapping(l, d, ln);
        ptr::copy_nonoverlapping(r, d.add(ln), rn);
        return;
    }
    let h = (ln + rn) / 2;
    let i = corank(l, ln, r, rn, h);
    let (mut alf, mut ale, mut arf, mut are) = (l, l.add(i), r, r.add(h - i));
    let (mut blf, mut ble, mut brf, mut bre) = (l.add(i), l.add(ln), r.add(h - i), r.add(rn));
    let (mut adf, mut ade, mut bdf, mut bde) = (d, d.add(h), d.add(h), d.add(ln + rn));
    let steps = i.min(h - i).min(ln - i).min(rn - (h - i));
    for _ in 0..steps {
        front(&mut alf, &mut arf, &mut adf);
        front(&mut blf, &mut brf, &mut bdf);
        back(&mut ale, &mut are, &mut ade);
        back(&mut ble, &mut bre, &mut bde);
    }
    let len = |a: *const T, b: *const T| b.offset_from(a) as usize;
    rounds(alf, len(alf, ale), arf, len(arf, are), adf);
    rounds(blf, len(blf, ble), brf, len(brf, bre), bdf);
}

/// `partition_point` from the start by exponential steps (1, 2, 4, ...)
/// then a binary search in the last step: O(log of the answer), not of the
/// whole slice (a stretch is short beside the rest of a long run).
#[inline]
fn gallop<T>(s: &[T], pred: impl Fn(&T) -> bool) -> usize {
    let mut lo = 0;
    let mut step = 1;
    while lo + step <= s.len() && pred(&s[lo + step - 1]) {
        lo += step;
        step *= 2;
    }
    let hi = (lo + step).min(s.len() + 1) - 1;
    lo + s[lo..hi].partition_point(pred)
}

/// Stretch length from which the skewed merges gallop (TimSort 7).
pub const GALLOP_MIN: usize = 8;

/// A merge gallops when its short run is under a quarter (galloping from
/// 1:16 only, out of place, was measured neutral to 13 % slower).
pub fn skewed(short: usize, len: usize) -> bool { 4 * short < len }

/// Out-of-place stable merge of `a` and `b` into `d`, all disjoint (v2
/// `merge_out`): ordered → copied; head and tail blocks already ordered
/// against the other run copied; balanced → predictable ? branchy
/// two-ended : 4 chains; skewed → galloping stretches.
pub unsafe fn merge_out<T: Copy + PartialOrd>(
    a: *const T,
    la: usize,
    b: *const T,
    lb: usize,
    d: *mut T,
) {
    merge_out_p(a, la, b, lb, d, None)
}

/// `merge_out` with the predictability already decided (`Some`).
pub unsafe fn merge_out_p<T: Copy + PartialOrd>(
    a: *const T,
    la: usize,
    b: *const T,
    lb: usize,
    d: *mut T,
    pred: Option<bool>,
) {
    if la == 0 || lb == 0 || *a.add(la - 1) <= *b {
        ptr::copy_nonoverlapping(a, d, la);
        ptr::copy_nonoverlapping(b, d.add(la), lb);
        count(la + lb);
        return;
    }
    let (sa, sb) = (std::slice::from_raw_parts(a, la), std::slice::from_raw_parts(b, lb));
    let head = sb.partition_point(|x| *x < sa[0]);
    let tail = la - sa.partition_point(|x| *x <= sb[lb - 1]);
    let len = la + lb;
    ptr::copy_nonoverlapping(b, d, head);
    ptr::copy_nonoverlapping(a.add(la - tail), d.add(len - tail), tail);
    count(head + tail);
    let (sa, sb) = (&sa[..la - tail], &sb[head..]);
    let d = d.add(head);
    let (la, lb) = (sa.len(), sb.len());
    if la == 0 || lb == 0 {
        ptr::copy_nonoverlapping(sa.as_ptr(), d, la);
        ptr::copy_nonoverlapping(sb.as_ptr(), d.add(la), lb);
        count(la + lb);
        return;
    }
    if !skewed(la.min(lb), la + lb) {
        if pred.unwrap_or_else(|| predictable(sa, sb)) {
            count(la + lb);
            bidir::<T, true>(sa.as_ptr(), la, sb.as_ptr(), lb, d);
        } else {
            four(sa.as_ptr(), la, sb.as_ptr(), lb, d);
        }
        return;
    }
    // Skewed: stretches, galloping from GALLOP_MIN on.
    count(la + lb);
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < la && j < lb {
        let v = sb[j];
        let mut c = 0;
        while c < GALLOP_MIN && i + c < la && sa[i + c] <= v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += gallop(&sa[i + c..], |x| *x <= v);
        }
        ptr::copy_nonoverlapping(sa.as_ptr().add(i), d.add(k), c);
        i += c;
        k += c;
        if i == la {
            break;
        }
        let v = sa[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < lb && sb[j + c] < v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += gallop(&sb[j + c..], |x| *x < v);
        }
        ptr::copy_nonoverlapping(sb.as_ptr().add(j), d.add(k), c);
        j += c;
        k += c;
    }
    ptr::copy_nonoverlapping(sa.as_ptr().add(i), d.add(k), la - i);
    k += la - i;
    ptr::copy_nonoverlapping(sb.as_ptr().add(j), d.add(k), lb - j);
}

/// Below this many elements in the run outside, a hole merge finishes on
/// one chain (v2 `gap::SMALL`).
const HOLE_MIN: usize = 64;

/// Stable merge into `out[0..la + lb)`; the run marked in place (`a` if
/// `a_in_place`, else `b`) lies at the end of `out`, the other elsewhere.
pub unsafe fn hole_fwd<T: Copy + PartialOrd>(
    mut a: *const T,
    mut la: usize,
    mut b: *const T,
    mut lb: usize,
    mut out: *mut T,
    a_in_place: bool,
) {
    let mut pred = None;
    if (if a_in_place { lb } else { la }) >= HOLE_MIN {
        pred = pred_once(a, la, b, lb);
    }
    loop {
        // The hole at the start of out: as long as the run outside.
        let p = if a_in_place { lb } else { la };
        if p == 0 {
            return;
        }
        if p < HOLE_MIN {
            fwd(a, la, b, lb, out);
            count(la + lb);
            return;
        }
        let i = corank(a, la, b, lb, p);
        // Output out[0..p) = the hole; inputs: the run outside, and the
        // start of the run in place, at out[p..): disjoint.
        piece(a, i, b, p - i, out, pred);
        a = a.add(i);
        la -= i;
        b = b.add(p - i);
        lb -= p - i;
        out = out.add(p);
    }
}

/// Mirror of `hole_fwd`: the run in place lies at the start of `out`, the
/// merge goes from the end.
pub unsafe fn hole_bwd<T: Copy + PartialOrd>(
    a: *const T,
    mut la: usize,
    b: *const T,
    mut lb: usize,
    out: *mut T,
    a_in_place: bool,
) {
    let mut pred = None;
    if (if a_in_place { lb } else { la }) >= HOLE_MIN {
        pred = pred_once(a, la, b, lb);
    }
    loop {
        let other = if a_in_place { lb } else { la };
        if other == 0 {
            return;
        }
        if other < HOLE_MIN {
            bwd(a, la, b, lb, out);
            count(la + lb);
            return;
        }
        // Output out[k..m) = the hole at the end; inputs: the run outside,
        // and the end of the run in place, below k: disjoint.
        let k = la + lb - other;
        let i = corank(a, la, b, lb, k);
        piece(a.add(i), la - i, b.add(k - i), lb - (k - i), out.add(k), pred);
        la = i;
        lb = k - i;
    }
}
