#[allow(unused_imports)]
use rayon::prelude::*;

#[allow(unused_imports)]
use super::super::{RadixSort, Radixable, RadixKey};

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
use super::super::sorts::rollercoaster_sort::rollercoaster_sort;
#[allow(unused_imports)]
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
#[allow(unused_imports)]
use super::super::sorts::lsd_stable_sort::lsd_stable_radixsort;
#[allow(unused_imports)]
use super::super::sorts::msd_sort::msd_radixsort;
#[allow(unused_imports)]
use super::super::sorts::msd_stable_sort::msd_stable_radixsort;
#[allow(unused_imports)]
use super::super::sorts::ska_sort::ska_sort;
#[allow(unused_imports)]
use super::super::sorts::peeka_sort::peeka_sort;
#[allow(unused_imports)]
use super::super::sorts::thiel_sort::thiel_radixsort;
#[allow(unused_imports)]
use super::super::sorts::voracious_sort::{voracious_sort, voracious_sort_heu};

#[allow(unused_imports)]
use super::super::generators::boolean::*;
#[allow(unused_imports)]
use super::super::generators::char::*;
#[allow(unused_imports)]
use super::super::generators::custom::*;
#[allow(unused_imports)]
use super::super::generators::float_32::*;
#[allow(unused_imports)]
use super::super::generators::float_64::*;
#[allow(unused_imports)]
use super::super::generators::signed_i128::*;
#[allow(unused_imports)]
use super::super::generators::signed_i16::*;
#[allow(unused_imports)]
use super::super::generators::signed_i32::*;
#[allow(unused_imports)]
use super::super::generators::signed_i64::*;
#[allow(unused_imports)]
use super::super::generators::signed_i8::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u128::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u16::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u32::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u64::*;
#[allow(unused_imports)]
use super::super::generators::unsigned_u8::*;

use super::test_helpers::helper_sort_aux;

fn helper_sort<T, K>(
    test_name: &str,
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
    // runs: usize,
    // sizes: &Vec<usize>,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
    // T: Ord,
{
    let sorts_name = vec![
        // "Trait Vora",
        // "Trait Vora Stable",
        "Trait Vora MT",
        // "Rust Std",
        // "Rust Uns",
        // "Counting sort",
        // "Rollercoaster",
        // "LSD",
        // "MSD",
        // "DLSD",
        // "Ska Sort",
        // "American",
        // "Thiel",
        // "Raw Voracious sort",
        // "Raw Regions sort",
        "Rayon pll uns",
        // "Rayon pll stable",
    ];

    let runs = 50;
    let thread_n = 16;
    let with_check = false;

    let sizes: Vec<usize> = vec![
        // 500,1000, 10000, 50000,100000,500000,1000000, 10000000,
        1000000,5000000,10000000,20000000,50000000,100000000,200000000
        // 10, 30, 50, 70, 100
        // 130,150,170,200,230,250,
        // 300,350,400,500,600,700,800,900,
        // 1000,1500,2000,3000,4000,5000,6000,7000,8000,9000,
        // 10_000,20_000,30_000,
        // 50_000,60_000,70_000,80_000,90_000,
        // 100_000,
        // 200_000,300_000,400_000,500_000,600_000,700_000,800_000,900_000,
        // 1_000_000,
        // 1_100_000,1_200_000,1_300_000,1_400_000,1_500_000,1_600_000,1_700_000,
        // 1_800_000,1_900_000,
        // 2_000_000,
        // 2_100_000,2_200_000,2_300_000,2_400_000,2_500_000,2_600_000,2_700_000,2_800_000,2_900_000,
        // 3_000_000,
        // 3_100_000,3_200_000,3_300_000,3_400_000,3_500_000,3_600_000,3_700_000,3_800_000,3_900_000,
        // 4_000_000,
        // 5_000_000,
        // 6_000_000,7_000_000,8_000_000,9_000_000,
        // 10_000_000,
        // 20_000_000,
        // 30_000_000,40_000_000,
        // 50_000_000,
        // 60_000_000,
        // 70_000_000,80_000_000,90_000_000,
        // 100_000_000,
        // 200_000_000,300_000_000,400_000_000,
        // 500_000_000,600_000_000,700_000_000,800_000_000,900_000_000,
        // 500_000_000,
        // 1_000_000_000,
    ];

    println!("Number of iterations: {}", runs);
    // println!("Number of threads: {}", thread_n);
    // println!("With check: {}", with_check);
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
            // helper_sort_aux(&|arr: &mut [T]| arr.voracious_sort(),runs,*size,generator,with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.voracious_stable_sort(),runs,*size,generator, with_check);
            helper_sort_aux(&|arr: &mut [T]| arr.voracious_mt_sort(thread_n),runs,*size,generator,with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort(), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort_unstable(),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| counting_sort(arr, 8),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| insertion_sort(arr),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| rollercoaster_sort(arr, 8),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| lsd_stable_radixsort(arr, 8),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| lsd_radixsort(arr, 8),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| msd_radixsort(arr, 8), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| msd_stable_radixsort(arr, 8), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| dlsd_radixsort(arr, 8),runs,*size,generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| ska_sort(arr, 8), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| american_flag_sort(arr, 8), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| thiel_radixsort(arr, 8), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| voracious_sort(arr, 8), runs, *size, generator, with_check);

            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,    75_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   100_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   200_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   300_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   400_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   500_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   600_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   650_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   700_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   800_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8,   900_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_000_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_100_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_200_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_300_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_400_000, thread_n), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| peeka_sort(arr, 8, 1_500_000, thread_n), runs, *size, generator, with_check);

            // helper_sort_aux(&|arr: &mut [T]| arr.par_sort_unstable(), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.par_sort(), runs, *size, generator, with_check);
            helper_sort_aux(&|arr: &mut [T]| arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator, with_check);
            // helper_sort_aux(&|arr: &mut [T]| arr.par_sort_by(|a, b| a.partial_cmp(b).unwrap()), runs, *size, generator, with_check);
            println!();
        }
    }
}

