//! Records (structs) to sort by a key: keys of 16 to 64 bits (u16, i16,
//! u32, i32, f32, u64, i64, f64, char) in records of 8 to 128 bytes, the
//! key first or last; a record holding a `String` (not `Copy`); a composite
//! key from two fields. The keys follow the distributions of their type;
//! the payload's first word is the record's original index (stability
//! checks).

use super::int_extra::{extra_i32, extra_i64, extra_u32, extra_u64};
use super::narrow_extra::{extra_char, extra_i16, extra_u16};

pub type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

/// Key first, then `W` payload words (the first: original index).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Rec<K, const W: usize> {
    pub key: K,
    pub pay: [u32; W],
}

/// Payload first, key last.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct RecEnd<K, const W: usize> {
    pub pay: [u32; W],
    pub key: K,
}

/// A record that is not `Copy`: u64 key, a heap string (32 bytes).
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct RecString {
    pub key: u64,
    pub name: String,
}

/// Composite key: (`hi`, `lo`) compared as `(hi << 32) | lo` (16 bytes).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct RecPair {
    pub hi: u32,
    pub lo: u32,
    pub index: u64,
}

/// Competitors that sort records through a trait on the record: rdst
/// (`RadixKey`) and voracious 1.x (`Radixable`).
impl<K: rdst::RadixKey, const W: usize> rdst::RadixKey for Rec<K, W> {
    const LEVELS: usize = K::LEVELS;
    #[inline]
    fn get_level(&self, level: usize) -> u8 {
        self.key.get_level(level)
    }
}

macro_rules! v1_rec {
    ($($k:ty),*) => {$(
        impl<const W: usize> crate::Radixable<$k> for Rec<$k, W> {
            type Key = $k;
            #[inline]
            fn key(&self) -> $k {
                self.key
            }
        }
    )*};
}
v1_rec!(u16, u32, u64, i64);

impl RecPair {
    pub fn key(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }
}

/// Key generators of each key type (own list + extras).
pub fn keys_u16() -> Gens<u16> {
    let mut g = super::unsigned_u16::generators_u16();
    g.extend(extra_u16::generators());
    g
}
pub fn keys_i16() -> Gens<i16> {
    let mut g = super::signed_i16::generators_i16();
    g.extend(extra_i16::generators());
    g
}
pub fn keys_u32() -> Gens<u32> {
    let mut g = super::unsigned_u32::generators_u32();
    g.extend(extra_u32::generators());
    g
}
pub fn keys_i32() -> Gens<i32> {
    let mut g = super::signed_i32::generators_i32();
    g.extend(extra_i32::generators());
    g
}
pub fn keys_u64() -> Gens<u64> {
    let mut g = super::unsigned_u64::generators_u64();
    g.extend(extra_u64::generators());
    g
}
pub fn keys_i64() -> Gens<i64> {
    let mut g = super::signed_i64::generators_i64();
    g.extend(extra_i64::generators());
    g
}
pub fn keys_f32() -> Gens<f32> {
    super::float_32::generators_f32()
}
pub fn keys_f64() -> Gens<f64> {
    super::float_64::generators_f64()
}
pub fn keys_char() -> Gens<char> {
    let mut g = super::char::generators_char();
    g.extend(extra_char::generators());
    g
}

