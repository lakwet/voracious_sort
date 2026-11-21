//! Unit tests of `v2/runs/verge/scan.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// The block scans against a plain one, from every start, on short
/// arrays of few values (many runs and plateaus of every length).
#[test]
fn against_plain_scans() {
    let mut r = Rng(23);
    for &n in [1usize, 2, 5, 16, 17, 40, 100].iter() {
        let a: Vec<u8> = (0..n).map(|_| (r.next() % 3) as u8).collect();
        let fwd = |s: usize, ok: &dyn Fn(u8, u8) -> bool| {
            let mut i = s;
            while i + 1 < n && ok(a[i], a[i + 1]) {
                i += 1;
            }
            i + 1
        };
        let back = |s: usize, min: usize, ok: &dyn Fn(u8, u8) -> bool| {
            let mut i = s;
            while i > min && ok(a[i - 1], a[i]) {
                i -= 1;
            }
            i
        };
        for s in 0..n {
            assert_eq!(explore_forward_asc(&a, s), fwd(s, &|x, y| x <= y), "asc n={} s={}", n, s);
            assert_eq!(explore_forward_desc(&a, s), fwd(s, &|x, y| x >= y), "desc n={} s={}", n, s);
            assert_eq!(
                explore_forward_plateau(&a, s),
                fwd(s, &|x, y| x == y),
                "eq n={} s={}",
                n,
                s
            );
            for min in 0..=s {
                assert_eq!(explore_backward_asc(&a, s, min), back(s, min, &|x, y| x <= y));
                assert_eq!(explore_backward_desc(&a, s, min), back(s, min, &|x, y| x >= y));
                assert_eq!(explore_backward_plateau(&a, s, min), back(s, min, &|x, y| x == y));
            }
        }
    }
}
