use super::super::{RadixKey, RadixSort, Radixable};

use super::super::sorts::american_flag_sort::american_flag_sort;
use super::super::sorts::boolean_sort::boolean_sort;
use super::super::sorts::comparative_sort::insertion_sort;
use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::msd_stable_sort::msd_stable_radixsort;
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::ska_sort::ska_sort;
use super::super::sorts::thiel_sort::thiel_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;

use super::super::generators::boolean::*;
use super::super::generators::char::*;
use super::super::generators::custom::*;
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

fn helper_sort_aux<T, K>(
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generator: &dyn Fn(usize) -> Vec<T>,
    runs: usize,
    array_size: usize,
    stable: bool,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    for _ in 0..runs {
        let mut array = generator(array_size);
        let mut check = array.to_vec();
        sort(&mut array);
        if stable {
            check.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        }
        assert_eq!(check, array);
    }
}

fn helper_sort<T, K>(
    stable: bool,
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
    runs: usize,
    array_size: usize,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    generators.iter().for_each(|(generator, _gen_name)| {
        // println!("generator name: {}", _gen_name);
        helper_sort_aux(sort, generator, runs, array_size, stable);
    });
}

#[test]
fn test_sort_boolean_sort() {
    helper_sort(false, &|a| boolean_sort(a), generators_bool(), 1, 0);
    helper_sort(false, &|a| boolean_sort(a), generators_bool(), 1, 1);
    helper_sort(false, &|a| boolean_sort(a), generators_bool(), 2, 10_000);
}

#[test]
fn test_sort_counting_sort_u8() {
    helper_sort(true, &|a| counting_sort(a, 8), generators_u8(), 1, 0);
    helper_sort(true, &|a| counting_sort(a, 8), generators_u8(), 1, 1);
    helper_sort(true, &|a| counting_sort(a, 8), generators_u8(), 2, 10_000);
}

#[test]
fn test_sort_counting_sort_u16() {
    helper_sort(true, &|a| counting_sort(a, 16), generators_u16(), 1, 0);
    helper_sort(true, &|a| counting_sort(a, 16), generators_u16(), 1, 1);
    helper_sort(true, &|a| counting_sort(a, 16), generators_u16(), 2, 10_000);
}

#[test]
fn test_sort_counting_sort_i8() {
    helper_sort(true, &|a| counting_sort(a, 8), generators_i8(), 1, 0);
    helper_sort(true, &|a| counting_sort(a, 8), generators_i8(), 1, 1);
    helper_sort(true, &|a| counting_sort(a, 8), generators_i8(), 2, 10_000);
}

#[test]
fn test_sort_counting_sort_i16() {
    helper_sort(true, &|a| counting_sort(a, 16), generators_i16(), 1, 0);
    helper_sort(true, &|a| counting_sort(a, 16), generators_i16(), 1, 1);
    helper_sort(true, &|a| counting_sort(a, 16), generators_i16(), 2, 10_000);
}

