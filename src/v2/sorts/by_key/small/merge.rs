//! Stable merges of sorted runs of records.

use std::mem::MaybeUninit;

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer the size of the
/// shorter one: each element moved once (the shorter run copied out, the
/// merge writing from the end or the start into the array).
pub(super) fn merge_two<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E) {
    let n = arr.len();
    if m == 0 || m == n || enc(&arr[m - 1]) <= enc(&arr[m]) {
        return;
    }
    let a = arr.as_mut_ptr();
    // SAFETY: the shorter run is moved out to the buffer; the merge then
    // fills the array from the side where the moved-out run was, never
    // overtaking the unread elements of the other run (the write position
    // is always at least as far from that side as the read position), and
    // every element is moved once; nothing is dropped or duplicated. `enc`
    // already ran on every element.
    unsafe {
        if n - m <= m {
            // Right run out; merge from the end.
            let len = n - m;
            let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a.add(m), b, len);
            let (mut i, mut j, mut k) = (m, len, n);
            while i > 0 && j > 0 {
                // Stable: on equal keys the right element goes last.
                let take_left = enc(&*a.add(i - 1)) > enc(&*b.add(j - 1));
                k -= 1;
                if take_left {
                    i -= 1;
                    a.add(k).write(std::ptr::read(a.add(i)));
                } else {
                    j -= 1;
                    a.add(k).write(std::ptr::read(b.add(j)));
                }
            }
            std::ptr::copy_nonoverlapping(b, a.add(k - j), j);
        } else {
            // Left run out; merge from the start.
            let len = m;
            let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a, b, len);
            let (mut i, mut j, mut k) = (0, m, 0);
            while i < len && j < n {
                // Stable: on equal keys the left element goes first.
                let take_right = enc(&*a.add(j)) < enc(&*b.add(i));
                if take_right {
                    a.add(k).write(std::ptr::read(a.add(j)));
                    j += 1;
                } else {
                    a.add(k).write(std::ptr::read(b.add(i)));
                    i += 1;
                }
                k += 1;
            }
            std::ptr::copy_nonoverlapping(b.add(i), a.add(k), len - i);
        }
    }
}

/// Stable merge of the sorted segments `bounds[i]..bounds[i + 1]`, pairs of
/// neighbours per round, alternating between the array and a buffer.
pub(super) fn merge_all<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E) {
    let n = arr.len();
    if bounds.len() <= 2 {
        return;
    }
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    let mut in_buf = false;
    // SAFETY: each round moves every element exactly once from one side to
    // the other (each segment pair merged or copied into the same range);
    // at the end the elements are copied back if they are in the buffer.
    // Nothing is dropped or duplicated; `enc` already ran on every element.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        while bounds.len() > 2 {
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            let mut next = vec![0];
            let mut i = 0;
            while i + 2 < bounds.len() {
                let (s, m, e) = (bounds[i], bounds[i + 1], bounds[i + 2]);
                merge_into(src, dst, s, m, e, enc);
                next.push(e);
                i += 2;
            }
            if i + 1 < bounds.len() {
                let (s, e) = (bounds[i], bounds[i + 1]);
                std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
                next.push(e);
            }
            bounds = next;
            in_buf = !in_buf;
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
}

/// Stable merge of `src[s..m]` and `src[m..e]` into `dst[s..e]`.
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
unsafe fn merge_into<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m > s && m < e && enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let (mut i, mut j, mut k) = (s, m, s);
    while i < m && j < e {
        let right = enc(&*src.add(j)) < enc(&*src.add(i));
        let from = if right { j } else { i };
        dst.add(k).write(std::ptr::read(src.add(from)));
        j += right as usize;
        i += (!right) as usize;
        k += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(i), dst.add(k), m - i);
    k += m - i;
    std::ptr::copy_nonoverlapping(src.add(j), dst.add(k), e - j);
}
