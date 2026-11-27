//! Dev only: the u32 / i32 chain against its own LSD engine and blazing-radix,
//! in a binary that builds in seconds (the test crate takes minutes).
//!
//! cargo run --release --example ab_u32 --features "bench ab"
//! Env: T=u32|i32 (default u32); SORTS="v2,lsd,std,blz" (`v2` the library,
//! `lsd` the LSD engine alone, no detector (`dlsd`, `roller`: those engines alone): v2 - lsd on a cell whose path
//! ends in the LSD is what the chain's failed questions cost);
//! `copy` the A/B copy of the chain (`ab_copy32.rs`), `copy:MASK` with the
//! questions of MASK skipped (see `ab_copy32::SKIP`);
//! DOM_SIZES="300,1000,..."; TOTAL (keys per measure, default 4M);
//! GEN=text (only the distributions containing text); GENS=a,b (only these, short names as printed); CHECK=1 (every sort
//! against `sort_unstable` first); FRESH=1 (new pages for every allocation
//! of 128 KiB or more: `common/fresh_pages.rs`).
//! TRACE build (feature `v2_trace`): the decision labels of v2 per cell.
//! Output: ns per key, copy subtracted, median of REPS (7), random order.

#[allow(dead_code, unused_imports)]
#[path = "../src/generators/mod.rs"]
mod generators;

#[path = "common/fresh_pages.rs"]
mod fresh_pages;
#[global_allocator]
static ALLOC: fresh_pages::Fresh = fresh_pages::Fresh;

use std::time::Instant;
use voracious_radix_sort::{
    ab_i32_copy, ab_i32_engine, ab_i32_lsd, ab_u32_copy, ab_u32_engine, ab_u32_lsd, devour_sort,
};

type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

trait Key: voracious_radix_sort::DevourSort + Send + Sync + std::fmt::Debug + 'static {
    fn gens() -> Gens<Self>;
    fn lsd(w: &mut [Self]);
    fn blz(w: &mut [Self]);
    fn turbo(w: &mut [Self]);
    fn engine(name: &str, w: &mut [Self]);
    fn copy(w: &mut [Self]);
}

impl Key for u32 {
    fn gens() -> Gens<u32> {
        let mut g = generators::unsigned_u32::generators_u32();
        g.extend(generators::int_extra::extra_u32::generators());
        g
    }
    fn lsd(w: &mut [u32]) { ab_u32_lsd(w) }
    fn blz(w: &mut [u32]) { blazing_radix::radix_sort::<u32>(w) }
    fn turbo(w: &mut [u32]) { turbosort::sort(w) }
    fn engine(name: &str, w: &mut [u32]) { ab_u32_engine(name, w) }
    fn copy(w: &mut [u32]) { ab_u32_copy(w) }
}

impl Key for i32 {
    fn gens() -> Gens<i32> {
        let mut g = generators::signed_i32::generators_i32();
        g.extend(generators::int_extra::extra_i32::generators());
        g
    }
    fn lsd(w: &mut [i32]) { ab_i32_lsd(w) }
    fn blz(w: &mut [i32]) { blazing_radix::radix_sort::<i32>(w) }
    fn turbo(w: &mut [i32]) { turbosort::sort(w) }
    fn engine(name: &str, w: &mut [i32]) { ab_i32_engine(name, w) }
    fn copy(w: &mut [i32]) { ab_i32_copy(w) }
}

fn run<T: Key>(name: &str, w: &mut [T])
{
    match name {
        "std" => w.sort_unstable(),
        "blz" => T::blz(w),
        "turbo" => T::turbo(w),
        "glide" => glidesort::sort(w),
        "lsd" => T::lsd(w),
        "copy" => T::copy(w),
        c if c.starts_with("copy:") => {
            voracious_radix_sort::ab_copy32_skip(c[5..].parse().unwrap());
            T::copy(w);
            voracious_radix_sort::ab_copy32_skip(0);
        },
        "dlsd" | "dlsdr" | "dlsds" | "dlsdo" | "rollo" | "roller" | "lsdc" => T::engine(name, w),
        c if c.starts_with("lsdc:") => {
            voracious_radix_sort::ab_copy32_skip(c[5..].parse().unwrap());
            T::engine("lsdc", w);
            voracious_radix_sort::ab_copy32_skip(0);
        },
        _ => devour_sort(w),
    }
}

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

