//! Merges of two runs of records: through a buffer of the shorter one
//! (`merge_two`), the pipe organ (`merge_asc_desc`), the buffers prefaulted.

use std::mem::MaybeUninit;

use super::*;

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer the size of the
/// shorter one: each element moved once (the shorter run copied out, the
/// merge writing from the end or the start into the array).
pub(crate) fn merge_two<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E, branchy: bool) {
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(m.min(arr.len() - m));
    prefault(&mut buf);
    merge_two_with(arr, m, enc, &mut buf, branchy);
}

/// `merge_two` with a buffer of at least the shorter run (capacity).
/// `branchy`: branches on the comparison (faster when the merge is
/// predictable), else selects without branches.
pub(crate) fn merge_two_with<T, E: Fn(&T) -> u128>(
    arr: &mut [T],
    m: usize,
    enc: &E,
    buf: &mut Vec<MaybeUninit<T>>,
    branchy: bool,
) {
    // Balanced runs only: a short run merged into a long one is mostly
    // block moves, which the one-chain merge with its trim does as well.
    if !branchy && 16 * m.min(arr.len() - m) >= arr.len() {
        return merge_two_x(arr, m, enc, buf);
    }
    if branchy {
        merge_two_body::<T, E, true>(arr, m, enc, buf)
    } else {
        merge_two_body::<T, E, false>(arr, m, enc, buf)
    }
}

#[inline(always)]
/// `merge_two_with` once the buffer holds the shorter run.
pub(crate) fn merge_two_body<T, E: Fn(&T) -> u128, const BRANCHY: bool>(
    arr: &mut [T],
    m: usize,
    enc: &E,
    buf: &mut Vec<MaybeUninit<T>>,
) {
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
            debug_assert!(buf.capacity() >= len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a.add(m), b, len);
            // The left keys above every right key move as one block.
            let top = enc(&*b.add(len - 1));
            let p = std::slice::from_raw_parts(a, m).partition_point(|x| enc(x) <= top);
            std::ptr::copy(a.add(p), a.add(p + len), m - p);
            let (mut i, mut j, mut k) = (p, len, p + len);
            while i > 0 && j > 0 {
                // Stable: on equal keys the right element goes last.
                let take_left = enc(&*a.add(i - 1)) > enc(&*b.add(j - 1));
                k -= 1;
                if BRANCHY {
                    if take_left {
                        i -= 1;
                        a.add(k).write(std::ptr::read(a.add(i)));
                    } else {
                        j -= 1;
                        a.add(k).write(std::ptr::read(b.add(j)));
                    }
                    continue;
                }
                let from = pick(take_left, a.add(i - 1) as *const T, b.add(j - 1) as *const T);
                a.add(k).write(std::ptr::read(from));
                i -= take_left as usize;
                j -= (!take_left) as usize;
            }
            std::ptr::copy_nonoverlapping(b, a.add(k - j), j);
        } else {
            // Left run out; merge from the start.
            let len = m;
            debug_assert!(buf.capacity() >= len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a, b, len);
            // The right keys below every left key move as one block.
            let bottom = enc(&*b);
            let q = m + std::slice::from_raw_parts(a.add(m), n - m)
                .partition_point(|x| enc(x) < bottom);
            std::ptr::copy(a.add(m), a, q - m);
            let (mut i, mut j, mut k) = (0, q, q - m);
            while i < len && j < n {
                // Stable: on equal keys the left element goes first.
                let take_right = enc(&*a.add(j)) < enc(&*b.add(i));
                if BRANCHY {
                    if take_right {
                        a.add(k).write(std::ptr::read(a.add(j)));
                        j += 1;
                    } else {
                        a.add(k).write(std::ptr::read(b.add(i)));
                        i += 1;
                    }
                    k += 1;
                    continue;
                }
                let from = pick(take_right, a.add(j) as *const T, b.add(i) as *const T);
                a.add(k).write(std::ptr::read(from));
                j += take_right as usize;
                i += (!take_right) as usize;
                k += 1;
            }
            std::ptr::copy_nonoverlapping(b.add(i), a.add(k), len - i);
        }
    }
}

/// Touches every page of the buffer's capacity before the merges: the page
/// faults of a fresh allocation cost about twice as much when they happen
/// in the middle of a merge (32-byte records at 1M: 7.0 ns for the first
/// round against 3.0 for zeroing + 1.4 for the round).
pub(crate) fn prefault<T>(buf: &mut Vec<MaybeUninit<T>>) {
    // SAFETY: bytes of the spare capacity, written only.
    // WATCH: these bytes are never read, so LLVM may drop the fill when the
    // surrounding code changes (seen once after a refactor of
    // `merge_all_kway`: 0.01 ns per element instead of 0.74, and the faults
    // back in the first merge round; journal §71). If the first merge round
    // slows down again, write one byte per page with `write_volatile`.
    unsafe {
        std::ptr::write_bytes(
            buf.as_mut_ptr() as *mut u8,
            0,
            buf.capacity() * std::mem::size_of::<T>(),
        )
    };
}

/// Stable merge of the ascending run `arr[..m]` and the strictly descending
/// run `arr[m..]`: the descending run copied reversed into a buffer, then
/// the two merged from the back into the array (the write position never
/// passes the unread part of the first run). Branches: pipe organs
/// interleave regularly.
pub(crate) fn merge_asc_desc<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E) {
    let n = arr.len();
    let len = n - m;
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
    prefault(&mut buf);
    // SAFETY: the descending run is moved out reversed; each later write
    // lands on a slot already read (the back of the array first holds the
    // moved-out run, then slots of the first run already taken); every
    // element is moved once more. `enc` already ran on every element.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        for k in 0..len {
            b.add(k).write(std::ptr::read(a.add(n - 1 - k)));
        }
        // The descending run, reversed in the buffer, is the right run of a
        // right gap merge.
        gap_right(a, m, b, len, enc);
    }
}
