#![allow(dead_code)]

use std::time::Instant;

#[allow(unused_imports)]
use afsort::AFSortable;
#[allow(unused_imports)]
use rdxsort::*;
#[allow(unused_imports)]
use rayon::prelude::*;


use super::super::{RadixSort, Radixable};

#[allow(unused_imports)]
use super::super::sorts::american_flag_sort::american_flag_sort;
#[allow(unused_imports)]
use super::super::sorts::boolean_sort::boolean_sort;
#[allow(unused_imports)]
use super::super::sorts::comparative_sort::insertion_sort;
#[allow(unused_imports)]
use super::super::sorts::counting_sort::counting_sort;
#[allow(unused_imports)]
use super::super::sorts::dlsd_sort::dlsd_radixsort;
#[allow(unused_imports)]
use super::super::sorts::lsd_mt_sort::lsd_mt_radixsort;
#[allow(unused_imports)]
use super::super::sorts::lsd_sort::lsd_radixsort;
#[allow(unused_imports)]
use super::super::sorts::msd_sort::msd_radixsort;
#[allow(unused_imports)]
use super::super::sorts::ska_sort::ska_sort;
#[allow(unused_imports)]
use super::super::sorts::regions_sort::regions_sort;
#[allow(unused_imports)]
use super::super::sorts::thiel_sort::thiel_radixsort;
#[allow(unused_imports)]
use super::super::sorts::voracious_sort::voracious_sort;

use super::super::generators::boolean::*;
use super::super::generators::char::*;
#[allow(unused_imports)]
use super::super::generators::custom::*;
#[allow(unused_imports)]
use super::super::generators::float_32::*;
#[allow(unused_imports)]
use super::super::generators::float_64::*;
#[allow(unused_imports)]
use super::super::generators::signed_i128::*;
use super::super::generators::signed_i16::*;
use super::super::generators::signed_i32::*;
use super::super::generators::signed_i64::*;
use super::super::generators::signed_i8::*;
use super::super::generators::string::*;
use super::super::generators::tuple::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u128::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;
use super::super::generators::unsigned_u64::*;
use super::super::generators::unsigned_u8::*;

fn std_deviation(data: &Vec<u64>, mean: u64, size: usize) -> f32 {
    let variance = data
        .iter()
        .map(|value| {
            let diff = if mean >= *value {
                mean - *value
            } else {
                *value - mean
            };

            (diff * diff) as f64
        })
        .sum::<f64>()
        / size as f64;

    variance.sqrt() as f32
}

fn helper_sort_aux<T>(
    sort: &dyn Fn(&mut [T]) -> (),
    runs: usize,
    size: usize,
    generator: &dyn Fn(usize) -> Vec<T>,
) where
    T: Radixable + Copy + PartialOrd + std::fmt::Debug,
    // T: rdxsort::RdxSortTemplate,
    // [T]: rdxsort::RdxSort,
    // T: afsort::DigitAt,
    // T: Ord,
{
    let mut nanos: Vec<u64> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let mut array = generator(size);
        // let mut check = array.to_vec();

        let start = Instant::now();
        sort(&mut array);
        let ns: u64 = start.elapsed().as_nanos() as u64;
        nanos.push(ns);

        // check.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        // assert_eq!(check, array);
    }

    let sum: u64 = nanos.iter().sum();
    let mean: u64 = sum / runs as u64;
    let std_dev: f32 = std_deviation(&nanos, mean, size);
    let per_item: f32 = (mean as f64 / size as f64) as f32;

    // \u{1b} => escape for terminal
    // 0 => no color
    // 0;30 => gray
    // 0;31 => red
    // 1;31 => red
    // 0;32 => green
    // 0;33 => brown
    // 0;34 => blue
    // 1;34 => light blue
    // 0;37 => light gray

    // print time, standard deviation and time per item
    print!("\u{1b}[0;32m{}us\u{1b}[0m\t\u{1b}[1;31m{:.0}ns\u{1b}[0m\t(\u{1b}[0;33m{:.2}ns\u{1b}[0m)\t", mean / 1000, std_dev, per_item);
}

