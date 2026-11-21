//! Unit tests of `v2/decide/shape.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Sorted or reversed keys, a sawtooth and jittered timestamps are
/// ordered (locally, globally, or both); random keys are not.
#[test]
fn orders() {
    let mut r = Rng(43);
    let n = 10_000u64;
    let asc: Vec<u64> = (0..n).collect();
    let desc: Vec<u64> = (0..n).rev().collect();
    let saw: Vec<u64> = (0..n).map(|i| i % 500).collect();
    let jitter: Vec<u64> = (0..n).map(|i| i * 100 + r.next() % 1000).collect();
    let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
    for a in [&asc, &desc, &saw].iter() {
        assert!(locally_ordered(a));
    }
    for a in [&asc, &desc, &jitter].iter() {
        assert!(globally_ordered(a));
    }
    assert!(!locally_ordered(&random) && !globally_ordered(&random));
}

/// Uniform keys are not clustered; many tight groups far apart are (a
/// sample of 256 keys from 200 groups: most neighbours in different
/// groups, more than 5 % in the same one).
#[test]
fn clusters() {
    let mut r = Rng(47);
    let mut uniform: Vec<u64> = (0..256).map(|_| r.next()).collect();
    uniform.sort_unstable();
    assert!(!clustered(&uniform));
    let centres: Vec<u64> = (0..200).map(|_| r.next() & !0xFFFF).collect();
    let mut groups: Vec<u64> =
        (0..256).map(|_| centres[(r.next() % 200) as usize] | (r.next() & 0xFFFF)).collect();
    groups.sort_unstable();
    assert!(clustered(&groups));
}
