//! Unit tests of `v2/types/mod.rs` (a child module: sees its private items).

use crate::v2::testing::*;

/// The chain of each key type (`sort`), on every shape and size: the
/// std order (floats: `total_cmp`, NaN and -0.0 included). The crate's
/// tests in src/tests do more (records, every threshold); these also
/// run in the default build.
macro_rules! ints {
    ($($t:ident: $u:ty),*) => {$(
        for_each_case::<$u>(|shape, raw| {
            let a: Vec<$t> = raw.iter().map(|&x| x as $t).collect();
            let mut got = a.clone();
            super::$t::sort(&mut got);
            let mut expected = a.clone();
            expected.sort_unstable();
            assert!(got == expected, "{} {} n={}", stringify!($t), shape, a.len());
        });
    )*};
}

#[test]
fn integer_chains() {
    ints!(u8: u8, i8: u8, u16: u16, i16: u16, u32: u32, i32: u32);
    ints!(u64: u64, i64: u64, u128: u128, i128: u128);
}

#[test]
fn char_chain() {
    for_each_case::<u32>(|shape, raw| {
        let a: Vec<char> =
            raw.iter().map(|&x| std::char::from_u32(x % 0x11_0000).unwrap_or('\u{D7FF}')).collect();
        let mut got = a.clone();
        super::char::sort(&mut got);
        let mut expected = a.clone();
        expected.sort_unstable();
        assert!(got == expected, "char {} n={}", shape, a.len());
    });
}

#[test]
fn bool_chain() {
    // Low bit (about half true), and few true (shapes kept in order).
    for map in [|x: u8| x & 1 == 1, |x: u8| x > 250] {
        for_each_case::<u8>(|shape, raw| {
            let a: Vec<bool> = raw.iter().map(|&x| map(x)).collect();
            let mut got = a.clone();
            super::bool::sort(&mut got);
            let mut expected = a.clone();
            expected.sort_unstable();
            assert!(got == expected, "bool {} n={}", shape, a.len());
        });
    }
}

macro_rules! floats {
    ($f:ident, $u:ty) => {
        for_each_case::<$u>(|shape, raw| {
            let a: Vec<$f> = raw.iter().map(|&x| <$f>::from_bits(x)).collect();
            let mut got = a.clone();
            super::$f::sort(&mut got);
            let mut expected = a.clone();
            expected.sort_unstable_by(|x, y| x.total_cmp(y));
            let same = got.iter().zip(&expected).all(|(x, y)| x.to_bits() == y.to_bits());
            assert!(same, "{} {} n={}", stringify!($f), shape, a.len());
        });
    };
}

#[test]
fn float_chains() {
    floats!(f32, u32);
    floats!(f64, u64);
}
