//! Unit tests of `v2/decide/path.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Random keys hold no run Verge would keep; a sorted array (or one
/// with a long run in the middle) does.
#[test]
fn skip_verge() {
    let mut r = Rng(53);
    for &n in [4096usize, 100_000].iter() {
        let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
        assert!(can_skip_verge(&random), "random n={}", n);
        let mut run = random.clone();
        run[n / 4..3 * n / 4].sort_unstable();
        assert!(!can_skip_verge(&run), "run n={}", n);
        let asc: Vec<u64> = (0..n as u64).collect();
        assert!(!can_skip_verge(&asc), "sorted n={}", n);
    }
}

/// Two long ascending runs of random keys start the array: their merge
/// is unpredictable. Two runs one after the other (or a short prefix)
/// are not reported.
#[test]
fn random_runs() {
    let mut r = Rng(59);
    let n = 100_000;
    let mut a: Vec<u64> = (0..n).map(|_| r.next()).collect();
    a[..n / 2].sort_unstable();
    a[n / 2..].sort_unstable();
    assert!(starts_with_random_runs(&a, n / 2));
    assert!(!starts_with_random_runs(&a, 10));
    let apart: Vec<u64> =
        (0..n as u64).map(|i| if i < n as u64 / 2 { i } else { i + n as u64 }).collect();
    assert!(!starts_with_random_runs(&apart, n / 2));
}
