//! Portable benchmark of `float_sort`, meant to be run on several machines
//! (x86, ARM, Apple...) to check that what was tuned on one machine holds.
//!
//! Run it (release build, one test thread; ignored by a plain `cargo test`):
//!
//! ```text
//! cargo test --release --lib --features voracious_multithread bench_portable -- --ignored --nocapture --test-threads=1
//! ```
//!
//! (The test modules are only built with the `voracious_multithread` feature.)
//!
//! (On Windows on ARM use the native toolchain, e.g. `cargo +stable-aarch64-pc-windows-msvc`:
//! an x86_64 toolchain runs under emulation and distorts the timings.)
//!
//! Environment variables (all optional):
//! - `BENCH_PARTS`: comma-separated parts among `sort`, `knobs`, `aa` (all by default);
//! - `BENCH_TYPES`: types of the `sort` part among `f64`, `f32`, `u32`, `u64`,
//!   `i32`, `i64` (all by default; integers: `int_sort` against std and
//!   `voracious_sort`);
//! - `BENCH_MAX`: largest array size (default 10_000_000);
//! - `BENCH_MEM_MB`: memory budget of one measurement in MiB (default 2048);
//! - `BENCH_REPS`: repetitions per measurement, the median is kept (default 3);
//! - `BENCH_OUT`: output directory (default `target/bench_portable`).
//!
//! Method: for each size, batches of arrays from each generator; each variant
//! sorts a fresh copy of the same batch; the order of the variants rotates
//! between repetitions (no variant always runs first, after the generation or
//! on a cold cache); the median of the repetitions is kept, in ns per key. In
//! the first repetition every result is checked bit for bit against
//! `sort_unstable_by(total_cmp)`.
//!
//! Parts:
//! - `sort`: `float_sort` against `sort_unstable_by(total_cmp)` (std),
//!   `voracious_sort` (this crate's generic sort) and, for `f32`,
//!   blazing-radix, on every generator; summary per size: geometric mean of
//!   the ratios, distributions won, worst distributions.
//! - `knobs`: the machine-dependent settings, each varied while the others
//!   keep their value: the L2 size the thresholds are derived from, the
//!   rollercoaster leaf size, the DLSD staged scatter, the fused LSD
//!   cache-aliasing fix, the in-place permutation (ska swap or blocks). If
//!   the best setting is the default one, the tuning holds on this machine.
//! - `aa`: two identical variants of `float_sort`, to see the noise of the
//!   machine (any difference smaller than this is meaningless).
//!
//! Output: CSV files and `system.txt` in `BENCH_OUT`, summaries on stdout.

use std::io::Write;
use std::sync::atomic::Ordering::Relaxed;

use super::super::dedicated::cache_info::{cache_info, L2_OVERRIDE};
use super::super::dedicated::float_dlsd::{dlsd64, NO_STAGE};
use super::super::dedicated::float_lsd::{w32, w64, NO_ALIAS_FIX};
use super::super::dedicated::float_roller::{roller32, roller64, BLOCK_PERMUTE, LEAF_OVERRIDE};
use super::super::dedicated::float_sort::float_sort;
use super::super::generators::float_32::*;
use super::super::generators::float_64::*;
use super::super::generators::signed_i32::generators_i32;
use super::super::generators::signed_i64::generators_i64;
use super::super::generators::unsigned_u32::generators_u32;
use super::super::generators::unsigned_u64::generators_u64;
use super::super::int_sort;
use super::super::RadixSort;

type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

struct Config {
    max: usize,
    mem: usize,
    reps: usize,
    out: std::path::PathBuf,
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|s| s.replace('_', "").parse().ok()).unwrap_or(default)
}

fn config() -> Config {
    Config {
        max: env_usize("BENCH_MAX", 10_000_000),
        mem: env_usize("BENCH_MEM_MB", 2048) << 20,
        reps: env_usize("BENCH_REPS", 3).max(1),
        out: std::env::var("BENCH_OUT").map(Into::into).unwrap_or_else(|_| "target/bench_portable".into()),
    }
}

/// A sort to measure: `setup` sets the knobs it needs (it runs before each
/// timed batch, outside the timing), `sort` sorts one array.
struct Variant<'a, T> {
    name: &'a str,
    setup: Box<dyn Fn() + 'a>,
    sort: Box<dyn Fn(&mut [T]) + 'a>,
}

fn variant<'a, T>(name: &'a str, sort: impl Fn(&mut [T]) + 'a) -> Variant<'a, T> {
    Variant { name, setup: Box::new(|| {}), sort: Box::new(sort) }
}