/// Records built from a list of key generators.
pub fn wrap<K: Copy + 'static, T: 'static>(keys: Gens<K>, make: fn(Vec<K>) -> Vec<T>) -> Gens<T> {
    keys.into_iter()
        .map(|(g, name)| {
            let f = move |n: usize| make(g(n));
            (Box::leak(Box::new(f)) as &'static dyn Fn(usize) -> Vec<T>, name)
        })
        .collect()
}

/// Key first, `W` words of payload, index in the first.
pub fn rec<K: Copy, const W: usize>(keys: Vec<K>) -> Vec<Rec<K, W>> {
    keys.into_iter()
        .enumerate()
        .map(|(i, key)| {
            let mut pay = [0x5A5A_5A5Au32; W];
            pay[0] = i as u32;
            Rec { key, pay }
        })
        .collect()
}

/// Key last.
pub fn rec_end<K: Copy, const W: usize>(keys: Vec<K>) -> Vec<RecEnd<K, W>> {
    keys.into_iter()
        .enumerate()
        .map(|(i, key)| {
            let mut pay = [0x5A5A_5A5Au32; W];
            pay[0] = i as u32;
            RecEnd { pay, key }
        })
        .collect()
}

pub fn rec_string(keys: Vec<u64>) -> Vec<RecString> {
    keys.into_iter().enumerate().map(|(i, key)| RecString { key, name: format!("#{i}") }).collect()
}

pub fn rec_pair(keys: Vec<u64>) -> Vec<RecPair> {
    keys.into_iter().enumerate().map(|(i, k)| RecPair { hi: (k >> 32) as u32, lo: k as u32, index: i as u64 }).collect()
}

/// Keys built to test stability (correctness tests only): many equal keys
/// in every shape the sort treats apart (constant, few values around the
/// counting limit, presorted with duplicates, non-strictly descending, runs
/// and teeth with equal keys inside and across them).
pub mod stability {
    use rand::RngExt;

    pub fn equal(n: usize) -> Vec<u64> {
        vec![7; n]
    }
    pub fn two_values(n: usize) -> Vec<u64> {
        (0..n).map(|_| rand::rng().random_range(0..2)).collect()
    }
    pub fn few16(n: usize) -> Vec<u64> {
        (0..n).map(|_| rand::rng().random_range(0..16)).collect()
    }
    // At the limit of the few-keys counting, and just above it.
    pub fn few256(n: usize) -> Vec<u64> {
        (0..n).map(|_| rand::rng().random_range(0..256)).collect()
    }
    pub fn few257(n: usize) -> Vec<u64> {
        (0..n).map(|_| rand::rng().random_range(0..257)).collect()
    }
    pub fn dups1000(n: usize) -> Vec<u64> {
        (0..n).map(|_| rand::rng().random_range(0..1000)).collect()
    }
    pub fn dups_quarter(n: usize) -> Vec<u64> {
        let k = (n as u64 / 4).max(1);
        (0..n).map(|_| rand::rng().random_range(0..k)).collect()
    }
    // Ascending with runs of 8 equal keys (the presorted path).
    pub fn asc_dups(n: usize) -> Vec<u64> {
        (0..n as u64).map(|i| i / 8).collect()
    }
    // Descending, not strictly: must not be reversed.
    pub fn desc_dups(n: usize) -> Vec<u64> {
        (0..n as u64).map(|i| (n as u64 - i) / 8).collect()
    }
    // Strictly descending but for one equal pair at the end.
    pub fn desc_last_equal(n: usize) -> Vec<u64> {
        let mut v: Vec<u64> = (0..n as u64).map(|i| 2 * (n as u64 - i)).collect();
        if n >= 2 {
            v[n - 1] = v[n - 2];
        }
        v
    }
    // Teeth sharing their values, with duplicates inside each tooth.
    pub fn saw_dups(n: usize) -> Vec<u64> {
        let limit = ((n as f64 / (n as f64).log2().max(1.0) / 0.9) as u64).max(2);
        (0..n as u64).map(|i| (i % limit) / 4).collect()
    }
    // Ascending then descending, duplicates in both halves and across.
    pub fn pipe_dups(n: usize) -> Vec<u64> {
        let h = n as u64 / 2;
        (0..n as u64).map(|i| if i < h { i / 3 } else { (n as u64 - i) / 3 }).collect()
    }
    // 8 sorted runs over 100 values: many ties between runs.
    pub fn runs_dups(n: usize) -> Vec<u64> {
        let mut v: Vec<u64> = (0..n).map(|_| rand::rng().random_range(0..100)).collect();
        for r in 0..8 {
            v[r * n / 8..(r + 1) * n / 8].sort_unstable();
        }
        v
    }
    // Sorted with duplicates for 90 %, a random tail over the same values.
    pub fn asc_tail_dups(n: usize) -> Vec<u64> {
        let cut = n * 9 / 10;
        let top = (cut as u64 / 4).max(1);
        (0..n).map(|i| if i < cut { i as u64 / 4 } else { rand::rng().random_range(0..top) }).collect()
    }
    // One value for 60 %, the rest among 100.
    pub fn heavy_dups(n: usize) -> Vec<u64> {
        (0..n).map(|_| { let mut r = rand::rng(); if r.random_range(0..10) < 6 { 50 } else { r.random_range(0..100) } }).collect()
    }
    pub fn alternate(n: usize) -> Vec<u64> {
        (0..n as u64).map(|i| i % 2).collect()
    }

    pub fn generators() -> super::Gens<u64> {
        vec![
            (&equal, "stab equal"),
            (&two_values, "stab 2 values"),
            (&few16, "stab 16 values"),
            (&few256, "stab 256 values"),
            (&few257, "stab 257 values"),
            (&dups1000, "stab 1000 values"),
            (&dups_quarter, "stab n/4 values"),
            (&asc_dups, "stab asc dups"),
            (&desc_dups, "stab desc dups"),
            (&desc_last_equal, "stab desc last equal"),
            (&saw_dups, "stab saw dups"),
            (&pipe_dups, "stab pipe dups"),
            (&runs_dups, "stab runs dups"),
            (&asc_tail_dups, "stab asc tail dups"),
            (&heavy_dups, "stab heavy"),
            (&alternate, "stab alternate"),
        ]
    }

    /// The same keys converted to another key type (orders and equalities
    /// kept: the values are small).
    pub fn as_keys<K: Copy + 'static>(convert: fn(u64) -> K) -> super::Gens<K> {
        generators()
            .into_iter()
            .map(|(g, name)| {
                let f = move |n: usize| g(n).into_iter().map(convert).collect();
                (Box::leak(Box::new(f)) as &'static dyn Fn(usize) -> Vec<K>, name)
            })
            .collect()
    }
}
