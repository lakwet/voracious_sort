//! Dev only: A/B of the u16 chain against blazing-radix, in a binary that
//! builds in seconds (the test crate takes minutes).
//!
//! cargo run --release --example ab_u16 --features "bench ab"
//! Env: SORTS="v2,copy,..." (`v2` the library, `copy` the A/B copy of
//! `src/v2/devour/ab_copy.rs`, `std`, `turbo`, `blz`; SKIP_MASKS="x:name,..."
//! is read as the names), DOM_SIZES="200,500,...", TOTAL (keys per measure,
//! default 2M), CHECK=1 (shape test first: every size 33..1100, then every
//! 997 keys up to 70 000).
//! Output: ns per key, copy subtracted, median of 7, random order per rep.

#[allow(dead_code, unused_imports)]
#[path = "../src/generators/mod.rs"]
mod generators;

#[path = "common/fresh_pages.rs"]
mod fresh_pages;
#[global_allocator]
static ALLOC: fresh_pages::Fresh = fresh_pages::Fresh;

use std::time::Instant;
use voracious_radix_sort::{ab_u16_copy, devour_sort};

fn rng(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// One sort by its name (see the module doc).
fn run(name: &str, w: &mut [u16]) {
    match name {
        "std" => w.sort_unstable(),
        "blz" => blazing_radix::radix_sort::<u16>(w),
        "copy" => ab_u16_copy(w),
        "cnt" => {
            // Floor: the counting pass alone (no writing).
            let mut c = vec![0u32; 1 << 16];
            for &k in w.iter() {
                c[k as usize] += 1;
            }
            std::hint::black_box(&c);
        },
        t if t.starts_with("turbo") => turbosort::sort(w),
        _ => devour_sort(w),
    }
}

fn order(rep: usize, j: usize, k: usize) -> usize {
    let mut p: Vec<usize> = (0..k).collect();
    let mut seed = 0x9E37_79B9_7F4A_7C15u64 ^ (rep as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
    for i in (1..k).rev() {
        let r = rng(&mut seed) as usize % (i + 1);
        p.swap(i, r);
    }
    p[j]
}

/// Random shapes (few values, straight and wrapped ranges, one byte, random)
/// against `sort_unstable`, for every sort.
fn check(masks: &[(u64, String)]) {
    let mut seed = 0x1234_5678_9abc_def1u64;
    for n in (33..1100).chain((1100..70_000).step_by(997)) {
        for shape in 0..9u64 {
            let base = rng(&mut seed) as u16;
            let a: Vec<u16> = (0..n)
                .map(|_| {
                    let r = rng(&mut seed);
                    match shape {
                        0 => base.wrapping_add((r % 3) as u16),
                        1 => base.wrapping_add((r % 256) as u16),
                        2 => (r % 256) as u16 ^ 0x8080,
                        3 => {
                            if r & 1 == 0 {
                                (r >> 8) as u16 % 100
                            } else {
                                u16::MAX - (r >> 8) as u16 % 100
                            }
                        },
                        4 => 0x7F80 + (r % 256) as u16,
                        5 => base.wrapping_add((r % 300) as u16),
                        6 => ((r % 256) as u16) << 8 | (base & 0xFF),
                        7 => (r % 5) as u16 * 1000,
                        _ => r as u16,
                    }
                })
                .collect();
            let mut r = a.clone();
            r.sort_unstable();
            for m in masks.iter().filter(|m| m.1 != "cnt") {
                let mut w = a.clone();
                run(&m.1, &mut w);
                assert!(w == r, "n {} shape {} sort {}", n, shape, m.1);
            }
        }
    }
    // Every distribution beyond the short preparation.
    let mut gens = generators::unsigned_u16::generators_u16();
    gens.extend(generators::narrow_extra::extra_u16::generators());
    for n in [1100, 1500, 2000, 3001, 5000, 10_007, 40_000, 70_000, 140_000] {
        for _ in 0..3 {
            for (gen, title) in gens.iter() {
                let a = gen(n);
                let mut r = a.clone();
                r.sort_unstable();
                for m in masks.iter().filter(|m| m.1 != "cnt") {
                    let mut w = a.clone();
                    run(&m.1, &mut w);
                    assert!(w == r, "n {} {} sort {}", n, title, m.1);
                }
            }
        }
    }
    println!("check ok");
}

/// TRACE build (feature `v2_trace`, debug is enough): the decision labels
/// of one array per size and distribution, for each sort.
#[cfg(feature = "v2_trace")]
fn trace(masks: &[(u64, String)]) {
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![2000, 10_000, 100_000]);
    let mut gens = generators::unsigned_u16::generators_u16();
    gens.extend(generators::narrow_extra::extra_u16::generators());
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let a = gen(n);
            for m in masks {
                let mut w = a.clone();
                let _ = voracious_radix_sort::ab_trace_take();
                run(&m.1, &mut w);
                let t = voracious_radix_sort::ab_trace_take();
                println!(
                    "{:>7} {:14} {:8} {}",
                    n,
                    title.trim_start_matches("--").trim_end_matches(':').trim(),
                    m.1,
                    t.join(" ")
                );
            }
        }
    }
}