fn name_of(title: &str) -> String {
    title.trim_start_matches("--").trim_end_matches(':').trim().replace(' ', "_")
}

fn gens_wanted<T: Key>() -> Gens<T> {
    let want = std::env::var("GEN").ok();
    // GENS=a,b,c: only the distributions whose short name (as printed,
    // `_` for spaces) is in the list.
    let list: Option<Vec<String>> = std::env::var("GENS").ok().filter(|v| !v.is_empty()).map(|v| v.split(',').map(String::from).collect());
    T::gens()
        .into_iter()
        .filter(|(_, t)| want.as_ref().is_none_or(|w| t.contains(w.as_str())))
        .filter(|(_, t)| list.as_ref().is_none_or(|l| l.iter().any(|x| *x == name_of(t))))
        .collect()
}

fn check<T: Key>(sorts: &[String], sizes: &[usize])
{
    for &n in sizes {
        for (gen, title) in gens_wanted::<T>().iter() {
            let a = gen(n);
            let mut r = a.clone();
            r.sort_unstable();
            for s in sorts {
                let mut w = a.clone();
                run(s, &mut w);
                assert!(w == r, "n {} {} sort {}", n, title, s);
            }
        }
    }
    println!("check ok");
}

#[cfg(feature = "v2_trace")]
fn trace<T: Key>(sizes: &[usize])
{
    for &n in sizes {
        for (gen, title) in gens_wanted::<T>().iter() {
            let mut w = gen(n);
            let _ = voracious_radix_sort::ab_trace_take();
            // TRACE_SORT: the sort traced (default v2).
            run::<T>(&std::env::var("TRACE_SORT").unwrap_or("v2".into()), &mut w);
            let t = voracious_radix_sort::ab_trace_take();
            println!("{:>9} {:14} {}", n, name_of(title), t.join(" "));
        }
    }
}

