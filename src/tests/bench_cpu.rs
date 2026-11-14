//! CPU benchmark of the v2 sorts against the other sorts, to run on other
//! machines: `make benchcpu` (or `make benchcpu-quick`).
//!
//! Everything goes to one file at the repository root,
//! `bench-cpu-<host>-<date>.txt` (written as it goes: a run stopped early
//! still leaves its results):
//! - the machine: CPU, cores, L2 as reported, page size, OS, rustc, commit;
//! - per part, type and size: the ns per key of every sort on every
//!   distribution (`row` lines, CSV), then a summary of the ratios
//!   other / v2 (> 1: v2 faster): geometric mean, median, p10, wins;
//! - every result is checked to be sorted in the first repetition
//!   (`CHECK FAILED` lines otherwise; voracious 1.x mixes -0.0 and +0.0, a
//!   known v1 bug, on `Zeros +-`).
//!
//! Parts (`BENCH_PARTS`, all but `tune` by default):
//! - `aa`: v2 against itself, the noise of the machine;
//! - `int`, `float`, `wide`: integers, floats, 128-bit integers against
//!   std, voracious 1.x and the radix crates (blazing, radsort, rdst,
//!   turbosort) where they take the type;
//! - `records`: sort by key (stable) of records of 8, 16, 32 and 128 bytes
//!   (`v2` the work copy `records2`, `v2 old` the current `records`) against
//!   std (stable and unstable), glidesort, radsort, rdst, turbosort and
//!   voracious 1.x;
//! - `rparams`: the CPU parameters of the records sort: the record size
//!   from which run merges go 4-way instead of 2-way (`FOUR_WAY_MIN`), on
//!   the shapes with runs, records of 16, 32 and 128 bytes; and the digit
//!   width of the LSD on 16-byte records (11 bits against 8);
//! - `l2`: v2 with the L2 size it derives its thresholds from forced to
//!   half, twice and four times the detected one, and to 12 MiB (the
//!   Snapdragon X the thresholds were tuned on): if the detected one is
//!   the fastest, the tuning holds on this machine (four times: whether a
//!   small L2 backed by a large L3 should count for more);
//! - `tune` (`make benchtune`, a few hours): one cache-derived decision at
//!   a time (`v2::cache::Budget`), its budget forced to L2 / 4, L2 x 4,
//!   L2 x 16 and the L3, the others at their default, 10k to 10M, on the
//!   types it touches: which cache level each decision should follow on
//!   this CPU. `BENCH_TUNE`: decisions to run (`lsd11,roller32,...`).
//!
//! Sizes: 1k to 10M for everything; 30M and 100M for the main types only
//! (`BIG_LABELS`), and only when an array, its copy and a buffer fit in the
//! memory budget.
//!
//! Settings: `BENCH_MIN` / `BENCH_MAX` (smallest / largest size, default
//! 1k / 100M), `BENCH_REPS` (repetitions, the median is kept, default 3; at
//! least 5 arrays sorted per measurement), `BENCH_MEM_MB` (memory budget,
//! default half the RAM), `BENCH_TYPES` (filter), `BENCH_OUT` (output
//! directory, default the repository root), `BENCH_PIN=0` (do not pin the
//! thread to a fast core on hybrid CPUs; `slow`: pin it to a slow one).
//! A panic is written to the file before the run stops.

use std::io::Write;
use std::sync::atomic::Ordering::Relaxed;

use super::super::generators as g;
use super::super::generators::records::*;
use super::super::v2::cache::{l2_bytes, page_bytes, reported_l2, reported_levels, Budget, BUDGETS, BUDGET_OVERRIDE, L2_OVERRIDE};
use super::super::v2::floats::ravenous_sort2;
use super::super::v2::integers::devour_sort2;
use super::super::v2::records::devour_sort_by_key;
use super::super::v2::records2;
use super::super::RadixSort;

type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

const SIZES: [usize; 12] = [1_000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000, 3_000_000, 10_000_000, 30_000_000, 100_000_000];
/// Beyond this size only the types of `BIG_LABELS` are measured.
const BIG: usize = 10_000_000;
const BIG_LABELS: [&str; 13] = ["aa u64", "u32", "u64", "i64", "f32", "f64", "u128", "records u32 8B", "records u64 16B", "records u64 32B", "l2 u32", "l2 u64", "l2 f64"];
/// L2 of the Snapdragon X the thresholds were tuned on (`l2` part: the same
/// engine choices as there).
const ARM_L2: usize = 12 << 20;
/// Keys per batch of arrays (small sizes: many arrays per measurement).
const BATCH_KEYS: usize = 4_000_000;
/// Most bytes in a batch (large records).
const BATCH_BYTES: usize = 256 << 20;
/// At least this many arrays sorted per measurement, over the repetitions.
const MIN_ARRAYS: usize = 5;

struct Config {
    min: usize,
    max: usize,
    reps: usize,
    parts: Vec<String>,
    types: Option<Vec<String>>,
    memory: usize,
}

