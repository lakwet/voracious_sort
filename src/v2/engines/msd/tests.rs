//! Unit tests of `v2/engines/msd.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Records (key, position) by their key: in key order, none lost. Keys
/// of 64 bits (range within u64) and of 128 bits (wider range).
#[test]
fn records() {
    for_each_case::<u64>(|shape, keys| {
        let a: Vec<(u64, u32)> = keys.iter().enumerate().map(|(i, &k)| (k, i as u32)).collect();
        check(&format!("u64 {}", shape), a, |x: &(u64, u32)| x.0 as u128);
    });
    for_each_case::<u128>(|shape, keys| {
        let a: Vec<(u128, u32)> = keys.iter().enumerate().map(|(i, &k)| (k, i as u32)).collect();
        check(&format!("u128 {}", shape), a, |x: &(u128, u32)| x.0);
    });
}

fn check<T: Copy + Ord, E: Fn(&T) -> u128>(what: &str, a: Vec<T>, enc: E) {
    if a.is_empty() {
        return;
    }
    let lo = a.iter().map(&enc).min().unwrap();
    let hi = a.iter().map(&enc).max().unwrap();
    let mut got = a.clone();
    msd(&mut got, &enc, lo, hi);
    assert!(got.windows(2).all(|w| enc(&w[0]) <= enc(&w[1])), "{} n={}: not sorted", what, a.len());
    assert!(same_values(&got, &a), "{} n={}: records lost", what, a.len());
}
