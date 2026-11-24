//! Dev only: i16 sorts compared in a binary that builds in seconds.
//!
//! cargo run --release --example ab_i16 --features "bench ab"
//! Env: SORTS="v2,copy,std,turbo" (v2: the library; copy: the A/B copy of
//! `src/v2/devour/ab_copy.rs`; blazing is always the last column),
//! DOM_SIZES, TOTAL, REPS (default 7), COLD=1, CSV=path + CPU=name.
//! Output: ns per key, copy subtracted, median of the reps, random order.

#[allow(dead_code, unused_imports)]
#[path = "../src/generators/mod.rs"]
mod generators;

use std::time::Instant;
use voracious_radix_sort::{ab_i16_copy, ab_i16_gen, devour_sort};

fn rng(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
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

/// CHECK=1: `gen` against `sort_unstable` on random shapes and on every
/// distribution, before the timings.
fn check() {
    let mut seed = 0x1234_5678_9abc_def1u64;
    for n in 2..1200 {
        for shape in 0..9u64 {
            let base = rng(&mut seed) as u16;
            let a: Vec<i16> = (0..n)
                .map(|_| {
                    let r = rng(&mut seed);
                    (match shape {
                        0 => base.wrapping_add((r % 3) as u16),
                        1 => base.wrapping_add((r % 256) as u16),
                        2 => (r % 256) as u16 ^ 0x8080,
                        3 => (r % 200) as u16 ^ 0xFF9C,
                        4 => 0x7F80 + (r % 256) as u16,
                        5 => base.wrapping_add((r % 300) as u16),
                        6 => ((r % 256) as u16) << 8 | (base & 0xFF),
                        7 => (r % 5) as u16 * 1000,
                        _ => r as u16,
                    }) as i16
                })
                .collect();
            let mut r = a.clone();
            r.sort_unstable();
            let mut w = a.clone();
            ab_i16_gen(&mut w);
            assert!(w == r, "n {} shape {}", n, shape);
        }
    }
    let mut gens = generators::signed_i16::generators_i16();
    gens.extend(generators::narrow_extra::extra_i16::generators());
    for n in [
        65, 100, 300, 700, 1099, 1100, 1500, 2000, 3001, 5000, 10_007, 40_000, 70_000, 140_000,
        300_000,
    ] {
        for _ in 0..2 {
            for (gen, title) in gens.iter() {
                let a = gen(n);
                let mut r = a.clone();
                r.sort_unstable();
                let mut w = a.clone();
                ab_i16_gen(&mut w);
                assert!(w == r, "n {} {}", n, title);
            }
        }
    }
    println!("check ok");
}

/// TRACE build (feature `v2_trace`): the decision labels of the generic
/// chain, one array per size and distribution.
#[cfg(feature = "v2_trace")]
fn trace() {
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![5000, 100_000, 1_000_000]);
    let mut gens = generators::signed_i16::generators_i16();
    gens.extend(generators::narrow_extra::extra_i16::generators());
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let mut w = gen(n);
            let _ = voracious_radix_sort::ab_trace_take();
            ab_i16_gen(&mut w);
            let t = voracious_radix_sort::ab_trace_take();
            println!(
                "{:>8} {:14} {}",
                n,
                title.trim_start_matches("--").trim_end_matches(':').trim(),
                t.join(" ")
            );
        }
    }
}

/// PROF=gen|blz|blzu: one sort in a loop for PROF_SECS (default 4) on
/// PROF_N keys of PROF_GEN (for perf stat).
fn prof(which: &str) {
    let n: usize = std::env::var("PROF_N").ok().map_or(1_000_000, |v| v.parse().unwrap());
    let want = std::env::var("PROF_GEN").unwrap_or("Unif".into());
    let secs: f64 = std::env::var("PROF_SECS").ok().map_or(4.0, |v| v.parse().unwrap());
    let mut gens = generators::signed_i16::generators_i16();
    gens.extend(generators::narrow_extra::extra_i16::generators());
    let gen = gens.iter().find(|(_, t)| t.contains(want.as_str())).expect("PROF_GEN").0;
    let a = gen(n);
    let mut w = a.clone();
    let t = std::time::Instant::now();
    let mut count = 0usize;
    while t.elapsed().as_secs_f64() < secs {
        w.copy_from_slice(&a);
        match which {
            "blz" => blazing_radix::radix_sort::<i16>(&mut w),
            "blzu" => blazing_radix::radix_sort::<u16>(unsafe {
                std::slice::from_raw_parts_mut(w.as_mut_ptr() as *mut u16, w.len())
            }),
            _ => ab_i16_gen(&mut w),
        }
        count += 1;
    }
    println!(
        "{} {} n={} : {:.3} ns/key (copy included), {} sorts",
        which,
        want,
        n,
        t.elapsed().as_nanos() as f64 / (count * n) as f64,
        count
    );
}

