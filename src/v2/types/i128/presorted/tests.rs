//! Unit tests of `v2/types/i128/presorted.rs` (a child module: sees its private items).

use super::forward_asc;

#[test]
fn forward_asc_matches_scan() {
    for n in 0..40usize {
        for bad in 0..n {
            let mut a: Vec<u128> = (0..n as u128).collect();
            a[bad] = 1000;
            if bad + 1 < n {
                let want = crate::v2::runs::verge::explore_forward_asc(&a, 0);
                assert_eq!(forward_asc(&a), want, "n {} bad {}", n, bad);
            }
        }
        let a: Vec<u128> = (0..n as u128).collect();
        assert_eq!(forward_asc(&a), n);
    }
}
