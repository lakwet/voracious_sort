//! Unit tests of `v2/inspect/presorted.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Constant, ascending and descending arrays are sorted (`Ok`); any
/// other comes back permuted (`Err`) with a true ascending prefix.
#[test]
fn presorted_shapes() {
    for_each_case::<u64>(|shape, a| {
        if a.is_empty() {
            return;
        }
        let what = format!("{} n={}", shape, a.len());
        let mut got = a.clone();
        match presorted(&mut got) {
            Ok(()) => assert!(sorted_by(&got, |k| k), "{}: Ok but not sorted", what),
            Err(known) => {
                let p = known.ascending_prefix.min(got.len());
                assert!(sorted_by(&got[..p], |k| k), "{}: wrong ascending prefix", what);
                assert!(!sorted_by(&a, |k| k), "{}: sorted input refused", what);
            },
        }
        assert!(same_values(&got, &a), "{}: values changed", what);
        // (The byte-swapped way of the patterns is not in order.)
        let desc = sorted_by(&a, std::cmp::Reverse);
        if sorted_by(&a, |k| k) || desc {
            assert!(presorted(&mut a.clone()).is_ok(), "{}: not found presorted", what);
        }
    });
}

/// A sorted array (ascending or reversed) with a few elements moved is
/// repaired; a random one is left a permutation.
#[test]
fn nearly_sorted() {
    let mut r = Rng(5);
    for &n in [64usize, 100, 1000, 4096, 20_000].iter() {
        for &reversed in [false, true].iter() {
            for &moved in [1usize, 3].iter() {
                let mut a: Vec<u64> = (0..n as u64).map(|i| i * 3).collect();
                for _ in 0..moved {
                    let (i, j) = ((r.next() % n as u64) as usize, (r.next() % n as u64) as usize);
                    a.swap(i, j);
                }
                if reversed {
                    a.reverse();
                }
                let mut got = a.clone();
                let (sorted, _) = repair_nearly_sorted(&mut got, Unsorted::default());
                assert!(
                    sorted && sorted_by(&got, |k| k),
                    "n={} moved {} reversed {}",
                    n,
                    moved,
                    reversed
                );
            }
        }
        let a: Vec<u64> = (0..n).map(|_| r.next()).collect();
        let mut got = a.clone();
        let (sorted, _) = repair_nearly_sorted(&mut got, Unsorted::default());
        assert!(!sorted || sorted_by(&got, |k| k), "random n={}", n);
        assert!(same_values(&got, &a), "random n={}: values changed", n);
    }
}

/// The 32 probes count every spread pair once.
#[test]
fn probes() {
    let a: Vec<u32> = (0..1000).collect();
    let p = probe_pairs(&a);
    assert_eq!((p.ascending, p.descending, p.equal()), (32, 0, 0));
    let d: Vec<u32> = (0..1000).rev().collect();
    assert_eq!(probe_pairs(&d).descending, 32);
    assert_eq!(probe_pairs(&[7u32; 100]).equal(), 32);
}