/// STATS=1: per distribution, distinct high bytes, low bytes and values,
/// and the table of a count with the less dispersed byte renumbered
/// (k x 256 counters).
fn stats() {
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![10_000, 100_000, 1_000_000]);
    let mut gens = generators::unsigned_u16::generators_u16();
    gens.extend(generators::narrow_extra::extra_u16::generators());
    println!(
        "{:>8} {:14} {:>5} {:>5} {:>6} {:>8} {:>7}",
        "size", "gen", "hi", "lo", "vals", "table", "KiB"
    );
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let a = gen(n);
            let (mut hi, mut lo) = ([false; 256], [false; 256]);
            let mut v = vec![false; 1 << 16];
            for &k in &a {
                hi[(k >> 8) as usize] = true;
                lo[(k & 255) as usize] = true;
                v[k as usize] = true;
            }
            let h = hi.iter().filter(|&&x| x).count();
            let l = lo.iter().filter(|&&x| x).count();
            let d = v.iter().filter(|&&x| x).count();
            let table = h.min(l) * 256;
            let name = title.trim_start_matches("--").trim_end_matches(':').trim();
            println!(
                "{:>8} {:14} {:>5} {:>5} {:>6} {:>8} {:>7}",
                n,
                name,
                h,
                l,
                d,
                table,
                table * 4 / 1024
            );
        }
    }
}

/// PROF=v2|copy|blz|std|turbo: only that sort, on PROF_GEN (default Unif)
/// arrays of PROF_N keys (default 1000), repeated for ~PROF_SECS seconds
/// (default 4), for `perf stat` / `perf record`. Prints ns per key.
fn prof(which: &str) {
    let n: usize = std::env::var("PROF_N").ok().map_or(1000, |v| v.parse().unwrap());
    let want = std::env::var("PROF_GEN").unwrap_or("Unif".into());
    let secs: f64 = std::env::var("PROF_SECS").ok().map_or(4.0, |v| v.parse().unwrap());
    let mut gens = generators::unsigned_u16::generators_u16();
    gens.extend(generators::narrow_extra::extra_u16::generators());
    let gen = gens
        .iter()
        .find(|(_, t)| t.trim_start_matches("--").trim_end_matches(':').trim() == want)
        .expect("PROF_GEN")
        .0;
    let count = (2_000_000 / n).max(1);
    let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
    let mut work = arrays.clone();
    let t = Instant::now();
    let mut sorts = 0usize;
    while t.elapsed().as_secs_f64() < secs {
        for (w, a) in work.iter_mut().zip(arrays.iter()) {
            w.copy_from_slice(a);
            run(which, w);
        }
        sorts += count;
    }
    let ns = t.elapsed().as_nanos() as f64 / (sorts * n) as f64;
    println!("{} {} n={} : {:.3} ns/key (copy included), {} sorts", which, want, n, ns, sorts);
}