fn variant_with<'a, T>(name: &'a str, setup: impl Fn() + 'a, sort: impl Fn(&mut [T]) + 'a) -> Variant<'a, T> {
    Variant { name, setup: Box::new(setup), sort: Box::new(sort) }
}

/// Default values of every knob (set after each knob sweep).
fn reset_knobs() {
    L2_OVERRIDE.store(0, Relaxed);
    LEAF_OVERRIDE.store(0, Relaxed);
    NO_STAGE.store(false, Relaxed);
    NO_ALIAS_FIX.store(false, Relaxed);
    BLOCK_PERMUTE.store(false, Relaxed);
}

fn short_name(name: &str) -> String {
    name.trim_start_matches("-- ").trim_end_matches(':').trim().to_string()
}

/// Number of arrays per batch for `size` keys of `bytes` bytes: about 4M keys
/// per batch, within the memory budget (batch + working copy + scratch); 0 if
/// even one array does not fit.
fn batch_count(cfg: &Config, size: usize, bytes: usize) -> usize {
    let per_array = size * bytes * 3;
    if per_array > cfg.mem {
        return 0;
    }
    (4_000_000 / size).max(1).min(cfg.mem / per_array)
}

/// Median ns per key of each variant, for each generator, at `size`.
/// Returns (generator, medians) rows.
fn measure<T: Copy + PartialOrd>(
    cfg: &Config,
    gens: &Gens<T>,
    size: usize,
    variants: &[Variant<T>],
    to_bits: fn(&T) -> u64,
    total_cmp: fn(&T, &T) -> std::cmp::Ordering,
) -> Vec<(String, Vec<f64>)> {
    let count = batch_count(cfg, size, std::mem::size_of::<T>());
    if count == 0 {
        return Vec::new();
    }
    let nv = variants.len();
    let mut rows = Vec::new();
    for (gen, name) in gens.iter() {
        let mut times: Vec<Vec<f64>> = vec![Vec::new(); nv];
        for rep in 0..cfg.reps {
            let batch: Vec<Vec<T>> = (0..count).map(|_| gen(size)).collect();
            let expected: Option<Vec<Vec<u64>>> = if rep == 0 {
                Some(
                    batch
                        .iter()
                        .map(|a| {
                            let mut e = a.clone();
                            e.sort_unstable_by(total_cmp);
                            e.iter().map(to_bits).collect()
                        })
                        .collect(),
                )
            } else {
                None
            };
            for k in 0..nv {
                let v = (k + rep) % nv;
                (variants[v].setup)();
                let mut work = batch.clone();
                let start = std::time::Instant::now();
                for a in work.iter_mut() {
                    (variants[v].sort)(a);
                }
                times[v].push(start.elapsed().as_nanos() as f64 / (count * size) as f64);
                if let Some(exp) = &expected {
                    for (a, e) in work.iter().zip(exp.iter()) {
                        assert!(
                            a.iter().map(to_bits).eq(e.iter().copied()),
                            "wrong result: {} on {} at {}",
                            variants[v].name,
                            short_name(name),
                            size
                        );
                    }
                }
            }
            reset_knobs();
        }
        let med = times
            .iter_mut()
            .map(|t| {
                t.sort_by(|a, b| a.partial_cmp(b).unwrap());
                t[t.len() / 2]
            })
            .collect();
        rows.push((short_name(name), med));
    }
    rows
}

fn geo(values: impl Iterator<Item = f64>) -> f64 {
    let (mut sum, mut n) = (0.0, 0);
    for v in values {
        sum += v.ln();
        n += 1;
    }
    (sum / n.max(1) as f64).exp()
}

fn write_rows(file: &mut std::fs::File, ty: &str, size: usize, rows: &[(String, Vec<f64>)]) {
    for (g, med) in rows {
        let vals: Vec<String> = med.iter().map(|v| format!("{:.3}", v)).collect();
        writeln!(file, "{},{},{},{}", ty, g, size, vals.join(",")).unwrap();
    }
    file.flush().unwrap();
}

fn system_info(cfg: &Config) -> String {
    let info = cache_info();
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into());
    format!(
        "arch {}\nos {}\nthreads {}\nL2 {} KiB (detected: {})\npage {} B\nrustc {}\nmax size {}\nmemory budget {} MiB\nrepetitions {}\n",
        std::env::consts::ARCH,
        std::env::consts::OS,
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        info.l2 >> 10,
        info.l2_detected,
        info.page,
        rustc.trim(),
        cfg.max,
        cfg.mem >> 20,
        cfg.reps
    )
}