fn helper_sort<T>(
    test_name: &str,
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
) where
    T: Radixable + Copy + PartialOrd + std::fmt::Debug,
    // T: rdxsort::RdxSortTemplate,
    // [T]: rdxsort::RdxSort,
    // T: afsort::DigitAt,
    T: Ord,
{
    let runs = 3;
    let thread_n = 63;

    let sizes: Vec<usize> = vec![
        // 250, 500, 750, 1000, 1500, 2000, 2500, 5000, 7500, 10000,
        // 12000, 14000, 16000, 18000, 20000, 25000, 30000,
        500,
        50_000,
        // 50_000,60_000,70_000,80_000,90_000,
        100_000,
        // 200_000,300_000,400_000,500_000,600_000,700_000,800_000,900_000,
        1_000_000,
        // 2_000_000,3_000_000,4_000_000,
        5_000_000,
        // 6_000_000,7_000_000,
        // 8_000_000,9_000_000,
        10_000_000,
        50_000_000,
        100_000_000,
        // 200_000_000,300_000_000,400_000_000,
        500_000_000,
        // 600_000_000,700_000_000,800_000_000,800_000_000,900_000_000,
        1_000_000_000,
    ];

    let sorts_name = vec![
        "Trait Vora",
        // "Trait DLSD",
        // "Rust Std",
        "Rust Uns",
        // "Fast LSD (lib)",
        // "Raw DLSD",
        // "LSD",
        // "MSD",
        // "Ska Sort",
        // "American",
        // "Thiel",
        // "Raw Voracious sort",
        // "LSD MT",
        // "Regions sort 4000",
        // "Regions sort 6000",
        // "Regions sort 8000",
        // "Regions sort 16000",
        // "Regions sort 32000",
        // "Regions sort 64000",
        // "Regions sort 128000",
        // "Regions sort 256000",
        // "Regions sort 512000",
        "Regions sort 1000000",
        // "Regions sort test 1000000",
        // "AF sort (lib)",
        "Rayon pll uns",
    ];

    println!("Number of iterations: {}", runs);
    println!("Number of threads: {}", thread_n);
    print!("=== {} ===", test_name);
    print!("\u{1b}[1;34m");
    for sort_name in sorts_name.iter() {
        print!("\t{}\t", sort_name);
    }
    println!("\u{1b}[0m");
    for size in sizes.iter() {
        println!("Array size: {}", size);
        for (generator, gen_name) in generators.iter() {
            print!("{}", gen_name);

            helper_sort_aux(&|arr: &mut [T]| arr.voracious_sort(),runs,*size,generator,);
            // helper_sort_aux(&|arr: &mut [T]| arr.dlsd_sort(),runs,*size,generator);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort(), runs, *size, generator);
            helper_sort_aux(&|arr: &mut [T]| arr.sort_unstable(),runs,*size,generator);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| arr.rdxsort(), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| dlsd_radixsort(arr, 8),runs,*size,generator);
            // helper_sort_aux(&|arr: &mut [T]| lsd_radixsort(arr, 8),runs,*size,generator);
            // helper_sort_aux(&|arr: &mut [T]| msd_radixsort(arr, 8), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| ska_sort(arr, 8), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| american_flag_sort(arr, 8), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| thiel_radixsort(arr, 8), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| voracious_sort(arr, 8), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| lsd_mt_radixsort(arr, 8, thread_n),runs,*size,generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 4000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 6000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 8000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 16000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 32000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 64000), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 128000, thread_n), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 256000, thread_n), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 512000, thread_n), runs, *size, generator);
            helper_sort_aux(&|arr: &mut [T]| regions_sort(arr, 8, 1000000, thread_n), runs, *size, generator);
            // helper_sort_aux(&|arr: &mut [T]| arr.af_sort_unstable(), runs, *size, generator);
            helper_sort_aux(&|arr: &mut [T]| arr.par_sort_unstable(), runs, *size, generator);

            println!();
        }
    }
}

#[test]
fn speed_test_bool() {
    helper_sort("Test boolean", generators_bool());
}

#[test]
fn speed_test_char() {
    helper_sort("Test char", generators_char());
}

#[test]
fn speed_test_u8() {
    helper_sort("Test u8", generators_u8());
}

#[test]
fn speed_test_u16() {
    helper_sort("Test u16", generators_u16());
}

#[test]
fn speed_test_u32() {
    helper_sort("Test u32", generators_u32());
}