fn main() {
    // The sorts measured: SORTS="v2,copy", or the names of SKIP_MASKS.
    let masks: Vec<(u64, String)> = std::env::var("SORTS")
        .or_else(|_| std::env::var("SKIP_MASKS"))
        .unwrap_or("v2,copy".into())
        .split(',')
        .map(|m| (0, m.rsplit(':').next().unwrap().to_string()))
        .collect();
    if std::env::var("CHECK").is_ok() {
        check(&masks);
    }
    #[cfg(feature = "v2_trace")]
    {
        trace(&masks);
        return;
    }
    if std::env::var("STATS").is_ok() {
        stats();
        return;
    }
    if let Ok(which) = std::env::var("PROF") {
        prof(&which);
        return;
    }
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![70, 100, 150, 200, 300, 500, 700, 1000]);
    // COLD=1: below 1M keys every sort on a fresh copy (allocated, page
    // faults paid, the copy subtracted as usual), 32M keys per measure by
    // default (inputs far beyond the last cache, so each array is cold when
    // sorted); from 1M keys the work arrays are reused (already out of cache).
    let cold = std::env::var("COLD").is_ok();
    // REPS: repetitions per cell (median kept), 7 by default.
    let reps: usize = std::env::var("REPS").ok().map_or(7, |v| v.parse().unwrap());
    let total: usize = std::env::var("TOTAL")
        .ok()
        .map_or(if cold { 32_000_000 } else { 2_000_000 }, |v| v.parse().unwrap());
    let mut gens = generators::unsigned_u16::generators_u16();
    gens.extend(generators::narrow_extra::extra_u16::generators());
    let head: String = masks.iter().map(|m| format!(" {:>8}", m.1)).collect();
    // CSV=path: also a bench file the bench viewer reads (label u16, ns per
    // key, copy subtracted; CPU=name for its machine line).
    let mut csv = std::env::var("CSV").ok().map(|p| std::fs::File::create(p).unwrap());
    if let Some(f) = csv.as_mut() {
        use std::io::Write;
        let cpu = std::env::var("CPU").unwrap_or("unknown".into());
        writeln!(
            f,
            "== machine
cpu,{}
settings,ab_u16 example: median of {} interleaved, random order, copy subtracted, {} keys per \
             measure{}",
            cpu,
            reps,
            total,
            if cold { ", cold (fresh copies, inputs beyond the caches)" } else { "" }
        )
        .unwrap();
        let names: Vec<&str> = masks.iter().map(|m| m.1.as_str()).collect();
        writeln!(
            f,
            "== u16 (ab_u16 example; ns per key)
cols,u16,size,gen,{},blazing",
            names.join(",")
        )
        .unwrap();
    }
    println!("{:>9} {:12}{} {:>8} {:>8}", "size", "gen", head, "blazing", "copy");
    let started = Instant::now();
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String =
                title.trim_start_matches("--").trim_end_matches(':').trim().replace(' ', "_");
            let count = (total / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = masks.len() + 2;
            let mut times = vec![Vec::new(); k];
            for rep in 0..reps {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let sort_one = |w: &mut [u16]| {
                        if v < masks.len() {
                            run(&masks[v].1, w);
                        } else if v == masks.len() {
                            blazing_radix::radix_sort::<u16>(w);
                        }
                    };
                    // TRIM=1: freed memory given back to the system before each
                    // measure, as the official bench does (glibc only).
                    #[cfg(all(target_os = "linux", target_env = "gnu"))]
                    if std::env::var("TRIM").is_ok() {
                        extern "C" {
                            fn malloc_trim(pad: usize) -> i32;
                        }
                        // SAFETY: glibc's malloc_trim, no precondition.
                        unsafe { malloc_trim(0) };
                    }
                    let t = Instant::now();
                    // Cold below 1M keys only: beyond, the arrays are out of
                    // the caches anyway, and a fresh copy of tens of MiB (its
                    // page faults) would outweigh the sort being measured.
                    if cold && n < 1_000_000 {
                        for a in arrays.iter() {
                            let mut w = a.clone();
                            sort_one(&mut w);
                            std::hint::black_box(&w);
                        }
                    } else {
                        for (w, a) in work.iter_mut().zip(arrays.iter()) {
                            w.copy_from_slice(a);
                            sort_one(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if !cold
                        && rep == 0
                        && v <= masks.len()
                        && (v == masks.len() || masks[v].1 != "cnt")
                    {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            let med = |x: &Vec<f64>| -> f64 {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            let cells: String =
                (0..=masks.len()).map(|v| format!(" {:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:12}{} {:8.3}", n, name, cells, copy);
            if let Some(f) = csv.as_mut() {
                use std::io::Write;
                let vals: Vec<String> = (0..=masks.len())
                    .map(|v| format!("{:.4}", (med(&times[v]) - copy).max(0.001)))
                    .collect();
                let gen_name = title.trim_start_matches("--").trim_end_matches(':').trim();
                writeln!(f, "row,u16,{},{},{}", n, gen_name, vals.join(",")).unwrap();
            }
        }
    }
    if let Some(f) = csv.as_mut() {
        use std::io::Write;
        writeln!(f, "== done in {} min", (started.elapsed().as_secs() + 30) / 60).unwrap();
    }
}
