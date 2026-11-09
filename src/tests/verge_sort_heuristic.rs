use super::super::algo::verge_sort_heuristic::{
    backward_orientation, compute_big_enough_run, explore_around,
    explore_backward_asc, explore_backward_desc, explore_backward_plateau,
    explore_forward_asc, explore_forward_desc, explore_forward_plateau,
    explore_simple_forward, forward_orientation, jump,
    verge_sort_preprocessing, BackwardGrowth, ForwardGrowth, Orientation,
};
use super::super::generators::unsigned_u64::*;

fn default_sort(arr: &mut [u64], _radix: usize) { arr.sort_unstable(); }

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

/// The fast Verge variant must take exactly the same decisions as the
/// original one: same explorations, same separators, same array (stable).
#[test]
fn test_verge_fast_same_as_original() {
    use super::super::algo::verge_sort_fast as fast;
    use super::super::algo::verge_sort_heuristic as orig;
    use super::super::generators::float_32::generators_f32;

    // Explorations on every start position of patterned arrays.
    let patterns: Vec<Vec<u64>> = vec![
        (0..300).collect(),
        (0..300).rev().collect(),
        vec![7; 300],
        (0..300).map(|i| (i / 37) as u64).collect(),
        (0..300).map(|i| if i % 50 < 25 { i as u64 } else { 1000 - i as u64 }).collect(),
        (0..300).map(|i| ((i * 7919) % 13) as u64).collect(),
        (0..300).map(|i| if i == 150 { 0 } else { i as u64 }).collect(),
    ];
    for p in patterns.iter() {
        let mut a = p.clone();
        for s in 0..a.len() {
            assert_eq!(fast::explore_forward_asc(&a, s), orig::explore_forward_asc(&mut a, s));
            assert_eq!(fast::explore_forward_desc(&a, s), orig::explore_forward_desc(&mut a, s));
            assert_eq!(fast::explore_forward_plateau(&a, s), orig::explore_forward_plateau(&mut a, s));
            for m in [0, s / 2, s].iter() {
                assert_eq!(fast::explore_backward_asc(&a, s, *m), orig::explore_backward_asc(&mut a, s, *m));
                assert_eq!(fast::explore_backward_desc(&a, s, *m), orig::explore_backward_desc(&mut a, s, *m));
                assert_eq!(fast::explore_backward_plateau(&a, s, *m), orig::explore_backward_plateau(&mut a, s, *m));
            }
        }
    }

    // Whole pre-processing on every f32 distribution and several sizes.
    let fallback = |a: &mut [f32], _r: usize| a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    for size in [200usize, 1_000, 10_000, 100_000].iter() {
        for (gen, name) in generators_f32().iter() {
            let input = gen(*size);
            let mut a = input.clone();
            let mut b = input.clone();
            let mut c = input.clone();
            let sa = orig::verge_sort_preprocessing(&mut a, 8, &fallback);
            let sb = fast::verge_sort_preprocessing_fast::<_, true>(&mut b, 8, &fallback);
            let sc = fast::verge_sort_preprocessing_fast::<_, false>(&mut c, 8, &fallback);
            assert_eq!(sa, sb, "{} {} separators (stable)", name, size);
            assert_eq!(sa, sc, "{} {} separators (unstable)", name, size);
            let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
            assert!(bits(&a) == bits(&b), "{} {} array (stable)", name, size);
            // Unstable: same multiset, and each segment sorted.
            for w in sc.windows(2) {
                assert!(c[w[0]..w[1]].windows(2).all(|p| p[0] <= p[1]), "{} {} segment", name, size);
            }
            let mut ma = bits(&a);
            let mut mc = bits(&c);
            ma.sort_unstable();
            mc.sort_unstable();
            assert!(ma == mc, "{} {} multiset", name, size);
        }
    }
}

/// The fast k-way merge must give exactly the result of the original one
/// (stable merge: equal keys keep their order).
#[test]
fn test_k_way_merge_fast_same_as_original() {
    use super::super::algo::k_way_merge::k_way_merge;
    use super::super::algo::k_way_merge_fast::k_way_merge_fast;

    #[derive(Clone, Copy, Debug)]
    struct S {
        k: u32,
        id: u32,
    }
    impl PartialEq for S {
        fn eq(&self, o: &Self) -> bool {
            self.k == o.k
        }
    }
    impl PartialOrd for S {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            self.k.partial_cmp(&o.k)
        }
    }

    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for case in 0..3000 {
        let n = 2 + (next() % 400) as usize;
        let range = [2u64, 10, 1000, 1 << 30][case % 4];
        // Random run boundaries, including tiny runs (insert path).
        let mut seps = vec![0usize];
        while *seps.last().unwrap() < n {
            let len = match next() % 4 {
                0 => 1 + (next() % 3) as usize,
                1 => 1 + (next() % 20) as usize,
                _ => 1 + (next() % 200) as usize,
            };
            seps.push((seps.last().unwrap() + len).min(n));
        }
        let mut arr: Vec<S> = (0..n).map(|i| S { k: (next() % range) as u32, id: i as u32 }).collect();
        for w in seps.windows(2) {
            arr[w[0]..w[1]].sort_by(|a, b| a.k.cmp(&b.k));
        }
        // Special shapes: sorted + moved element (Front / Middle).
        if case % 10 == 0 {
            arr = (0..n).map(|i| S { k: i as u32, id: i as u32 }).collect();
            arr[n - 1].k = [0u32, (n / 2) as u32][case % 20 / 10];
            seps = vec![0, n - 1, n];
        }
        let mut a = arr.clone();
        let mut b = arr.clone();
        k_way_merge(&mut a, &mut seps.clone());
        k_way_merge_fast(&mut b, &mut seps.clone());
        let ka: Vec<(u32, u32)> = a.iter().map(|s| (s.k, s.id)).collect();
        let kb: Vec<(u32, u32)> = b.iter().map(|s| (s.k, s.id)).collect();
        assert!(ka == kb, "case {} n {} seps {:?}", case, n, seps);
    }
}
