use super::super::algo::verge_sort_heuristic::{
    backward_orientation, compute_big_enough_run, explore_around,
    explore_backward_asc, explore_backward_desc, explore_backward_plateau,
    explore_forward_asc, explore_forward_desc, explore_forward_plateau,
    explore_simple_forward, forward_orientation, jump,
    verge_sort_preprocessing, BackwardGrowth, ForwardGrowth, Orientation,
};
use super::super::generators::unsigned_u64::*;

fn default_sort(arr: &mut [u64], _radix: usize) {
    arr.sort_unstable();
}

#[test]
fn test_verge_sort_heuristic_compute_big_enough_run() {
    assert_eq!(compute_big_enough_run(100), 15);
    assert_eq!(compute_big_enough_run(129), 18);
    assert_eq!(compute_big_enough_run(1_000), 100);
    assert_eq!(compute_big_enough_run(100_000), 6_020);
    assert_eq!(compute_big_enough_run(1_000_000), 50_171);
}

#[test]
fn test_verge_sort_heuristic_explore_forward_asc() {
    let mut arr = vec![5, 1, 2, 3, 4, 5, 5, 3, 4, 2, 1, 10];
    assert_eq!(explore_forward_asc(&mut arr, 5), 7);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_forward_asc(&mut arr, 5), 6);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 11];
    assert_eq!(explore_forward_asc(&mut arr, 5), 12);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_forward_asc(&mut arr, 5), 11);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_forward_asc(&mut arr, 5), 6);

    let mut arr = vec![9, 9, 8, 8, 7, 7, 6, 6, 6, 5, 4, 3];
    assert_eq!(explore_forward_desc(&mut arr, 0), 12);
}

#[test]
fn test_verge_sort_heuristic_explore_backward_asc() {
    let mut arr = vec![5, 1, 2, 3, 4, 5, 5, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_asc(&mut arr, 5, 0), 1);

    let mut arr = vec![0, 1, 2, 3, 4, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_asc(&mut arr, 5, 0), 0);

    let mut arr = vec![5, 1, 2, 7, 5, 5, 6, 6, 6, 7, 8, 11];
    assert_eq!(explore_backward_asc(&mut arr, 5, 0), 4);

    let mut arr = vec![5, 1, 2, 3, 7, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_backward_asc(&mut arr, 5, 0), 5);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_backward_asc(&mut arr, 0, 0), 0);

    let mut arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 10];
    assert_eq!(explore_backward_asc(&mut arr, 11, 0), 0);
}

#[test]
fn test_verge_sort_heuristic_explore_forward_desc() {
    let mut arr = vec![5, 1, 2, 3, 4, 5, 5, 7, 4, 2, 1, 10];
    assert_eq!(explore_forward_desc(&mut arr, 5), 7);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 7, 3, 4, 2, 1, 10];
    assert_eq!(explore_forward_desc(&mut arr, 5), 6);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 4, 3, 3, 2, 1, 1];
    assert_eq!(explore_forward_desc(&mut arr, 5), 12);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 4, 3, 3, 2, 1, 10];
    assert_eq!(explore_forward_desc(&mut arr, 5), 11);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_forward_desc(&mut arr, 5), 6);

    let mut arr = vec![9, 9, 8, 8, 7, 7, 6, 6, 6, 5, 4, 3];
    assert_eq!(explore_forward_desc(&mut arr, 0), 12);
}

#[test]
fn test_verge_sort_heuristic_explore_backward_desc() {
    let mut arr = vec![0, 8, 7, 6, 6, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_desc(&mut arr, 5, 0), 1);

    let mut arr = vec![8, 8, 7, 6, 6, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_desc(&mut arr, 5, 0), 0);

    let mut arr = vec![5, 1, 2, 4, 6, 5, 6, 6, 6, 7, 8, 11];
    assert_eq!(explore_backward_desc(&mut arr, 5, 0), 4);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_backward_desc(&mut arr, 5, 0), 5);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_backward_desc(&mut arr, 0, 0), 0);

    let mut arr = vec![9, 9, 8, 8, 7, 7, 6, 6, 6, 5, 4, 3];
    assert_eq!(explore_backward_desc(&mut arr, 11, 0), 0);
}

