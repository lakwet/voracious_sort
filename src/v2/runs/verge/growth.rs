//! Growth patterns of the Verge pre-processing: from a probe position, the
//! run around it is explored backward and forward (ascending, descending or
//! plateau of equal keys), and the pair of directions found tells which
//! run it is.

use super::scan::*;

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
