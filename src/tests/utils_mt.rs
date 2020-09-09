use super::super::sorts::utils_mt::{aggregate_histograms, compute_offset_mt};

#[test]
fn test_utils_mt_compute_offset_mt() {
    let mut arr: Vec<u32> = vec![0b0000_0111_0000_0000_0000_0000_0000_0000];
    let (offset, raw_offset) = compute_offset_mt(&mut arr, 8);
    assert_eq!(offset, 0);
    assert_eq!(raw_offset, 5);
}

#[test]
fn test_utils_mt_aggregate_histograms() {
    let h1 = vec![0, 1, 2, 3, 4];
    let h2 = vec![0, 1, 2, 3, 4];
    let h3 = vec![0, 1, 2, 3, 4];
    let histograms = vec![h1, h2, h3];

    let g = aggregate_histograms(&histograms);

    assert_eq!(g[0], 0);
    assert_eq!(g[1], 3);
    assert_eq!(g[2], 6);
    assert_eq!(g[3], 9);
    assert_eq!(g[4], 12);
}
