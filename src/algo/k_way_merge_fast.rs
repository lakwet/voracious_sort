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
        let needed = left.min(right);
        if copy.len() < needed {
            copy.resize(needed, arr[0]);
        }
        merge2(arr, copy, start, middle, end);
    }
}

/// Whether a branchy merge of the sorted runs `run[..mid]` and `run[mid..]`
/// is predictable, from its first 64 steps: few switches between the runs
/// (long stretches), or a sequence of choices (left or right run) periodic
/// with a period of at most 16 (runs holding the same values alternate). A
/// branch predictor learns such patterns (about 1 ns per element and per
/// merge round measured); on irregular interleavings (random runs) it
/// mispredicts about one step in two (about 3.5-4 ns per element and per
/// round, as much as a whole radix sort).
pub(crate) fn predictable_merge<T: Copy + PartialOrd>(run: &[T], mid: usize) -> bool {
    let (mut i, mut j) = (0, mid);
    let mut bits = 0u64;
    let mut steps = 0;
    while steps < 64 && i < mid && j < run.len() {
        let take_left = run[i] <= run[j];
        bits |= u64::from(take_left) << steps;
        if take_left {
            i += 1;
        } else {
            j += 1;
        }
        steps += 1;
    }
    if steps < 64 {
        return true;
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
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
