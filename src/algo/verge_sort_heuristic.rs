use super::super::{RadixKey, Radixable};

#[derive(PartialEq, Debug)]
pub enum Orientation {
    IsAsc,
    IsDesc,
    IsPlateau,
    IsNone,
}

#[derive(PartialEq, Debug)]
pub enum GrowthPattern {
    AscThenDesc,
    DescThenAsc,
    AscOnly,
    DescOnly,
    PlateauOnly,
    Neither,
}

// A: Ascending, D: Descending, P: Plateau, N: None
#[derive(PartialEq, Debug)]
pub enum BackwardGrowth {
    AP,
    DP,
    AN,
    DN,
    NN,
    NP,
}

// A: Ascending, D: Descending, P: Plateau, N: None
#[derive(PartialEq, Debug)]
pub enum ForwardGrowth {
    PA,
    PD,
    NA,
    ND,
    NN,
    PN,
}

#[inline]
pub fn compute_big_enough_run(size: usize) -> usize {
    let div = (size as f64).log2();
    ((size as f64) / div) as usize
}

#[inline]
pub fn explore_forward_asc<T>(arr: &mut [T], start: usize) -> usize
where
    T: PartialOrd,
{
    if start == arr.len() - 1 {
        return arr.len();
    }

    let mut i = start;

    let quantity = arr.len() - i;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start + q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j + 1);
                let a2 = arr.get_unchecked(j + 2);
                let a3 = arr.get_unchecked(j + 3);
                let a4 = arr.get_unchecked(j + 4);

                // using a function as a parameter for the compare
                // drastically impact performance
                let b0 = a0 <= a1;
                let b1 = a1 <= a2 && b0;
                let b2 = a2 <= a3 && b1;
                let b3 = a3 <= a4 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j + 4;
                } else if b1 {
                    return j + 3;
                } else if b0 {
                    return j + 2;
                } else {
                    return j + 1;
                }
            }
        }
    }

    i = if quotient > 10 { start + ((quotient - 1) * 4) - 1 } else { i };

    while i < arr.len() - 1 {
        if arr[i] <= arr[i + 1] {
            i += 1;
        } else {
            return i + 1;
        }
    }

    i + 1
}

#[inline]
pub fn explore_forward_desc<T>(arr: &mut [T], start: usize) -> usize
where
    T: PartialOrd,
{
    if start == arr.len() - 1 {
        return arr.len();
    }

    let mut i = start;

    let quantity = arr.len() - i;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start + q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j + 1);
                let a2 = arr.get_unchecked(j + 2);
                let a3 = arr.get_unchecked(j + 3);
                let a4 = arr.get_unchecked(j + 4);

                let b0 = a0 >= a1;
                let b1 = a1 >= a2 && b0;
                let b2 = a2 >= a3 && b1;
                let b3 = a3 >= a4 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j + 4;
                } else if b1 {
                    return j + 3;
                } else if b0 {
                    return j + 2;
                } else {
                    return j + 1;
                }
            }
        }
    }

    i = if quotient > 10 { start + ((quotient - 1) * 4) - 1 } else { i };

    while i < arr.len() - 1 {
        if arr[i] >= arr[i + 1] {
            i += 1;
        } else {
            return i + 1;
        }
    }

    i + 1
}

#[inline]
pub fn explore_backward_asc<T: PartialOrd>(
    arr: &mut [T],
    start: usize,
    min_boundary: usize,
) -> usize {
    let mut i = start;

    let quantity = i - min_boundary;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start - q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j - 1);
                let a2 = arr.get_unchecked(j - 2);
                let a3 = arr.get_unchecked(j - 3);
                let a4 = arr.get_unchecked(j - 4);

                let b0 = a1 <= a0;
                let b1 = a2 <= a1 && b0;
                let b2 = a3 <= a2 && b1;
                let b3 = a4 <= a3 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j - 3;
                } else if b1 {
                    return j - 2;
                } else if b0 {
                    return j - 1;
                } else {
                    return j;
                }
            }
        }
    }

    i = if quotient > 10 { start - ((quotient - 1) * 4) + 1 } else { i };

    while i > min_boundary {
        if arr[i - 1] <= arr[i] {
            i -= 1;
        } else {
            break;
        }
    }

    i
}

