use rayon::prelude::*;

use super::super::{RadixKey, RadixSort, Radixable};

use super::super::sorts::american_flag_sort::american_flag_sort;
use super::super::sorts::boolean_sort::boolean_sort;
use super::super::sorts::comparative_sort::insertion_sort;
use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::rollercoaster_sort::rollercoaster_sort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::lsd_stable_sort::lsd_stable_radixsort;
use super::super::sorts::msd_stable_sort::msd_stable_radixsort;
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::peeka_sort::peeka_sort;
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

pub fn helper_sort_aux<T, K>(
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generator: &dyn Fn(usize) -> Vec<T>,
    array_size: usize,
    stable: bool,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    let mut array = generator(array_size);
    let mut check = array.to_vec();
    sort(&mut array);
    if stable {
        check.par_sort_by(|a, b| a.partial_cmp(b).unwrap());
    } else {
        check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    }
    assert_eq!(check, array);
}

pub fn helper_sort<T, K>(
    stable: bool,
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
    array_size: usize,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    generators.iter().for_each(|(generator, _gen_name)| {
        // println!("generator name: {}", _gen_name);
        helper_sort_aux(sort, generator, array_size, stable);
    });
}

#[test]
fn test_sort_boolean_sort() {
    for size in [0, 1, 10_000].iter() {
        helper_sort(false, &|a| boolean_sort(a), generators_bool(), *size);
    }
}

#[test]
fn test_sort_counting_sort() {
    for size in [0, 1, 10_000].iter() {
        helper_sort(false, &|a| counting_sort(a, 1), generators_bool(), *size);
        helper_sort(false, &|a| counting_sort(a, 8), generators_u8(), *size);
        helper_sort(false, &|a| counting_sort(a, 16), generators_u16(), *size);
        helper_sort(false, &|a| counting_sort(a, 8), generators_i8(), *size);
        helper_sort(false, &|a| counting_sort(a, 16), generators_i16(), *size);
    }
}