fn main() {
    if std::env::var("CACHE").is_ok() {
        let (l2, big) = voracious_radix_sort::ab_cache();
        println!("L2 {} KiB, largest {} KiB", l2 / 1024, big / 1024);
        return;
    }
    if let Ok(which) = std::env::var("PROF") {
        prof(&which);
        return;
    }
    if std::env::var("CHECK").is_ok() {
        check();
    }
    #[cfg(feature = "v2_trace")]
    {
        trace();
        return;
    }
    let sorts: Vec<String> = std::env::var("SORTS")
        .unwrap_or("v2,wrap,std,turbo".into())
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![1000, 10_000, 100_000, 1_000_000]);
    let cold = std::env::var("COLD").is_ok();
    let reps: usize = std::env::var("REPS").ok().map_or(7, |v| v.parse().unwrap());
    let total: usize = std::env::var("TOTAL")
        .ok()
        .map_or(if cold { 32_000_000 } else { 2_000_000 }, |v| v.parse().unwrap());
    let mut gens = generators::signed_i16::generators_i16();
    gens.extend(generators::narrow_extra::extra_i16::generators());
    let mut csv = std::env::var("CSV").ok().map(|p| std::fs::File::create(p).unwrap());
    if let Some(f) = csv.as_mut() {
        use std::io::Write;
        let cpu = std::env::var("CPU").unwrap_or("unknown".into());
        writeln!(
            f,
            "== machine\ncpu,{}\nsettings,ab_i16 example: median of {} interleaved, random order, \
             copy subtracted, {} keys per measure{}",
            cpu,
            reps,
            total,
            if cold { ", cold (fresh copies, inputs beyond the caches)" } else { "" }
        )
        .unwrap();
        // For the bench viewer: the library is "v2" (its reference).
        let names: Vec<&str> = sorts
            .iter()
            .map(|s| match s.as_str() {
                "gen" => "v2",
                "turbo" => "turbosort",
                x => x,
            })
            .collect();
        writeln!(
            f,
            "== i16 (ab_i16 example; ns per key)\ncols,i16,size,gen,{},blazing",
            names.join(",")
        )
        .unwrap();
    }
    let head: String = sorts.iter().map(|s| format!(" {:>8}", s)).collect();
    println!("{:>9} {:12}{} {:>8} {:>8}", "size", "gen", head, "blazing", "copy");
    let started = Instant::now();
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name = title.trim_start_matches("--").trim_end_matches(':').trim();
            let count = (total / n).max(1);
            let arrays: Vec<Vec<i16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 2;
            let mut times = vec![Vec::new(); k];
            for rep in 0..reps {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let sort_one = |w: &mut [i16]| {
                        if v < sorts.len() {
                            match sorts[v].as_str() {
                                "std" => w.sort_unstable(),
                                "turbo" => turbosort::sort(w),
                                "copy" => ab_i16_copy(w),
                                "gen" => ab_i16_gen(w),
                                // blazing u16 on the keys xored with the sign bit (the
                                // i16 order with the u16 code; timing only), and the
                                // xor alone (to subtract).
                                "blzx" => {
                                    let b = unsafe {
                                        std::slice::from_raw_parts_mut(
                                            w.as_mut_ptr() as *mut u16,
                                            w.len(),
                                        )
                                    };
                                    for x in b.iter_mut() {
                                        *x ^= 0x8000;
                                    }
                                    blazing_radix::radix_sort::<u16>(b);
                                },
                                "xor" => {
                                    let b = unsafe {
                                        std::slice::from_raw_parts_mut(
                                            w.as_mut_ptr() as *mut u16,
                                            w.len(),
                                        )
                                    };
                                    for x in b.iter_mut() {
                                        *x ^= 0x8000;
                                    }
                                    std::hint::black_box(&b);
                                },
                                // blazing on the same bits read as u16 (timing only).
                                "blzu" => blazing_radix::radix_sort::<u16>(unsafe {
                                    std::slice::from_raw_parts_mut(
                                        w.as_mut_ptr() as *mut u16,
                                        w.len(),
                                    )
                                }),
                                // The same work in the u16 instance (wrong
                                // order for i16: timing only, COLD=1).
                                "asu16" => voracious_radix_sort::ab_u16_gen(unsafe {
                                    std::slice::from_raw_parts_mut(
                                        w.as_mut_ptr() as *mut u16,
                                        w.len(),
                                    )
                                }),
                                _ => devour_sort(w),
                            }
                        } else if v == sorts.len() {
                            blazing_radix::radix_sort::<i16>(w);
                        }
                    };
                    let t = Instant::now();
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
                    if !cold && rep == 0 && v <= sorts.len() {
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
                (0..=sorts.len()).map(|v| format!(" {:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:12}{} {:8.3}", n, name.replace(' ', "_"), cells, copy);
            if let Some(f) = csv.as_mut() {
                use std::io::Write;
                let vals: Vec<String> = (0..=sorts.len())
                    .map(|v| format!("{:.4}", (med(&times[v]) - copy).max(0.001)))
                    .collect();
                writeln!(f, "row,i16,{},{},{}", n, name, vals.join(",")).unwrap();
            }
        }
    }
    if let Some(f) = csv.as_mut() {
        use std::io::Write;
        writeln!(f, "== done in {} min", started.elapsed().as_secs() / 60).unwrap();
    }
}
