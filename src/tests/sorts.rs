use super::super::{RadixSort, Radixable, RadixableForContainer};

use super::super::sorts::american_flag_sort::american_flag_sort;
use super::super::sorts::boolean_sort::boolean_sort;
use super::super::sorts::comparative_sort::insertion_sort;
use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::msd_string_sort::msd_string_radixsort;
use super::super::sorts::ska_sort::ska_sort;
use super::super::sorts::thiel_sort::thiel_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;

use super::super::generators::boolean::*;
use super::super::generators::char::*;
use super::super::generators::custom::*;
use super::super::generators::float_32::*;
use super::super::generators::float_64::*;
use super::super::generators::signed_i16::*;
use super::super::generators::signed_i32::*;
use super::super::generators::signed_i64::*;
use super::super::generators::signed_i8::*;
use super::super::generators::string::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;
use super::super::generators::unsigned_u64::*;
use super::super::generators::unsigned_u8::*;

use super::super::generators::tuple::*;

fn helper_sort_aux<T>(
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generator: &dyn Fn(usize) -> Vec<T>,
    runs: usize,
    array_size: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>,
    T: Copy + PartialOrd + std::fmt::Debug,
    [T]: RadixableForContainer<T = T>,
{
    for _ in 0..runs {
        let mut array = generator(array_size);
        let mut check = array.to_vec();
        sort(&mut array);
        check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(check, array);
    }
}

fn helper_sort<T>(
    sort: &dyn Fn(&mut Vec<T>) -> (),
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
    runs: usize,
    array_size: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>,
    T: Copy + PartialOrd + std::fmt::Debug,
    [T]: RadixableForContainer<T = T>,
{
    generators.iter().for_each(|(generator, gen_name)| {
        println!("generator name: {}", gen_name);
        helper_sort_aux(sort, generator, runs, array_size);
    });
}

#[test]
fn test_sort_insertion_sort_u32() {
    helper_sort(&|arr| insertion_sort(arr), generators_u32(), 2, 5_000);
}

#[test]
fn test_sort_insertion_sort_u64() {
    helper_sort(&|arr| insertion_sort(arr), generators_u64(), 2, 5_000);
}

#[test]
fn test_sort_american_flag_sort_u8() {
    helper_sort(
        &|arr| american_flag_sort(arr, 8),
        generators_u8(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_american_flag_sort_u16() {
    helper_sort(
        &|arr| american_flag_sort(arr, 8),
        generators_u16(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_american_flag_sort_u32() {
    helper_sort(
        &|arr| american_flag_sort(arr, 8),
        generators_u32(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_american_flag_sort_u64() {
    helper_sort(
        &|arr| american_flag_sort(arr, 8),
        generators_u64(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_boolean_sort() {
    helper_sort(&|arr| boolean_sort(arr), generators_bool(), 10, 10_000);
}

#[test]
fn test_sort_counting_sort_u8() {
    helper_sort(&|arr| counting_sort(arr, 8), generators_u8(), 10, 10_000);
}

#[test]
fn test_sort_lsd_sort_u8() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_lsd_sort_u16() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_lsd_sort_u32() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_lsd_sort_u64() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_u8() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_u16() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_u32() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_u64() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_ska_sort_u8() {
    helper_sort(&|arr| ska_sort(arr, 8), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_ska_sort_u16() {
    helper_sort(&|arr| ska_sort(arr, 8), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_ska_sort_u32() {
    helper_sort(&|arr| ska_sort(arr, 8), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_ska_sort_u64() {
    helper_sort(&|arr| ska_sort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_thiel_sort_u8() {
    helper_sort(&|arr| thiel_radixsort(arr, 8), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_thiel_sort_u16() {
    helper_sort(&|arr| thiel_radixsort(arr, 8), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_thiel_sort_u32() {
    helper_sort(&|arr| thiel_radixsort(arr, 8), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_thiel_sort_u64() {
    helper_sort(&|arr| thiel_radixsort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_raw_voracious_sort_u8() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_raw_voracious_sort_u16() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_raw_voracious_sort_u32() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_raw_voracious_sort_u64() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_raw_dlsd_sort_u64() {
    helper_sort(&|arr| dlsd_radixsort(arr, 8), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_msd_string_sort_string() {
    let runs = 2;
    let size = 100_000;

    for _ in 0..runs {
        let array = helper_random_array_uniform_string(size, 40);
        let mut array = array
            .iter()
            .map(|element| unsafe { element.get_unchecked(..) })
            .collect::<Vec<&str>>();
        let mut check = array.to_vec();

        msd_string_radixsort(&mut array);

        check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(check, array);
    }
}

#[test]
fn test_sort_raw_voracious_sort_f32() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_f32(), 2, 500_000);
}

#[test]
fn test_sort_raw_voracious_sort_f64() {
    helper_sort(&|arr| voracious_sort(arr, 8), generators_f64(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_f32() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_f32(), 2, 500_000);
}

#[test]
fn test_sort_msd_sort_f64() {
    helper_sort(&|arr| msd_radixsort(arr, 8), generators_f64(), 2, 500_000);
}

#[test]
fn test_sort_lsd_sort_f32() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_f32(), 2, 500_000);
}

#[test]
fn test_sort_lsd_sort_f64() {
    helper_sort(&|arr| lsd_radixsort(arr, 8), generators_f64(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_bool() {
    helper_sort(&|arr| arr.voracious_sort(), generators_bool(), 2, 10_000);
}

#[test]
fn test_sort_voracious_sort_char() {
    helper_sort(&|arr| arr.voracious_sort(), generators_char(), 4, 500_000);
}

#[test]
fn test_sort_voracious_sort_u8() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_u16() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_u32() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_u64() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_i8() {
    helper_sort(&|arr| arr.voracious_sort(), generators_i8(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_i16() {
    helper_sort(&|arr| arr.voracious_sort(), generators_i16(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_i32() {
    helper_sort(&|arr| arr.voracious_sort(), generators_i32(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_i64() {
    helper_sort(&|arr| arr.voracious_sort(), generators_i64(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_boolbool() {
    helper_sort(
        &|arr| arr.voracious_sort(),
        generators_boolbool(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_voracious_sort_tuple_boolu8() {
    helper_sort(&|arr| arr.voracious_sort(), generators_boolu8(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_u8bool() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u8bool(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_boolu16() {
    helper_sort(
        &|arr| arr.voracious_sort(),
        generators_boolu16(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_voracious_sort_tuple_u8u8() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u8u8(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_u32u32() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u32u32(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_u64u64() {
    helper_sort(&|arr| arr.voracious_sort(), generators_u64u64(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_tuple_i64i64() {
    helper_sort(&|arr| arr.voracious_sort(), generators_i64i64(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_string() {
    let runs = 2;
    let size = 100_000;

    for _ in 0..runs {
        let arr = helper_random_array_uniform_string(size, 40);
        let mut arr = arr
            .iter()
            .map(|element| unsafe { element.get_unchecked(..) })
            .collect::<Vec<&str>>();
        let mut check = arr.to_vec();

        arr.voracious_sort();

        check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(check, arr);
    }
}

#[test]
fn test_sort_voracious_sort_f32() {
    helper_sort(&|arr| arr.voracious_sort(), generators_f32(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_f64() {
    helper_sort(&|arr| arr.voracious_sort(), generators_f64(), 2, 500_000);
}

#[test]
#[should_panic]
fn test_sort_voracious_sort_f32_panic() {
    let mut array = vec![0.0, 12.3, 37.122, -27.872, std::f32::NAN, -18.001];
    array.voracious_sort()
}

#[test]
#[should_panic]
fn test_sort_voracious_sort_f64_panic() {
    let mut array = vec![0.0, 12.3, 37.122, -27.872, std::f64::NAN, -18.001];
    array.voracious_sort()
}

#[test]
fn test_sort_voracious_sort_custom() {
    helper_sort(&|arr| arr.voracious_sort(), generators_custom(), 2, 500_000);
}

#[test]
fn test_sort_voracious_sort_my_struct() {
    helper_sort(
        &|arr| arr.voracious_sort(),
        generators_mystruct(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_voracious_sort_structwithf64() {
    helper_sort(
        &|arr| arr.voracious_sort(),
        generators_structwithf64(),
        2,
        500_000,
    );
}

#[test]
fn test_sort_dlsd_sort_bool() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_bool(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_char() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_char(), 4, 500_000);
}

#[test]
fn test_sort_dlsd_sort_u8() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u8(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_u16() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u16(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_u32() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u32(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_u64() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u64(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_i8() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_i8(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_i16() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_i16(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_i32() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_i32(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_i64() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_i64(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_boolbool() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_boolbool(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_boolu16() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_boolu16(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_u8u8() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u8u8(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_boolu8() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_boolu8(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_u8bool() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u8bool(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_u32u32() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u32u32(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_u64u64() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_u64u64(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_tuple_i64i64() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_i64i64(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_string() {
    let runs = 2;
    let size = 1_000;

    for _ in 0..runs {
        let arr = helper_random_array_uniform_string(size, 20);
        let mut arr = arr
            .iter()
            .map(|element| unsafe { element.get_unchecked(..) })
            .collect::<Vec<&str>>();
        let mut check = arr.to_vec();

        arr.dlsd_sort();

        check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(check, arr);
    }
}

#[test]
fn test_sort_dlsd_sort_f32() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_f32(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_f64() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_f64(), 2, 500_000);
}

#[test]
#[should_panic]
fn test_sort_dlsd_sort_f32_panic() {
    let mut array = vec![0.0, 12.3, 37.122, -27.872, std::f32::NAN, -18.001];
    array.dlsd_sort()
}

#[test]
#[should_panic]
fn test_sort_dlsd_sort_f64_panic() {
    let mut array = vec![0.0, 12.3, 37.122, -27.872, std::f64::NAN, -18.001];
    array.dlsd_sort()
}

#[test]
fn test_sort_dlsd_sort_custom() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_custom(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_mystruct() {
    helper_sort(&|arr| arr.dlsd_sort(), generators_mystruct(), 2, 500_000);
}

#[test]
fn test_sort_dlsd_sort_structwithf64() {
    helper_sort(
        &|arr| arr.dlsd_sort(),
        generators_structwithf64(),
        2,
        500_000,
    );
}
