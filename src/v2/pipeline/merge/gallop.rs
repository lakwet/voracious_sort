//! Galloping merges (TimSort): stretches of one run moved as blocks once
//! longer than `GALLOP_MIN`, found by exponential then binary search.

/// Stretch length from which the skewed merge gallops (TimSort's
/// `MIN_GALLOP` is 7): shorter stretches are moved one element at a time.
pub(super) const GALLOP_MIN: usize = 8;

/// Number of trailing elements of the sorted `a` greater than `v`, by
/// exponential search from the end, then binary search: O(log count).
#[inline]
fn count_back_greater<T: Copy + PartialOrd>(a: &[T], v: T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[n - hi] > v {
        lo = hi;
        hi *= 2;
    }
    let hi = hi.min(n + 1);
    let (s, e) = (n + 1 - hi, n - lo);
    lo + (e - s) - a[s..e].partition_point(|x| *x <= v)
}

/// Number of leading elements of the sorted `a` lower than `v` (exponential
/// then binary search).
#[inline]
fn count_front_lower<T: Copy + PartialOrd>(a: &[T], v: T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[hi - 1] < v {
        lo = hi;
        hi *= 2;
    }
    let e = hi.min(n + 1) - 1;
    lo + a[lo..e].partition_point(|x| *x < v)
}

/// Stable merge of a long left run `[start, middle)` and a short right run
/// `[middle, end)` (copied to `buf`), from the back: before each right
/// element, the left elements greater than it are moved as one block
/// (galloping, as in TimSort), instead of one at a time. A short run
/// merged into a long one (a sawtooth with a short last tooth, a few late
/// elements) moves the long stretches with memmove.
pub(super) fn gallop_back<T: Copy + PartialOrd>(arr: &mut [T], buf: &mut [T], start: usize, middle: usize, end: usize) {
    buf.copy_from_slice(&arr[middle..end]);
    let (mut j, mut i, mut pos) = (middle, buf.len(), end);
    while i > 0 && j > start {
        let v = buf[i - 1];
        // Short stretches element by element; from GALLOP_MIN on, galloping.
        let mut c = 0;
        while c < GALLOP_MIN && c < j - start && arr[j - 1 - c] > v {
            arr[pos - 1 - c] = arr[j - 1 - c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_back_greater(&arr[start..j - c], v);
            arr.copy_within(j - c - more..j - c, pos - c - more);
            c += more;
        }
        pos -= c;
        j -= c;
        pos -= 1;
        arr[pos] = v;
        i -= 1;
    }
    arr[start..start + i].copy_from_slice(&buf[..i]);
}

/// Mirror of `gallop_back`: a short left run (copied to `buf`) merged into
/// a long right run from the front, the right elements lower than each
/// left element moved as one block.
pub(super) fn gallop_forward<T: Copy + PartialOrd>(arr: &mut [T], buf: &mut [T], start: usize, middle: usize, end: usize) {
    buf.copy_from_slice(&arr[start..middle]);
    let m = buf.len();
    let (mut j, mut i, mut pos) = (middle, 0, start);
    while i < m && j < end {
        let v = buf[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < end && arr[j + c] < v {
            arr[pos + c] = arr[j + c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_front_lower(&arr[j + c..end], v);
            arr.copy_within(j + c..j + c + more, pos + c);
            c += more;
        }
        pos += c;
        j += c;
        arr[pos] = v;
        pos += 1;
        i += 1;
    }
    arr[pos..pos + (m - i)].copy_from_slice(&buf[i..]);
}
