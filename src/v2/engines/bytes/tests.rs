//! Unit tests of `v2/engines/bytes.rs` (a child module: sees its private items).

use super::*;
use crate::v2::testing::*;

#[test]
fn counting_and_order() {
    for_each_case::<u8>(|shape, raw| {
        if raw.is_empty() {
            return;
        }
        let mut expected = raw.to_vec();
        expected.sort_unstable();
        let mut got = raw.to_vec();
        count::<0>(&mut got);
        assert!(got == expected, "u8 {} n={}", shape, raw.len());
        // i8 through its bits.
        let signed: Vec<i8> = raw.iter().map(|&x| x as i8).collect();
        let mut bits: Vec<u8> = signed.iter().map(|&x| x as u8).collect();
        count::<0x80>(&mut bits);
        let got: Vec<i8> = bits.iter().map(|&x| x as i8).collect();
        let mut expected = signed.clone();
        expected.sort_unstable();
        assert!(got == expected, "i8 {} n={}", shape, raw.len());
        // The order check: true exactly on sorted, reversed, constant.
        let mut a = raw.to_vec();
        let sorted = a.windows(2).all(|p| p[0] <= p[1]);
        let reversed = a.windows(2).all(|p| p[0] >= p[1]);
        assert_eq!(sorted_or_reversed(&mut a), sorted || reversed, "{}", shape);
        if sorted || reversed {
            assert!(a.windows(2).all(|p| p[0] <= p[1]), "{}", shape);
        } else {
            assert_eq!(a, raw.to_vec(), "{}", shape);
        }
    });
}