#[test]
fn test_verge_sort_heuristic_explore_backward_plateau() {
    let mut arr = vec![0, 5, 5, 5, 5, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_plateau(&mut arr, 5, 0), 1);

    let mut arr = vec![5, 5, 5, 5, 5, 5, 4, 3, 4, 2, 1, 10];
    assert_eq!(explore_backward_plateau(&mut arr, 5, 0), 0);

    let mut arr = vec![5, 1, 2, 4, 5, 5, 6, 6, 6, 7, 8, 11];
    assert_eq!(explore_backward_plateau(&mut arr, 5, 0), 4);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_backward_plateau(&mut arr, 5, 0), 5);

    let mut arr = vec![5, 1, 2, 3, 6, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_backward_plateau(&mut arr, 5, 0), 5);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_backward_plateau(&mut arr, 0, 0), 0);

    let mut arr = vec![5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
    assert_eq!(explore_backward_plateau(&mut arr, 9, 0), 0);
}

#[test]
fn test_verge_sort_heuristic_explore_forward_plateau() {
    let mut arr = vec![0, 8, 7, 6, 6, 5, 5, 5, 5, 5, 5, 10];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 11);

    let mut arr = vec![0, 8, 7, 6, 6, 5, 5, 5, 5, 5, 5, 5];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 12);

    let mut arr = vec![5, 1, 2, 4, 6, 5, 5, 6, 6, 7, 8, 11];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 7);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 7];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 6);

    let mut arr = vec![5, 1, 2, 3, 4, 5, 4, 6, 6, 7, 8, 7];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 6);

    let mut arr = vec![5, 1, 2, 3, 4, 5];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 6);

    let mut arr = vec![5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
    assert_eq!(explore_forward_plateau(&mut arr, 5), 12);
}

#[test]
fn test_verge_sort_heuristic_jump() {
    let arr_length = 1000;
    let jump_size = 100;
    let position = 0;
    assert_eq!(jump(arr_length, position, jump_size), 100);

    let position = 800;
    assert_eq!(jump(arr_length, position, jump_size), 900);

    let position = 900;
    assert_eq!(jump(arr_length, position, jump_size), 1000);

    let position = 950;
    assert_eq!(jump(arr_length, position, jump_size), 1000);
}

#[test]
fn test_verge_sort_heuristic_backward_orientation() {
    let mut arr = vec![0, 1, 2, 3, 0];
    assert_eq!(backward_orientation(&mut arr, 2), Orientation::IsAsc);

    let mut arr = vec![0, 2, 2, 3, 0];
    assert_eq!(backward_orientation(&mut arr, 2), Orientation::IsPlateau);

    let mut arr = vec![0, 3, 2, 3, 0];
    assert_eq!(backward_orientation(&mut arr, 2), Orientation::IsDesc);

    let mut arr = vec![0, 1, 2, 3, 0];
    assert_eq!(backward_orientation(&mut arr, 0), Orientation::IsNone);
}

#[test]
fn test_verge_sort_heuristic_forward_orientation() {
    let mut arr = vec![0, 1, 2, 3, 0];
    assert_eq!(forward_orientation(&mut arr, 2), Orientation::IsAsc);

    let mut arr = vec![0, 2, 2, 2, 0];
    assert_eq!(forward_orientation(&mut arr, 2), Orientation::IsPlateau);

    let mut arr = vec![0, 3, 2, 1, 0];
    assert_eq!(forward_orientation(&mut arr, 2), Orientation::IsDesc);

    let mut arr = vec![0, 1, 2, 3, 0];
    assert_eq!(forward_orientation(&mut arr, 4), Orientation::IsNone);
}

