//! Unit tests of `v2/sorts/ravenous.rs` (a child module: sees its private items).

use crate::v2::testing::*;

/// NaNs set aside: negative ones first, positive ones last, each group
/// in `total_cmp` order, the rest returned without NaN; the signed
/// zeros of a sorted array put -0.0 first.
macro_rules! check {
    ($f:ident, $m:ident, $u:ty) => {
        use super::$m as m;
        let mut r = Rng(61);
        let specials =
            [<$f>::NAN, -<$f>::NAN, <$f>::from_bits(<$f>::NAN.to_bits() | 1), 0.0, -0.0, 1.0];
        for &n in SIZES.iter() {
            for &rate in [0u64, 1, 30, 100].iter() {
                let a: Vec<$f> = (0..n)
                    .map(|_| {
                        if r.next() % 100 < rate {
                            specials[(r.next() % 6) as usize]
                        } else {
                            (r.next() % 100) as $f
                        }
                    })
                    .collect();
                let mut got = a.clone();
                let rest_len = m::nan_aside(&mut got).len();
                let neg = got.iter().take_while(|v| v.is_nan() && v.is_sign_negative()).count();
                let pos = n - neg - rest_len;
                let what = format!("{} n={} rate={}", stringify!($f), n, rate);
                assert!(got[neg..neg + rest_len].iter().all(|v| !v.is_nan()), "{}: NaN left", what);
                assert!(
                    got[neg + rest_len..].iter().all(|v| v.is_nan() && v.is_sign_positive()),
                    "{}",
                    what
                );
                assert_eq!(neg + pos, a.iter().filter(|v| v.is_nan()).count(), "{}", what);
                let ends: Vec<$f> =
                    got[..neg].iter().chain(&got[neg + rest_len..]).copied().collect();
                assert!(
                    ends.windows(2).all(|w| w[0].total_cmp(&w[1]).is_le()),
                    "{}: NaN order",
                    what
                );
                let bits = |v: &[$f]| v.iter().map(|x| x.to_bits()).collect::<Vec<$u>>();
                assert!(same_values(&bits(&got), &bits(&a)), "{}: values changed", what);
                // Signed zeros: sorted by `<` (mixed), then put in order.
                let mut z: Vec<$f> = got[neg..neg + rest_len].to_vec();
                z.sort_unstable_by(|x, y| x.partial_cmp(y).unwrap());
                m::order_signed_zeros(&mut z);
                assert!(z.windows(2).all(|w| w[0].total_cmp(&w[1]).is_le()), "{}: zeros", what);
            }
        }
    };
}

#[test]
fn nan_and_zeros() {
    {
        check!(f32, f32_bits, u32);
    }
    {
        check!(f64, f64_bits, u64);
    }
}
