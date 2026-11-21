//! The in-place 2-way merge of the merge rounds.

use super::classic::merge2;
use super::gallop::{gallop_back, gallop_forward};

/// Up to this many elements, a run is inserted instead of merged.
const INSERT_MAX: usize = 16;

/// Stable merge of the sorted runs `[start, middle)` and `[middle, end)`, in
/// place: the elements already at their place trimmed (binary searches); a
/// run below the other rotated; a tiny run inserted; balanced runs in the
/// cache merged from both ends into `copy`; skewed runs by galloping;
/// others by the classic merge (shorter run copied out). `only`: the one
/// merge of two runs covering the array.
pub(super) fn merge_in_place<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut Vec<T>,
    start: usize,
    middle: usize,
    end: usize,
    only: bool,
) {
    if start >= middle || middle >= end || arr[middle - 1] <= arr[middle] {
        return;
    }
    // Left elements <= the first right element are already in place (stable:
    // equal elements of the left run stay first).
    let first_right = arr[middle];
    let start = start + arr[start..middle].partition_point(|x| *x <= first_right);
    // Right elements >= the last left element are already in place.
    let last_left = arr[middle - 1];
    let end = middle + arr[middle..end].partition_point(|x| *x < last_left);

    let left = middle - start;
    let right = end - middle;
    if arr[end - 1] < arr[start] {
        // The right run lies entirely below the left one (e.g. a rotated
        // sorted array): a block rotation, no merge.
        arr[start..end].rotate_left(left);
        return;
    }
    if right <= INSERT_MAX {
        // A few right elements: copied out, then merged from the back, each
        // left element moved once (block moves between binary searches);
        // one rotation per inserted element moved the prefix again and
        // again.
        let mut tmp = [arr[middle]; INSERT_MAX];
        tmp[..right].copy_from_slice(&arr[middle..end]);
        let (mut j, mut pos) = (middle, end);
        for &v in tmp[..right].iter().rev() {
            // Left elements above `v` (stable: the equal ones stay first).
            let p = start + arr[start..j].partition_point(|x| *x <= v);
            arr.copy_within(p..j, pos - (j - p));
            pos -= j - p + 1;
            j = p;
            arr[pos] = v;
        }
    } else if left <= INSERT_MAX {
        // Mirror: a few left elements merged from the front, before the
        // equal right ones.
        let mut tmp = [arr[start]; INSERT_MAX];
        tmp[..left].copy_from_slice(&arr[start..middle]);
        let (mut j, mut pos) = (middle, start);
        for &v in tmp[..left].iter() {
            let q = j + arr[j..end].partition_point(|x| *x < v);
            arr.copy_within(j..q, pos);
            pos += q - j;
            j = q;
            arr[pos] = v;
            pos += 1;
        }
    } else {
        // Balanced runs (the shorter at least a quarter) within half the L2:
        // two-ended merge into the scratch, copied back. Skewed runs keep the
        // merge below (shorter run copied, long predictable stretches), and
        // so do merges beyond the cache, where the extra copy costs more.
        let len = end - start;
        let in_cache = len * std::mem::size_of::<T>()
            <= crate::v2::cache::budget(crate::v2::cache::Budget::MergeTwoWay) / 2;
        // The only merge (two runs: a pipe organ), balanced, beyond the
        // cache: the gap merge (shorter run in the buffer, four chains, no
        // copy back) instead of the one-chain merge below. In the cache the
        // two-ended merge into the scratch stays faster; in the rounds of
        // many runs (sawtooth) the gap merge loses.
        if only && !in_cache && 4 * left.min(right) >= len {
            super::gap::merge_two_gap(&mut arr[start..end], middle - start, copy);
            return;
        }
        if 4 * left.min(right) >= len && in_cache {
            if copy.len() < len {
                copy.resize(len, arr[0]);
            }
            // Blocks already ordered against the other run go straight to
            // their place: the left elements above the last right one end the
            // output, the right elements below the first left one start it.
            let src = &arr[start..end];
            let mid = middle - start;
            let tail = mid - src[..mid].partition_point(|x| *x <= src[len - 1]);
            let head = src[mid..].partition_point(|x| *x < src[0]);
            let dst = &mut copy[..len];
            dst[..head].copy_from_slice(&src[mid..mid + head]);
            dst[len - tail..].copy_from_slice(&src[mid - tail..mid]);
            super::merge_balanced(
                &src[..mid - tail],
                &src[mid + head..],
                &mut dst[head..len - tail],
            );
            arr[start..end].copy_from_slice(&copy[..len]);
            return;
        }
        let needed = left.min(right);
        if copy.len() < needed {
            copy.resize(needed, arr[0]);
        }
        if 4 * needed < len {
            if right < left {
                gallop_back(arr, &mut copy[..right], start, middle, end);
            } else {
                gallop_forward(arr, &mut copy[..left], start, middle, end);
            }
            return;
        }
        merge2(arr, copy, start, middle, end);
    }
}
