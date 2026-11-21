//! The public API called as a user calls it: every method of `RadixSort`
//! on slices and `Vec`, the free functions, the edge sizes, a key type of
//! the user (`SortKey`).

use voracious_radix_sort::{devour_sort, ravenous_sort, RadixSort, SortKey};

/// A small deterministic generator (xorshift).
fn values(n: usize, seed: u64) -> Vec<u64> {
    let mut x = seed | 1;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        })
        .collect()
}

#[test]
fn every_method_on_vec_and_slice() {
    for n in [0usize, 1, 2, 3, 31, 32, 33, 64, 65, 1000, 100_000] {
        let base = values(n, 7);
        let mut expected = base.clone();
        expected.sort_unstable();

        let mut v = base.clone();
        v.voracious_sort();
        assert_eq!(v, expected, "Vec voracious_sort n={}", n);

        let mut v = base.clone();
        v.as_mut_slice().voracious_stable_sort();
        assert_eq!(v, expected, "slice voracious_stable_sort n={}", n);

        let mut v = base.clone();
        devour_sort(&mut v);
        assert_eq!(v, expected, "devour_sort n={}", n);

        #[cfg(feature = "voracious_multithread")]
        {
            let mut v = base.clone();
            v.voracious_mt_sort(4);
            assert_eq!(v, expected, "voracious_mt_sort n={}", n);
        }
    }
}

#[test]
fn every_primitive_type() {
    let base = values(5000, 11);
    macro_rules! check {
        ($($t:ty),*) => {$(
            let mut v: Vec<$t> = base.iter().map(|&x| x as $t).collect();
            let mut expected = v.clone();
            expected.sort_unstable();
            v.voracious_sort();
            assert_eq!(v, expected, stringify!($t));
        )*};
    }
    check!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

    let mut b: Vec<bool> = base.iter().map(|&x| x & 1 == 1).collect();
    let mut expected = b.clone();
    expected.sort_unstable();
    b.voracious_sort();
    assert_eq!(b, expected, "bool");

    let mut c: Vec<char> =
        base.iter().map(|&x| char::from_u32((x % 0xD000) as u32).unwrap()).collect();
    let mut expected = c.clone();
    expected.sort_unstable();
    c.voracious_sort();
    assert_eq!(c, expected, "char");

    let mut f: Vec<f64> = base.iter().map(|&x| (x as i64) as f64 / 3.0).collect();
    f.extend([f64::NAN, -f64::NAN, -0.0, 0.0, f64::INFINITY, f64::NEG_INFINITY]);
    let mut expected = f.clone();
    expected.sort_unstable_by(|a, b| a.total_cmp(b));
    let mut g = f.clone();
    f.voracious_sort();
    ravenous_sort(&mut g);
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&f), bits(&expected), "f64 voracious_sort");
    assert_eq!(bits(&g), bits(&expected), "f64 ravenous_sort");

    let mut h: Vec<f32> = base.iter().map(|&x| (x as i32) as f32 / 3.0).collect();
    let mut expected = h.clone();
    expected.sort_unstable_by(|a, b| a.total_cmp(b));
    h.voracious_sort();
    assert_eq!(
        h.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        expected.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        "f32"
    );
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Event {
    time: i64,
    weight: f32,
    id: u32,
}

#[test]
fn records_by_key() {
    for n in [0usize, 1, 2, 100, 10_000, 300_000] {
        let base: Vec<Event> = values(n, 3)
            .iter()
            .enumerate()
            .map(|(i, &x)| Event {
                time: (x % 1000) as i64 - 500,
                weight: (x >> 40) as f32 - 1e5,
                id: i as u32,
            })
            .collect();

        let mut expected = base.clone();
        expected.sort_by_key(|e| e.time);
        let mut v = base.clone();
        v.voracious_sort_by_key(|e| e.time);
        assert_eq!(v, expected, "stable by i64 key n={}", n);

        let mut expected = base.clone();
        expected.sort_by(|a, b| a.weight.total_cmp(&b.weight));
        let mut v = base.clone();
        v.voracious_sort_by_key(|e| e.weight);
        assert_eq!(v, expected, "stable by f32 key n={}", n);

        let mut v = base.clone();
        v.voracious_sort_unstable_by_key(|e| e.time);
        assert!(v.windows(2).all(|w| w[0].time <= w[1].time), "unstable order n={}", n);
        let mut ids: Vec<u32> = v.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        assert!(
            ids.iter().enumerate().all(|(i, &id)| id == i as u32),
            "unstable keeps every record n={}",
            n
        );
    }
}

/// A key type of the user: a date compared as (year, month, day).
#[derive(Clone, Copy)]
struct Date {
    year: i16,
    month: u8,
    day: u8,
}

impl SortKey for Date {
    fn encode(self) -> u128 {
        (((self.year as u16 ^ 0x8000) as u128) << 16)
            | ((self.month as u128) << 8)
            | self.day as u128
    }
}

#[test]
fn key_type_of_the_user() {
    let base: Vec<(Date, u32)> = values(20_000, 5)
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            (
                Date {
                    year: (x % 400) as i16 - 200,
                    month: (x >> 20) as u8 % 12 + 1,
                    day: (x >> 30) as u8 % 28 + 1,
                },
                i as u32,
            )
        })
        .collect();
    let mut expected = base.clone();
    expected.sort_by_key(|(d, _)| (d.year, d.month, d.day));
    let mut v = base;
    v.voracious_sort_by_key(|(d, _)| *d);
    assert!(v.iter().map(|x| x.1).eq(expected.iter().map(|x| x.1)));
}
