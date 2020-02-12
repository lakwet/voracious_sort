#![allow(dead_code)]

use rayon;

use std::time::Instant;

use super::super::*;

use super::super::algo::k_way_merge::{k_way_merge, k_way_merge_mt};
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::sorts::utils::{
    get_histogram, get_histogram_mt, Params,
};

use super::super::sorts::regions_sort::*;

use super::super::generators::unsigned_u32::*;
use super::super::generators::unsigned_u64::*;

fn default_sort_u64(arr: &mut [u64], _radix: usize) {
    arr.sort_unstable();
}

fn default_sort_u32(arr: &mut [u32], _radix: usize) {
    arr.sort_unstable();
}

#[test]
fn speed_other_k_way_merge() {
    let sizes = vec![
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
    ];

    let thread_n = 3;

    for size in sizes.iter() {
        println!("=== Array size: {}", *size);
        let mut arr = helper_random_array_asc_sawtooth_u64(*size);
        let mut copy = arr.to_vec();
        let mut buffer = arr.to_vec();
        let mut separators = verge_sort_preprocessing(&mut arr, 8, &default_sort_u64);
        let mut sep_copy = separators.to_vec();

        let start = Instant::now();
        k_way_merge(&mut arr, &mut separators);
        println!("K way merge (u64): {}us", start.elapsed().as_micros() as u64);

        let start = Instant::now();
        k_way_merge_mt(&mut copy, &mut buffer, &mut sep_copy, thread_n);
        println!("K way merge MT (u64): {}us", start.elapsed().as_micros() as u64);

        for i in 1..copy.len() {
            assert!(copy[i - 1] <= copy[i]);
        }

        let mut arr = helper_random_array_asc_sawtooth_u32(*size);
        let mut copy = arr.to_vec();
        let mut buffer = arr.to_vec();
        let mut separators = verge_sort_preprocessing(&mut arr, 8, &default_sort_u32);
        let mut sep_copy = separators.to_vec();

        let start = Instant::now();
        k_way_merge(&mut arr, &mut separators);
        println!("K way merge (u32): {}us", start.elapsed().as_micros() as u64);

        let start = Instant::now();
        k_way_merge_mt(&mut copy, &mut buffer, &mut sep_copy, thread_n);
        println!("K way merge MT (u32): {}us", start.elapsed().as_micros() as u64);

        for i in 1..copy.len() {
            assert!(copy[i - 1] <= copy[i]);
        }
    }
}

#[test]
fn speed_other_get_histogram_mt() {
    let sizes = vec![
        1_000,
        10_000,
        100_000,
        180_000, 200_000, 220_000,
        250_000, 750_000,
        1_000_000,
        10_000_000,
        100_000_000,
    ];

    let radix = 8;
    let threads_n = vec![2, 3, 4];
    let parts_n = vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32];

    println!("**Compare get_histogram vs get_histogram_mt**");
    for part_n in parts_n.iter() {
        println!("  === part_n: {}", *part_n);
        print!("      Array size:");
        for size in sizes.iter() {
            print!("      {}", *size);
        }
        println!();
        for thread_n in threads_n.iter() {
            print!("      thread_n: {}: ", *thread_n);
            let mut res = Vec::new();
            for size in sizes.iter() {
                let mut arr = helper_random_array_uniform_u32(*size);

                let dummy = arr[0];
                let (offset, _) = dummy.compute_offset(&mut arr, radix);
                let max_level = dummy.compute_max_level(offset, radix);
                let p = Params::new(0, radix, offset, max_level);
                let (mask, shift) = dummy.get_mask_and_shift(&p);

                let start = Instant::now();
                get_histogram(&mut arr, &p, mask, shift);
                let t1 = start.elapsed().as_micros() as u64;

                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(*thread_n)
                    .build()
                    .unwrap();

                let start = Instant::now();
                get_histogram_mt(&mut arr, &p, mask, shift, &pool, *part_n);
                let t2 = start.elapsed().as_micros() as u64;

                res.push((t1, t2));
            }
            println!("      {:?}", res);
        }
    }
}

#[test]
fn speed_other_compare_block_sizes() {
    let sizes = vec![
        1_000,
        10_000,
        50_000,
        100_000,
        250_000,
        1_000_000,
        2_000_000,
        10_000_000,
        20_000_000,
        50_000_000,
        100_000_000,
    ];

    let radix = 8;
    let threads_n = vec![2, 3, 4];
    let block_sizes = vec![
        20000,32000,64000,128000,256000,512000,1000000,2000000,
        4000000,5000000,10000000,
    ];

    println!("**Compare regions_sort**");
    for block_size in block_sizes.iter() {
        println!("  === block_size: {}", *block_size);
        print!("      Array size:");
        for size in sizes.iter() {
            print!("\t\t{}", *size);
        }
        println!();
        for thread_n in threads_n.iter() {
            print!("      thread_n: {}: ", *thread_n);
            let mut res = Vec::new();
            for size in sizes.iter() {
                let mut arr = helper_random_array_uniform_u32(*size);
                let start = Instant::now();
                regions_sort(&mut arr, radix, *block_size, *thread_n);
                let t1 = start.elapsed().as_nanos() as u64;
                res.push((t1 / 1000, t1 as f64 / *size as f64));
            }
            for r in res.iter() {
                print!("[{}, (\u{1b}[0;33m{:.2}ns\u{1b}[0m)]", r.0, r.1);
            }
            println!();
        }
    }
}

#[test]
fn speed_other_regions_graph_processing() {
    let sizes = vec![
        1_000_000, 5_000_000,
        10_000_000, 50_000_000,100_000_000,
        500_000_000,
        1_000_000_000,
    ];
    let block_sizes = [
        200_000, 500_000, 1000000,
    ];
    let radix = 8;
    let thread_n = 4;

    for block_size in block_sizes.iter() {
        for size in sizes.iter() {
            let mut arr = helper_random_array_uniform_u32(*size);
            let dummy = arr[0];
            let (offset, _) = dummy.compute_offset(&mut arr, radix);
            let max_level = dummy.compute_max_level(offset, radix);
            let p = Params::new(0, radix, offset, max_level);
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_n)
                .build()
                .unwrap();
            let histograms = local_sorting(&mut arr, &p, *block_size, &pool, thread_n);

            let mut regions_graph = RegionsGraph::new(p.radix_range);
            let global_histogram = regions_graph.build_regions_graph(&histograms);
            let sorted_countries = sort_countries(&global_histogram);

            let start = Instant::now();
            for country in sorted_countries.iter() {
                let swaps = regions_graph.two_cycle(*country);
                perform_swaps(&mut arr, swaps);
                let swaps = regions_graph.two_path(*country);
                perform_swaps(&mut arr, swaps);
            }
            let t1 = start.elapsed().as_nanos() as u64;
            println!(
                "Size: {}, block_size: {}, Time: {}ns ({}ns)",
                *size,
                *block_size,
                t1,
                t1 as f64 / *size as f64,
            );
        }
    }
}