impl Config {
    fn from_env() -> Config {
        let list = |name: &str| std::env::var(name).ok().map(|s| s.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>());
        let number = |name: &str, default: usize| std::env::var(name).ok().and_then(|s| s.replace('_', "").parse().ok()).unwrap_or(default);
        Config {
            min: number("BENCH_MIN", 0),
            max: number("BENCH_MAX", 100_000_000),
            reps: number("BENCH_REPS", 3).max(1),
            parts: list("BENCH_PARTS").unwrap_or_else(|| ["aa", "int", "float", "wide", "records", "rparams", "l2"].iter().map(|s| s.to_string()).collect()),
            types: list("BENCH_TYPES"),
            memory: number("BENCH_MEM_MB", (total_ram().unwrap_or(16 << 30) / 2) >> 20) << 20,
        }
    }

    fn part(&self, part: &str) -> bool {
        self.parts.iter().any(|p| p == part)
    }

    fn wants(&self, part: &str, ty: &str) -> bool {
        self.part(part) && self.types.as_ref().map_or(true, |t| t.iter().any(|x| x == ty))
    }
}

/// The output file, every line also printed.
struct Report {
    file: std::fs::File,
}

impl Report {
    fn line(&mut self, text: &str) {
        println!("{}", text);
        writeln!(self.file, "{}", text).unwrap();
        self.file.flush().unwrap();
    }
}

/// A sort to measure.
struct Variant<'a, T> {
    name: &'a str,
    sort: Box<dyn Fn(&mut [T]) + 'a>,
}

fn variant<'a, T>(name: &'a str, sort: impl Fn(&mut [T]) + 'a) -> Variant<'a, T> {
    Variant { name, sort: Box::new(sort) }
}

fn short_name(name: &str) -> String {
    name.trim_start_matches("-- ").trim_end_matches(':').trim().replace(',', ";")
}

fn geo(values: &[f64]) -> f64 {
    (values.iter().map(|v| v.ln()).sum::<f64>() / values.len().max(1) as f64).exp()
}

/// Median ns per key of each variant on each distribution at `size`. The
/// variants sort copies of the same batch, in an order rotating between
/// repetitions; `ordered(a, b)`: `a` may come before `b`.
fn measure<T: Clone>(report: &mut Report, cfg: &Config, label: &str, gens: &Gens<T>, size: usize, variants: &[Variant<T>], ordered: fn(&T, &T) -> bool) -> Vec<(String, Vec<f64>)> {
    let bytes = std::mem::size_of::<T>().max(1);
    let count = (BATCH_KEYS / size).min(BATCH_BYTES / (size * bytes)).max(1);
    let reps = cfg.reps.max((MIN_ARRAYS + count - 1) / count);
    let nv = variants.len();
    let mut rows = Vec::new();
    for (gen, name) in gens.iter() {
        let mut times: Vec<Vec<f64>> = vec![Vec::new(); nv];
        for rep in 0..reps {
            let batch: Vec<Vec<T>> = (0..count).map(|_| gen(size)).collect();
            for k in 0..nv {
                let v = (k + rep) % nv;
                let mut work = batch.clone();
                let start = std::time::Instant::now();
                // A sort that panics loses its measurement, not the run.
                let sorted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for a in work.iter_mut() {
                        (variants[v].sort)(a);
                    }
                }));
                if sorted.is_err() {
                    report.line(&format!("SORT PANICKED,{},{},{},{}", label, size, short_name(name), variants[v].name));
                    times[v].push(f64::NAN);
                    continue;
                }
                times[v].push(start.elapsed().as_nanos() as f64 / (count * size) as f64);
                if rep == 0 && !work.iter().all(|a| a.windows(2).all(|p| ordered(&p[0], &p[1]))) {
                    report.line(&format!("CHECK FAILED,{},{},{},{}", label, size, short_name(name), variants[v].name));
                }
            }
        }
        let medians = times
            .iter_mut()
            .map(|t| {
                t.sort_by(|a, b| a.total_cmp(b));
                t[t.len() / 2]
            })
            .collect();
        rows.push((short_name(name), medians));
    }
    rows
}

/// One type of one part: raw rows and a summary per size.
fn bench<T: Clone>(report: &mut Report, cfg: &Config, label: &str, gens: &Gens<T>, variants: &[Variant<T>], ordered: fn(&T, &T) -> bool, min_size: usize) {
    let names: Vec<&str> = variants.iter().map(|v| v.name).collect();
    report.line("");
    report.line(&format!("== {} ({} distributions; ns per key; ratios other / {}, > 1: {} faster)", label, gens.len(), names[0], names[0]));
    report.line(&format!("cols,{},size,gen,{}", label, names.join(",")));
    for &size in SIZES.iter().filter(|&&s| s <= cfg.max && s >= min_size.max(cfg.min)) {
        if size > BIG && !BIG_LABELS.contains(&label) {
            continue;
        }
        // The batch, its working copy and a buffer of the same size.
        if size > BIG && size.saturating_mul(std::mem::size_of::<T>()).saturating_mul(4) > cfg.memory {
            report.line(&format!("skip,{},{},memory budget (BENCH_MEM_MB)", label, size));
            continue;
        }
        let started = std::time::Instant::now();
        let rows = measure(report, cfg, label, gens, size, variants, ordered);
        for (gen, ns) in rows.iter() {
            let ns: Vec<String> = ns.iter().map(|x| format!("{:.3}", x)).collect();
            report.line(&format!("row,{},{},{},{}", label, size, gen, ns.join(",")));
        }
        let first: Vec<f64> = rows.iter().map(|r| r.1[0]).collect();
        report.line(&format!("sum,{},{},{} geo {:.2} ns ({:.0} s)", label, size, names[0], geo(&first), started.elapsed().as_secs_f64()));
        for v in 1..variants.len() {
            let mut ratios: Vec<f64> = rows.iter().map(|r| r.1[v] / r.1[0]).collect();
            let wins = ratios.iter().filter(|&&x| x > 1.0).count();
            ratios.sort_by(|a, b| a.total_cmp(b));
            report.line(&format!(
                "sum,{},{},{} geo {:.2} med {:.2} p10 {:.2} wins {}/{}",
                label,
                size,
                names[v],
                geo(&ratios),
                ratios[ratios.len() / 2],
                ratios[ratios.len() / 10],
                wins,
                ratios.len()
            ));
        }
    }
}

