//! The in-place 2-way merge of the merge rounds.

use super::bidir::merge_bidir;
use super::classic::merge2;
use super::gallop::{gallop_back, gallop_forward};

/// Up to this many elements, a run is inserted instead of merged.
const INSERT_MAX: usize = 16;

/// Stable merge of the sorted runs `[start, middle)` and `[middle, end)`, in
/// place: the elements already at their place trimmed (binary searches); a
/// run below the other rotated; a tiny run inserted; balanced runs in the
/// cache merged from both ends into `copy`; skewed runs by galloping;
/// others by the classic merge (shorter run copied out).
pub(super) fn merge_in_place<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut Vec<T>,
    start: usize,
    middle: usize,
    end: usize,
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
        // Insert the right elements, in order, after the equal left ones.
        let mut lo = start;
        for idx in middle..end {
            let value = arr[idx];
            let pos = lo + arr[lo..idx].partition_point(|x| *x <= value);
            arr[pos..=idx].rotate_right(1);
            lo = pos + 1;
        }
    } else if left <= INSERT_MAX {
        // Insert the left elements, last first, before the equal right ones.
        let mut hi = end;
        for idx in (start..middle).rev() {
            let value = arr[idx];
            let pos = idx + 1 + arr[idx + 1..hi].partition_point(|x| *x < value);
            arr[idx..pos].rotate_left(1);
            hi = pos - 1;
        }
    } else {
        // Balanced runs (the shorter at least a quarter) within half the L2:
        // two-ended merge into the scratch, copied back. Skewed runs keep the
        // merge below (shorter run copied, long predictable stretches), and
        // so do merges beyond the cache, where the extra copy costs more.
        let len = end - start;
        if 4 * left.min(right) >= len
            && len * std::mem::size_of::<T>() <= super::super::super::cache::budget(super::super::super::cache::Budget::MergeTwoWay) / 2
        {
            if copy.len() < len {
                copy.resize(len, arr[0]);
            }
            // Blocks already ordered against the other run go straight to
            // their place: the left elements above the last right one end the
            // output, the right elements below the first left one start it
            // (u16 sawtooth with two teeth at 100k keys: 0.70 to 0.47 ns
            // per key).
            let src = &arr[start..end];
            let mid = middle - start;
            let tail = mid - src[..mid].partition_point(|x| *x <= src[len - 1]);
            let head = src[mid..].partition_point(|x| *x < src[0]);
            let dst = &mut copy[..len];
            dst[..head].copy_from_slice(&src[mid..mid + head]);
            dst[len - tail..].copy_from_slice(&src[mid - tail..mid]);
            merge_bidir(&src[..mid - tail], &src[mid + head..], &mut dst[head..len - tail]);
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
