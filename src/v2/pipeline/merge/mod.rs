//! Merging sorted segments (after Verge): rounds of 2-way merges.

mod bidir;
mod classic;
mod gallop;
mod predictable;
mod two_way;

pub(crate) use predictable::predictable_merge;

use bidir::merge_bidir;
use gallop::GALLOP_MIN;
use two_way::merge_in_place;

/// Merges the sorted segments delimited by `separators` (stable), pairs of
/// neighbours per round. From 3 segments, in the cache, the rounds alternate
/// between the array and a buffer (ping-pong: no copy back after each
/// merge); else each merge is done in place.
pub(crate) fn merge_runs<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    if separators.len() <= 2 {
        return;
    }
    // Ping-pong while the array and its buffer fit in a quarter of the L2,
    // from 4096 keys (journal §52).
    let bytes = 2 * arr.len() * std::mem::size_of::<T>();
    if separators.len() >= 4
        && arr.len() >= 4096
        && bytes <= super::super::cache::budget(super::super::cache::Budget::PingPong) / 4
    {
        pingpong(arr, separators);
        return;
    }
    // Several merges: allocate the scratch once (as the original merge).
    let mut copy: Vec<T> = if separators.len() > 3 {
        vec![arr[0]; arr.len() / 2 + 2]
    } else {
        Vec::new()
    };
    while separators.len() > 2 {
        let half = (separators.len() - 1) / 2;
        for i in 0..half {
            let i2 = i * 2;
            merge_in_place(arr, &mut copy, separators[i2], separators[i2 + 1], separators[i2 + 2]);
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
}

/// Ping-pong rounds (see `merge_runs`).
fn pingpong<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    let n = arr.len();
    let mut buf: Vec<T> = vec![arr[0]; n];
    let mut in_buf = false;
    while separators.len() > 2 {
        {
            let (src, dst): (&[T], &mut [T]) = if in_buf { (&buf[..], &mut arr[..]) } else { (&arr[..], &mut buf[..]) };
            let segs = separators.len() - 1;
            let mut i = 0;
            while i + 1 < segs {
                let (s, m, e) = (separators[i], separators[i + 1], separators[i + 2]);
                merge_out(&src[s..m], &src[m..e], &mut dst[s..e]);
                i += 2;
            }
            if i < segs {
                let (s, e) = (separators[i], separators[i + 1]);
                dst[s..e].copy_from_slice(&src[s..e]);
            }
        }
        in_buf = !in_buf;
        let half = (separators.len() - 1) / 2;
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
    if in_buf {
        arr.copy_from_slice(&buf[..n]);
    }
}

/// Stable merge of the sorted runs `a` (first) and `b` into `dst`, out of
/// place: blocks already in order copied whole; balanced runs by the
/// two-ended branch-free merge; skewed runs by stretches, element by
/// element up to `GALLOP_MIN`, then galloping and one block copy.
fn merge_out<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
    if a.is_empty() || b.is_empty() || a[a.len() - 1] <= b[0] {
        dst[..a.len()].copy_from_slice(a);
        dst[a.len()..].copy_from_slice(b);
        return;
    }
    // Head and tail blocks already ordered against the other run.
    let head = b.partition_point(|x| *x < a[0]);
    let tail = a.len() - a.partition_point(|x| *x <= b[b.len() - 1]);
    let len = dst.len();
    dst[..head].copy_from_slice(&b[..head]);
    dst[len - tail..].copy_from_slice(&a[a.len() - tail..]);
    let (a, b) = (&a[..a.len() - tail], &b[head..]);
    let dst = &mut dst[head..len - tail];
    if a.is_empty() || b.is_empty() {
        dst[..a.len()].copy_from_slice(a);
        dst[a.len()..].copy_from_slice(b);
        return;
    }
    // Balanced runs: two-ended branch-free merge (as the in-place version;
    // a branchy merge does not beat it even on predictable interleavings).
    if 4 * a.len().min(b.len()) >= a.len() + b.len() {
        merge_bidir(a, b, dst);
        return;
    }
    // Skewed runs: stretches, galloping from `GALLOP_MIN` on.
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        // Stretch of `a` not above b[j] (stable: a first on ties).
        let v = b[j];
        let mut c = 0;
        while c < GALLOP_MIN && i + c < a.len() && a[i + c] <= v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += a[i + c..].partition_point(|x| *x <= v);
        }
        dst[k..k + c].copy_from_slice(&a[i..i + c]);
        i += c;
        k += c;
        if i == a.len() {
            break;
        }
        // Stretch of `b` below a[i].
        let v = a[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < b.len() && b[j + c] < v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += b[j + c..].partition_point(|x| *x < v);
        }
        dst[k..k + c].copy_from_slice(&b[j..j + c]);
        j += c;
        k += c;
    }
    dst[k..k + (a.len() - i)].copy_from_slice(&a[i..]);
    k += a.len() - i;
    dst[k..].copy_from_slice(&b[j..]);
}
