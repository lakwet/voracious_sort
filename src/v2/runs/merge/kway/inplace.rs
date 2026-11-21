//! In-place merge of two neighbouring runs through a buffer of the shorter
//! one (at most n/2), v2 `two_way::merge_in_place` and `gallop.rs`:
//!
//! ```text
//! already ordered            → nothing
//! elements already in place  → trimmed (binary searches at both ends)
//! right run below the left   → blocks swapped through the buffer
//! a run of ≤ 16              → inserted from its side (block moves)
//! skewed (`kernel::skewed`)  → galloping merge from the shorter run's side
//! balanced                   → shorter run copied out, hole merge (4 chains)
//! ```
//! Difference with v2: balanced runs in the cache are not merged into a
//! scratch of the full length and copied back (that needs a buffer of n).

use super::kernel::{count, hole_bwd, hole_fwd, skewed, GALLOP_MIN};
use core::ptr;

/// A right run of at most this many elements is copied to the stack and
/// inserted from the back (no buffer needed).
const INSERT_MAX: usize = 16;

/// Stable merge of `v[start..mid]` and `v[mid..end]`; `buf` holds at least
/// `min(mid - start, end - mid)` elements.
pub fn merge_two<T: Copy + PartialOrd>(
    v: &mut [T],
    buf: *mut T,
    start: usize,
    mid: usize,
    end: usize,
) {
    if start >= mid || mid >= end || v[mid - 1] <= v[mid] {
        return;
    }
    let first_right = v[mid];
    let start = start + v[start..mid].partition_point(|x| *x <= first_right);
    let last_left = v[mid - 1];
    let end = mid + v[mid..end].partition_point(|x| *x < last_left);
    let (left, right) = (mid - start, end - mid);
    if v[end - 1] < v[start] {
        unsafe { swap_blocks(v, buf, start, mid, end) };
        return;
    }
    if right <= INSERT_MAX {
        let mut tmp = [v[mid]; INSERT_MAX];
        tmp[..right].copy_from_slice(&v[mid..end]);
        let (mut j, mut pos) = (mid, end);
        for &x in tmp[..right].iter().rev() {
            let p = start + v[start..j].partition_point(|y| *y <= x);
            v.copy_within(p..j, pos - (j - p));
            count(j - p + 1);
            pos -= j - p + 1;
            j = p;
            v[pos] = x;
        }
        return;
    }
    if left <= INSERT_MAX {
        let mut tmp = [v[start]; INSERT_MAX];
        tmp[..left].copy_from_slice(&v[start..mid]);
        let (mut j, mut pos) = (mid, start);
        for &x in tmp[..left].iter() {
            let q = j + v[j..end].partition_point(|y| *y < x);
            v.copy_within(j..q, pos);
            count(q - j + 1);
            pos += q - j;
            j = q;
            v[pos] = x;
            pos += 1;
        }
        return;
    }
    let len = end - start;
    let short = left.min(right);
    let p = v.as_mut_ptr();
    unsafe {
        if skewed(short, len) {
            if right < left {
                gallop_back(v, buf, start, mid, end);
            } else {
                gallop_forward(v, buf, start, mid, end);
            }
            return;
        }
        count(short);
        if left <= right {
            ptr::copy_nonoverlapping(p.add(start), buf, left);
            hole_fwd(buf, left, p.add(mid), right, p.add(start), false);
        } else {
            ptr::copy_nonoverlapping(p.add(mid), buf, right);
            hole_bwd(p.add(start), left, buf, right, p.add(start), true);
        }
    }
}

/// `v[s..mid]` and `v[mid..e]` exchanged through `buf` (the shorter one
/// out, the longer moved, the shorter back): plain copies, unlike a
/// rotation in place.
pub unsafe fn swap_blocks<T: Copy>(v: &mut [T], buf: *mut T, s: usize, mid: usize, e: usize) {
    let p = v.as_mut_ptr();
    let (la, lb) = (mid - s, e - mid);
    if la <= lb {
        ptr::copy_nonoverlapping(p.add(s), buf, la);
        ptr::copy(p.add(mid), p.add(s), lb);
        ptr::copy_nonoverlapping(buf, p.add(s + lb), la);
    } else {
        ptr::copy_nonoverlapping(p.add(mid), buf, lb);
        ptr::copy(p.add(s), p.add(s + lb), la);
        ptr::copy_nonoverlapping(buf, p.add(s), lb);
    }
    count(e - s + la.min(lb));
}