fn bench<T: Key>(sorts: &[String], sizes: &[usize])
{
    // COLD=1: below 1M keys every sort on a fresh copy (allocated, page
    // faults paid, the copy subtracted), 32M keys per measure by default
    // (inputs far beyond the last cache); from 1M keys the work arrays are
    // reused (already out of cache). As `ab_u16`.
    let cold = std::env::var("COLD").is_ok();
    let reps: usize = std::env::var("REPS").ok().map_or(7, |v| v.parse().unwrap());
    let total: usize = std::env::var("TOTAL")
        .ok()
        .map_or(if cold { 32_000_000 } else { 4_000_000 }, |v| v.parse().unwrap());
    let head: String = sorts.iter().map(|s| format!(" {:>8}", s)).collect();
    println!("{:>9} {:14}{} {:>8}", "size", "gen", head, "memcpy");
    // CSV=path: a bench file for the bench viewer (CPU=name, LABEL=type).
    let label = std::env::var("T").unwrap_or("u32".into());
    let mut csv = std::env::var("CSV").ok().map(|p| std::fs::File::create(p).unwrap());
    if let Some(f) = csv.as_mut() {
        use std::io::Write;
        let cpu = std::env::var("CPU").unwrap_or("unknown".into());
        writeln!(
            f,
            "== machine
cpu,{}
settings,ab_u32 example: median of {} interleaved, random order, copy subtracted, {} keys per measure{}",
            cpu,
            reps,
            total,
            if cold { ", cold (fresh copies, inputs beyond the caches)" } else { "" }
        )
        .unwrap();
        // Readable names for the viewer.
        let names: Vec<&str> = sorts
            .iter()
            .map(|s| match s.as_str() {
                "v2" => "voracious 2.1",
                "std" => "std",
                "blz" => "blazing",
                "turbo" => "turbosort",
                "glide" => "glidesort",
                c if c.starts_with("copy") => "voracious new",
                o => o,
            })
            .collect();
        writeln!(f, "== {} (ab_u32 example; ns per key)
cols,{},size,gen,{}", label, label, names.join(","))
            .unwrap();
    }
    for &n in sizes {
        for (gen, title) in gens_wanted::<T>().iter() {
            let count = (total / n).max(1);
            let arrays: Vec<Vec<T>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            // The sorts, then the copy alone (subtracted).
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..reps {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    if cold && n < 1_000_000 {
                        for a in arrays.iter() {
                            let mut w = a.clone();
                            if v < sorts.len() {
                                run(&sorts[v], &mut w);
                            }
                            std::hint::black_box(&w);
                        }
                    } else {
                        for (w, a) in work.iter_mut().zip(arrays.iter()) {
                            w.copy_from_slice(a);
                            if v < sorts.len() {
                                run(&sorts[v], w);
                            }
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if !cold && rep == 0 && v < sorts.len() {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", sorts[v], n, title);
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
                (0..sorts.len()).map(|v| format!(" {:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:14}{} {:8.3}", n, name_of(title), cells, copy);
            if let Some(f) = csv.as_mut() {
                use std::io::Write;
                let vals: Vec<String> = (0..sorts.len())
                    .map(|v| format!("{:.4}", (med(&times[v]) - copy).max(0.001)))
                    .collect();
                let gen_name = title.trim_start_matches("--").trim_end_matches(':').trim();
                writeln!(f, "row,{},{},{},{}", label, n, gen_name, vals.join(",")).unwrap();
            }
        }
    }
}

/// PROF=sort: only that sort, on PROF_GEN (default Unif) arrays of PROF_N
/// keys (default 1000), for ~PROF_SECS seconds (default 4), for `perf`.
fn prof<T: Key>(which: &str) {
    let n: usize = std::env::var("PROF_N").ok().map_or(1000, |v| v.parse().unwrap());
    let want = std::env::var("PROF_GEN").unwrap_or("Unif".into());
    let secs: f64 = std::env::var("PROF_SECS").ok().map_or(4.0, |v| v.parse().unwrap());
    let gen = T::gens().into_iter().find(|(_, t)| name_of(t) == want).expect("PROF_GEN").0;
    let count = (2_000_000 / n).max(1);
    let arrays: Vec<Vec<T>> = (0..count).map(|_| gen(n)).collect();
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
    println!("{} {} n={} : {:.3} ns/key (copy included)", which, want, n, ns);
}

fn go<T: Key>()
{
    let sorts: Vec<String> = std::env::var("SORTS")
        .unwrap_or("v2,lsd,std,blz".into())
        .split(',')
        .map(String::from)
        .collect();
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![300, 500, 1000, 2000, 5000, 10_000, 50_000, 100_000, 1_000_000]);
    if std::env::var("CHECK").is_ok() {
        check::<T>(&sorts, &sizes);
    }
    if std::env::var("DSTATS").is_ok() {
        // The stack DLSD's decisions per cell (u32 only), one array each.
        for &n in &sizes {
            for (gen, title) in u32::gens().iter() {
                if std::env::var("GEN").ok().is_some_and(|w| !title.contains(w.as_str())) {
                    continue;
                }
                let mut w = gen(n);
                let _ = voracious_radix_sort::ab_dlsd_stats_take();
                if let Some(m) = std::env::var("DSKIP").ok().and_then(|v| v.parse().ok()) {
                    voracious_radix_sort::ab_copy32_skip(m);
                }
                u32::engine("dlsds", &mut w);
                let st = voracious_radix_sort::ab_dlsd_stats_take();
                let calls: Vec<String> = st
                    .iter()
                    .take(4)
                    .map(|c| format!("n{} top{} pre{} w{}x{} f{}", c.0, c.1, c.2, c.3, c.4, c.5))
                    .collect();
                let staged = 0;
                println!(
                    "{:>8} {:14} calls {:>4} staged {}  {}",
                    n,
                    name_of(title),
                    st.len(),
                    staged,
                    calls.join(" | ")
                );
            }
        }
        return;
    }
    if let Ok(which) = std::env::var("PROF") {
        prof::<T>(&which);
        return;
    }
    #[cfg(feature = "v2_trace")]
    {
        trace::<T>(&sizes);
        return;
    }
    #[allow(unreachable_code)]
    bench::<T>(&sorts, &sizes);
}

fn main() {
    fresh_pages::init();
    match std::env::var("T").as_deref() {
        Ok("i32") => go::<i32>(),
        _ => go::<u32>(),
    }
}
