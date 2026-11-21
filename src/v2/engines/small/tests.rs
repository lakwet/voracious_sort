//! Unit tests of `v2/engines/small/mod.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Every length up to the limit of each sort, in every shape, unsigned
/// and signed order (codec: flip the sign bit).
macro_rules! check {
    ($u:ty) => {
        let flip = |k: $u| k ^ 1 << (<$u>::BITS - 1);
        for n in 0..=MERGE_MAX {
            for (shape, raw) in shapes(n, n as u64 + 1) {
                for way in 0..2 {
                    let a: Vec<$u> = raw.iter().map(|&x| <$u>::from_raw(x)[way]).collect();
                    let what = format!("{} {} n={}", stringify!($u), shape, n);
                    let mut expected = a.clone();
                    expected.sort_unstable();
                    let mut got = a.clone();
                    merge_sort(&mut got, |k| k, |k| k);
                    assert!(got == expected, "merge_sort {}", what);
                    let mut got = a.clone();
                    merge_sort(&mut got, flip, flip);
                    assert!(
                        sorted_by(&got, flip) && same_values(&got, &a),
                        "merge_sort signed {}",
                        what
                    );
                    let mut got = a.clone();
                    block_merge_sort(&mut got);
                    assert!(got == expected, "block_merge_sort {}", what);
                    if n <= NETWORK_MAX {
                        let mut got = a.clone();
                        network_sort(&mut got, |k| k, |k| k);
                        assert!(got == expected, "network_sort {}", what);
                        let mut got = a.clone();
                        network_sort(&mut got, flip, flip);
                        assert!(
                            sorted_by(&got, flip) && same_values(&got, &a),
                            "network_sort signed {}",
                            what
                        );
                    }
                }
            }
        }
    };
}

#[test]
fn every_length() {
    check!(u8);
    check!(u16);
    check!(u32);
    check!(u64);
    check!(u128);
}