/// The variants of one decision for the `tune` part: its default budget,
/// then forced to L2 / 4, L2 x 4, L2 x 16 and the L3 (when one is reported
/// above the L2), the other decisions untouched.
fn tune_variants<'a, T: 'a>(b: Budget, sort: impl Fn(&mut [T]) + Copy + 'a) -> Vec<Variant<'a, T>> {
    let l2 = l2_bytes();
    let mut sizes: Vec<(&'static str, usize)> = vec![("L2 / 4", l2 / 4), ("L2 x 4", l2 * 4), ("L2 x 16", l2 * 16)];
    if let Some(l3) = reported_levels()[2].filter(|&l3| l3 > l2) {
        sizes.push(("L3", l3));
    }
    let mut variants = vec![variant("default", move |a: &mut [T]| sort(a))];
    for (name, bytes) in sizes {
        variants.push(variant(name, move |a: &mut [T]| {
            BUDGET_OVERRIDE[b as usize].store(bytes, Relaxed);
            sort(a);
            BUDGET_OVERRIDE[b as usize].store(0, Relaxed);
        }));
    }
    variants
}

fn all<T>(mut a: Gens<T>, b: Gens<T>) -> Gens<T> {
    a.extend(b);
    a
}

fn command(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn cpu_name() -> String {
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        if let Some(line) = info.lines().find(|l| l.starts_with("model name") || l.starts_with("Model")) {
            return line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        }
    }
    if cfg!(windows) {
        let key = r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0";
        if let Some(out) = command("reg", &["query", key, "/v", "ProcessorNameString"]) {
            if let Some(line) = out.lines().find(|l| l.contains("ProcessorNameString")) {
                return line.split("REG_SZ").nth(1).unwrap_or("").trim().to_string();
            }
        }
    }
    command("sysctl", &["-n", "machdep.cpu.brand_string"]).unwrap_or_else(|| "unknown".into())
}

fn host_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .or_else(|| command("hostname", &[]))
        .unwrap_or_else(|| "machine".into())
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

/// UTC date and time as `YYYY-MM-DD-HHMM`.
fn utc_stamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + (month <= 2) as i64;
    format!("{:04}-{:02}-{:02}-{:02}{:02}", year, month, day, rest / 3600, rest % 3600 / 60)
}

/// L1d, L2, L3 as reported (smallest of each level).
fn cache_levels() -> String {
    let show = |name: &str, v: Option<usize>| format!("{} {}", name, v.map_or("?".to_string(), |b| format!("{} KiB", b >> 10)));
    let [l1, l2, l3] = reported_levels();
    format!("{}, {}, {}", show("L1d", l1), show("L2", l2), show("L3", l3))
}

/// Instruction sets the build may use (`-C target-cpu=native` adds them).
fn target_features() -> String {
    let mut f = Vec::new();
    if cfg!(target_feature = "sse4.2") {
        f.push("sse4.2");
    }
    if cfg!(target_feature = "avx2") {
        f.push("avx2");
    }
    if cfg!(target_feature = "bmi2") {
        f.push("bmi2");
    }
    if cfg!(target_feature = "avx512f") {
        f.push("avx512f");
    }
    if cfg!(target_feature = "neon") {
        f.push("neon");
    }
    if cfg!(target_feature = "sve") {
        f.push("sve");
    }
    if f.is_empty() {
        "baseline".into()
    } else {
        f.join(" ")
    }
}

/// Physical memory in bytes.
fn total_ram() -> Option<usize> {
    if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
        let line = info.lines().find(|l| l.starts_with("MemTotal:"))?;
        let kib: usize = line.split_whitespace().nth(1)?.parse().ok()?;
        return Some(kib << 10);
    }
    if cfg!(windows) {
        let out = command("powershell", &["-NoProfile", "-Command", "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory"])?;
        return out.trim().parse().ok();
    }
    command("sysctl", &["-n", "hw.memsize"])?.trim().parse().ok()
}