fn sort_part<T: Copy + PartialOrd>(
    cfg: &Config,
    ty: &str,
    gens: &Gens<T>,
    variants: &[Variant<T>],
    to_bits: fn(&T) -> u64,
    total_cmp: fn(&T, &T) -> std::cmp::Ordering,
) {
    let names: Vec<&str> = variants.iter().map(|v| v.name).collect();
    let mut file = std::fs::File::create(cfg.out.join(format!("sort_{}.csv", ty))).unwrap();
    writeln!(file, "type,gen,size,{}", names.join(",")).unwrap();
    println!("\n== sort {} (ns per key; ratios > 1: {} faster)", ty, names[0]);
    for &size in [1_000usize, 10_000, 100_000, 1_000_000, 10_000_000].iter().filter(|&&s| s <= cfg.max) {
        let rows = measure(cfg, gens, size, variants, to_bits, total_cmp);
        if rows.is_empty() {
            println!("{:>9}: skipped (memory budget)", size);
            continue;
        }
        write_rows(&mut file, ty, size, &rows);
        let mut line = format!("{:>9}: {} {:.2} ns", size, names[0], geo(rows.iter().map(|r| r.1[0])));
        for v in 1..variants.len() {
            let ratio = geo(rows.iter().map(|r| r.1[v] / r.1[0]));
            let wins = rows.iter().filter(|r| r.1[0] < r.1[v]).count();
            line += &format!(" | {}/fs {:.2} (fs wins {}/{})", names[v], ratio, wins, rows.len());
        }
        println!("{}", line);
        // Worst distributions: float_sort against the best other variant.
        let mut worst: Vec<(f64, &str)> = rows
            .iter()
            .map(|r| {
                let best_other = r.1[1..].iter().cloned().fold(f64::MAX, f64::min);
                (r.1[0] / best_other, r.0.as_str())
            })
            .collect();
        worst.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let w: Vec<String> = worst.iter().take(3).filter(|x| x.0 > 1.0).map(|x| format!("{} {:.2}x", x.1, x.0)).collect();
        if !w.is_empty() {
            println!("           slower than the best other on: {}", w.join(", "));
        }
    }
}

/// One knob swept on one sort: prints the geometric mean of each setting and
/// marks the default.
fn knob<T: Copy + PartialOrd>(
    cfg: &Config,
    file: &mut std::fs::File,
    title: &str,
    ty: &str,
    gens: &Gens<T>,
    sizes: &[usize],
    variants: &[Variant<T>],
    to_bits: fn(&T) -> u64,
    total_cmp: fn(&T, &T) -> std::cmp::Ordering,
) {
    println!("\n-- {} ({}; default first)", title, ty);
    for &size in sizes.iter().filter(|&&s| s <= cfg.max) {
        let rows = measure(cfg, gens, size, variants, to_bits, total_cmp);
        if rows.is_empty() {
            println!("{:>9}: skipped (memory budget)", size);
            continue;
        }
        let g: Vec<f64> = (0..variants.len()).map(|v| geo(rows.iter().map(|r| r.1[v]))).collect();
        let best = (0..g.len()).min_by(|&a, &b| g[a].partial_cmp(&g[b]).unwrap()).unwrap();
        let cells: Vec<String> = (0..g.len())
            .map(|v| format!("{}{} {:.2}", if v == best { "*" } else { "" }, variants[v].name, g[v]))
            .collect();
        println!("{:>9}: {}", size, cells.join(" | "));
        for (v, val) in g.iter().enumerate() {
            writeln!(file, "{},{},{},{},{:.3}", title, ty, size, variants[v].name, val).unwrap();
        }
        file.flush().unwrap();
    }
}

fn bits64(a: &mut [f64]) -> &mut [u64] {
    // SAFETY: same size and alignment, every bit pattern valid for both.
    unsafe { &mut *(a as *mut [f64] as *mut [u64]) }
}

fn bits32(a: &mut [f32]) -> &mut [u32] {
    // SAFETY: same size and alignment, every bit pattern valid for both.
    unsafe { &mut *(a as *mut [f32] as *mut [u32]) }
}

fn f64_bits(x: &f64) -> u64 {
    x.to_bits()
}

fn f32_bits(x: &f32) -> u64 {
    x.to_bits() as u64
}

fn f64_cmp(a: &f64, b: &f64) -> std::cmp::Ordering {
    a.total_cmp(b)
}

fn f32_cmp(a: &f32, b: &f32) -> std::cmp::Ordering {
    a.total_cmp(b)
}

