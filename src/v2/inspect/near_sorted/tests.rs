//! Unit tests of `v2/inspect/near_sorted.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Ascending arrays with a few misplaced elements (at the start, the
/// end, anywhere, equal keys) are repaired; when the repair gives up,
/// the array is a permutation of the input.
#[test]
fn repair() {
    let mut r = Rng(3);
    for &n in [0usize, 1, 2, 10, 100, 1000, 20_000].iter() {
        let base: Vec<u32> = (0..n as u32).map(|i| i / 3).collect();
        assert!(repair_ascending(&mut base.clone()), "sorted n={}", n);
        if n < 2 {
            continue;
        }
        for case in 0..4 {
            let mut a = base.clone();
            match case {
                0 => a.swap(0, n - 1),
                1 => a[n - 1] = 0,
                2 => a[0] = u32::MAX,
                _ => {
                    let i = (r.next() % n as u64) as usize;
                    a[i] = (r.next() % (n as u64 / 3 + 1)) as u32;
                },
            }
            let mut got = a.clone();
            if repair_ascending(&mut got) {
                assert!(sorted_by(&got, |k| k), "case {} n={}: not sorted", case, n);
            }
            assert!(same_values(&got, &a), "case {} n={}: values changed", case, n);
            // Prefix known sorted: same outcome.
            let mut from = a.clone();
            if repair_ascending_from(&mut from, 1) {
                assert!(sorted_by(&from, |k| k), "case {} n={} from 1", case, n);
            }
            assert!(same_values(&from, &a));
        }
        let a: Vec<u32> = (0..n).map(|_| r.next() as u32).collect();
        let mut got = a.clone();
        if repair_ascending(&mut got) {
            assert!(sorted_by(&got, |k| k), "random n={}", n);
        }
        assert!(same_values(&got, &a), "random n={}: values changed", n);
    }
}
