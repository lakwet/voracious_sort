use std::fmt::Debug;
use std::io::Write;

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
use super::super::sorts::lsd_sort::lsd_radixsort_heu;
use super::super::{
    counting_sort, dlsd_radixsort, lsd_radixsort, msd_radixsort, rollercoaster_sort, RadixKey,
    Radixable,
};

const REPS: usize = 9;
/// Each measure sorts a batch of arrays totalling about this many elements.
const BATCH_ELEMENTS: usize = 1_000_000;
const SIZES: [usize; 14] = [20, 50, 100, 150, 200, 300, 450, 700, 1000, 1500, 2500, 5000, 10_000, 50_000];
const OUT_DIR: &str = "C:/Users/Axell/AppData/Local/Temp/claude/8d3631a030c6f17e/0a77c6ab-b07b-40e7-80c1-fdf30fae5edd/scratchpad";

fn stats(v: &mut Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = v[v.len() / 2];
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (v.len() - 1) as f64).sqrt();
    (med, sd / mean * 100.0)
}

/// Rust sort_unstable vs the radix algorithm the type dispatches to above
/// its current threshold, called directly (no threshold involved).
fn bench_type<T, K>(
    type_name: &str,
    algo_name: &str,
    gens: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
    radix: &dyn Fn(&mut [T]),
    out: &mut std::fs::File,
) where
    T: Radixable<K> + Debug,
    K: RadixKey,
{
    for &size in SIZES.iter() {
        for (gen, gen_name) in gens.iter() {
            let count = (BATCH_ELEMENTS / size).max(1);
            let mut t: Vec<Vec<f64>> = vec![Vec::new(); 2];
            for rep in 0..REPS {
                let batch: Vec<Vec<T>> = (0..count).map(|_| gen(size)).collect();
                let mut results: Vec<Vec<Vec<T>>> = vec![Vec::new(); 2];
                for k in 0..2 {
                    let v = (k + rep) % 2;
                    let mut work = batch.clone();
                    let start = std::time::Instant::now();
                    for a in work.iter_mut() {
                        if v == 0 { a.sort_unstable_by(|x, y| x.partial_cmp(y).unwrap()); } else { radix(a); }
                    }
                    t[v].push(start.elapsed().as_nanos() as f64 / (count * size) as f64);
                    results[v] = work;
                }
                assert!(results[0] == results[1], "{} {} {} {}", type_name, algo_name, gen_name, size);
            }
            let (s, ssd) = stats(&mut t[0]);
            let (r, rsd) = stats(&mut t[1]);
            let g = gen_name.trim_start_matches("-- ").trim_end_matches(':').trim();
            writeln!(out, "{},{},{},{},{:.3},{:.1},{:.3},{:.1}", type_name, algo_name, g, size, s, ssd, r, rsd).unwrap();
            out.flush().unwrap();
        }
    }
    println!("done {} {}", type_name, algo_name);
}

#[test]
fn tmp_alltypes() {
    // Generators use rayon: thousands of tiny par_iter calls hit an internal
    // rayon error, so the global pool is limited to one thread.
    let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();
    let path = format!("{}/thresholds_{}.csv", OUT_DIR, std::env::consts::ARCH);
    let mut out = std::fs::File::create(path).unwrap();
    writeln!(out, "type,algo,gen,size,std,std_sd,radix,radix_sd").unwrap();

    bench_type("u8", "counting", generators_u8(), &|a| counting_sort(a, 8), &mut out);
    bench_type("i8", "counting", generators_i8(), &|a| counting_sort(a, 8), &mut out);
    bench_type("u16", "lsd_heu", generators_u16(), &|a| lsd_radixsort_heu(a, 8, 100_000), &mut out);
    bench_type("i16", "lsd_heu", generators_i16(), &|a| lsd_radixsort_heu(a, 8, 150_000), &mut out);
    bench_type("u32", "dlsd", generators_u32(), &|a| dlsd_radixsort(a, 8), &mut out);
    bench_type("u64", "dlsd", generators_u64(), &|a| dlsd_radixsort(a, 8), &mut out);
    bench_type("usize", "dlsd", generators_usize(), &|a| dlsd_radixsort(a, 8), &mut out);
    bench_type("u128", "msd", generators_u128(), &|a| msd_radixsort(a, 8), &mut out);
    bench_type("i128", "msd", generators_i128(), &|a| msd_radixsort(a, 8), &mut out);
    bench_type("i32", "lsd_heu", generators_i32(), &|a| lsd_radixsort_heu(a, 8, 100_000), &mut out);
    bench_type("i64", "roller", generators_i64(), &|a| rollercoaster_sort(a, 8), &mut out);
    bench_type("isize", "roller", generators_isize(), &|a| rollercoaster_sort(a, 8), &mut out);
    bench_type("f32", "lsd", generators_f32(), &|a| lsd_radixsort(a, 8), &mut out);
    bench_type("f64", "dlsd", generators_f64(), &|a| dlsd_radixsort(a, 8), &mut out);
    bench_type("f64", "roller", generators_f64(), &|a| rollercoaster_sort(a, 8), &mut out);
    bench_type("char", "lsd_heu", generators_char(), &|a| lsd_radixsort_heu(a, 7, 11_000), &mut out);
    bench_type("struct u64", "dlsd", generators_structu64(), &|a| dlsd_radixsort(a, 8), &mut out);
    bench_type("struct i64", "msd", generators_structi64(), &|a| msd_radixsort(a, 8), &mut out);
    bench_type("struct f64", "roller", generators_structf64(), &|a| rollercoaster_sort(a, 8), &mut out);
    bench_type("struct f32", "lsd", generators_structf32(), &|a| lsd_radixsort(a, 8), &mut out);
    bench_type("struct i32", "lsd", generators_structi32(), &|a| lsd_radixsort(a, 8), &mut out);
}
