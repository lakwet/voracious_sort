//! Verge on the records: long runs kept, the gaps sorted, all merged.

use super::super::pipeline::verge::compute_big_enough_run;
use super::merge::{merge_all, merge_two};
use super::sort_encoded;




/// Verge on the elements: runs of at least n / log2(n) keys (ascending, or
/// strictly descending and then reversed: still stable) are kept, the gaps
/// between them sorted, and all the segments merged stably. Returns false
/// (nothing done) without such a run.
pub(super) fn verge<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    let n = arr.len();
    let long = compute_big_enough_run(n).max(2);
    // Runs: (start, end, descending).
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    let mut i = 0;
    while i + 1 < n {
        let start = i;
        let desc = enc(&arr[i + 1]) < enc(&arr[i]);
        let mut j = i + 1;
        if desc {
            while j < n && enc(&arr[j]) < enc(&arr[j - 1]) {
                j += 1;
            }
        } else {
            while j < n && enc(&arr[j]) >= enc(&arr[j - 1]) {
                j += 1;
            }
        }
        if j - start >= long {
            runs.push((start, j, desc));
        }
        i = j;
    }
    if runs.is_empty() {
        return false;
    }
    // Two long runs interleaving unpredictably (sorted random sources):
    // merging costs more than sorting by key (the merge rule of the radix
    // path, on the first 64 steps of their merge).
    if runs.len() >= 2 && !predictable(arr, runs[0], runs[1], enc) {
        return false;
    }
    // Segments: the runs and the gaps between them, in order.
    let mut bounds = vec![0];
    for &(s, e, desc) in runs.iter() {
        let at = *bounds.last().unwrap();
        if s > at {
            sort_encoded(&mut arr[at..s], enc);
            bounds.push(s);
        }
        if desc {
            arr[s..e].reverse();
        }
        bounds.push(e);
    }
    let at = *bounds.last().unwrap();
    if at < n {
        sort_encoded(&mut arr[at..], enc);
        bounds.push(n);
    }
    if bounds.len() == 3 {
        merge_two(arr, bounds[1], enc, true);
    } else {
        merge_all(arr, &bounds, enc, true);
    }
    true
}

/// Whether merging two runs would be predictable (same rule as
/// `predictable_merge`): on the first 64 steps of their merge (runs read in
/// increasing order), at most 16 switches between the two runs, or a
/// pattern periodic with a period of at most 16 (pipe organs alternate one
/// by one).
fn predictable<T, E: Fn(&T) -> u128>(arr: &[T], a: (usize, usize, bool), b: (usize, usize, bool), enc: &E) -> bool {
    let at = |r: (usize, usize, bool), i: usize| if r.2 { enc(&arr[r.1 - 1 - i]) } else { enc(&arr[r.0 + i]) };
    let (la, lb) = (a.1 - a.0, b.1 - b.0);
    let (mut i, mut j, mut bits) = (0, 0, 0u64);
    for step in 0..64 {
        if i == la || j == lb {
            return true;
        }
        let left = at(a, i) <= at(b, j);
        bits |= (left as u64) << step;
        if left {
            i += 1;
        } else {
            j += 1;
        }
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}
