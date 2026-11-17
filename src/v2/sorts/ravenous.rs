//! # Ravenous sort: radix sort for `f32` and `f64`
//!
//! Floats are sorted through their bits, encoded so that unsigned order is
//! the [`f64::total_cmp`] order (`key = bits ^ (sign ? 0xFF..FF : 0x80..00)`;
//! `-0.0 < +0.0`). The arrays are only reinterpreted, never copied.
//!
//! The public entry and its trait; each type sorts through its own decision
//! chain (`types::f32`, `types::f64`). Shared here: the float encodings and
//! helpers (`f32_bits`, `f64_bits`).

use crate::v2::engines::small::{merge_sort, MERGE_MAX};

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// Float types sortable by [`ravenous_sort2`].
pub trait RavenousSort2: sealed::Sealed + Copy + PartialOrd {
    #[doc(hidden)]
    fn ravenous_sort2_impl(arr: &mut [Self]);
}

/// Sorts `f32` or `f64` slices in [`f64::total_cmp`] order.
///
/// ```text
/// use voracious_radix_sort::ravenous_sort2;
///
/// let mut v = vec![1.5f64, -0.5, 3.0, -2.25];
/// ravenous_sort2(&mut v);
/// assert_eq!(v, vec![-2.25, -0.5, 1.5, 3.0]);
/// ```
#[inline]
pub fn ravenous_sort2<T: RavenousSort2>(arr: &mut [T]) { T::ravenous_sort2_impl(arr); }

macro_rules! float_width {
    ($float:ty, $uint:ty, $sint:ty, $m:ident) => {
        pub(crate) mod $m {
            /// The floats as their raw bits.
            #[inline(always)]
            pub(crate) fn bits(arr: &mut [$float]) -> &mut [$uint] {
                // SAFETY: same size and alignment, every bit pattern valid
                // for both; the slice reborrows `arr` exclusively.
                unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut $uint, arr.len()) }
            }

            #[inline(always)]
            pub(crate) fn encode(bits: $uint) -> $uint {
                const SIGN: $uint = 1 << (<$uint>::BITS - 1);
                bits ^ ((((bits as $sint) >> (<$uint>::BITS - 1)) as $uint) | SIGN)
            }

            #[inline(always)]
            pub(crate) fn decode(key: $uint) -> $uint {
                const SIGN: $uint = 1 << (<$uint>::BITS - 1);
                key ^ ((key >> (<$uint>::BITS - 1)).wrapping_sub(1) | SIGN)
            }

            /// Small arrays: encode, sort as integers, decode.
            pub(crate) fn sort_small(bits: &mut [$uint]) {
                if bits.len() <= super::MERGE_MAX {
                    super::merge_sort(bits, encode, decode);
                    return;
                }
                bits.iter_mut().for_each(|b| *b = encode(*b));
                bits.sort_unstable();
                bits.iter_mut().for_each(|b| *b = decode(*b));
            }

            /// -0.0 and +0.0 compare equal in the generic steps of the radix
            /// path (repair, Verge, merges): they come out together but
            /// mixed. Their block is found by two binary searches and
            /// rewritten, -0.0 first.
            pub(crate) fn order_signed_zeros(arr: &mut [$float]) {
                let lo = arr.partition_point(|x| *x < 0.0);
                let hi = lo + arr[lo..].partition_point(|x| *x <= 0.0);
                if hi - lo < 2 {
                    return;
                }
                let negative = arr[lo..hi].iter().filter(|x| x.is_sign_negative()).count();
                arr[lo..lo + negative].fill(-0.0);
                arr[lo + negative..hi].fill(0.0);
            }

            /// NaNs moved to the ends in `total_cmp` order (negative first,
            /// positive last); the rest returned. One vectorised read when
            /// there is none (the generic steps of the radix path compare
            /// with `<`: a NaN broke the repair, Verge, the merges).
            #[inline]
            pub(crate) fn nan_aside(arr: &mut [$float]) -> &mut [$float] {
                if !arr.iter().fold(false, |a, x| a | x.is_nan()) {
                    return arr;
                }
                split_nan(arr)
            }

            #[cold]
            #[inline(never)]
            fn split_nan(arr: &mut [$float]) -> &mut [$float] {
                let mut nans: Vec<$float> = arr.iter().copied().filter(|x| x.is_nan()).collect();
                nans.sort_unstable_by(|x, y| x.total_cmp(y));
                let neg = nans.iter().filter(|x| x.is_sign_negative()).count();
                let n = arr.len();
                let mut kept = 0;
                for i in 0..n {
                    let x = arr[i];
                    if !x.is_nan() {
                        arr[kept] = x;
                        kept += 1;
                    }
                }
                // Kept values to the middle, then the NaNs around them.
                arr.copy_within(0..kept, neg);
                arr[..neg].copy_from_slice(&nans[..neg]);
                arr[neg + kept..].copy_from_slice(&nans[neg..]);
                &mut arr[neg..neg + kept]
            }

            /// Total order on the floats (for the heavy hitters detector).
            #[inline(always)]
            pub(crate) fn less(x: $float, y: $float) -> bool { x.total_cmp(&y).is_lt() }

            #[inline(always)]
            pub(crate) fn same(x: $float, y: $float) -> bool { x.to_bits() == y.to_bits() }
        }
    };
}

float_width!(f32, u32, i32, f32_bits);
float_width!(f64, u64, i64, f64_bits);

impl RavenousSort2 for f32 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f32]) { crate::v2::types::f32::sort(arr); }
}

impl RavenousSort2 for f64 {
    #[inline]
    fn ravenous_sort2_impl(arr: &mut [f64]) { crate::v2::types::f64::sort(arr); }
}

#[cfg(test)]
mod tests {
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
                    assert!(
                        got[neg..neg + rest_len].iter().all(|v| !v.is_nan()),
                        "{}: NaN left",
                        what
                    );
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
}