fn machine(report: &mut Report, cfg: &Config) {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let reported = reported_l2().map_or("not read".to_string(), |b| format!("{} KiB", b >> 10));
    let lines = [
        format!("cpu,{}", cpu_name()),
        format!("arch,{}", std::env::consts::ARCH),
        format!("os,{}", std::env::consts::OS),
        format!("threads,{}", threads),
        format!("ram,{}", total_ram().map_or("unknown".to_string(), |b| format!("{:.1} GiB", b as f64 / (1u64 << 30) as f64))),
        format!("l2,{} KiB used (reported: {})", l2_bytes() >> 10, reported),
        format!("caches,{}", cache_levels()),
        format!("page,{} B", page_bytes()),
        format!("target features,{}", target_features()),
        format!("rustc,{}", command("rustc", &["--version"]).unwrap_or_else(|| "unknown".into())),
        format!("commit,{}", command("git", &["describe", "--always", "--dirty"]).unwrap_or_else(|| "unknown".into())),
        format!("settings,sizes {} to {} reps {} memory budget {} MiB parts {}", cfg.min, cfg.max, cfg.reps, cfg.memory >> 20, cfg.parts.join(" ")),
    ];
    report.line("== machine");
    for l in lines.iter() {
        report.line(l);
    }
}

fn ord_total_f32(a: &f32, b: &f32) -> bool {
    a.total_cmp(b).is_le()
}

fn ord_total_f64(a: &f64, b: &f64) -> bool {
    a.total_cmp(b).is_le()
}

macro_rules! ordered_by_key {
    ($t:ty) => {
        |a: &$t, b: &$t| a.key <= b.key
    };
}

