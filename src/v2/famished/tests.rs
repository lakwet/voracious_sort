//! Unit tests of `v2/sorts/by_key/mod.rs` (a child module: sees its private items).

use std::mem::MaybeUninit;

use super::*;
use crate::v2::testing::*;

/// A record: its key, its first position (shows the stability) and a
/// payload of `N` bytes (small records, and records from 32 bytes for
/// the 4-way merges).
#[derive(Clone, Copy, Debug)]
struct Rec<const N: usize> {
    key: u64,
    pos: u32,
    pay: [u8; N],
}

fn enc<const N: usize>(r: &Rec<N>) -> u128 { r.key as u128 }

fn records<const N: usize>(keys: &[u64]) -> Vec<Rec<N>> {
    keys.iter().enumerate().map(|(i, &key)| Rec { key, pos: i as u32, pay: [i as u8; N] }).collect()
}

/// `got` is the stable sort of `a` by key, payloads intact.
fn stable<const N: usize>(what: &str, a: &[Rec<N>], got: &[Rec<N>]) {
    let mut expected = a.to_vec();
    expected.sort_by_key(|r| r.key);
    let same = got.len() == a.len()
        && got
            .iter()
            .zip(&expected)
            .all(|(x, y)| x.key == y.key && x.pos == y.pos && x.pay == y.pay);
    assert!(same, "{} n={}: not the stable order", what, a.len());
}

/// `got` holds the records of `a` (when a try gave up).
fn kept<const N: usize>(what: &str, a: &[Rec<N>], got: &[Rec<N>]) {
    let key = |r: &Rec<N>| (r.key, r.pos);
    let (x, y): (Vec<_>, Vec<_>) = (a.iter().map(key).collect(), got.iter().map(key).collect());
    assert!(same_values(&x, &y), "{} n={}: records lost", what, a.len());
}

/// Cuts `0..n` into about `runs` sorted segments: the bounds.
fn runs_of<const N: usize>(a: &mut [Rec<N>], runs: usize, r: &mut Rng) -> Vec<usize> {
    let n = a.len();
    let mut b: Vec<usize> = (1..runs).map(|_| (r.next() % (n as u64 + 1)) as usize).collect();
    b.push(0);
    b.push(n);
    b.sort_unstable();
    b.dedup();
    for w in b.windows(2) {
        a[w[0]..w[1]].sort_by_key(|x| x.key);
    }
    b
}

/// The two-run merges (branchy or not, with or without a given buffer,
/// the gap merge, ascending then descending) and the k-way merges, on
/// every shape, keys with and without repeats.
fn merges<const N: usize>() {
    let mut r = Rng(41);
    for_each_case::<u64>(|shape, keys| {
        let n = keys.len();
        for &distinct in [5u64, u64::MAX].iter() {
            let keys: Vec<u64> = keys.iter().map(|&k| k % distinct).collect();
            let a: Vec<Rec<N>> = records(&keys);
            let what = |w: &str| format!("{} {} {} distinct {}", w, N, shape, distinct);
            let m = (r.next() % (n as u64 + 1)) as usize;
            let mut two = a.clone();
            two[..m].sort_by_key(|x| x.key);
            two[m..].sort_by_key(|x| x.key);
            for &branchy in [false, true].iter() {
                let mut got = two.clone();
                merge::merge_two(&mut got, m, &enc, branchy);
                stable(&what("merge_two"), &a, &got);
                let mut buf: Vec<MaybeUninit<Rec<N>>> = Vec::with_capacity(n);
                let mut got = two.clone();
                merge::merge_two_with(&mut got, m, &enc, &mut buf, branchy);
                stable(&what("merge_two_with"), &a, &got);
            }
            let mut buf: Vec<MaybeUninit<Rec<N>>> = Vec::with_capacity(n);
            let mut got = two.clone();
            merge::merge_two_x(&mut got, m, &enc, &mut buf);
            stable(&what("merge_two_x"), &a, &got);
            // Ascending, then strictly descending (distinct keys only).
            if distinct == u64::MAX {
                let mut organ = two.clone();
                let mut keys2: Vec<u64> = organ[m..].iter().map(|x| x.key).collect();
                keys2.dedup();
                if keys2.len() == n - m {
                    organ[m..].reverse();
                    let mut got = organ.clone();
                    merge::merge_asc_desc(&mut got, m, &enc);
                    stable(&what("merge_asc_desc"), &organ, &got);
                }
            }
            for &runs in [2usize, 3, 5, 8, 33].iter() {
                let mut segs = a.clone();
                let bounds = runs_of(&mut segs, runs, &mut r);
                for &branchy in [false, true].iter() {
                    let mut got = segs.clone();
                    merge::merge_all_half(&mut got, &bounds, &enc, branchy);
                    stable(&what("merge_all_half"), &a, &got);
                    let mut got = segs.clone();
                    merge::merge_all_kway(&mut got, &bounds, &enc, branchy);
                    stable(&what("merge_all_kway"), &a, &got);
                }
            }
        }
    });
}

#[test]
fn merges_small_records() { merges::<4>(); }

#[test]
fn merges_large_records() { merges::<40>(); }