fn knobs_part(cfg: &Config) {
    let mut file = std::fs::File::create(cfg.out.join("knobs.csv")).unwrap();
    writeln!(file, "knob,type,size,setting,ns_per_key").unwrap();
    println!("\n== knobs (geometric mean, ns per key; * = best)");
    let g64 = generators_f64();
    let g32 = generators_f32();

    // L2 size the thresholds derive from (detected first).
    let detected = cache_info().l2;
    let mut l2s = vec![detected];
    for &v in [1usize << 20, 2 << 20, 4 << 20, 16 << 20, 32 << 20].iter() {
        if v != detected {
            l2s.push(v);
        }
    }
    let names: Vec<String> = l2s
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{}{}KiB", if i == 0 { "detected=" } else { "" }, v >> 10))
        .collect();
    let v64: Vec<Variant<f64>> = l2s
        .iter()
        .zip(names.iter())
        .map(|(&l2, n)| {
            let o = if l2 == detected { 0 } else { l2 };
            variant_with(n.as_str(), move || L2_OVERRIDE.store(o, Relaxed), |a: &mut [f64]| float_sort(a))
        })
        .collect();
    knob(cfg, &mut file, "L2 size (float_sort)", "f64", &g64, &[250_000, 1_000_000, 4_000_000], &v64, f64_bits, f64_cmp);
    let v32: Vec<Variant<f32>> = l2s
        .iter()
        .zip(names.iter())
        .map(|(&l2, n)| {
            let o = if l2 == detected { 0 } else { l2 };
            variant_with(n.as_str(), move || L2_OVERRIDE.store(o, Relaxed), |a: &mut [f32]| float_sort(a))
        })
        .collect();
    knob(cfg, &mut file, "L2 size (float_sort)", "f32", &g32, &[1_000_000, 4_000_000], &v32, f32_bits, f32_cmp);

    // Rollercoaster leaf size (keys).
    let leaves = [32_768usize, 8_192, 16_384, 65_536, 131_072];
    let leaf_names: Vec<String> = leaves.iter().map(|l| format!("{}k", l >> 10)).collect();
    let v: Vec<Variant<f64>> = leaves
        .iter()
        .zip(leaf_names.iter())
        .map(|(&l, n)| variant_with(n.as_str(), move || LEAF_OVERRIDE.store(l, Relaxed), |a: &mut [f64]| roller64(bits64(a))))
        .collect();
    knob(cfg, &mut file, "leaf keys (rollercoaster)", "f64", &g64, &[4_000_000], &v, f64_bits, f64_cmp);
    let v: Vec<Variant<f32>> = leaves
        .iter()
        .zip(leaf_names.iter())
        .map(|(&l, n)| variant_with(n.as_str(), move || LEAF_OVERRIDE.store(l, Relaxed), |a: &mut [f32]| roller32(bits32(a))))
        .collect();
    knob(cfg, &mut file, "leaf keys (rollercoaster)", "f32", &g32, &[4_000_000], &v, f32_bits, f32_cmp);

    // DLSD staged scatter (cache aliasing).
    let v: Vec<Variant<f64>> = vec![
        variant_with("staging", || NO_STAGE.store(false, Relaxed), |a: &mut [f64]| dlsd64(bits64(a))),
        variant_with("no staging", || NO_STAGE.store(true, Relaxed), |a: &mut [f64]| dlsd64(bits64(a))),
    ];
    knob(cfg, &mut file, "staged scatter (DLSD)", "f64", &g64, &[1_000_000, 4_000_000], &v, f64_bits, f64_cmp);

    // Fused LSD cache-aliasing fix.
    let v: Vec<Variant<f32>> = vec![
        variant_with("alias fix", || NO_ALIAS_FIX.store(false, Relaxed), |a: &mut [f32]| w32::lsd(bits32(a))),
        variant_with("no alias fix", || NO_ALIAS_FIX.store(true, Relaxed), |a: &mut [f32]| w32::lsd(bits32(a))),
    ];
    knob(cfg, &mut file, "aliasing fix (fused LSD)", "f32", &g32, &[1_000_000, 10_000_000], &v, f32_bits, f32_cmp);
    let v: Vec<Variant<f64>> = vec![
        variant_with("alias fix", || NO_ALIAS_FIX.store(false, Relaxed), |a: &mut [f64]| w64::lsd(bits64(a))),
        variant_with("no alias fix", || NO_ALIAS_FIX.store(true, Relaxed), |a: &mut [f64]| w64::lsd(bits64(a))),
    ];
    knob(cfg, &mut file, "aliasing fix (fused LSD)", "f64", &g64, &[1_000_000, 10_000_000], &v, f64_bits, f64_cmp);

    // In-place permutation of the rollercoaster MSD pass.
    let v: Vec<Variant<f64>> = vec![
        variant_with("ska swap", || BLOCK_PERMUTE.store(false, Relaxed), |a: &mut [f64]| roller64(bits64(a))),
        variant_with("blocks", || BLOCK_PERMUTE.store(true, Relaxed), |a: &mut [f64]| roller64(bits64(a))),
    ];
    knob(cfg, &mut file, "permutation (rollercoaster)", "f64", &g64, &[10_000_000, 50_000_000], &v, f64_bits, f64_cmp);
    reset_knobs();
}

