//! One decision chain per key type: everything that decides how an array of
//! that type is sorted (small arrays, presorted check, detector order, radix
//! path rules, engine of each chunk, thresholds). A fix for one type is made
//! in its file only, so no other type moves; a type that needs a changed
//! piece of shared code gets a folder with its copy (`u128/presorted.rs`).
//! Method of 2026-11-17: copy, route, change the copy only; factor what is
//! shared once every type wins.
//!
//! Shared by all (not decisions): the engines (`engines`), the detectors
//! themselves (`special`), the inspections (`inspect`, `decide::shape`,
//! the helpers of `decide::path`), runs and merges (`runs`), the key
//! encodings (`sorts::devour::keys`, `sorts::ravenous`).
//!
//! Inventory, against the shared chain the files were copied from
//! (2026-11-17):
//!
//! | file        | differences                                             |
//! |-------------|---------------------------------------------------------|
//! | u8, i8      | none                                                    |
//! | u16, i16    | no `dominant` (slower on the Ryzen 3950X: 1.09, 1.04)   |
//! | u32, i32    | none                                                    |
//! | u64, i64    | none                                                    |
//! | u128, i128  | own `presorted` (v1 scan by 4 pairs), behind `new_rule` |
//! | f32, f64    | none                                                    |
//! | char        | none (its own copy of the u32 chain)                    |
//!
//! `usize` / `isize` go to the type of their width (`sorts::devour`).
//!
//! Records (`records`): one folder per key type (bool, u8 ... i128, f32,
//! f64, char), one chain per record size in it (r1, r2, r4, r8, r16, r32,
//! r64; from the key's own size up). Chains of at most 8 bytes: copies of
//! the small-records path; above: copies of the large-records chain. No
//! differences yet. The pieces they share re-sort through the caller's
//! chain (`ends`, `heavy_keys`, Verge take it as an argument).

pub(crate) mod char;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i128;
pub(crate) mod i16;
pub(crate) mod i32;
pub(crate) mod i64;
pub(crate) mod i8;
pub(crate) mod records;
pub(crate) mod u128;
pub(crate) mod u16;
pub(crate) mod u32;
pub(crate) mod u64;
pub(crate) mod u8;

#[cfg(test)]
mod tests {
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
            let a: Vec<char> = raw
                .iter()
                .map(|&x| std::char::from_u32(x % 0x11_0000).unwrap_or('\u{D7FF}'))
                .collect();
            let mut got = a.clone();
            super::char::sort(&mut got);
            let mut expected = a.clone();
            expected.sort_unstable();
            assert!(got == expected, "char {} n={}", shape, a.len());
        });
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
}
