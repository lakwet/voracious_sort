//! Run explorations: blocks of 32 pairs compared without branches (the
//! comparisons combined with `&`, so the loop vectorizes), only the failing
//! block searched for the exact position.

use std::convert::TryInto;

const BLOCK: usize = 32;

/// First index `j > start` such that `ok(arr[j - 1], arr[j])` is false, or
/// `arr.len()`.
#[inline(always)]
fn scan_forward<T, F: Fn(&T, &T) -> bool>(arr: &[T], start: usize, ok: F) -> usize {
    let n = arr.len();
    if start + 1 >= n {
        return n;
    }
    let mut i = start;
    while i + BLOCK < n {
        let block: &[T; BLOCK + 1] = arr[i..i + BLOCK + 1].try_into().unwrap();
        let mut good = true;
        for k in 0..BLOCK {
            good &= ok(&block[k], &block[k + 1]);
        }
        if !good {
            break;
        }
        i += BLOCK;
    }
    while i + 1 < n {
        if ok(&arr[i], &arr[i + 1]) {
            i += 1;
        } else {
            return i + 1;
        }
    }
    n
}

/// Smallest `j` in `min..=start` such that `ok(arr[k - 1], arr[k])` holds for
/// every `k` in `j + 1..=start`.
#[inline(always)]
fn scan_backward<T, F: Fn(&T, &T) -> bool>(arr: &[T], start: usize, min: usize, ok: F) -> usize {
    let mut i = start;
    while i >= min + BLOCK {
        let block: &[T; BLOCK + 1] = arr[i - BLOCK..i + 1].try_into().unwrap();
        let mut good = true;
        for k in 0..BLOCK {
            good &= ok(&block[k], &block[k + 1]);
        }
        if !good {
            break;
        }
        i -= BLOCK;
    }
    while i > min {
        if ok(&arr[i - 1], &arr[i]) {
            i -= 1;
        } else {
            break;
        }
    }
    i
}

#[inline]
pub fn explore_forward_asc<T: PartialOrd>(arr: &[T], start: usize) -> usize {
    scan_forward(arr, start, |a, b| a <= b)
}

#[inline]
pub fn explore_forward_desc<T: PartialOrd>(arr: &[T], start: usize) -> usize {
    scan_forward(arr, start, |a, b| a >= b)
}

#[inline]
pub fn explore_forward_plateau<T: PartialOrd>(arr: &[T], start: usize) -> usize {
    scan_forward(arr, start, |a, b| a == b)
}

#[inline]
pub fn explore_backward_asc<T: PartialOrd>(arr: &[T], start: usize, min: usize) -> usize {
    scan_backward(arr, start, min, |a, b| a <= b)
}

#[inline]
pub fn explore_backward_desc<T: PartialOrd>(arr: &[T], start: usize, min: usize) -> usize {
    scan_backward(arr, start, min, |a, b| a >= b)
}

#[inline]
pub fn explore_backward_plateau<T: PartialOrd>(arr: &[T], start: usize, min: usize) -> usize {
    scan_backward(arr, start, min, |a, b| a == b)
}