// #[test]
// fn speed_test_bool() {
//     helper_sort("Test boolean", generators_bool());
// }

// #[test]
// fn speed_test_char() {
//     helper_sort("Test char", generators_char());
// }

// #[test]
// fn speed_test_f32() {
//     helper_sort("Test f32", generators_f32());
// }

#[test]
fn speed_test_f64() {
    helper_sort("Test f64", generators_f64());
}

// #[test]
// fn speed_test_u8() {
//     helper_sort("Test u8", generators_u8());
// }

// #[test]
// fn speed_test_u16() {
//     helper_sort("Test u16", generators_u16());
// }

#[test]
fn speed_test_u32() {
    helper_sort("Test u32", generators_u32());
}

#[test]
fn speed_test_u64() {
    helper_sort("Test u64", generators_u64());
}

// #[test]
// fn speed_test_u128() {
//     helper_sort("Test u128", generators_u128());
// }

// #[test]
// fn speed_test_usize() {
//     helper_sort("Test usize", generators_usize());
// }

// #[test]
// fn speed_test_i8() {
//     helper_sort("Test i8", generators_i8());
// }

// #[test]
// fn speed_test_i16() {
//     helper_sort("Test i16", generators_i16());
// }

#[test]
fn speed_test_i32() {
    helper_sort("Test i32", generators_i32());
}

#[test]
fn speed_test_i64() {
    helper_sort("Test i64", generators_i64());
}

// #[test]
// fn speed_test_i128() {
//     helper_sort("Test i128", generators_i128());
// }

// #[test]
// fn speed_test_isize() {
//     helper_sort("Test isize", generators_isize());
// }

// #[test]
// fn speed_test_structbool() {
//     helper_sort("Test struct{isize, bool}", generators_structbool());
// }

// #[test]
// fn speed_test_structchar() {
//     helper_sort("Test struct{isize, char}", generators_structchar());
// }

// #[test]
// fn speed_test_structf32() {
//     helper_sort("Test struct{isize, f32}", generators_structf32());
// }

// #[test]
// fn speed_test_structf64() {
//     helper_sort("Test struct{isize, f64}", generators_structf64());
// }

