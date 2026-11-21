//! Memory checks of the v2 sorts under Miri: `cargo +nightly miri test
//! miri_` (`make miri`). Small arrays, and the cache sizes forced small
//! (`L2_OVERRIDE`) so that the engines meant for large arrays (rollercoaster,
//! LSD beyond the cache, DLSD, merges, detectors) run on a few thousand
//! keys. Without Miri these are ordinary (fast) tests.

use crate::v2::cache::L2_OVERRIDE;
use crate::v2::sorts::by_key::{devour_sort_by_key, devour_sort_unstable_by_key};
use crate::v2::sorts::devour::devour_sort;
use crate::v2::sorts::ravenous::ravenous_sort;

/// Forced L2 (and largest cache) in bytes: thresholds of a few hundred to a
/// few thousand keys.
const SMALL_CACHE: usize = 16 * 1024;

fn sizes() -> &'static [usize] {
    if cfg!(miri) {
        &[0, 1, 2, 3, 17, 33, 64, 65, 200, 700, 1500, 3000]
    } else {
        &[0, 1, 2, 3, 17, 33, 64, 65, 200, 700, 1500, 3000, 20_000, 100_000]
    }
}

/// xorshift64*: no generator crate (they run on rayon), same keys under Miri.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Shapes that reach the detectors and the run handling: random, narrow,
/// few values, sorted, reversed, saw, sorted with a random tail, equal.
fn shapes(n: usize, seed: u64) -> Vec<(&'static str, Vec<u64>)> {
    let mut r = Rng(seed | 1);
    let rand: Vec<u64> = (0..n).map(|_| r.next()).collect();
    let mut asc = rand.clone();
    asc.sort_unstable();
    let mut tail = asc.clone();
    for x in tail.iter_mut().skip(n * 9 / 10) {
        *x = r.next();
    }
    vec![
        ("random", rand.clone()),
        ("narrow", rand.iter().map(|x| x % 1000).collect()),
        ("few", rand.iter().map(|x| (x % 5) << 40).collect()),
        ("asc", asc.clone()),
        ("desc", asc.iter().rev().copied().collect()),
        ("saw", (0..n as u64).map(|i| (i % 97) << 20).collect()),
        ("tail", tail),
        ("equal", vec![42; n]),
    ]
}

fn with_small_cache(f: impl Fn()) {
    L2_OVERRIDE.store(SMALL_CACHE, std::sync::atomic::Ordering::Relaxed);
    f();
    L2_OVERRIDE.store(0, std::sync::atomic::Ordering::Relaxed);
}

macro_rules! check_ints {
    ($($t:ty),*) => {$(
        for &n in sizes() {
            for (name, keys) in shapes(n, n as u64 + 7) {
                let mut a: Vec<$t> = keys.iter().map(|&k| k as $t).collect();
                let mut e = a.clone();
                e.sort_unstable();
                devour_sort(&mut a);
                assert!(a == e, "{} n {} {}", stringify!($t), n, name);
            }
        }
    )*};
}

#[test]
fn miri_devour() {
    with_small_cache(|| {
        check_ints!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
        for &n in sizes() {
            for (name, keys) in shapes(n, 3) {
                let mut a: Vec<char> =
                    keys.iter().map(|&k| char::from_u32((k % 0xD000) as u32).unwrap()).collect();
                let mut e = a.clone();
                e.sort_unstable();
                devour_sort(&mut a);
                assert!(a == e, "char n {} {}", n, name);
            }
        }
    });
}

#[test]
fn miri_ravenous() {
    with_small_cache(|| {
        for &n in sizes() {
            for (name, keys) in shapes(n, 11) {
                let mut a: Vec<f64> =
                    keys.iter().map(|&k| (k >> 11) as f64 * 1e-3 - 4e12).collect();
                if n > 3 {
                    a[1] = f64::NAN;
                    a[2] = -0.0;
                    a[3] = f64::INFINITY;
                }
                let mut e = a.clone();
                e.sort_unstable_by(|x, y| x.total_cmp(y));
                ravenous_sort(&mut a);
                let same = a.iter().zip(e.iter()).all(|(x, y)| x.to_bits() == y.to_bits());
                assert!(same, "f64 n {} {}", n, name);
                let mut b: Vec<f32> = keys.iter().map(|&k| (k >> 40) as f32 - 5e6).collect();
                let mut e = b.clone();
                e.sort_unstable_by(|x, y| x.total_cmp(y));
                ravenous_sort(&mut b);
                let same = b.iter().zip(e.iter()).all(|(x, y)| x.to_bits() == y.to_bits());
                assert!(same, "f32 n {} {}", n, name);
            }
        }
    });
}

macro_rules! check_records {
    ($($w:literal),*) => {$(
        for &n in sizes() {
            for (name, keys) in shapes(n, 5) {
                // Key, original position (stability check), $w more words.
                let recs: Vec<(u64, u32, [u32; $w])> = keys
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| (k, i as u32, [7; $w]))
                    .collect();
                let bytes = std::mem::size_of::<(u64, u32, [u32; $w])>();
                let mut a = recs.clone();
                devour_sort_by_key(&mut a, |r| r.0);
                let mut e = recs.clone();
                e.sort_by_key(|r| r.0);
                assert!(a == e, "stable {}B n {} {}", bytes, n, name);
                let mut a = recs.clone();
                devour_sort_unstable_by_key(&mut a, |r| r.0);
                let ka: Vec<u64> = a.iter().map(|r| r.0).collect();
                let ke: Vec<u64> = e.iter().map(|r| r.0).collect();
                let mut pa: Vec<u32> = a.iter().map(|r| r.1).collect();
                pa.sort_unstable();
                assert!(ka == ke, "unstable {}B n {} {}", bytes, n, name);
                assert!(pa.iter().enumerate().all(|(i, &p)| p == i as u32), "lost");
            }
        }
    )*};
}

#[test]
fn miri_records() {
    with_small_cache(|| {
        // 16, 24, 32 and 128 bytes.
        check_records!(0, 2, 4, 28);
        // Float keys with NaN (both signs), -0.0, infinities: total order.
        for &n in sizes() {
            for (name, keys) in shapes(n, 13) {
                let mut recs: Vec<(f64, u32)> = keys
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| ((k >> 12) as f64 - 1e15, i as u32))
                    .collect();
                for (j, v) in [f64::NAN, -f64::NAN, -0.0, 0.0, f64::NEG_INFINITY].iter().enumerate()
                {
                    if j < n {
                        recs[j * n / 5].0 = *v;
                    }
                }
                let mut a = recs.clone();
                devour_sort_by_key(&mut a, |r| r.0);
                let mut e = recs.clone();
                e.sort_by(|x, y| x.0.total_cmp(&y.0));
                let same = a
                    .iter()
                    .zip(e.iter())
                    .all(|(x, y)| x.0.to_bits() == y.0.to_bits() && x.1 == y.1);
                assert!(same, "f64 key n {} {}", n, name);
            }
        }
        // Keys narrower than the record (u32 key in 8 bytes, u16 in 4).
        for &n in sizes() {
            for (name, keys) in shapes(n, 9) {
                let recs: Vec<(u32, u32)> =
                    keys.iter().enumerate().map(|(i, &k)| (k as u32, i as u32)).collect();
                let mut a = recs.clone();
                devour_sort_by_key(&mut a, |r| r.0);
                let mut e = recs.clone();
                e.sort_by_key(|r| r.0);
                assert!(a == e, "u32 key n {} {}", n, name);
            }
        }
    });
}