/// The tries of the chains (few keys, heavy keys, LSD on 11-bit
/// digits, packed sorts, verge, ends): sorted stably when they answer
/// `true` (or always, for the sorts), records kept when they give up.
/// From 64 records: the chains try them on no shorter array.
fn tries<const N: usize>() {
    let sort = |x: &mut [Rec<N>]| x.sort_by_key(|r| r.key);
    for_each_case::<u64>(|shape, keys| {
        let n = keys.len();
        if n < 64 {
            return;
        }
        let a: Vec<Rec<N>> = records(&keys);
        let what = |w: &str| format!("{} {} {}", w, N, shape);
        let check = |name: &str, done: bool, got: &[Rec<N>]| {
            if done {
                stable(&what(name), &a, got);
            } else {
                kept(&what(name), &a, got);
            }
        };
        let mut got = a.clone();
        let done = few::few_keys(&mut got, &enc, &Sample::new());
        check("few_keys", done, &got);
        assert!(done || shape != "few" || n < 128, "{}: few keys missed", what("few_keys"));
        let mut got = a.clone();
        let done = few::few_keys_wide(&mut got, &enc);
        check("few_keys_wide", done, &got);
        let mut got = a.clone();
        let done = heavy::heavy_keys(&mut got, &enc, &Sample::new(), &sort);
        check("heavy_keys", done, &got);
        let lo = keys.iter().min().copied().unwrap();
        let mut got = a.clone();
        lsd::lsd11(&mut got, &enc, lo);
        stable(&what("lsd11"), &a, &got);
        let bits = 128 - ((keys.iter().max().unwrap() - lo) as u128).leading_zeros();
        let mut got = a.clone();
        let done = packed::sort_in_place(&mut got, &enc, lo as u128, bits);
        check("packed sort_in_place", done, &got);
        if 32 - (n as u32).leading_zeros() + bits <= 128 {
            let mut got = a.clone();
            packed_buffer(&mut got, &enc, lo as u128, bits);
            stable(&what("packed_buffer"), &a, &got);
        }
        let mut got = a.clone();
        std_fallback(&mut got, &enc);
        stable(&what("std_fallback"), &a, &got);
        let mut got = a.clone();
        let done = verge::verge(&mut got, &enc, &sort);
        check("verge", done, &got);
        for &only in [false, true].iter() {
            let mut got = a.clone();
            let done = verge::verge_if(&mut got, &enc, only, &sort);
            check("verge_if", done, &got);
        }
        let mut got = a.clone();
        let done = ends(&mut got, &enc, &mut (0, false), &sort);
        check("ends", done, &got);
        assert!(done || !sorted_by(&keys, |k| k), "{}: sorted array missed", what("ends"));
    });
}

#[test]
fn tries_small_records() { tries::<4>(); }

#[test]
fn tries_large_records() { tries::<40>(); }

/// The entry points on every key type: stable (positions kept among
/// equal keys) and unstable (records kept, keys in order).
macro_rules! entries {
    ($($k:ty),*) => {$(
        for_each_case::<u64>(|shape, keys| {
            let a: Vec<($k, u32)> =
                keys.iter().enumerate().map(|(i, &x)| (x as $k, i as u32)).collect();
            let what = format!("{} {} n={}", stringify!($k), shape, a.len());
            let mut got = a.clone();
            famished_sort_by_key(&mut got, |x| x.0);
            let mut expected = a.clone();
            expected.sort_by_key(|x| x.0);
            assert!(got == expected, "stable {}", what);
            let mut got = a.clone();
            famished_sort_unstable_by_key(&mut got, |x| x.0);
            assert!(sorted_by(&got, |x| x.0) && same_values(&got, &a), "unstable {}", what);
        });
    )*};
}

#[test]
fn entry_points() {
    entries!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
    for_each_case::<u64>(|shape, keys| {
        let a: Vec<(bool, u32)> =
            keys.iter().enumerate().map(|(i, &x)| (x & 1 == 1, i as u32)).collect();
        let mut got = a.clone();
        famished_sort_by_key(&mut got, |x| x.0);
        let mut expected = a.clone();
        expected.sort_by_key(|x| x.0);
        assert!(got == expected, "bool {} n={}", shape, a.len());
    });
    // Floats: by `total_cmp` (NaN and -0.0 included), via their bits.
    for_each_case::<u64>(|shape, keys| {
        let a: Vec<(f64, u32)> =
            keys.iter().enumerate().map(|(i, &x)| (f64::from_bits(x), i as u32)).collect();
        let mut got = a.clone();
        famished_sort_by_key(&mut got, |x| x.0);
        let mut expected = a.clone();
        expected.sort_by(|x, y| x.0.total_cmp(&y.0));
        let same =
            got.iter().zip(&expected).all(|(x, y)| x.0.to_bits() == y.0.to_bits() && x.1 == y.1);
        assert!(same, "f64 {} n={}", shape, a.len());
        let b: Vec<(f32, u32)> =
            keys.iter().enumerate().map(|(i, &x)| (f32::from_bits(x as u32), i as u32)).collect();
        let mut got = b.clone();
        famished_sort_by_key(&mut got, |x| x.0);
        let mut expected = b.clone();
        expected.sort_by(|x, y| x.0.total_cmp(&y.0));
        let same =
            got.iter().zip(&expected).all(|(x, y)| x.0.to_bits() == y.0.to_bits() && x.1 == y.1);
        assert!(same, "f32 {} n={}", shape, b.len());
        let c: Vec<(char, u32)> = keys
            .iter()
            .enumerate()
            .map(|(i, &x)| (std::char::from_u32((x % 0x11_0000) as u32).unwrap_or('a'), i as u32))
            .collect();
        let mut got = c.clone();
        famished_sort_by_key(&mut got, |x| x.0);
        let mut expected = c.clone();
        expected.sort_by_key(|x| x.0);
        assert!(got == expected, "char {} n={}", shape, c.len());
    });
}