// #[test]
// fn speed_test_structu8() {
//     helper_sort("Test struct{isize, u8}", generators_structu8());
// }

// #[test]
// fn speed_test_structu16() {
//     helper_sort("Test struct{isize, u16}", generators_structu16());
// }

// #[test]
// fn speed_test_structu32() {
//     helper_sort("Test struct{isize, u32}", generators_structu32());
// }

// #[test]
// fn speed_test_structu64() {
//     helper_sort("Test struct{isize, u64}", generators_structu64());
// }

// #[test]
// fn speed_test_structu128() {
//     helper_sort("Test struct{isize, u128}", generators_structu128());
// }

// #[test]
// fn speed_test_structusize() {
//     helper_sort("Test struct{isize, usize}", generators_structusize());
// }

// #[test]
// fn speed_test_structi8() {
//     helper_sort("Test struct{isize, i8}", generators_structi8());
// }

// #[test]
// fn speed_test_structi16() {
//     helper_sort("Test struct{isize, i16}", generators_structi16());
// }

// #[test]
// fn speed_test_structi32() {
//     helper_sort("Test struct{isize, i32}", generators_structi32());
// }

// #[test]
// fn speed_test_structi64() {
//     helper_sort("Test struct{isize, i64}", generators_structi64());
// }

// #[test]
// fn speed_test_structi128() {
//     helper_sort("Test struct{isize, i128}", generators_structi128());
// }

// #[test]
// fn speed_test_structisize() {
//     helper_sort("Test struct{isize, isize}", generators_structisize());
// }

// #[test]
// fn speed_test_bench_all() {
//     let medium: Vec<usize> = vec![
//         10_000_000,
//         50_000_000,
//         75_000_000,
//     ];

//     let big: Vec<usize> = vec![
//         100_000_000,
//         250_000_000,
//         500_000_000,
//         750_000_000,
//         1_000_000_000,
//     ];

//     helper_sort("Test boolean", generators_bool(), 31, &medium);
//     helper_sort("Test char", generators_char(), 31, &medium);
//     helper_sort("Test u8", generators_u8(), 31, &medium);
//     helper_sort("Test u16", generators_u16(), 31, &medium);
//     helper_sort("Test u32", generators_u32(), 31, &medium);
//     helper_sort("Test u64", generators_u64(), 31, &medium);
//     helper_sort("Test u128", generators_u128(), 31, &medium);
//     helper_sort("Test i8", generators_i8(), 31, &medium);
//     helper_sort("Test i16", generators_i16(), 31, &medium);
//     helper_sort("Test i32", generators_i32(), 31, &medium);
//     helper_sort("Test i64", generators_i64(), 31, &medium);
//     helper_sort("Test i128", generators_i128(), 31, &medium);
//     helper_sort("Test f32", generators_f32(), 31, &medium);
//     helper_sort("Test f64", generators_f64(), 31, &medium);
//     helper_sort("Test struct{isize, f32}", generators_structf32(), 31, &medium);
//     helper_sort("Test struct{isize, f64}", generators_structf64(), 31, &medium);

//     helper_sort("Test boolean", generators_bool(), 11, &big);
//     helper_sort("Test char", generators_char(), 11, &big);
//     helper_sort("Test u8", generators_u8(), 11, &big);
//     helper_sort("Test u16", generators_u16(), 11, &big);
//     helper_sort("Test u32", generators_u32(), 11, &big);
//     helper_sort("Test u64", generators_u64(), 11, &big);
//     helper_sort("Test u128", generators_u128(), 11, &big);
//     helper_sort("Test i8", generators_i8(), 11, &big);
//     helper_sort("Test i16", generators_i16(), 11, &big);
//     helper_sort("Test i32", generators_i32(), 11, &big);
//     helper_sort("Test i64", generators_i64(), 11, &big);
//     helper_sort("Test i128", generators_i128(), 11, &big);
//     helper_sort("Test f32", generators_f32(), 11, &big);
//     helper_sort("Test f64", generators_f64(), 11, &big);
//     helper_sort("Test struct{isize, f32}", generators_structf32(), 11, &big);
//     helper_sort("Test struct{isize, f64}", generators_structf64(), 11, &big);
// }
