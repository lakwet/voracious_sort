//! Records (structs) to sort by a key: a key field and a payload of `P`
//! words, from 8 bytes (u32 key, u32 value) to 128 bytes. The keys come from
//! the distributions of the key type; the payload holds the record's
//! original index (stability checks) and filler.

use super::int_extra::{extra_u32, extra_u64};
use super::unsigned_u32::generators_u32;
use super::unsigned_u64::generators_u64;

/// A record: `key` and `P` payload words (the first one: original index).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Rec<K, const P: usize> {
    pub key: K,
    pub pay: [u64; P],
}

/// 8-byte record: u32 key, u32 value (original index).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Rec8 {
    pub key: u32,
    pub val: u32,
}

pub type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

fn wrap<K: Copy + 'static, T: 'static>(keys: Gens<K>, make: &'static (dyn Fn(Vec<K>) -> Vec<T> + Sync)) -> Gens<T> {
    keys.into_iter()
        .map(|(g, name)| {
            let f = move |n: usize| make(g(n));
            (Box::leak(Box::new(f)) as &'static dyn Fn(usize) -> Vec<T>, name)
        })
        .collect()
}

fn records<K: Copy, const P: usize>(keys: Vec<K>) -> Vec<Rec<K, P>> {
    keys.into_iter()
        .enumerate()
        .map(|(i, key)| {
            let mut pay = [0x5A5A_5A5A_5A5A_5A5Au64; P];
            if P > 0 {
                pay[0] = i as u64;
            }
            Rec { key, pay }
        })
        .collect()
}

fn u64_keys() -> Gens<u64> {
    let mut g = generators_u64();
    g.extend(extra_u64::generators());
    g
}

/// u32 key, u32 value: 8 bytes.
pub fn generators_rec8() -> Gens<Rec8> {
    let mut g = generators_u32();
    g.extend(extra_u32::generators());
    wrap(g, &|keys: Vec<u32>| keys.into_iter().enumerate().map(|(i, key)| Rec8 { key, val: i as u32 }).collect())
}

/// u64 key, one word: 16 bytes.
pub fn generators_rec16() -> Gens<Rec<u64, 1>> {
    wrap(u64_keys(), &|k| records::<u64, 1>(k))
}

/// u64 key, three words: 32 bytes.
pub fn generators_rec32() -> Gens<Rec<u64, 3>> {
    wrap(u64_keys(), &|k| records::<u64, 3>(k))
}

/// u64 key, seven words: 64 bytes.
pub fn generators_rec64() -> Gens<Rec<u64, 7>> {
    wrap(u64_keys(), &|k| records::<u64, 7>(k))
}

/// u64 key, fifteen words: 128 bytes.
pub fn generators_rec128() -> Gens<Rec<u64, 15>> {
    wrap(u64_keys(), &|k| records::<u64, 15>(k))
}

/// f64 key, three words: 32 bytes.
pub fn generators_rec_f64() -> Gens<Rec<f64, 3>> {
    wrap(super::float_64::generators_f64(), &|k| records::<f64, 3>(k))
}

/// u128 key, two words: 32 bytes.
pub fn generators_rec_u128() -> Gens<Rec<u128, 2>> {
    let mut g = super::unsigned_u128::generators_u128();
    g.extend(super::int128_extra::extra_u128::generators());
    wrap(g, &|k| records::<u128, 2>(k))
}
