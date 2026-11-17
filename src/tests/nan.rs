//! NaN in the float sorts (1.2.2): no panic, negative NaNs first, positive
//! NaNs last, the other values sorted, nothing lost; equal keys of a stable
//! sort keep their order.

use std::cmp::Ordering;

#[cfg(feature = "voracious_multithread")]
use super::super::peeka_sort;
use super::super::{
    american_flag_sort, counting_sort, dlsd_radixsort, lsd_radixsort, lsd_stable_radixsort,
    msd_radixsort, msd_stable_radixsort, rollercoaster_sort, ska_sort, thiel_radixsort,
    voracious_sort, RadixSort, Radixable,
};

const SIZES: [usize; 9] = [0, 1, 2, 3, 10, 100, 1000, 10_000, 200_000];

/// Values with NaNs of both signs every `every` elements (none if 0),
/// plus -0.0, +0.0, infinities.
fn values(n: usize, every: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            if every > 0 && i % every == 0 {
                if i % (2 * every) == 0 {
                    f64::NAN
                } else {
                    -f64::NAN
                }
            } else {
                match i % 13 {
                    0 => -0.0,
                    1 => 0.0,
                    2 => f64::INFINITY,
                    3 => f64::NEG_INFINITY,
                    _ => ((i * 7919) % 1001) as f64 - 500.5,
                }
            }
        })
        .collect()
}

fn check<F: Copy + PartialOrd + std::fmt::Debug>(
    before: &[F],
    after: &[F],
    neg_nan: impl Fn(F) -> bool,
    bits: impl Fn(F) -> u64,
    what: &str,
) {
    assert_eq!(before.len(), after.len(), "{}", what);
    let is_nan = |x: F| x.partial_cmp(&x).is_none();
    let a = after.iter().take_while(|&&x| is_nan(x) && neg_nan(x)).count();
    let b = after.len() - after.iter().rev().take_while(|&&x| is_nan(x) && !neg_nan(x)).count();
    assert!(a <= b, "{}", what);
    let mid = &after[a..b];
    assert!(mid.iter().all(|&x| !is_nan(x)), "{}: NaN in the middle", what);
    assert!(mid.windows(2).all(|w| w[0] <= w[1]), "{}: not sorted", what);
    let mut x: Vec<u64> = before.iter().map(|&v| bits(v)).collect();
    let mut y: Vec<u64> = after.iter().map(|&v| bits(v)).collect();
    x.sort_unstable();
    y.sort_unstable();
    assert!(x == y, "{}: values lost", what);
}

macro_rules! all_sorts {
    ($t:ty, $conv:expr, $neg:expr, $bits:expr) => {{
        let sorts: Vec<(&str, Box<dyn Fn(&mut Vec<$t>)>)> = vec![
            ("voracious_sort()", Box::new(|v: &mut Vec<$t>| v.voracious_sort())),
            ("voracious_stable_sort()", Box::new(|v: &mut Vec<$t>| v.voracious_stable_sort())),
            #[cfg(feature = "voracious_multithread")]
            ("voracious_mt_sort()", Box::new(|v: &mut Vec<$t>| v.voracious_mt_sort(4))),
            ("american_flag_sort", Box::new(|v: &mut Vec<$t>| american_flag_sort(v, 8))),
            ("counting_sort", Box::new(|v: &mut Vec<$t>| counting_sort(v, 8))),
            ("dlsd_radixsort", Box::new(|v: &mut Vec<$t>| dlsd_radixsort(v, 8))),
            ("lsd_radixsort", Box::new(|v: &mut Vec<$t>| lsd_radixsort(v, 8))),
            ("lsd_stable_radixsort", Box::new(|v: &mut Vec<$t>| lsd_stable_radixsort(v, 8))),
            ("msd_radixsort", Box::new(|v: &mut Vec<$t>| msd_radixsort(v, 8))),
            ("msd_stable_radixsort", Box::new(|v: &mut Vec<$t>| msd_stable_radixsort(v, 8))),
            ("rollercoaster_sort", Box::new(|v: &mut Vec<$t>| rollercoaster_sort(v, 8))),
            ("ska_sort", Box::new(|v: &mut Vec<$t>| ska_sort(v, 8))),
            ("thiel_radixsort", Box::new(|v: &mut Vec<$t>| thiel_radixsort(v, 8))),
            ("voracious_sort", Box::new(|v: &mut Vec<$t>| voracious_sort(v, 8))),
            #[cfg(feature = "voracious_multithread")]
            ("peeka_sort", Box::new(|v: &mut Vec<$t>| peeka_sort(v, 8, 100_000, 4))),
        ];
        for (name, sort) in sorts.iter() {
            for &n in SIZES.iter() {
                // counting_sort: integers in a narrow range only (its
                // histogram spans the key range).
                if *name == "counting_sort" && n > 0 {
                    continue;
                }
                for &every in [0usize, 1, 3, 100, 5000].iter() {
                    let before: Vec<$t> = values(n, every).into_iter().map($conv).collect();
                    let mut after = before.clone();
                    sort(&mut after);
                    let what = format!("{} {} n {} nan every {}", name, stringify!($t), n, every);
                    check(&before, &after, $neg, $bits, &what);
                }
            }
        }
    }};
}

#[test]
fn test_nan_f64_all_sorts() {
    all_sorts!(f64, |x| x, |x: f64| x.is_sign_negative(), |x: f64| x.to_bits());
}

#[test]
fn test_nan_f32_all_sorts() {
    all_sorts!(
        f32,
        |x: f64| x as f32,
        |x: f32| x.is_sign_negative(),
        |x: f32| x.to_bits() as u64
    );
}

#[derive(Clone, Copy, Debug)]
struct Rec {
    key: f64,
    pos: usize,
}

impl PartialEq for Rec {
    fn eq(&self, other: &Self) -> bool { self.key == other.key }
}

impl PartialOrd for Rec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Radixable<f64> for Rec {
    type Key = f64;

    fn key(&self) -> f64 { self.key }
}

#[test]
fn test_nan_struct_float_key() {
    for &n in SIZES.iter() {
        for &every in [0usize, 1, 3, 100].iter() {
            let before: Vec<Rec> = values(n, every)
                .into_iter()
                .enumerate()
                .map(|(pos, k)| Rec { key: (k * 0.01).round(), pos })
                .collect();
            let mut after = before.clone();
            after.voracious_sort();
            let what = format!("struct n {} every {}", n, every);
            check(&before, &after, |r: Rec| r.key.is_sign_negative(), |r: Rec| r.pos as u64, &what);
            // Stable: equal keys keep their order (NaNs included, by bits).
            let mut after = before.clone();
            after.voracious_stable_sort();
            check(&before, &after, |r: Rec| r.key.is_sign_negative(), |r: Rec| r.pos as u64, &what);
            for w in after.windows(2) {
                if w[0].key.to_bits() == w[1].key.to_bits() {
                    assert!(w[0].pos < w[1].pos, "{}: not stable", what);
                }
            }
        }
    }
}