#[test]
fn test_sort_insertion_sort() {
    for size in [0, 1, 5_000].iter() {
        helper_sort(false, &|a| insertion_sort(a), generators_bool(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_char(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_f32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_f64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_u8(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_u16(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_u32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_u64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_u128(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_usize(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_i8(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_i16(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_i32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_i64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_i128(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_isize(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structbool(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structchar(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structf32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structf64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structu8(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structu16(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structu32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structu64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structu128(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structusize(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structi8(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structi16(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structi32(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structi64(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structi128(), *size);
        helper_sort(false, &|a| insertion_sort(a), generators_structisize(), *size);
    }
}

#[test]
fn test_sort_insertion_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    insertion_sort(&mut a);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_insertion_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    insertion_sort(&mut a);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_american_flag_sort() {
    for size in [0, 1, 100, 50_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| american_flag_sort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_american_flag_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    american_flag_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_american_flag_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    american_flag_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_dlsd_radixsort() {
    // This sort has a variable radix, so no need to check other radix.
    for size in [0, 1, 200, 50_000, 100_000].iter() { for radix in [8].iter() {
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| dlsd_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_dlsd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    dlsd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_dlsd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    dlsd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_radixsort() {
    // Can't sort 128bits key (because of the histogram)
    for size in [0, 1, 200, 10_000].iter() { for radix in [8].iter() {
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_u64(), *size);
        // helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_i64(), *size);
        // helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structu64(), *size);
        // helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structi64(), *size);
        // helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| lsd_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_lsd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    lsd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    lsd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_stable_radixsort() {
    // Can't sort 128bits key (because of the histogram)
    for size in [0, 1, 200, 10_000].iter() { for radix in [8].iter() {
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_u64(), *size);
        // helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_i64(), *size);
        // helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structu64(), *size);
        // helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structi64(), *size);
        // helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| lsd_stable_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_lsd_stable_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    lsd_stable_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_lsd_stable_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    lsd_stable_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_stable_radixsort() {
    for size in [0, 1, 200, 10_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| msd_stable_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_msd_stable_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    msd_stable_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_stable_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    msd_stable_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_radixsort() {
    for size in [0, 1, 200, 10_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| msd_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_msd_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    msd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_msd_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    msd_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_ska_sort() {
    for size in [0, 1, 100, 2000, 20_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| ska_sort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| ska_sort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_ska_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    ska_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_ska_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    ska_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_thiel_radixsort() {
    // This sort is not used and not fully implemented, no need to test other radix.
    // It won't work for 128bits key.
    for size in [0, 1, 200, 20_000].iter() { for radix in [8].iter() {
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_u64(), *size);
        // helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_i64(), *size);
        // helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structu64(), *size);
        // helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structi64(), *size);
        // helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| thiel_radixsort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_thiel_radixsort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    thiel_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_thiel_radixsort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    thiel_radixsort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_voracious_sort() {
    for size in [0, 1, 200, 40_000, 100_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| voracious_sort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_voracious_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(40_000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    voracious_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_voracious_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(40_000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    voracious_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_rollercoaster_sort() {
    for size in [0, 1, 200, 40_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_bool(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_char(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_f32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_f64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_u8(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_u16(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_u32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_u64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_u128(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_usize(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_i8(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_i16(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_i32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_i64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_i128(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_isize(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structbool(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structchar(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structf32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structf64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structu8(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structu16(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structu32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structu64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structu128(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structusize(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structi8(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structi16(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structi32(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structi64(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structi128(), *size);
        helper_sort(false, &|a| rollercoaster_sort(a, *radix), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_rollercoaster_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    rollercoaster_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_rollercoaster_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    rollercoaster_sort(&mut a, 8);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_peeka_sort() {
    for size in [0, 1, 10_000, 30_000, 500_000].iter() { for radix in [7, 8].iter() {
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_bool(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_char(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_f32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_f64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_u8(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_u16(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_u32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_u64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_u128(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_usize(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_i8(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_i16(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_i32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_i64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_i128(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_isize(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structbool(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structchar(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structf32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structf64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structu8(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structu16(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structu32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structu64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structu128(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structusize(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structi8(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structi16(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structi32(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structi64(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structi128(), *size);
        helper_sort(false, &|a| peeka_sort(a, *radix, 100_000, 4), generators_structisize(), *size);
    } }
}

#[test]
fn test_sort_peeka_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(40_000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    peeka_sort(&mut a, 8, 100_000, 4);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_peeka_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(40_000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    peeka_sort(&mut a, 8, 100_000, 4);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious() {
    // No need to test Vec<bool> since it is a dedicated sort.
    for size in [0, 1, 200, 400, 500, 20_000, 40_000, 90_000, 200_000, 3_100_000].iter() {
        helper_sort(false, &|a| a.voracious_sort(), generators_char(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_f32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_f64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_u8(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_u16(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_u32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_u64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_u128(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_usize(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_i8(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_i16(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_i32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_i64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_i128(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_isize(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structbool(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structchar(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structf32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structf64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structu8(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structu16(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structu32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structu64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structu128(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structusize(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structi8(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structi16(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structi32(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structi64(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structi128(), *size);
        helper_sort(false, &|a| a.voracious_sort(), generators_structisize(), *size);
    }
}

#[test]
fn test_sort_trait_voracious_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    a.voracious_sort();
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    a.voracious_sort();
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_stable() {
    // No need to test Vec<bool> since it is a dedicated sort.
    for size in [0, 1, 200, 400, 500, 20_000, 40_000, 90_000, 200_000, 3_100_000].iter() {
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_char(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_f32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_f64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_u8(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_u16(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_u32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_u64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_u128(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_usize(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_i8(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_i16(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_i32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_i64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_i128(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_isize(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structbool(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structchar(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structf32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structf64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structu8(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structu16(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structu32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structu64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structu128(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structusize(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi8(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi16(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi32(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi64(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structi128(), *size);
        helper_sort(false, &|a| a.voracious_stable_sort(), generators_structisize(), *size);
    }
}

#[test]
fn test_sort_trait_voracious_stable_f32_inf() {
    let mut a = helper_random_array_uniform_f32(2_000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    a.voracious_stable_sort();
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_stable_f64_inf() {
    let mut a = helper_random_array_uniform_f64(2_000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    a.voracious_stable_sort();
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_mt_sort() {
    for size in [0, 1, 10_000, 30_000, 500_000].iter() {
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_bool(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_char(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_f32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_f64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_u8(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_u16(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_u32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_u64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_u128(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_usize(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_i8(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_i16(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_i32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_i64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_i128(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_isize(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structbool(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structchar(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structf32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structf64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structu8(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structu16(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structu32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structu64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structu128(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structusize(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structi8(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structi16(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structi32(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structi64(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structi128(), *size);
        helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structisize(), *size);
    }
}

#[test]
fn test_sort_trait_voracious_mt_sort_f32_inf() {
    let mut a = helper_random_array_uniform_f32(40_000);
    a[1999] = std::f32::NEG_INFINITY;
    a[0] = std::f32::INFINITY;
    let mut check = a.to_vec();
    a.voracious_mt_sort(4);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}

#[test]
fn test_sort_trait_voracious_mt_sort_f64_inf() {
    let mut a = helper_random_array_uniform_f64(40_000);
    a[1999] = std::f64::NEG_INFINITY;
    a[0] = std::f64::INFINITY;
    let mut check = a.to_vec();
    a.voracious_mt_sort(4);
    check.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(a, check);
}