#[test]
#[ignore]
fn bench_cpu() {
    let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();
    let cfg = Config::from_env();
    let dir: std::path::PathBuf = std::env::var("BENCH_OUT").map(Into::into).unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").into());
    let path = dir.join(format!("bench-cpu-{}-{}.txt", host_name(), utc_stamp()));
    let mut report = Report { file: std::fs::File::create(&path).unwrap() };
    // A panic (a sort failing, out of memory...) is written to the file too.
    let panic_path = path.clone();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&panic_path) {
            let _ = writeln!(f, "PANIC,{}", info.to_string().replace('\n', " "));
        }
        default_hook(info);
    }));
    let started = std::time::Instant::now();
    let core = pin::pin_fastest();
    machine(&mut report, &cfg);
    report.line(&format!("core,{}", core));
    let r = &mut report;

    // Noise: v2 against itself.
    if cfg.part("aa") {
        let u64s = all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators());
        bench(r, &cfg, "aa u64", &u64s, &[variant("v2", |a: &mut [u64]| devour_sort2(a)), variant("v2 again", |a: &mut [u64]| devour_sort2(a))], |a, b| a <= b, 0);
        bench(r, &cfg, "aa f64", &g::float_64::generators_f64(), &[variant("v2", |a: &mut [f64]| ravenous_sort2(a)), variant("v2 again", |a: &mut [f64]| ravenous_sort2(a))], ord_total_f64, 0);
    }

    // Integers.
    {
        use rdst::RadixSort as Rdst;
        macro_rules! int {
            ($t:ty, $gens:expr, [$($extra:expr),*]) => {
                if cfg.wants("int", stringify!($t)) {
                    let variants: Vec<Variant<$t>> = vec![
                        variant("v2", |a: &mut [$t]| devour_sort2(a)),
                        variant("std", |a: &mut [$t]| a.sort_unstable()),
                        variant("v1", |a: &mut [$t]| a.voracious_sort()),
                        $($extra),*
                    ];
                    bench(r, &cfg, stringify!($t), &$gens, &variants, |a, b| a <= b, 0);
                }
            };
        }
        int!(u32, all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators()), [
            variant("blazing", |a: &mut [u32]| blazing_radix::radix_sort::<u32>(a)),
            variant("radsort", |a: &mut [u32]| radsort::sort(a)),
            variant("rdst", |a: &mut [u32]| a.radix_sort_unstable()),
            variant("turbosort", |a: &mut [u32]| turbosort::sort(a))
        ]);
        int!(u64, all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators()), [
            variant("blazing", |a: &mut [u64]| blazing_radix::radix_sort::<u64>(a)),
            variant("radsort", |a: &mut [u64]| radsort::sort(a)),
            variant("rdst", |a: &mut [u64]| a.radix_sort_unstable()),
            variant("turbosort", |a: &mut [u64]| turbosort::sort(a))
        ]);
        int!(i32, all(g::signed_i32::generators_i32(), g::int_extra::extra_i32::generators()), [
            variant("blazing", |a: &mut [i32]| blazing_radix::radix_sort::<i32>(a)),
            variant("radsort", |a: &mut [i32]| radsort::sort(a)),
            variant("turbosort", |a: &mut [i32]| turbosort::sort(a))
        ]);
        int!(i64, all(g::signed_i64::generators_i64(), g::int_extra::extra_i64::generators()), [
            variant("blazing", |a: &mut [i64]| blazing_radix::radix_sort::<i64>(a)),
            variant("radsort", |a: &mut [i64]| radsort::sort(a)),
            variant("turbosort", |a: &mut [i64]| turbosort::sort(a))
        ]);
        int!(u16, all(g::unsigned_u16::generators_u16(), g::narrow_extra::extra_u16::generators()), [
            variant("blazing", |a: &mut [u16]| blazing_radix::radix_sort::<u16>(a)),
            variant("radsort", |a: &mut [u16]| radsort::sort(a)),
            variant("rdst", |a: &mut [u16]| a.radix_sort_unstable()),
            variant("turbosort", |a: &mut [u16]| turbosort::sort(a))
        ]);
        int!(i16, all(g::signed_i16::generators_i16(), g::narrow_extra::extra_i16::generators()), [
            variant("blazing", |a: &mut [i16]| blazing_radix::radix_sort::<i16>(a)),
            variant("radsort", |a: &mut [i16]| radsort::sort(a))
        ]);
        int!(u8, all(g::unsigned_u8::generators_u8(), g::narrow_extra::extra_u8::generators()), [
            variant("blazing", |a: &mut [u8]| blazing_radix::radix_sort::<u8>(a)),
            variant("radsort", |a: &mut [u8]| radsort::sort(a)),
            variant("rdst", |a: &mut [u8]| a.radix_sort_unstable()),
            variant("turbosort", |a: &mut [u8]| turbosort::sort(a))
        ]);
        int!(i8, all(g::signed_i8::generators_i8(), g::narrow_extra::extra_i8::generators()), [
            variant("blazing", |a: &mut [i8]| blazing_radix::radix_sort::<i8>(a)),
            variant("radsort", |a: &mut [i8]| radsort::sort(a))
        ]);
        int!(char, all(g::char::generators_char(), g::narrow_extra::extra_char::generators()), [
            variant("radsort", |a: &mut [char]| radsort::sort(a))
        ]);
    }

    // Floats.
    if cfg.wants("float", "f32") {
        let variants: Vec<Variant<f32>> = vec![
            variant("v2", |a: &mut [f32]| ravenous_sort2(a)),
            variant("std", |a: &mut [f32]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("v1", |a: &mut [f32]| a.voracious_sort()),
            variant("blazing", |a: &mut [f32]| blazing_radix::radix_sort::<f32>(a)),
            variant("radsort", |a: &mut [f32]| radsort::sort(a)),
            variant("turbosort", |a: &mut [f32]| turbosort::sort(a)),
        ];
        bench(r, &cfg, "f32", &all(g::float_32::generators_f32(), g::float_extra::extra_f32::generators()), &variants, ord_total_f32, 0);
    }
    if cfg.wants("float", "f64") {
        let variants: Vec<Variant<f64>> = vec![
            variant("v2", |a: &mut [f64]| ravenous_sort2(a)),
            variant("std", |a: &mut [f64]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("v1", |a: &mut [f64]| a.voracious_sort()),
            variant("radsort", |a: &mut [f64]| radsort::sort(a)),
            variant("turbosort", |a: &mut [f64]| turbosort::sort(a)),
        ];
        bench(r, &cfg, "f64", &all(g::float_64::generators_f64(), g::float_extra::extra_f64::generators()), &variants, ord_total_f64, 0);
    }

    // 128-bit integers.
    {
        use rdst::RadixSort as Rdst;
        macro_rules! wide {
            ($t:ty, $gens:expr) => {
                if cfg.wants("wide", stringify!($t)) {
                    let variants: Vec<Variant<$t>> = vec![
                        variant("v2", |a: &mut [$t]| devour_sort2(a)),
                        variant("std", |a: &mut [$t]| a.sort_unstable()),
                        variant("v1", |a: &mut [$t]| a.voracious_sort()),
                        variant("radsort", |a: &mut [$t]| radsort::sort(a)),
                        variant("rdst", |a: &mut [$t]| a.radix_sort_unstable()),
                    ];
                    bench(r, &cfg, stringify!($t), &$gens, &variants, |a, b| a <= b, 0);
                }
            };
        }
        wide!(u128, all(g::unsigned_u128::generators_u128(), g::int128_extra::extra_u128::generators()));
        wide!(i128, all(g::signed_i128::generators_i128(), g::int128_extra::extra_i128::generators()));
    }

    // Records sorted by key (stable; rdst and v1 are not stable).
    {
        use rdst::RadixSort as Rdst;
        macro_rules! records {
            ($label:expr, $ty:ty, $gens:expr) => {
                if cfg.wants("records", $label) {
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("v2", |a: &mut [$ty]| records2::devour_sort_by_key(a, |x| x.key)),
                        variant("v2 old", |a: &mut [$ty]| devour_sort_by_key(a, |x| x.key)),
                        variant("std", |a: &mut [$ty]| a.sort_by_key(|x| x.key)),
                        variant("std unstable", |a: &mut [$ty]| a.sort_unstable_by_key(|x| x.key)),
                        variant("std cached", |a: &mut [$ty]| a.sort_by_cached_key(|x| x.key)),
                        variant("glidesort", |a: &mut [$ty]| glidesort::sort_by_key(a, |x| x.key)),
                        variant("radsort", |a: &mut [$ty]| radsort::sort_by_key(a, |x| x.key)),
                        variant("rdst", |a: &mut [$ty]| a.radix_sort_unstable()),
                        variant("turbosort", |a: &mut [$ty]| turbosort::sort_by_key(a, |x| x.key)),
                        variant("v1", |a: &mut [$ty]| a.voracious_sort()),
                    ];
                    bench(r, &cfg, concat!("records ", $label), &$gens, &variants, ordered_by_key!($ty), 0);
                }
            };
        }
        records!("u32 8B", Rec<u32, 1>, wrap(keys_u32(), rec::<u32, 1>));
        records!("u64 16B", Rec<u64, 2>, wrap(keys_u64(), rec::<u64, 2>));
        records!("u64 32B", Rec<u64, 6>, wrap(keys_u64(), rec::<u64, 6>));
        records!("u32 128B", Rec<u32, 31>, wrap(keys_u32(), rec::<u32, 31>));
    }

    // CPU parameters of the records sort: from which record size the run
    // merges go 4-way (one move per two levels) instead of 2-way (cheaper
    // steps): the default 32 was measured on the Snapdragon X.
    if cfg.part("rparams") {
        use super::super::v2::records2::FOUR_WAY_MIN;
        let with = |b: usize| move || FOUR_WAY_MIN.store(b, Relaxed);
        macro_rules! rparams {
            ($label:expr, $ty:ty, $gens:expr) => {
                if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == "records")) {
                    let runs: Gens<$ty> = $gens.into_iter().filter(|(_, n)| ["Runs", "Saw", "Killer", "Almost", "Pipe"].iter().any(|k| n.contains(k))).collect();
                    let (w16, w32, w64, wnever) = (with(16), with(32), with(64), with(usize::MAX));
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("4-way from 32B", move |a: &mut [$ty]| {
                            w32();
                            records2::devour_sort_by_key(a, |x| x.key);
                        }),
                        variant("4-way from 16B", move |a: &mut [$ty]| {
                            w16();
                            records2::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                        variant("4-way from 64B", move |a: &mut [$ty]| {
                            w64();
                            records2::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                        variant("2-way only", move |a: &mut [$ty]| {
                            wnever();
                            records2::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                    ];
                    bench(r, &cfg, concat!("rparams ", $label), &runs, &variants, ordered_by_key!($ty), 100_000);
                }
            };
        }
        rparams!("u64 16B", Rec<u64, 2>, wrap(keys_u64(), rec::<u64, 2>));
        rparams!("u64 32B", Rec<u64, 6>, wrap(keys_u64(), rec::<u64, 6>));
        rparams!("u32 128B", Rec<u32, 31>, wrap(keys_u32(), rec::<u32, 31>));

        // Digit width of the LSD on 16-byte records in disorder beyond the
        // cache: 11 bits (2048 buckets, six passes for 64-bit keys) against
        // bytes (256 buckets, eight passes). 2048 write streams need more
        // L1 lines and TLB entries than some cores have (Zen 2: 32 KiB L1).
        if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == "records")) {
            use super::super::v2::records2::LSD11;
            type R = Rec<u64, 2>;
            let disorder: Gens<R> = wrap(keys_u64(), rec::<u64, 2>)
                .into_iter()
                .filter(|(_, n)| ["Unif", "Normale", "Int 0..n", "Zipf", "Composite", "Clusters", "Bimodal", "Timestamps", "Aligned", "Prefix"].iter().any(|k| n.contains(k)))
                .collect();
            let variants: Vec<Variant<R>> = vec![
                variant("LSD 11 bits", |a: &mut [R]| {
                    LSD11.store(true, Relaxed);
                    records2::devour_sort_by_key(a, |x| x.key);
                }),
                variant("LSD 8 bits", |a: &mut [R]| {
                    LSD11.store(false, Relaxed);
                    records2::devour_sort_by_key(a, |x| x.key);
                    LSD11.store(true, Relaxed);
                }),
            ];
            bench(r, &cfg, "rparams lsd digit u64 16B", &disorder, &variants, ordered_by_key!(R), 1_000_000);
        }
    }

    // One cache-derived decision at a time: its budget forced to a quarter,
    // 4x and 16x the L2 and to the L3, the others left at the L2 (the `l2`
    // part moves them all together). Tells which cache level each decision
    // should follow on this CPU. `BENCH_TUNE`: decisions to run (names of
    // `BUDGETS`), all by default.
    if cfg.part("tune") {
        let wanted: Option<Vec<String>> = std::env::var("BENCH_TUNE").ok().map(|s| s.split(',').map(|x| x.trim().to_string()).collect());
        let runs = |n: &str| ["Runs", "Saw", "Killer", "Almost", "Pipe", "Asc", "Desc", "Front", "Middle", "Organ"].iter().any(|k| n.contains(k));
        // `BUDGET_OVERRIDE` is indexed by the enum order.
        assert!(BUDGETS.iter().enumerate().all(|(i, &(b, _))| b as usize == i));
        for &(b, name) in BUDGETS.iter() {
            if wanted.as_ref().map_or(false, |w| !w.iter().any(|x| x == name)) {
                continue;
            }
            macro_rules! tune {
                ($ty:ty, $tyname:expr, $gens:expr, $sort:expr, $ordered:expr) => {
                    if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == $tyname)) {
                        let variants: Vec<Variant<$ty>> = tune_variants(b, $sort);
                        bench(r, &cfg, &format!("tune {} {}", name, $tyname), &$gens, &variants, $ordered, 10_000);
                    }
                };
            }
            let u32s = || all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators());
            let u64s = || all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators());
            let only_runs = |v: Gens<u64>| -> Gens<u64> { v.into_iter().filter(|(_, n)| runs(n)).collect() };
            match b {
                Budget::Lsd11 | Budget::Roller32 => {
                    tune!(u32, "u32", u32s(), |a: &mut [u32]| devour_sort2(a), |a: &u32, b: &u32| a <= b);
                }
                Budget::Roller64 | Budget::AliasInt => {
                    tune!(u64, "u64", u64s(), |a: &mut [u64]| devour_sort2(a), |a: &u64, b: &u64| a <= b);
                }
                Budget::CountLanes | Budget::WideCount => {
                    tune!(u32, "u32", u32s(), |a: &mut [u32]| devour_sort2(a), |a: &u32, b: &u32| a <= b);
                    tune!(u64, "u64", u64s(), |a: &mut [u64]| devour_sort2(a), |a: &u64, b: &u64| a <= b);
                }
                Budget::Roller128 => {
                    let gens = all(g::unsigned_u128::generators_u128(), g::int128_extra::extra_u128::generators());
                    tune!(u128, "u128", gens, |a: &mut [u128]| devour_sort2(a), |a: &u128, b: &u128| a <= b);
                }
                Budget::RollerF32 => {
                    tune!(f32, "f32", g::float_32::generators_f32(), |a: &mut [f32]| ravenous_sort2(a), ord_total_f32);
                }
                Budget::RollerF64 | Budget::AliasFloat => {
                    tune!(f64, "f64", g::float_64::generators_f64(), |a: &mut [f64]| ravenous_sort2(a), ord_total_f64);
                }
                Budget::MergeTwoWay | Budget::PingPong => {
                    tune!(u64, "u64", only_runs(u64s()), |a: &mut [u64]| devour_sort2(a), |a: &u64, b: &u64| a <= b);
                }
                Budget::Records => {
                    if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == "records")) {
                        type R8 = Rec<u32, 1>;
                        type R16 = Rec<u64, 2>;
                        type R32 = Rec<u64, 6>;
                        let v8: Vec<Variant<R8>> = tune_variants(b, |a: &mut [R8]| records2::devour_sort_by_key(a, |x| x.key));
                        bench(r, &cfg, "tune records 8B", &wrap(keys_u32(), rec::<u32, 1>), &v8, ordered_by_key!(R8), 10_000);
                        let v16: Vec<Variant<R16>> = tune_variants(b, |a: &mut [R16]| records2::devour_sort_by_key(a, |x| x.key));
                        bench(r, &cfg, "tune records 16B", &wrap(keys_u64(), rec::<u64, 2>), &v16, ordered_by_key!(R16), 10_000);
                        let v32: Vec<Variant<R32>> = tune_variants(b, |a: &mut [R32]| records2::devour_sort_by_key(a, |x| x.key));
                        bench(r, &cfg, "tune records 32B", &wrap(keys_u64(), rec::<u64, 6>), &v32, ordered_by_key!(R32), 10_000);
                    }
                }
            }
        }
    }

    // Thresholds derived from the L2: detected size against half and twice.
    if cfg.part("l2") {
        let l2 = l2_bytes();
        let with_l2 = |bytes: usize| move || L2_OVERRIDE.store(bytes, Relaxed);
        macro_rules! l2 {
            ($t:ty, $gens:expr, $sort:expr, $ordered:expr) => {
                if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == stringify!($t))) {
                    let (half, double, quad, arm) = (with_l2(l2 / 2), with_l2(l2 * 2), with_l2(l2 * 4), with_l2(ARM_L2));
                    let variants: Vec<Variant<$t>> = vec![
                        variant("detected L2", |a: &mut [$t]| $sort(a)),
                        variant("L2 / 2", move |a: &mut [$t]| {
                            half();
                            $sort(a);
                            L2_OVERRIDE.store(0, Relaxed);
                        }),
                        variant("L2 x 2", move |a: &mut [$t]| {
                            double();
                            $sort(a);
                            L2_OVERRIDE.store(0, Relaxed);
                        }),
                        variant("L2 x 4", move |a: &mut [$t]| {
                            quad();
                            $sort(a);
                            L2_OVERRIDE.store(0, Relaxed);
                        }),
                        variant("L2 12 MiB", move |a: &mut [$t]| {
                            arm();
                            $sort(a);
                            L2_OVERRIDE.store(0, Relaxed);
                        }),
                    ];
                    bench(r, &cfg, concat!("l2 ", stringify!($t)), &$gens, &variants, $ordered, 100_000);
                }
            };
        }
        l2!(u32, all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators()), devour_sort2, |a: &u32, b: &u32| a <= b);
        l2!(u64, all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators()), devour_sort2, |a: &u64, b: &u64| a <= b);
        l2!(f64, g::float_64::generators_f64(), ravenous_sort2, ord_total_f64);
        if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == "records")) {
            let (half, double, quad, arm) = (with_l2(l2 / 2), with_l2(l2 * 2), with_l2(l2 * 4), with_l2(ARM_L2));
            type R = Rec<u64, 6>;
            let variants: Vec<Variant<R>> = vec![
                variant("detected L2", |a: &mut [R]| devour_sort_by_key(a, |x| x.key)),
                variant("L2 / 2", move |a: &mut [R]| {
                    half();
                    devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 x 2", move |a: &mut [R]| {
                    double();
                    devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 x 4", move |a: &mut [R]| {
                    quad();
                    devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 12 MiB", move |a: &mut [R]| {
                    arm();
                    devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
            ];
            bench(r, &cfg, "l2 records u64 32B", &wrap(keys_u64(), rec::<u64, 6>), &variants, ordered_by_key!(R), 100_000);
        }
    }

    report.line("");
    #[cfg(windows)]
    report.line(&format!("core,ended on processor {}", pin::current()));
    report.line(&format!("== done in {:.0} min", started.elapsed().as_secs_f64() / 60.0));
    println!("\nResults written to {}", path.display());
}

