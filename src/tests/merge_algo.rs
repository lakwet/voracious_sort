use super::super::algo::k_way_merge::{k_way_merge, merge2};
use super::super::algo::merge_algo::{
    binary_search_max, binary_search_min, smart_k_way_merge, smart_merge2,
    smart_pre_merge2,
};
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::generators::unsigned_u32::*;
use super::super::generators::unsigned_u64::*;

fn default_sort<T: Ord>(arr: &mut [T], _radix: usize) {
    arr.sort_unstable();
}

#[test]
fn test_merge_algo_merge2() {
    for _ in 0..50 {
        let size = 1_000;
        let middle = 200;
        let mut copy = vec![0; (size / 2) + 2];
        let mut arr = helper_random_array_uniform_u64(size);
        let mut check = arr.to_vec();
        let (a, b) = arr.split_at_mut(middle);
        a.sort_unstable();
        b.sort_unstable();

        merge2(&mut arr, &mut copy, 0, middle, size);

        check.sort_unstable();
        assert_eq!(check, arr);
    }
    for _ in 0..50 {
        let size = 1_000;
        let middle = 700;
        let mut copy = vec![0; (size / 2) + 2];
        let mut arr = helper_random_array_uniform_u64(size);
        let mut check = arr.to_vec();
        let (a, b) = arr.split_at_mut(middle);
        a.sort_unstable();
        b.sort_unstable();

        merge2(&mut arr, &mut copy, 0, middle, size);

        check.sort_unstable();
        assert_eq!(check, arr);
    }
}

#[test]
fn test_merge_algo_k_way_merge() {
    let size = 10_000;

    for _ in 0..100 {
        let mut arr = helper_random_array_almost_desc_u64(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(check, arr);

        let mut arr = helper_random_array_uniform_u64(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(check, arr);

        let mut arr = helper_random_array_almost_asc_u64(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(check, arr);

        let mut arr = helper_random_array_sqrt_u64(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(check, arr);

        let mut arr = helper_random_array_zipf_u64(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(arr.as_mut_slice(), 8, &default_sort);
        k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(check, arr);
    }
}

#[test]
fn test_merge_algo_binary_search_min_and_max() {
    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 7, 7, 8, 9, 10, 11];
    assert_eq!(binary_search_max(&mut arr, 1, 9, 6), 5);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 7, 7, 8, 9, 10, 11];
    assert_eq!(binary_search_max(&mut arr, 1, 9, 9), 8);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10, 11];
    assert_eq!(binary_search_max(&mut arr, 1, 9, 6), 6);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10, 11];
    assert_eq!(binary_search_max(&mut arr, 1, 9, 10), 8);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 11, 11];
    assert_eq!(binary_search_max(&mut arr, 1, 10, 11), 9);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10, 11];
    assert_eq!(binary_search_min(&mut arr, 1, 9, 6), 5);

    //                 0 1 2 3 4 5 6 7 8 9  10
    let mut arr = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10, 11];
    assert_eq!(binary_search_min(&mut arr, 1, 9, 1), 1);

    let size = 1000;
    let min = 11;
    let max = 903;
    for _ in 0..100 {
        let mut arr = helper_random_array_uniform_u32(size);
        arr.sort_unstable();
        let value = helper_random_array_uniform_u32(1)[0];
        let position = binary_search_max(&mut arr, min, max, value);
        for i in min..position {
            assert!(arr[i] <= value);
        }
        if position < max - 1 {
            for i in position..max {
                assert!(arr[i] >= value);
            }
        }
    }

    for _ in 0..100 {
        let mut arr = helper_random_array_uniform_u64(size);
        arr.sort_unstable();
        let value = helper_random_array_uniform_u64(1)[0];
        let position = binary_search_min(&mut arr, min, max, value);
        for i in min..position {
            assert!(arr[i] <= value);
        }
        for i in position..max {
            assert!(arr[i] >= value);
        }
    }
}

#[test]
fn test_merge_algo_smart_merge2() {
    //                         0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17
    let mut source = vec![0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 0, 0, 1, 2, 3, 4, 5, 0];
    let mut destination =
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let check = vec![0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 0, 0, 0, 0, 0];

    smart_merge2(&mut source, &mut destination, 5, 10, 11, 17, 2);

    assert_eq!(check, destination);

    let half = 10_000;
    for _ in 0..100 {
        let mut part1 = helper_random_array_uniform_u32(half);
        let mut part2 = helper_random_array_uniform_u32(half);
        part1.sort_unstable();
        part2.sort_unstable();
        part1.extend(part2.iter());
        let mut dest = vec![0; 2 * half];
        let mut check = part1.to_vec();
        smart_merge2(&mut part1, &mut dest, 0, half, half, 2 * half, 0);
        check.sort_unstable();

        assert_eq!(check, dest);
    }
}

#[test]
fn test_merge_algo_smart_pre_merge2() {
    // test when nothing to do
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 10, 10, 11, 11, 2);
    assert_eq!(check, dest);

    // test when one slice's length == 0
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 10, 10, 13, 16, 2);
    assert_eq!(check, dest);

    // test when the other slice's length == 0
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 13, 16, 10, 10, 2);
    assert_eq!(check, dest);

    // test when both slice's length == 1
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 2, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 13, 14, 10, 11, 2);
    assert_eq!(check, dest);

    // test when slices are already sorted
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 1, 2, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 9, 11, 12, 14, 2);
    assert_eq!(check, dest);

    // test when one slice contains the other one
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 0, 1, 2, 2, 3, 3, 4, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 0, 10, 13, 16, 2);
    assert_eq!(check, dest);

    // test when one slice contains the other one, but reversed
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 0, 1, 2, 2, 3, 3, 4, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 13, 16, 0, 10, 2);
    assert_eq!(check, dest);

    // test when one slice partially overlap the other
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 0, 1, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 0, 6, 14, 20, 2);
    assert_eq!(check, dest);

    // test when one slice partially overlap the other, but reversed
    //                    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
    let mut source = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ];
    let mut dest = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let check = vec![
        0, 0, 0, 1, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    smart_pre_merge2(&mut source, &mut dest, 14, 20, 0, 6, 2);
    assert_eq!(check, dest);

    let half = 1_000;
    for _ in 0..10 {
        let mid = half + half / 2;
        let mut arr = helper_random_array_uniform_u32(mid);
        let mut part2 = helper_random_array_uniform_u32(half - half / 2);
        arr.sort_unstable();
        part2.sort_unstable();
        arr.extend(part2.iter());
        let mut check = arr.to_vec();
        check.sort_unstable();
        let mut destination = vec![0; 2 * half];
        smart_pre_merge2(&mut arr, &mut destination, 0, mid, mid, 2 * half, 0);
        assert_eq!(check, destination);
    }
}

#[test]
fn test_merge_algo_smart_k_way_merge() {
    let size = 10_000;
    for _ in 0..100 {
        let mut arr = helper_random_array_almost_asc_u32(size);
        let mut check = arr.to_vec();
        let mut separators =
            verge_sort_preprocessing(&mut arr, 8, &default_sort);
        smart_k_way_merge(&mut arr, &mut separators);
        check.sort_unstable();
        assert_eq!(arr, check);
    }
}
