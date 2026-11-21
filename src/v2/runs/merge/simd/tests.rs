//! Unit tests of `v2/runs/merge/simd/mod.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

/// Two sorted runs of every length pair up to 40 and a few long ones,
/// merged in each mode (0 unsigned, 1 signed, 2 float bits) against
/// the merged order of the same keys.
macro_rules! check {
    ($u:ty, $merge:expr, $key:expr) => {{
        let key: fn($u) -> $u = $key;
        let mut r = Rng(19);
        let lens: Vec<usize> = (0..40).chain([255, 1000, 20_000].iter().copied()).collect();
        for &la in lens.iter() {
            for &lb in [0usize, 1, 3, 4, 5, 17, 39, 1000].iter() {
                let mut a: Vec<$u> = (0..la).map(|_| r.next() as $u).collect();
                let mut b: Vec<$u> = (0..lb).map(|_| r.next() as $u).collect();
                a.sort_unstable_by_key(|&x| key(x));
                b.sort_unstable_by_key(|&x| key(x));
                let mut dst = vec![0 as $u; la + lb];
                $merge(&a, &b, &mut dst);
                let mut expected: Vec<$u> = a.iter().chain(&b).copied().collect();
                expected.sort_unstable_by_key(|&x| key(x));
                let same = dst.iter().zip(&expected).all(|(&x, &y)| key(x) == key(y));
                assert!(same, "{} {} {}+{}", stringify!($merge), stringify!($key), la, lb);
            }
        }
    }};
}

#[test]
fn merges() {
    check!(u32, merge_u32::<0>, |x| x);
    check!(u32, merge_u32::<1>, |x| x ^ 1 << 31);
    check!(u32, merge_u32::<2>, float_key_32);
    check!(u64, merge_u64::<0>, |x| x);
    check!(u64, merge_u64::<1>, |x| x ^ 1 << 63);
    check!(u64, merge_u64::<2>, float_key_64);
}