#[inline]
pub fn explore_backward_desc<T: PartialOrd>(
    arr: &mut [T],
    start: usize,
    min_boundary: usize,
) -> usize {
    let mut i = start;

    let quantity = i - min_boundary;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start - q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j - 1);
                let a2 = arr.get_unchecked(j - 2);
                let a3 = arr.get_unchecked(j - 3);
                let a4 = arr.get_unchecked(j - 4);

                let b0 = a1 >= a0;
                let b1 = a2 >= a1 && b0;
                let b2 = a3 >= a2 && b1;
                let b3 = a4 >= a3 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j - 3;
                } else if b1 {
                    return j - 2;
                } else if b0 {
                    return j - 1;
                } else {
                    return j;
                }
            }
        }
    }

    i = if quotient > 10 { start - ((quotient - 1) * 4) + 1 } else { i };

    while i > min_boundary {
        if arr[i - 1] >= arr[i] {
            i -= 1;
        } else {
            break;
        }
    }

    i
}

#[inline]
pub fn explore_backward_plateau<T: PartialOrd>(
    arr: &mut [T],
    start: usize,
    min_boundary: usize,
) -> usize {
    let mut i = start;

    let quantity = start - min_boundary;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start - q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j - 1);
                let a2 = arr.get_unchecked(j - 2);
                let a3 = arr.get_unchecked(j - 3);
                let a4 = arr.get_unchecked(j - 4);

                let b0 = a0 == a1;
                let b1 = a1 == a2 && b0;
                let b2 = a2 == a3 && b1;
                let b3 = a3 == a4 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j - 3;
                } else if b1 {
                    return j - 2;
                } else if b0 {
                    return j - 1;
                } else {
                    return j;
                }
            }
        }
    }

    i = if quotient > 10 { start - ((quotient - 1) * 4) + 1 } else { i };

    while i > min_boundary {
        if arr[i - 1] == arr[i] {
            i -= 1;
        } else {
            break;
        }
    }

    i
}

#[inline]
pub fn explore_forward_plateau<T: PartialOrd>(
    arr: &mut [T],
    start: usize,
) -> usize {
    if start == arr.len() - 1 {
        return arr.len();
    }

    let mut i = start;

    let quantity = arr.len() - i;
    let quotient = quantity / 4;

    if quotient > 10 {
        for q in 0..(quotient - 1) {
            let j = start + q * 4;
            unsafe {
                let a0 = arr.get_unchecked(j);
                let a1 = arr.get_unchecked(j + 1);
                let a2 = arr.get_unchecked(j + 2);
                let a3 = arr.get_unchecked(j + 3);
                let a4 = arr.get_unchecked(j + 4);

                let b0 = a0 == a1;
                let b1 = a1 == a2 && b0;
                let b2 = a2 == a3 && b1;
                let b3 = a3 == a4 && b2;

                if b3 {
                    // nothing
                } else if b2 {
                    return j + 4;
                } else if b1 {
                    return j + 3;
                } else if b0 {
                    return j + 2;
                } else {
                    return j + 1;
                }
            }
        }
    }

    i = if quotient > 10 { start + ((quotient - 1) * 4) - 1 } else { i };

    while i < arr.len() - 1 {
        if arr[i] == arr[i + 1] {
            i += 1;
        } else {
            return i + 1;
        }
    }

    i + 1
}

#[inline]
pub fn jump(arr_length: usize, position: usize, part_size: usize) -> usize {
    let target = position + part_size;

    if target >= arr_length {
        arr_length
    } else {
        target
    }
}

