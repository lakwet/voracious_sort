//! Unit tests of `v1/sorts/awardofthesky_sort.rs` (a child module: sees its private items).

use super::*;

#[test]
fn test_utils_get_full_histograms_except_last_chunk() {
    let p = Params::new(0, 4, 16, 4); // level, radix, offset, max_level
    let mut arr: Vec<u32> = vec![8, 5, 1024, 512, 256, 16_384, 64, 32];
    let histograms = get_full_histograms_except_last_chunk(&mut arr, &p);

    assert_eq!(histograms.len(), 3);
    assert_eq!(histograms[0], vec![7, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(histograms[1], vec![5, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(histograms[2], vec![6, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