/// Number of trailing elements of the sorted `a` greater than `x`
/// (exponential then binary search).
fn count_back_greater<T: PartialOrd>(a: &[T], x: &T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[n - hi] > *x {
        lo = hi;
        hi *= 2;
    }
    let hi = hi.min(n + 1);
    let (s, e) = (n + 1 - hi, n - lo);
    lo + (e - s) - a[s..e].partition_point(|y| y <= x)
}

/// Number of leading elements of the sorted `a` lower than `x`.
fn count_front_lower<T: PartialOrd>(a: &[T], x: &T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[hi - 1] < *x {
        lo = hi;
        hi *= 2;
    }
    let e = hi.min(n + 1) - 1;
    lo + a[lo..e].partition_point(|y| y < x)
}

/// A short right run (copied out) merged into a long left one from the
/// back, the left stretches above each right element moved as blocks.
unsafe fn gallop_back<T: Copy + PartialOrd>(
    v: &mut [T],
    buf: *mut T,
    start: usize,
    mid: usize,
    end: usize,
) {
    let m = end - mid;
    ptr::copy_nonoverlapping(v.as_ptr().add(mid), buf, m);
    count(m);
    gallop_back_from(v, std::slice::from_raw_parts(buf, m), start, mid, end);
}

/// `gallop_back` with the right run already out, in `b` (`v[mid..end]` free).
pub fn gallop_back_from<T: Copy + PartialOrd>(
    v: &mut [T],
    b: &[T],
    start: usize,
    mid: usize,
    end: usize,
) {
    let m = end - mid;
    let (mut j, mut i, mut pos) = (mid, m, end);
    let mut moved = 0;
    while i > 0 && j > start {
        let x = b[i - 1];
        let mut c = 0;
        while c < GALLOP_MIN && c < j - start && v[j - 1 - c] > x {
            v[pos - 1 - c] = v[j - 1 - c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_back_greater(&v[start..j - c], &x);
            v.copy_within(j - c - more..j - c, pos - c - more);
            c += more;
        }
        moved += c + 1;
        pos -= c + 1;
        j -= c;
        v[pos] = x;
        i -= 1;
    }
    v[start..start + i].copy_from_slice(&b[..i]);
    count(moved + i);
}

/// Mirror: a short left run (copied out) merged into a long right one from
/// the front.
unsafe fn gallop_forward<T: Copy + PartialOrd>(
    v: &mut [T],
    buf: *mut T,
    start: usize,
    mid: usize,
    end: usize,
) {
    let m = mid - start;
    ptr::copy_nonoverlapping(v.as_ptr().add(start), buf, m);
    count(m);
    gallop_forward_from(v, std::slice::from_raw_parts(buf, m), start, mid, end);
}

/// `gallop_forward` with the left run already out, in `b` (`v[start..mid]`
/// free).
pub fn gallop_forward_from<T: Copy + PartialOrd>(
    v: &mut [T],
    b: &[T],
    start: usize,
    mid: usize,
    end: usize,
) {
    let m = mid - start;
    let (mut j, mut i, mut pos) = (mid, 0, start);
    let mut moved = 0;
    while i < m && j < end {
        let x = b[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < end && v[j + c] < x {
            v[pos + c] = v[j + c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_front_lower(&v[j + c..end], &x);
            v.copy_within(j + c..j + c + more, pos + c);
            c += more;
        }
        moved += c + 1;
        pos += c;
        j += c;
        v[pos] = x;
        pos += 1;
        i += 1;
    }
    v[pos..pos + (m - i)].copy_from_slice(&b[i..]);
    count(moved + m - i);
}
