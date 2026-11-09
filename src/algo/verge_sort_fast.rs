//! Faster variant of the Verge sort pre-processing heuristic
//! (`verge_sort_heuristic`), same decisions and same separators:
//!
//! - run explorations check blocks of 32 pairs without branches (the
//!   comparisons are combined with `&`, so the loop can be vectorized), and
//!   only the failing block is searched for the exact position;
//! - `STABLE = false`: a descending run is just reversed (the stable variant
//!   also restores the order of equal elements, which needs one more pass).
//!
//! The original module is kept unchanged for the sorts that use it (stable
//! sorts and reference ports such as thiel_sort).

use super::verge_sort_heuristic::{
    compute_big_enough_run, get_growth_pattern, jump, BackwardGrowth, ForwardGrowth,
    GrowthPattern, Orientation,
};

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

#[inline]
fn backward_orientation<T: PartialOrd>(arr: &[T], position: usize) -> Orientation {
    if position == 0 {
        Orientation::IsNone
    } else if arr[position - 1] < arr[position] {
        Orientation::IsAsc
    } else if arr[position - 1] > arr[position] {
        Orientation::IsDesc
    } else {
        Orientation::IsPlateau
    }
}

#[inline]
fn forward_orientation<T: PartialOrd>(arr: &[T], position: usize) -> Orientation {
    if position >= arr.len() - 1 {
        Orientation::IsNone
    } else if arr[position] < arr[position + 1] {
        Orientation::IsAsc
    } else if arr[position] > arr[position + 1] {
        Orientation::IsDesc
    } else {
        Orientation::IsPlateau
    }
}

/// Same as `verge_sort_heuristic::explore_around`, with the fast scans.
#[inline]
pub fn explore_around<T: PartialOrd>(
    arr: &[T],
    position: usize,
    min_boundary: usize,
) -> ((BackwardGrowth, usize, usize), (ForwardGrowth, usize, usize)) {
    let (b_pattern, bp1, bp2) = match backward_orientation(arr, position) {
        Orientation::IsAsc => {
            let p = explore_backward_asc(arr, position, min_boundary);
            (BackwardGrowth::AN, position, p)
        },
        Orientation::IsDesc => {
            let p = explore_backward_desc(arr, position, min_boundary);
            (BackwardGrowth::DN, position, p)
        },
        Orientation::IsPlateau => {
            let bp1 = explore_backward_plateau(arr, position, min_boundary);
            if bp1 == min_boundary {
                (BackwardGrowth::NP, bp1, bp1)
            } else {
                match backward_orientation(arr, bp1) {
                    Orientation::IsAsc => {
                        let bp2 = explore_backward_asc(arr, bp1, min_boundary);
                        (BackwardGrowth::AP, bp1, bp2)
                    },
                    Orientation::IsDesc => {
                        let bp2 = explore_backward_desc(arr, bp1, min_boundary);
                        (BackwardGrowth::DP, bp1, bp2)
                    },
                    Orientation::IsPlateau => {
                        panic!("[Verge sort heuristic] Bad implementation.")
                    },
                    Orientation::IsNone => (BackwardGrowth::NP, bp1, bp1),
                }
            }
        },
        Orientation::IsNone => (BackwardGrowth::NN, position, position),
    };
    let (f_pattern, fp1, fp2) = match forward_orientation(arr, position) {
        Orientation::IsAsc => {
            let p = explore_forward_asc(arr, position);
            (ForwardGrowth::NA, position, p)
        },
        Orientation::IsDesc => {
            let p = explore_forward_desc(arr, position);
            (ForwardGrowth::ND, position, p)
        },
        Orientation::IsPlateau => {
            let fp1 = explore_forward_plateau(arr, position);
            let new_position = if fp1 == position { fp1 } else { fp1 - 1 };
            match forward_orientation(arr, new_position) {
                Orientation::IsAsc => {
                    let fp2 = explore_forward_asc(arr, fp1);
                    (ForwardGrowth::PA, fp1, fp2)
                },
                Orientation::IsDesc => {
                    let fp2 = explore_forward_desc(arr, fp1);
                    (ForwardGrowth::PD, fp1, fp2)
                },
                Orientation::IsPlateau => {
                    panic!("[Verge sort heuristic] Bad implementation.")
                },
                Orientation::IsNone => (ForwardGrowth::PN, fp1, fp1),
            }
        },
        Orientation::IsNone => (ForwardGrowth::NN, position + 1, position + 1),
    };

    ((b_pattern, bp1, bp2), (f_pattern, fp1, fp2))
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