/// Pins the bench thread to one core of the fastest kind on hybrid CPUs
/// (Intel P / E cores: Windows reports an efficiency class per logical
/// processor, higher is faster). Returns the `core` header line. `BENCH_PIN=0`
/// turns it off; CPUs whose cores are all alike are not pinned.
#[cfg(windows)]
mod pin {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemCpuSetInformation(info: *mut u8, len: u32, returned: *mut u32, process: isize, flags: u32) -> i32;
        fn SetThreadSelectedCpuSets(thread: isize, ids: *const u32, count: u32) -> i32;
        fn GetCurrentThread() -> isize;
        fn GetCurrentProcessorNumber() -> u32;
    }

    /// (cpu set id, logical processor, core, efficiency class) of every
    /// logical processor of group 0.
    fn cpu_sets() -> Vec<(u32, u8, u8, u8)> {
        let mut len: u32 = 0;
        // SAFETY: a null buffer with length 0 only asks for the needed size.
        unsafe { GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut len, 0, 0) };
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        // SAFETY: the buffer holds at least `len` bytes.
        if len == 0 || unsafe { GetSystemCpuSetInformation(buf.as_mut_ptr() as *mut u8, len, &mut len, 0, 0) } == 0 {
            return Vec::new();
        }
        // SAFETY: `len <= 8 * buf.len()` bytes were written by the call.
        let b = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, len as usize) };
        let u32_at = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        // SYSTEM_CPU_SET_INFORMATION: Size u32, Type u32, then Id u32,
        // Group u16, LogicalProcessorIndex u8, CoreIndex u8,
        // LastLevelCacheIndex u8, NumaNodeIndex u8, EfficiencyClass u8.
        let mut sets = Vec::new();
        let mut off = 0;
        while off + 19 <= b.len() {
            let size = u32_at(off) as usize;
            if size == 0 {
                break;
            }
            let group = u16::from_le_bytes([b[off + 12], b[off + 13]]);
            if u32_at(off + 4) == 0 && group == 0 {
                sets.push((u32_at(off + 8), b[off + 14], b[off + 15], b[off + 18]));
            }
            off += size;
        }
        sets
    }

    pub(super) fn pin_fastest() -> String {
        let sets = cpu_sets();
        let (lo, hi) = match (sets.iter().map(|s| s.3).min(), sets.iter().map(|s| s.3).max()) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => return "not pinned (cpu sets not read)".into(),
        };
        let fast = sets.iter().filter(|s| s.3 == hi).count();
        let kinds = format!("{} logical processors, {} of the fastest class {} (classes {}-{})", sets.len(), fast, hi, lo, hi);
        if std::env::var("BENCH_PIN").map_or(false, |v| v == "0") {
            return format!("not pinned (BENCH_PIN=0), current processor {}; {}", current(), kinds);
        }
        if lo == hi {
            return format!("not pinned (all cores alike), current processor {}; {}", current(), kinds);
        }
        // `BENCH_PIN=slow`: the slowest class instead (to measure the gap).
        let class = if std::env::var("BENCH_PIN").map_or(false, |v| v == "slow") { lo } else { hi };
        // A core of that class other than core 0 (interrupts) when there is one.
        let chosen: Vec<_> = sets.iter().filter(|s| s.3 == class).collect();
        let pick = chosen.iter().find(|s| s.2 != 0).unwrap_or(&chosen[0]);
        // SAFETY: GetCurrentThread returns a pseudo handle; one valid id.
        if unsafe { SetThreadSelectedCpuSets(GetCurrentThread(), &pick.0, 1) } == 0 {
            return format!("not pinned (SetThreadSelectedCpuSets failed); {}", kinds);
        }
        std::thread::yield_now();
        format!("pinned to logical processor {} (core {}, class {}), now on processor {}; {}", pick.1, pick.2, class, current(), kinds)
    }

    pub(super) fn current() -> u32 {
        // SAFETY: no arguments, cannot fail.
        unsafe { GetCurrentProcessorNumber() }
    }
}

#[cfg(not(windows))]
mod pin {
    pub(super) fn pin_fastest() -> String {
        "not pinned (only on Windows)".into()
    }
}
