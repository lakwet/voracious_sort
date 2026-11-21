use super::super::generators::char::*;
use super::super::generators::float_32::*;
use super::super::generators::float_64::*;
use super::super::generators::signed_i128::*;
use super::super::generators::signed_i16::*;
use super::super::generators::signed_i32::*;
use super::super::generators::signed_i64::*;
use super::super::generators::signed_i8::*;
use super::super::generators::unsigned_u128::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;
use super::super::generators::unsigned_u64::*;
use super::super::generators::unsigned_u8::*;
use crate::v1::sorts::utils::Params;
use crate::v1::traits::radix_key::RadixKey;
use crate::v1::Radixable;

// The dedicated `get_full_histograms` of each type must count, at each
// level, the same digits as a plain one-element-at-a-time count. A level
// missing in the unrolled loop made the f64 LSD panic (1.2.1).
//
// The radixes the trait methods use: 8 at every number of levels (the
// rollercoaster asks the LSD for the remaining levels only), and for char 7
// and 11 with the offset and the levels the LSD computes.
fn check_histograms<T, K>(arr: &mut [T], bits: usize, radixes: &[usize])
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut cases = Vec::new();
    for max_level in 1..=bits / 8 {
        cases.push((8, max_level));
    }
    for &radix in radixes {
        let (offset, _) = dummy.compute_offset(arr, radix);
        let max_level = dummy.compute_max_level(offset, radix);
        if max_level > 0 {
            cases.push((radix, max_level));
        }
    }
    for (radix, max_level) in cases {
        let p = Params::new(0, radix, 0, max_level);
        let mask = dummy.default_mask(radix);
        let shift = dummy.usize_to_keytype(radix);
        let mut expected = vec![vec![0usize; p.radix_range]; max_level];
        for item in arr.iter() {
            let mut v = item.into_key_type();
            for level in (0..max_level).rev() {
                expected[level][dummy.keytype_to_usize(v & mask)] += 1;
                v = v >> shift;
            }
        }
        let got = dummy.get_full_histograms(arr, &p);
        assert!(got == expected, "{} bits, radix {}, {} levels", bits, radix, max_level);
    }
}

#[test]
fn test_histograms_all_levels() {
    // 4 * 1000 + 3: the unrolled loop and the remainder.
    let n = 4_003;
    check_histograms(&mut helper_random_array_uniform_u8(n), 8, &[]);
    check_histograms(&mut helper_random_array_uniform_u16(n), 16, &[]);
    check_histograms(&mut helper_random_array_uniform_u32(n), 32, &[]);
    check_histograms(&mut helper_random_array_uniform_u64(n), 64, &[]);
    check_histograms(&mut helper_random_array_uniform_u128(n), 128, &[]);
    check_histograms(&mut helper_random_array_uniform_i8(n), 8, &[]);
    check_histograms(&mut helper_random_array_uniform_i16(n), 16, &[]);
    check_histograms(&mut helper_random_array_uniform_i32(n), 32, &[]);
    check_histograms(&mut helper_random_array_uniform_i64(n), 64, &[]);
    check_histograms(&mut helper_random_array_uniform_i128(n), 128, &[]);
    check_histograms(&mut helper_random_array_uniform_f32(n), 32, &[]);
    check_histograms(&mut helper_random_array_uniform_f64(n), 64, &[]);
    let mut us: Vec<usize> =
        helper_random_array_uniform_u64(n).iter().map(|&x| x as usize).collect();
    check_histograms(&mut us, 8 * std::mem::size_of::<usize>(), &[]);
    check_histograms(&mut helper_random_array_uniform_char(n), 32, &[7, 11]);
    let mut is: Vec<isize> =
        helper_random_array_uniform_i64(n).iter().map(|&x| x as isize).collect();
    check_histograms(&mut is, 8 * std::mem::size_of::<isize>(), &[]);
}

// u128 keys of every width: the LSD on 120-bit keys sorts 15 levels, whose
// remainder loop missed a level and panicked (1.2.1).
#[test]
fn test_lsd_u128_every_width() {
    use crate::v1::sorts::lsd_sort::lsd_radixsort;
    use crate::v1::RadixSort;
    for bits in (8..=128).step_by(8) {
        for n in [1_001usize, 5_003, 100_003] {
            let a: Vec<u128> = helper_random_array_uniform_u128(n)
                .iter()
                .map(|&x| if bits == 128 { x } else { x >> (128 - bits) })
                .collect();
            let mut expected = a.clone();
            expected.sort_unstable();
            let mut c = a.clone();
            lsd_radixsort(&mut c, 8);
            assert!(c == expected, "lsd_radixsort, {} bits, {} keys", bits, n);
            let mut c = a.clone();
            c.voracious_sort();
            assert!(c == expected, "voracious_sort, {} bits, {} keys", bits, n);
        }
    }
}
