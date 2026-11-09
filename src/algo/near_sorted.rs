//! Repair of nearly sorted arrays: a sorted array in which a few elements
//! are out of place (e.g. a few swaps of distant elements).
//!
//! One pass keeps the elements in order, compacted to the front, and sets
//! the misplaced ones aside: an element smaller than the last kept one is
//! set aside, unless the last kept one is the misplaced one (the element is
//! still >= the kept one before it), in which case that last kept one is set
//! aside instead. The few elements set aside are then sorted and inserted
//! back from the end, by binary search and block moves: the whole array is
//! moved once.
//!
//! The number of elements set aside is bounded along the way: beyond the
//! budget the repair gives up, after putting the elements set aside back
//! behind the kept ones (the array is then a permutation of the input whose
//! prefix is sorted), so the caller sorts it as usual. On random data it
//! gives up after a few dozen elements.

use super::verge_sort_fast::explore_forward_asc;

/// Repairs `arr` if it is ascending except for a few misplaced elements.
/// Returns `true` if `arr` is sorted, `false` if the repair gave up (`arr`
/// is then a permutation of the input).
pub fn repair_ascending<T: Copy + PartialOrd>(arr: &mut [T]) -> bool {
    let n = arr.len();
    let mut aside: Vec<T> = Vec::new();
    // The sorted prefix is found by the fast (vectorizable) scan: nothing is
    // written there.
    let mut kept = explore_forward_asc(arr, 0);
    let mut i = kept;
    // Indices of the last 8 elements set aside (ring), for the density test.
    let mut recent = [0usize; 8];
    let mut recent_count = 0;
    while i < n {
        let x = arr[i];
        let keep = if arr[kept - 1] <= x {
            arr[kept] = x;
            kept += 1;
            true
        } else if kept >= 2 && arr[kept - 2] <= x {
            // The last kept element is the misplaced one (too large).
            aside.push(arr[kept - 1]);
            arr[kept - 1] = x;
            true
        } else {
            aside.push(x);
            false
        };
        if aside.len() > recent_count {
            // Misplaced elements of a nearly sorted array are isolated: 8
            // of them within 64 elements means a disordered region.
            recent[recent_count % 8] = i;
            recent_count = aside.len();
            if recent_count >= 8 && i - recent[recent_count % 8] < 64 {
                arr[kept..kept + aside.len()].copy_from_slice(&aside);
                return false;
            }
        }
        if aside.len() > 8 + (i + 1) / 32 {
            // Give up: put the elements set aside back behind the kept ones.
            arr[kept..kept + aside.len()].copy_from_slice(&aside);
            return false;
        }
        i += 1;
        if keep && i < n {
            // The ascending run following a kept element is kept as a block.
            let end = explore_forward_asc(arr, i - 1);
            if end > i {
                if kept != i {
                    arr.copy_within(i..end, kept);
                }
                kept += end - i;
                i = end;
            }
        }
    }
    if aside.is_empty() {
        return true;
    }
    aside.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    // Insert from the end: each element set aside goes after the kept
    // elements <= it; the kept elements greater than it move once.
    let mut end = kept;
    let mut write = n;
    for &x in aside.iter().rev() {
        let p = arr[..end].partition_point(|y| *y <= x);
        let len = end - p;
        arr.copy_within(p..end, write - len);
        write -= len;
        write -= 1;
        arr[write] = x;
        end = p;
    }
    debug_assert_eq!(write, end);
    true
}

/// Repairs `arr` if it is nearly sorted in either direction (a descending
/// array, as seen on a sample of pairs, is reversed first). Returns `true`
/// if `arr` is sorted (ascending), `false` otherwise (`arr` is then a
/// permutation of the input).
pub fn repair_near_sorted<T: Copy + PartialOrd>(arr: &mut [T]) -> bool {
    let n = arr.len();
    if n < 64 {
        return false;
    }
    // Orientation from 32 spread adjacent pairs.
    let step = (n - 1) / 32;
    let (mut asc, mut desc) = (0, 0);
    for k in 0..32 {
        let i = k * step;
        if arr[i] < arr[i + 1] {
            asc += 1;
        } else if arr[i] > arr[i + 1] {
            desc += 1;
        }
    }
    if desc > asc && desc >= 24 {
        arr.reverse();
    } else if asc + (32 - asc - desc) < 24 {
        // Neither clearly ascending nor descending: not nearly sorted.
        return false;
    }
    repair_ascending(arr)
}
