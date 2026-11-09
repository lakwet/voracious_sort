//! Faster variant of `k_way_merge` (same pairwise merge order, same stable
//! result):
//!
//! - each 2-way merge first trims, by binary search, the elements already at
//!   their final place (head of the left run, tail of the right run);
//! - when one of the two remaining runs is tiny, its elements are inserted by
//!   binary search and a block shift (`rotate`, a memmove), instead of moving
//!   the other run one element at a time (e.g. a sorted array with one element
//!   moved: a single shift instead of a full merge).
//!
//! The original module is kept unchanged for the sorts that use it.

use super::k_way_merge::merge2;

/// Up to this many elements, a run is inserted instead of merged.
const INSERT_MAX: usize = 16;

/// Stable merge of the sorted runs `[start, middle)` and `[middle, end)`.
fn merge2_fast<T: Copy + PartialOrd>(
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
        let needed = left.min(right);
        if copy.len() < needed {
            copy.resize(needed, arr[0]);
        }
        merge2(arr, copy, start, middle, end);
    }
}

/// Same as `k_way_merge::k_way_merge` (same result), with the fast 2-way
/// merge.
pub fn k_way_merge_fast<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    if separators.len() <= 2 {
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
            merge2_fast(arr, &mut copy, separators[i2], separators[i2 + 1], separators[i2 + 2]);
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
}