#[test]
fn test_verge_sort_heuristic_explore_simple_forward() {
    let mut arr: Vec<u64> = vec![0, 0, 0, 1, 1];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsAsc);

    let mut arr: Vec<u64> = vec![0, 0, 0, 0, 0];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsPlateau);

    let mut arr: Vec<u64> = vec![2, 2, 2, 1, 1];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsDesc);

    let mut arr: Vec<u64> = vec![0, 2, 0, 1, 1];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsNone);

    let mut arr: Vec<u64> = vec![0, 1];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsAsc);

    let mut arr: Vec<u64> = vec![0, 0];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsPlateau);

    let mut arr: Vec<u64> = vec![2, 1];
    assert_eq!(explore_simple_forward(&mut arr), Orientation::IsDesc);
}

#[test]
fn test_verge_sort_heuristic_explore_around() {
    // test forward
    let mut arr = vec![0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0];
    let ((bpat, bp1, bp2), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::NN, 0, 0));
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::PA, 3, 5));

    let mut arr = vec![3, 3, 3, 2, 1, 9, 0, 0, 0, 0, 0];
    let ((_, _, _), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::PD, 3, 5));

    let mut arr = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let ((_, _, _), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::PN, 11, 11));

    let mut arr = vec![0, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0];
    let ((_, _, _), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::NA, 0, 5));

    let mut arr = vec![5, 4, 4, 2, 2, 3, 0, 0, 0, 0, 0];
    let ((_, _, _), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::ND, 0, 5));

    let mut arr = vec![5, 4, 4, 2, 2, 0, 0, 0, 0, 0, 0];
    let ((_, _, _), (fpat, fp1, fp2)) = explore_around(&mut arr, 0, 0);
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::ND, 0, 11));

    // test backward
    let mut arr = vec![0, 0, 0, 0, 0, 9, 1, 2, 3, 3, 3];
    let ((bpat, bp1, bp2), (fpat, fp1, fp2)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::AP, 8, 6));
    assert_eq!((fpat, fp1, fp2), (ForwardGrowth::NN, 11, 11));

    let mut arr = vec![0, 0, 0, 0, 0, 0, 5, 4, 3, 3, 3];
    let ((bpat, bp1, bp2), (_, _, _)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::DP, 8, 6));

    let mut arr = vec![3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];
    let ((bpat, bp1, bp2), (_, _, _)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::NP, 0, 0));

    let mut arr = vec![0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3];
    let ((bpat, bp1, bp2), (_, _, _)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::AP, 8, 0));

    let mut arr = vec![0, 0, 0, 0, 0, 0, 1, 2, 5, 4, 3];
    let ((bpat, bp1, bp2), (_, _, _)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::DN, 10, 8));

    let mut arr = vec![0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5];
    let ((bpat, bp1, bp2), (_, _, _)) = explore_around(&mut arr, 10, 0);
    assert_eq!((bpat, bp1, bp2), (BackwardGrowth::AN, 10, 0));
}

#[test]
fn test_verge_sort_heuristic_verge_sort_preprocessing() {
    for _ in 0..100 {
        let mut arr = helper_random_array_uniform_u64(10_000);
        let separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        for k in 0..(separators.len() - 1) {
            for i in separators[k]..(separators[k + 1] - 1) {
                assert!(arr[i] <= arr[i + 1]);
            }
        }

        let mut arr = helper_random_array_sqrt_u64(10_000);
        let separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        for k in 0..(separators.len() - 1) {
            for i in separators[k]..(separators[k + 1] - 1) {
                assert!(arr[i] <= arr[i + 1]);
            }
        }

        let mut arr = helper_random_array_zipf_u64(10_000);
        let separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        for k in 0..(separators.len() - 1) {
            for i in separators[k]..(separators[k + 1] - 1) {
                assert!(arr[i] <= arr[i + 1]);
            }
        }

        let mut arr = helper_random_array_almost_desc_u64(10_000);
        let separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        for k in 0..(separators.len() - 1) {
            for i in separators[k]..(separators[k + 1] - 1) {
                assert!(arr[i] <= arr[i + 1]);
            }
        }

        let mut arr = helper_random_array_almost_asc_u64(10_000);
        let separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);

        for k in 0..(separators.len() - 1) {
            for i in separators[k]..(separators[k + 1] - 1) {
                assert!(arr[i] <= arr[i + 1]);
            }
        }
    }
}