#[inline]
pub fn backward_orientation<T: PartialOrd>(
    arr: &mut [T],
    position: usize,
) -> Orientation {
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
pub fn forward_orientation<T: PartialOrd>(
    arr: &mut [T],
    position: usize,
) -> Orientation {
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

#[inline]
pub fn get_growth_pattern(
    b: BackwardGrowth,
    f: ForwardGrowth,
) -> GrowthPattern {
    match (b, f) {
        (BackwardGrowth::AP, ForwardGrowth::PD) => GrowthPattern::AscThenDesc,
        (BackwardGrowth::AP, ForwardGrowth::ND) => GrowthPattern::AscThenDesc,
        (BackwardGrowth::AN, ForwardGrowth::PD) => GrowthPattern::AscThenDesc,
        (BackwardGrowth::AN, ForwardGrowth::ND) => GrowthPattern::AscThenDesc,

        (BackwardGrowth::DP, ForwardGrowth::PA) => GrowthPattern::DescThenAsc,
        (BackwardGrowth::DP, ForwardGrowth::NA) => GrowthPattern::DescThenAsc,
        (BackwardGrowth::DN, ForwardGrowth::PA) => GrowthPattern::DescThenAsc,
        (BackwardGrowth::DN, ForwardGrowth::NA) => GrowthPattern::DescThenAsc,

        (BackwardGrowth::AP, ForwardGrowth::NN) => GrowthPattern::AscOnly,
        (BackwardGrowth::AP, ForwardGrowth::PA) => GrowthPattern::AscOnly,
        (BackwardGrowth::AP, ForwardGrowth::NA) => GrowthPattern::AscOnly,
        (BackwardGrowth::AP, ForwardGrowth::PN) => GrowthPattern::AscOnly,
        (BackwardGrowth::AN, ForwardGrowth::PA) => GrowthPattern::AscOnly,
        (BackwardGrowth::AN, ForwardGrowth::NA) => GrowthPattern::AscOnly,
        (BackwardGrowth::AN, ForwardGrowth::NN) => GrowthPattern::AscOnly,
        (BackwardGrowth::AN, ForwardGrowth::PN) => GrowthPattern::AscOnly,
        (BackwardGrowth::NN, ForwardGrowth::PA) => GrowthPattern::AscOnly,
        (BackwardGrowth::NN, ForwardGrowth::NA) => GrowthPattern::AscOnly,
        (BackwardGrowth::NP, ForwardGrowth::PA) => GrowthPattern::AscOnly,
        (BackwardGrowth::NP, ForwardGrowth::NA) => GrowthPattern::AscOnly,

        (BackwardGrowth::DP, ForwardGrowth::PD) => GrowthPattern::DescOnly,
        (BackwardGrowth::DP, ForwardGrowth::ND) => GrowthPattern::DescOnly,
        (BackwardGrowth::DP, ForwardGrowth::NN) => GrowthPattern::DescOnly,
        (BackwardGrowth::DP, ForwardGrowth::PN) => GrowthPattern::DescOnly,
        (BackwardGrowth::DN, ForwardGrowth::PD) => GrowthPattern::DescOnly,
        (BackwardGrowth::DN, ForwardGrowth::ND) => GrowthPattern::DescOnly,
        (BackwardGrowth::DN, ForwardGrowth::NN) => GrowthPattern::DescOnly,
        (BackwardGrowth::DN, ForwardGrowth::PN) => GrowthPattern::DescOnly,
        (BackwardGrowth::NN, ForwardGrowth::PD) => GrowthPattern::DescOnly,
        (BackwardGrowth::NN, ForwardGrowth::ND) => GrowthPattern::DescOnly,
        (BackwardGrowth::NP, ForwardGrowth::PD) => GrowthPattern::DescOnly,
        (BackwardGrowth::NP, ForwardGrowth::ND) => GrowthPattern::DescOnly,

        (BackwardGrowth::NN, ForwardGrowth::PN) => GrowthPattern::PlateauOnly,
        (BackwardGrowth::NP, ForwardGrowth::NN) => GrowthPattern::PlateauOnly,
        (BackwardGrowth::NP, ForwardGrowth::PN) => GrowthPattern::PlateauOnly,

        (BackwardGrowth::NN, ForwardGrowth::NN) => GrowthPattern::Neither,
    }
}

// This function is not for the Verge sort pre processing heuristic, but for
// an other heuristic in the Voracious Sort
#[inline]
pub fn explore_simple_forward<T: PartialOrd>(arr: &mut [T]) -> Orientation {
    match forward_orientation(arr, 0) {
        Orientation::IsAsc => {
            let p = explore_forward_asc(arr, 0);
            if p == arr.len() {
                Orientation::IsAsc
            } else {
                Orientation::IsNone
            }
        },
        Orientation::IsDesc => {
            let p = explore_forward_desc(arr, 0);
            if p == arr.len() {
                Orientation::IsDesc
            } else {
                Orientation::IsNone
            }
        },
        Orientation::IsPlateau => {
            let fp = explore_forward_plateau(arr, 0);
            match forward_orientation(arr, fp - 1) {
                Orientation::IsAsc => {
                    let p = explore_forward_asc(arr, fp);
                    if p == arr.len() {
                        Orientation::IsAsc
                    } else {
                        Orientation::IsNone
                    }
                },
                Orientation::IsDesc => {
                    let p = explore_forward_desc(arr, fp);
                    if p == arr.len() {
                        Orientation::IsDesc
                    } else {
                        Orientation::IsNone
                    }
                },
                Orientation::IsPlateau => {
                    panic!("[Verge sort heuristic] Bad implementation.")
                },
                Orientation::IsNone => Orientation::IsPlateau,
            }
        },
        Orientation::IsNone => {
            panic!("[Verge sort heuristic] Bad implementation.")
        },
    }
}

#[inline]
pub fn explore_around<T: PartialOrd>(
    arr: &mut [T],
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

#[inline]
fn handle_asc_then_desc<T, K>(
    arr: &mut [T],
    bp2: usize,
    bp1: usize,
    fp1: usize,
    fp2: usize,
    last_sorted: usize,
    big_enough: usize,
    separators: &mut Vec<usize>,
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize) -> (),
) -> (usize, usize)
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    //     bp2 bp1  position  fp1 fp2
    //        /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\
    //       /                   \
    //      /                     \
    if fp1 - bp2 >= big_enough {
        if bp2 - last_sorted > 0 {
            fallback_sort(&mut arr[last_sorted..bp2], radix);
            separators.push(bp2);
        }
        separators.push(fp1);
        (fp2, fp1)
    } else if fp2 - bp1 >= big_enough {
        if bp1 - last_sorted > 0 {
            fallback_sort(&mut arr[last_sorted..bp1], radix);
            separators.push(bp1);
        }
        separators.push(fp2);
        arr[bp1..fp2].reverse();
        (fp2, fp2)
    } else {
        (fp2, last_sorted)
    }
}

#[inline]
fn handle_desc_then_asc<T, K>(
    arr: &mut [T],
    bp2: usize,
    bp1: usize,
    fp1: usize,
    fp2: usize,
    last_sorted: usize,
    big_enough: usize,
    separators: &mut Vec<usize>,
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize) -> (),
) -> (usize, usize)
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    //     bp2 bp1   position   fp1 fp2
    //      \  |        |        |  /
    //       \ |        |        | /
    //        \|________|________|/
    if fp2 - bp1 >= big_enough {
        if bp1 - last_sorted > 0 {
            fallback_sort(&mut arr[last_sorted..bp1], radix);
            separators.push(bp1);
        }
        separators.push(fp2);
        (fp2, fp2)
    } else if fp1 - bp2 >= big_enough {
        if bp2 - last_sorted > 0 {
            fallback_sort(&mut arr[last_sorted..bp2], radix);
            separators.push(bp2);
        }
        separators.push(fp1);
        arr[bp2..fp1].reverse();
        (fp2, fp1)
    } else {
        (fp2, last_sorted)
    }
}

