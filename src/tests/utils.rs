use super::super::sorts::utils::{
    compute_max_level, compute_offset, copy_nonoverlapping,
    get_empty_histograms, get_histogram, get_partial_histograms,
    offset_from_bits, only_one_bucket_filled, prefix_sums, Params,
};
use super::super::Radixable;

#[test]
fn test_utils_copy_nonoverlapping() {
    let mut arr1 = vec![1, 1, 1, 1];
    let mut arr2 = vec![0, 0, 0, 0];
    copy_nonoverlapping(&mut arr1, &mut arr2, 3);

    assert_eq!(arr1, vec![1, 1, 1, 1]);
    assert_eq!(arr2, vec![1, 1, 1, 0]);
}

#[test]
fn test_utils_prefix_sums() {
    let histogram = vec![5, 7, 8, 4];
    let (p_sums, heads, tails) = prefix_sums(&histogram);

    let check_p_sums = vec![0, 5, 12, 20, 24];
    let check_heads = vec![0, 5, 12, 20];
    let check_tails = vec![5, 12, 20, 24];

    assert_eq!(p_sums, check_p_sums);
    assert_eq!(heads, check_heads);
    assert_eq!(tails, check_tails);
}

#[test]
fn test_utils_only_one_bucket_filled() {
    let histogram = vec![1, 0, 2, 5, 4, 7, 0, 1];
    assert_eq!(only_one_bucket_filled(&histogram), false);

    let histogram = vec![0, 0, 2, 0, 4, 0, 0, 0];
    assert_eq!(only_one_bucket_filled(&histogram), false);

    let histogram = vec![1, 0, 0, 0, 0, 0];
    assert_eq!(only_one_bucket_filled(&histogram), true);

    let histogram = vec![0, 0, 0, 0, 0, 1];
    assert_eq!(only_one_bucket_filled(&histogram), true);

    let histogram = vec![0, 0, 0, 1, 0, 0];
    assert_eq!(only_one_bucket_filled(&histogram), true);
}

#[test]
fn test_utils_offset_from_bits() {
    let mut arr: Vec<u32> = vec![0];
    let max: u32 = 0b0000_0111_0000_0000_0000_0000_0000_0000;
    let radix = 8;
    let bits = 32;
    let zero: u32 = 0;
    let one: u32 = 1;
    let (offset, raw_offset) =
        offset_from_bits(&mut arr, max, radix, bits, zero, one);
    assert_eq!(offset, 0);
    assert_eq!(raw_offset, 5);
}

#[test]
fn test_utils_compute_offset() {
    let mut arr: Vec<u32> = vec![0b0000_0111_0000_0000_0000_0000_0000_0000];
    let (offset, raw_offset) = compute_offset(&mut arr, 8);
    assert_eq!(offset, 0);
    assert_eq!(raw_offset, 5);

    let mut arr: Vec<char> = vec!['a'];
    // 'a': 0b0000_0000_0000_0000_0000_0000_0110_0001
    let (offset, raw_offset) = compute_offset(&mut arr, 8);
    assert_eq!(offset, 24);
    assert_eq!(raw_offset, 25);
}

#[test]
fn test_utils_compute_max_level() {
    assert_eq!(compute_max_level(32, 0, 8), 4);
    assert_eq!(compute_max_level(32, 7, 8), 4);
    assert_eq!(compute_max_level(32, 8, 8), 3);
    assert_eq!(compute_max_level(32, 9, 8), 3);
    assert_eq!(compute_max_level(32, 16, 8), 2);
    assert_eq!(compute_max_level(32, 20, 8), 2);
    assert_eq!(compute_max_level(32, 24, 8), 1);
    assert_eq!(compute_max_level(32, 30, 8), 1);
    assert_eq!(compute_max_level(32, 32, 8), 0);
}

#[test]
fn test_utils_get_empty_histograms() {
    let h = get_empty_histograms(2, 256);
    assert_eq!(h.len(), 2);
    assert_eq!(h[0].len(), 256);
    assert_eq!(h[1].len(), 256);
    for item in h[0].iter() {
        assert_eq!(*item, 0);
    }
}

#[test]
fn test_utils_get_histogram() {
    let mut v: Vec<u64> =
        vec![0, 1, 2, 3, 1, 1, 2, 1, 3, 0, 1, 2, 1, 0, 3, 1, 2, 3, 1, 0, 1, 0];
    let mut arr = v.as_mut_slice();
    let p = Params::new(0, 2, 62, 1); // level, radix, offset, max_level
    let (mask, shift) = arr[0].get_mask_and_shift(&p);
    let h = get_histogram(&mut arr, &p, mask, shift);

    let check = vec![5, 9, 4, 4];

    assert_eq!(h, check);
}

#[test]
fn test_utils_get_partial_histograms() {
    let p = Params::new(0, 4, 16, 4); // level, radix, offset, max_level
    let mut arr: Vec<u32> = vec![8, 5, 1024, 512, 256, 16_384, 64, 32];
    let histograms = get_partial_histograms(&mut arr, &p, 4);

    assert_eq!(histograms.len(), 4);
    assert_eq!(histograms[0], vec![
        7, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[1], vec![
        5, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[2], vec![
        6, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[3], vec![
        6, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0
    ]);

    let histograms = get_partial_histograms(&mut arr, &p, 2);

    assert_eq!(histograms.len(), 2);
    assert_eq!(histograms[0], vec![
        7, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(histograms[1], vec![
        5, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
}
