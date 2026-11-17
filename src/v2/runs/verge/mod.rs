//! Verge sort pre-processing (Verge sort heuristic, faster variant with the
//! same decisions and separators): runs of at least n / log2(n) elements
//! are kept (descending ones reversed), the parts between them sorted by
//! the caller; the returned separators delimit the sorted segments, to be
//! merged.

mod growth;
mod scan;

pub(crate) use scan::{explore_forward_asc, explore_forward_desc};

use growth::*;
use scan::*;

/// Shortest run kept: n / log2(n).
#[inline]
pub(crate) fn compute_big_enough_run(size: usize) -> usize {
    let div = (size as f64).log2();
    ((size as f64) / div) as usize
}

/// `position + part_size`, at most `arr_length`.
#[inline]
fn jump(arr_length: usize, position: usize, part_size: usize) -> usize {
    (position + part_size).min(arr_length)
}

/// Reverses a descending run. The stable variant also restores the original
/// order of each group of equal elements.
#[inline]
fn reverse_run<T: PartialOrd, const STABLE: bool>(run: &mut [T]) {
    run.reverse();
    if STABLE {
        let mut start = 0;
        while start < run.len() {
            let end = explore_forward_plateau(run, start).max(start + 1);
            run[start..end].reverse();
            start = end;
        }
    }
}

/// Keeps the run `[start, end)` as sorted (reversing it if `reverse`), after
/// sending the unsorted part before it to the fallback sort.
#[inline]
fn keep_run<T: Copy + PartialOrd, const STABLE: bool>(
    arr: &mut [T],
    start: usize,
    end: usize,
    reverse: bool,
    last_sorted: usize,
    separators: &mut Vec<usize>,
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize),
) {
    if start > last_sorted {
        fallback_sort(&mut arr[last_sorted..start], radix);
        separators.push(start);
    }
    separators.push(end);
    if reverse {
        reverse_run::<T, STABLE>(&mut arr[start..end]);
    }
}

/// Same decisions as `verge_sort_heuristic::handle_part`.
#[inline]
#[allow(clippy::too_many_arguments)]
fn handle_part<T: Copy + PartialOrd, const STABLE: bool>(
    arr: &mut [T],
    b_pattern: BackwardGrowth,
    f_pattern: ForwardGrowth,
    bp2: usize,
    bp1: usize,
    fp1: usize,
    fp2: usize,
    last_sorted: usize,
    big_enough: usize,
    separators: &mut Vec<usize>,
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize),
) -> (usize, usize) {
    let mut keep = |start: usize, end: usize, reverse: bool| {
        keep_run::<T, STABLE>(
            arr, start, end, reverse, last_sorted, separators, radix, fallback_sort,
        )
    };
    match get_growth_pattern(b_pattern, f_pattern) {
        GrowthPattern::AscThenDesc => {
            if fp1 - bp2 >= big_enough {
                keep(bp2, fp1, false);
                (fp2, fp1)
            } else if fp2 - bp1 >= big_enough {
                keep(bp1, fp2, true);
                (fp2, fp2)
            } else {
                (fp2, last_sorted)
            }
        },
        GrowthPattern::DescThenAsc => {
            if fp2 - bp1 >= big_enough {
                keep(bp1, fp2, false);
                (fp2, fp2)
            } else if fp1 - bp2 >= big_enough {
                keep(bp2, fp1, true);
                (fp2, fp1)
            } else {
                (fp2, last_sorted)
            }
        },
        GrowthPattern::AscOnly | GrowthPattern::PlateauOnly => {
            if fp2 - bp2 >= big_enough {
                keep(bp2, fp2, false);
                (fp2, fp2)
            } else {
                (fp2, last_sorted)
            }
        },
        GrowthPattern::DescOnly => {
            if fp2 - bp2 >= big_enough {
                keep(bp2, fp2, true);
                (fp2, fp2)
            } else {
                (fp2, last_sorted)
            }
        },
        GrowthPattern::Neither => {
            panic!("[Verge sort heuristic] Bad implementation.")
        },
    }
}

/// Same as `verge_sort_heuristic::verge_sort_preprocessing` (same
/// separators), with the fast explorations; `STABLE = false` skips the
/// restoration of equal elements order in reversed runs.
pub fn verge_sort_preprocessing_fast<T, const STABLE: bool>(
    arr: &mut [T],
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize),
) -> Vec<usize>
where
    T: Copy + PartialOrd,
{
    let size = arr.len();
    let big_enough = compute_big_enough_run(size);
    let mut last_sorted = 0;
    let mut separators: Vec<usize> = vec![0];
    let mut position = jump(size, 0, big_enough);

    while position < size {
        let ((b_pattern, bp1, bp2), (f_pattern, fp1, fp2)) =
            explore_around(arr, position, last_sorted);

        let (jump_position, new_last_sorted) = handle_part::<T, STABLE>(
            arr,
            b_pattern,
            f_pattern,
            bp2,
            bp1,
            fp1,
            fp2,
            last_sorted,
            big_enough,
            &mut separators,
            radix,
            fallback_sort,
        );

        last_sorted = new_last_sorted;
        position = jump(size, jump_position, big_enough);
    }

    if last_sorted < size {
        fallback_sort(&mut arr[last_sorted..size], radix);
        separators.push(size);
    }

    separators
}