#[inline]
fn handle_part<T, K>(
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
    fallback_sort: &dyn Fn(&mut [T], usize) -> (),
) -> (usize, usize)
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    match get_growth_pattern(b_pattern, f_pattern) {
        GrowthPattern::AscThenDesc => handle_asc_then_desc(
            arr,
            bp2,
            bp1,
            fp1,
            fp2,
            last_sorted,
            big_enough,
            separators,
            radix,
            fallback_sort,
        ),
        GrowthPattern::DescThenAsc => handle_desc_then_asc(
            arr,
            bp2,
            bp1,
            fp1,
            fp2,
            last_sorted,
            big_enough,
            separators,
            radix,
            fallback_sort,
        ),
        GrowthPattern::AscOnly | GrowthPattern::PlateauOnly => {
            //     bp2 bp1   position   fp1 fp2
            //       | |        |        | /
            //       | |________|________|/
            //       | /
            //       |/
            if fp2 - bp2 >= big_enough {
                if bp2 - last_sorted > 0 {
                    fallback_sort(&mut arr[last_sorted..bp2], radix);
                    separators.push(bp2);
                }
                separators.push(fp2);
                (fp2, fp2)
            } else {
                (fp2, last_sorted)
            }
        },
        GrowthPattern::DescOnly => {
            //     bp2 bp1   position  fp1 fp2
            //      \ |        |        |  |
            //       \|________|________|  |
            //                           \ |
            //                            \|
            if fp2 - bp2 >= big_enough {
                if bp2 - last_sorted > 0 {
                    fallback_sort(&mut arr[last_sorted..bp2], radix);
                    separators.push(bp2);
                }
                separators.push(fp2);
                arr[bp2..fp2].reverse();
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

pub fn verge_sort_preprocessing<T, K>(
    arr: &mut [T],
    radix: usize,
    fallback_sort: &dyn Fn(&mut [T], usize) -> (),
) -> Vec<usize>
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let size = arr.len();
    let big_enough = compute_big_enough_run(size);
    let mut last_sorted = 0;
    let mut separators: Vec<usize> = vec![0];
    let mut position = jump(size, 0, big_enough);

    while position < size {
        let ((b_pattern, bp1, bp2), (f_pattern, fp1, fp2)) =
            explore_around(arr, position, last_sorted);

        let (jump_position, new_last_sorted) = handle_part(
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
