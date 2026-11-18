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
/// End (exclusive) of the ascending, descending or constant run from
/// `start`; the backward scans: its start, not below `min`.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    /// The block scans against a plain one, from every start, on short
    /// arrays of few values (many runs and plateaus of every length).
    #[test]
    fn against_plain_scans() {
        let mut r = Rng(23);
        for &n in [1usize, 2, 5, 16, 17, 40, 100].iter() {
            let a: Vec<u8> = (0..n).map(|_| (r.next() % 3) as u8).collect();
            let fwd = |s: usize, ok: &dyn Fn(u8, u8) -> bool| {
                let mut i = s;
                while i + 1 < n && ok(a[i], a[i + 1]) {
                    i += 1;
                }
                i + 1
            };
            let back = |s: usize, min: usize, ok: &dyn Fn(u8, u8) -> bool| {
                let mut i = s;
                while i > min && ok(a[i - 1], a[i]) {
                    i -= 1;
                }
                i
            };
            for s in 0..n {
                assert_eq!(
                    explore_forward_asc(&a, s),
                    fwd(s, &|x, y| x <= y),
                    "asc n={} s={}",
                    n,
                    s
                );
                assert_eq!(
                    explore_forward_desc(&a, s),
                    fwd(s, &|x, y| x >= y),
                    "desc n={} s={}",
                    n,
                    s
                );
                assert_eq!(
                    explore_forward_plateau(&a, s),
                    fwd(s, &|x, y| x == y),
                    "eq n={} s={}",
                    n,
                    s
                );
                for min in 0..=s {
                    assert_eq!(explore_backward_asc(&a, s, min), back(s, min, &|x, y| x <= y));
                    assert_eq!(explore_backward_desc(&a, s, min), back(s, min, &|x, y| x >= y));
                    assert_eq!(explore_backward_plateau(&a, s, min), back(s, min, &|x, y| x == y));
                }
            }
        }
    }
}