fn aa_part(cfg: &Config) {
    println!("\n== A/A noise (two identical float_sort variants)");
    let mut file = std::fs::File::create(cfg.out.join("aa.csv")).unwrap();
    writeln!(file, "type,gen,size,a,b").unwrap();
    let g64 = generators_f64();
    let v: Vec<Variant<f64>> = vec![variant("A", |a: &mut [f64]| float_sort(a)), variant("B", |a: &mut [f64]| float_sort(a))];
    for &size in [10_000usize, 1_000_000].iter().filter(|&&s| s <= cfg.max) {
        let rows = measure(cfg, &g64, size, &v, f64_bits, f64_cmp);
        if rows.is_empty() {
            continue;
        }
        write_rows(&mut file, "f64", size, &rows);
        let ratios: Vec<f64> = rows.iter().map(|r| r.1[1] / r.1[0]).collect();
        let worst = ratios.iter().map(|r| if *r > 1.0 { *r } else { 1.0 / r }).fold(1.0, f64::max);
        println!(
            "{:>9}: B/A geometric mean {:.3}, worst single distribution {:.2}x (differences below this are noise)",
            size,
            geo(ratios.iter().cloned()),
            worst
        );
    }
}

#[test]
#[ignore]
fn bench_portable() {
    let cfg = config();
    std::fs::create_dir_all(&cfg.out).unwrap();
    let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();
    let info = system_info(&cfg);
    println!("{}", info);
    std::fs::write(cfg.out.join("system.txt"), &info).unwrap();
    let parts = std::env::var("BENCH_PARTS").unwrap_or_else(|_| "sort,knobs,aa".into());
    let parts: Vec<&str> = parts.split(',').map(|p| p.trim()).collect();
    if parts.contains(&"aa") {
        aa_part(&cfg);
    }
    let types = std::env::var("BENCH_TYPES").unwrap_or_else(|_| "f64,f32,u32,u64,i32,i64".into());
    let types: Vec<&str> = types.split(',').map(|t| t.trim()).collect();
    if parts.contains(&"sort") && types.contains(&"f64") {
        let v64: Vec<Variant<f64>> = vec![
            variant("float_sort", |a: &mut [f64]| float_sort(a)),
            variant("std", |a: &mut [f64]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("voracious", |a: &mut [f64]| a.voracious_sort()),
        ];
        sort_part(&cfg, "f64", &generators_f64(), &v64, f64_bits, f64_cmp);
    }
    if parts.contains(&"sort") && types.contains(&"f32") {
        let v32: Vec<Variant<f32>> = vec![
            variant("float_sort", |a: &mut [f32]| float_sort(a)),
            variant("std", |a: &mut [f32]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("voracious", |a: &mut [f32]| a.voracious_sort()),
            variant("blazing", |a: &mut [f32]| blazing_radix::radix_sort::<f32>(a)),
        ];
        sort_part(&cfg, "f32", &generators_f32(), &v32, f32_bits, f32_cmp);
    }
    macro_rules! int_part {
        ($t:ty, $name:expr, $gens:expr, $bits:expr) => {
            if parts.contains(&"sort") && types.contains(&$name) {
                let v: Vec<Variant<$t>> = vec![
                    variant("int_sort", |a: &mut [$t]| int_sort(a)),
                    variant("std", |a: &mut [$t]| a.sort_unstable()),
                    variant("voracious", |a: &mut [$t]| a.voracious_sort()),
                ];
                sort_part(&cfg, $name, &$gens, &v, $bits, |a: &$t, b: &$t| a.cmp(b));
            }
        };
    }
    int_part!(u32, "u32", generators_u32(), |x: &u32| *x as u64);
    int_part!(u64, "u64", generators_u64(), |x: &u64| *x);
    int_part!(i32, "i32", generators_i32(), |x: &i32| *x as u32 as u64);
    int_part!(i64, "i64", generators_i64(), |x: &i64| *x as u64);
    if parts.contains(&"knobs") {
        knobs_part(&cfg);
    }
    println!("\nCSV files and system.txt written to {}", cfg.out.display());
}