#[test]
fn speed_test_u64() {
    helper_sort("Test u64", generators_u64());
}

#[test]
fn speed_test_i8() {
    helper_sort("Test i8", generators_i8());
}

#[test]
fn speed_test_i16() {
    helper_sort("Test i16", generators_i16());
}

#[test]
fn speed_test_i32() {
    helper_sort("Test i32", generators_i32());
}

#[test]
fn speed_test_i64() {
    helper_sort("Test i64", generators_i64());
}

#[test]
fn speed_test_i128() {
    helper_sort("Test i128", generators_i128());
}

#[test]
fn speed_test_u128() {
    helper_sort("Test u128", generators_u128());
}

#[test]
fn speed_test_custom() {
    // helper_sort("Test Custom", generators_custom());
}

#[test]
fn speed_test_f32() {
    // helper_sort("Test f32", generators_f32());
}

#[test]
fn speed_test_f64() {
    // helper_sort("Test f64", generators_f64());
}

#[test]
fn speed_test_tuple_u8u8() {
    helper_sort("Test (u8, u8)", generators_u8u8());
}

#[test]
fn speed_test_tuple_boolbool() {
    helper_sort("Test (bool, bool)", generators_boolbool());
}

#[test]
fn speed_test_tuple_u32u32() {
    helper_sort("Test (u32, u32)", generators_u32u32());
}

#[test]
fn speed_test_tuple_boolu16() {
    helper_sort("Test (bool, u16)", generators_boolu16());
}

fn helper_sort_string_aux(
    sort: &dyn Fn(&mut Vec<&str>) -> (),
    runs: usize,
    string_size: usize,
    size: usize,
) {
    let generator = generators_string()[0].0;
    let mut nanos: Vec<u64> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let array = generator(size, string_size);
        let mut array = array
            .iter()
            .map(|element| unsafe { element.get_unchecked(..) })
            .collect::<Vec<&str>>();
        let mut check = array.to_vec();
        let start = Instant::now();
        sort(&mut array);
        let ns: u64 = start.elapsed().as_nanos() as u64;
        nanos.push(ns);
        check.sort_unstable();
        assert_eq!(check, array);
    }

    let sum: u64 = nanos.iter().sum();
    let mean: u64 = sum / runs as u64;
    let std_dev: f32 = std_deviation(&nanos, mean, size);
    let _min: u64 = *nanos.iter().min().unwrap();
    let _max: u64 = *nanos.iter().max().unwrap();
    let per_item: f32 = (mean as f64 / size as f64) as f32;

    print!("\u{1b}[0;32m{}us\u{1b}[0m\t\u{1b}[1;31m{:.0}ns\u{1b}[0m\t(\u{1b}[0;33m{:.2}ns\u{1b}[0m)\t", mean / 1000, std_dev, per_item);
}

fn helper_sort_string(test_name: &str) {
    // let runs = 3;
    // let string_size = 20;

    let sizes: Vec<usize> = vec![
        1_000, 10_000, 100_000, 500_000, 1_000_000, 5_000_000, 10_000_000,
        20_000_000,
    ];

    let sorts_name = vec![
        "Vora",
        "DLSD",
        "Rust Std",
        "Rust Uns",
        "LSD",
        "Af Sort (lib)",
    ];

    print!("=== {} ===", test_name);
    print!("\u{1b}[1;34m");
    for sort_name in sorts_name.iter() {
        print!("\t{}\t", sort_name);
    }
    println!("\u{1b}[0m");
    for size in sizes.iter() {
        println!("Array size: {}", size);
        print!("-- Unif       :");
        // helper_sort_string_aux(&|arr: &mut Vec<&str>| arr.voracious_sort(),runs,string_size,*size);
        // helper_sort_string_aux(&|arr: &mut Vec<&str>| arr.sort(),runs,string_size,*size);
        // helper_sort_string_aux(&|arr: &mut Vec<&str>| arr.sort_unstable(),runs,string_size,*size);
        // helper_sort_string_aux(&|arr: &mut Vec<&str>| arr.af_sort_unstable(),runs,string_size,*size);
        println!();
    }
}

#[test]
fn speed_test_string() {
    helper_sort_string("Test String");
}