#[test]
fn test_sort_insertion_sort_bool() {
    helper_sort(true, &|a| insertion_sort(a), generators_bool(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_bool(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_bool(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_char() {
    helper_sort(true, &|a| insertion_sort(a), generators_char(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_char(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_char(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_f32() {
    helper_sort(true, &|a| insertion_sort(a), generators_f32(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_f32(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_f32(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_f64() {
    helper_sort(true, &|a| insertion_sort(a), generators_f64(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_f64(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_f64(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    insertion_sort(&mut a);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_insertion_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    insertion_sort(&mut a);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_insertion_sort_u8() {
    helper_sort(true, &|a| insertion_sort(a), generators_u8(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_u8(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_u8(), 2, 5_000);
}
#[test]
fn test_sort_insertion_sort_u16() {
    helper_sort(true, &|a| insertion_sort(a), generators_u16(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_u16(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_u16(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_u32() {
    helper_sort(true, &|a| insertion_sort(a), generators_u32(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_u32(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_u32(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_u64() {
    helper_sort(true, &|a| insertion_sort(a), generators_u64(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_u64(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_u64(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_u128() {
    helper_sort(true, &|a| insertion_sort(a), generators_u128(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_u128(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_u128(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_i8() {
    helper_sort(true, &|a| insertion_sort(a), generators_i8(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_i8(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_i8(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_i16() {
    helper_sort(true, &|a| insertion_sort(a), generators_i16(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_i16(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_i16(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_i32() {
    helper_sort(true, &|a| insertion_sort(a), generators_i32(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_i32(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_i32(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_i64() {
    helper_sort(true, &|a| insertion_sort(a), generators_i64(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_i64(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_i64(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_i128() {
    helper_sort(true, &|a| insertion_sort(a), generators_i128(), 1, 0);
    helper_sort(true, &|a| insertion_sort(a), generators_i128(), 1, 1);
    helper_sort(true, &|a| insertion_sort(a), generators_i128(), 2, 5_000);
}

#[test]
fn test_sort_american_flag_sort_bool() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_char() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_f32() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_f64() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    american_flag_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_american_flag_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    american_flag_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_american_flag_sort_u8() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_american_flag_sort_u16() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_u32() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_u64() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_u128() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_i8() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_i16() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_i32() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_i64() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_american_flag_sort_i128() {
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| american_flag_sort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_bool() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_char() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_f32() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_f64() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    dlsd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_dlsd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    dlsd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_dlsd_radixsort_u8() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_dlsd_radixsort_u16() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_u32() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_u64() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_u128() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_i8() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_i16() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_i32() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_i64() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_dlsd_radixsort_i128() {
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| dlsd_radixsort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_bool() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_char() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_f32() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_f64() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    lsd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    lsd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_radixsort_u8() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_lsd_radixsort_u16() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_u32() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_u64() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_u64(), 2, 100_000);
}

// Cannot sort u128 with lsd_radixsort because of the histogram

#[test]
fn test_sort_lsd_radixsort_i8() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_i16() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_i32() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_lsd_radixsort_i64() {
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| lsd_radixsort(a, 8), generators_i64(), 2, 100_000);
}

// Cannot sort i128 with lsd_radixsort because of the histogram

#[test]
fn test_sort_msd_stable_radixsort_bool() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_char() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_f32() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_f64() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    msd_stable_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_stable_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    msd_stable_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_stable_radixsort_u8() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_msd_stable_radixsort_u16() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_u32() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_u64() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_u128() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_i8() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_i16() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_i32() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_i64() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_msd_stable_radixsort_i128() {
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| msd_stable_radixsort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_bool() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_char() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_f32() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_f64() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    msd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    msd_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_radixsort_u8() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_msd_radixsort_u16() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_u32() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_u64() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_u128() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_i8() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_i16() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_i32() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_i64() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_msd_radixsort_i128() {
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| msd_radixsort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_bool() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_char() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_f32() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_f64() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    ska_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_ska_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    ska_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_ska_sort_u8() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_ska_sort_u16() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_u32() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_u64() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_u128() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_i8() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_i16() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_i32() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_i64() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_ska_sort_i128() {
    helper_sort(false, &|a| ska_sort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| ska_sort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_bool() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_char() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_f32() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_f64() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    thiel_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_thiel_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    thiel_radixsort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_thiel_radixsort_u8() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_thiel_radixsort_u16() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_u32() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_u64() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_u128() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_i8() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_i16() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_i32() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_i64() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_thiel_radixsort_i128() {
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| thiel_radixsort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_bool() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_bool(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_bool(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_bool(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_char() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_char(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_char(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_f32() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f32(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f32(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_f64() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f64(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f64(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    voracious_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_voracious_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    voracious_sort(&mut a, 8);
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_voracious_sort_u8() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u8(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u8(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_voracious_sort_u16() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u16(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u16(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_u32() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u32(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u32(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_u64() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u64(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u64(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_u128() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u128(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u128(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_i8() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i8(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i8(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_i16() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i16(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i16(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_i32() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i32(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i32(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_i64() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i64(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i64(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_voracious_sort_i128() {
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i128(), 1, 0);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i128(), 1, 1);
    helper_sort(false, &|a| voracious_sort(a, 8), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_char() {
    helper_sort(false, &|a| a.voracious_sort(), generators_char(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_char(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_f32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_f32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_f32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_f64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_f64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_f64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    a.voracious_sort();
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    a.voracious_sort();
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_u8() {
    helper_sort(false, &|a| a.voracious_sort(), generators_u8(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_u8(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_trait_voracious_u16() {
    helper_sort(false, &|a| a.voracious_sort(), generators_u16(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_u16(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_u32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_u32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_u32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_u64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_u64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_u64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_u128() {
    helper_sort(false, &|a| a.voracious_sort(), generators_u128(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_u128(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_i8() {
    helper_sort(false, &|a| a.voracious_sort(), generators_i8(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_i8(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_i16() {
    helper_sort(false, &|a| a.voracious_sort(), generators_i16(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_i16(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_i32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_i32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_i32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_i64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_i64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_i64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_i128() {
    helper_sort(false, &|a| a.voracious_sort(), generators_i128(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_i128(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_char() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_char(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_char(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_char(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_f32() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f32(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f32(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_f64() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f64(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f64(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_f64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    a.voracious_stable_sort();
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_stable_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    a.voracious_stable_sort();
    check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_stable_u8() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u8(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u8(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u8(), 2, 100_000);
}
#[test]
fn test_sort_trait_voracious_stable_u16() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u16(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u16(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_u32() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u32(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u32(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_u64() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u64(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u64(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_u128() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u128(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u128(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_u128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_i8() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i8(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i8(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_i16() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i16(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i16(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_i32() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i32(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i32(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_i64() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i64(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i64(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_i128() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i128(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i128(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_i128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structbool() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structbool(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structbool(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structbool(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structchar() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structchar(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structchar(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structchar(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structf32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structf32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structf32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structf32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structf64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structf64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structf64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structf64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structu8() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structu8(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu8(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structu16() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structu16(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu16(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structu32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structu32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structu64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structu64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structu128() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structu128(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu128(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structu128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structi8() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structi8(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi8(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structi16() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structi16(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi16(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structi32() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structi32(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi32(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structi64() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structi64(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi64(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_structi128() {
    helper_sort(false, &|a| a.voracious_sort(), generators_structi128(), 1, 0);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi128(), 1, 1);
    helper_sort(false, &|a| a.voracious_sort(), generators_structi128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structbool() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structbool(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structbool(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structbool(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structchar() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structchar(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structchar(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structchar(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structf32() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf32(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf32(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structf64() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf64(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf64(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structf64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structu8() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu8(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu8(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structu16() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu16(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu16(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structu32() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu32(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu32(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structu64() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu64(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu64(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structu128() {
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu128(), 1, 0);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu128(), 1, 1);
    helper_sort(true, &|a| a.voracious_stable_sort(), generators_structu128(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structi8() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi8(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi8(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi8(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structi16() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi16(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi16(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi16(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structi32() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi32(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi32(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi32(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structi64() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi64(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi64(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi64(), 2, 100_000);
}

#[test]
fn test_sort_trait_voracious_stable_structi128() {
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi128(), 1, 0);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi128(), 1, 1);
    helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi128(), 2, 100_000);
}
