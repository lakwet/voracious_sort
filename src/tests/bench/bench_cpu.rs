//! The benchmarks of voracious 2.0 against 1.x, the standard sorts and the
//! best rivals of each type: `make bench`, `make bench-type T=u64`,
//! `make bench-records`, `make bench-big`, `make bench-big-type T=u64`
//! (see the Makefile and the README).
//!
//! Two files per run in `resultsbenchs/<machine>/` (`BENCH_OUT` to change
//! it), written as it goes (a run stopped early still leaves its results):
//! - `<bench>-<host>-<date>.txt`, the report for humans (also printed): how
//!   it reads and how it is measured, the machine, then per type and size a
//!   table of the duration of one sort, the ns per key and the ratio to v2
//!   of every sort on every distribution;
//! - `<bench>-<host>-<date>.csv`, the raw lines for `tools/bench_report.py`:
//!   the machine, `cols` / `row` lines (ns per key), `iter` lines (arrays
//!   per measurement, measurements), `sum` lines (ratios other / v2);
//! - every result is checked to be sorted in the first repetition
//!   (`CHECK FAILED` lines otherwise; voracious 1.x mixes -0.0 and +0.0, a
//!   known v1 bug, on `Zeros +-`).
//!
//! Parts (`BENCH_PARTS`, all but `tune` and `big` by default):
//! - `aa`: v2 against itself, the noise of the machine;
//! - `int`, `float`, `wide`: integers, floats, 128-bit integers against
//!   std, voracious 1.x and the radix crates (blazing, radsort, rdst,
//!   turbosort) where they take the type;
//! - `records`: sort by key (stable) of records of 8, 16, 32 and 128 bytes
//!   against std (stable and unstable), glidesort, radsort, rdst, turbosort
//!   and voracious 1.x;
//! - `rparams`: the CPU parameters of the records sort: the record size
//!   from which run merges go 4-way instead of 2-way (`FOUR_WAY_MIN`), on
//!   the shapes with runs, records of 16, 32 and 128 bytes; and the digit
//!   width of the LSD on 16-byte records (11 bits against 8);
//! - `l2`: v2 with the L2 size it derives its thresholds from forced to
//!   half, twice and four times the detected one, and to 12 MiB (the
//!   Snapdragon X the thresholds were tuned on): if the detected one is
//!   the fastest, the tuning holds on this machine (four times: whether a
//!   small L2 backed by a large L3 should count for more);
//! - `big` (dev part, `BENCH_PARTS=big`): 30M to 1G keys (30M, 100M, 300M,
//!   500M, 800M, 1G), one repetition by default, one array per
//!   measurement, u32 / u64 / f32 / f64 on ten shapes in disorder: the
//!   default against the LSD (or DLSD) forced and the rollercoaster forced;
//! - `tune` (dev part, `BENCH_PARTS=tune`): one cache-derived decision at
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
//! thread to a fast core on hybrid CPUs; `slow`: pin it to a slow one),
//! `BENCH_SIZES` (these sizes instead of the list, any value, e.g. `500,10000`; `BENCH_BIG_SIZES` for `big`), `BENCH_SORTS`
//! (only these sorts besides v2, e.g. `std,glidesort,blazing`),
//! `BENCH_MIN_ARRAYS` (arrays sorted per measurement at least, default 5),
//! `BENCH_MIN_SORTS` (arrays sorted at least at given sizes, over the
//! repetitions, e.g. `2000000:8,10000000:5`), `BENCH_AB=1` (integers: a
//! "v2 new" column, the v2 with the rule under test on, `v2::dev::ab`).
//! A panic is written to the file before the run stops.

use std::io::Write;
use std::sync::atomic::Ordering::Relaxed;

use crate::generators as g;
use crate::generators::records::*;
use crate::v1::RadixSort;
use crate::v2::cache::{
    l2_bytes, page_bytes, reported_l2, reported_levels, Budget, BUDGETS, BUDGET_OVERRIDE,
    L2_OVERRIDE,
};
use crate::v2::sorts::by_key;
use crate::v2::sorts::devour::devour_sort;
use crate::v2::sorts::ravenous::ravenous_sort;

type Gens<T> = Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>;

const SIZES: [usize; 13] = [
    1_000,
    10_000,
    30_000,
    50_000,
    100_000,
    200_000,
    600_000,
    1_000_000,
    3_000_000,
    5_000_000,
    10_000_000,
    30_000_000,
    100_000_000,
];
/// Beyond this size only the types of `BIG_LABELS` are measured.
const BIG: usize = 10_000_000;
const BIG_LABELS: [&str; 13] = [
    "aa u64",
    "u32",
    "u64",
    "i64",
    "f32",
    "f64",
    "u128",
    "records u32 8B",
    "records u64 16B",
    "records u64 32B",
    "l2 u32",
    "l2 u64",
    "l2 f64",
];
/// L2 of the Snapdragon X the thresholds were tuned on (`l2` part: the same
/// engine choices as there).
const ARM_L2: usize = 12 << 20;
/// Sizes of the `big` part.
const BIG_SIZES: [usize; 6] =
    [30_000_000, 100_000_000, 300_000_000, 500_000_000, 800_000_000, 1_000_000_000];
/// Keys per batch of arrays (small sizes: many arrays per measurement).
const BATCH_KEYS: usize = 4_000_000;
/// Most bytes in a batch (large records).
const BATCH_BYTES: usize = 256 << 20;
/// At least this many arrays sorted per measurement, over the repetitions.
const MIN_ARRAYS: usize = 5;

#[derive(Clone)]
struct Config {
    min: usize,
    max: usize,
    reps: usize,
    parts: Vec<String>,
    types: Option<Vec<String>>,
    memory: usize,
    /// At least this many arrays sorted per measurement (`big` part: 1).
    min_arrays: usize,
    /// At least this many arrays sorted at a given size, over the
    /// repetitions (`BENCH_MIN_SORTS=2000000:8,5000000:5`).
    min_sorts: Vec<(usize, usize)>,
}

impl Config {
    fn from_env() -> Config {
        let list = |name: &str| {
            std::env::var(name)
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>())
        };
        let number = |name: &str, default: usize| {
            std::env::var(name)
                .ok()
                .and_then(|s| s.replace('_', "").parse().ok())
                .unwrap_or(default)
        };
        Config {
            min: number("BENCH_MIN", 0),
            max: number("BENCH_MAX", 100_000_000),
            reps: number("BENCH_REPS", 3).max(1),
            parts: list("BENCH_PARTS").unwrap_or_else(|| {
                ["aa", "int", "float", "wide", "records", "rparams", "l2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            }),
            types: list("BENCH_TYPES"),
            memory: number("BENCH_MEM_MB", (total_ram().unwrap_or(16 << 30) / 2) >> 20) << 20,
            min_arrays: number("BENCH_MIN_ARRAYS", MIN_ARRAYS).max(1),
            min_sorts: list("BENCH_MIN_SORTS")
                .unwrap_or_default()
                .iter()
                .filter_map(|p| p.split_once(':'))
                .filter_map(|(n, k)| {
                    Some((n.trim().replace('_', "").parse().ok()?, k.trim().parse().ok()?))
                })
                .collect(),
        }
    }

    fn part(&self, part: &str) -> bool { self.parts.iter().any(|p| p == part) }

    /// `ty`: the type label; a records label ("u64 16B") is also wanted by
    /// its key type alone ("u64": every record of u64 keys).
    fn wants(&self, part: &str, ty: &str) -> bool {
        let key = |x: &String| part == "records" && ty.split(' ').next() == Some(x.as_str());
        self.part(part) && self.types.as_ref().is_none_or(|t| t.iter().any(|x| x == ty || key(x)))
    }
}

/// The output file, every line also printed.
/// The two output files: the raw lines (CSV, read by `tools/*.py`) and the
/// report for humans (also printed).
struct Report {
    file: std::fs::File,
    human: std::fs::File,
}

impl Report {
    /// A raw line.
    fn line(&mut self, text: &str) {
        writeln!(self.file, "{}", text).unwrap();
        self.file.flush().unwrap();
    }

    /// A line of the report for humans.
    fn text(&mut self, text: &str) {
        println!("{}", text);
        writeln!(self.human, "{}", text).unwrap();
        self.human.flush().unwrap();
    }
}

/// What the report shows and how it is measured, at its top.
const EXPLAINED: &str = "How to read this report
-----------------------
For each key type and array size, one line per distribution of the keys
and one column per sort. The title of each table gives the array size, how
many arrays each sort sorts per measurement (several for small arrays, so
that a measurement lasts long enough) and how many measurements are made
(the median is kept). Each cell gives three numbers:
  - duration: the time to sort one array of that size (ns, us, ms or s):
              in each measurement, the time of all its arrays divided by
              their number; then the median of the measurements;
  - ns/key:   that duration divided by the number of keys;
  - ratio:    the duration of that sort over the duration of voracious v2
              (> 1: v2 is faster; < 1: the other sort is faster).
The last lines of each table summarise each column: its geometric mean and
median over the distributions, and the number of distributions where v2 is
faster.

The sorts: voracious v2 (this version), voracious v1 (1.2.2), the sort of
the Rust standard library (stable or unstable, as the case) and the best
rivals of each type: the two fastest other crates on that type, and a
third one where it beats v2 on cases the two others do not (chosen on a
previous full bench over three machines).

How it is measured, to be close to real use:
  - every sort sorts the same arrays, made new for each measurement;
  - each sort is called once per array, as a program would (no buffer kept
    from one call to the next: each sort pays its own memory, page faults
    included; freed memory is given back to the system between sorts);
  - the order of the sorts changes at each measurement;
  - the crate is compiled as a user compiles it (release, no CPU-specific
    flags; `NATIVE=1` compiles every sort for this CPU, see the target
    features in the machine section) and pinned to one fast core, single
    thread;
  - every result is checked to be sorted.
";

/// A sort to measure.
struct Variant<'a, T> {
    name: &'a str,
    sort: Box<dyn Fn(&mut [T]) + 'a>,
}

/// The two fastest rivals of each type (geometric mean over every
/// distribution and size, benchbilan of 2026-11-17 on the Ryzen 3950X, Core
/// Ultra 185H and Core i5-4690K), and a third
/// one where it beats the v2 on cases the two do not
/// (in brackets its strongest case, time over
/// v2's). char: the only two rivals that sort it. Records without measures:
/// as the measured record of their size class.
fn best_rivals(label: &str) -> &'static [&'static str] {
    match label {
        // glidesort: Runs 8 10M 0.63, Sparse 90 % 1k 0.63-0.66.
        "u32" | "i32" => &["blazing", "turbosort", "glidesort"],
        // glidesort: Sparse 90 % / Zeros +- 1k 0.73-0.76.
        "f32" => &["blazing", "turbosort", "glidesort"],
        "u16" | "u8" => &["blazing", "turbosort"],
        // rdst: Clusters 10M 0.84.
        "u64" => &["blazing", "glidesort", "rdst"],
        // turbosort: Asc / Desc 0.89-0.91.
        "i64" => &["blazing", "glidesort", "turbosort"],
        // glidesort: Asc Saw 100k 0.79.
        "i16" => &["blazing", "radsort", "glidesort"],
        "i8" => &["blazing", "radsort"],
        // radsort: Timestamps 50k 0.71.
        "f64" => &["turbosort", "glidesort", "radsort"],
        "u128" | "i128" => &["radsort", "rdst"],
        "char" => &["radsort", "glidesort"],
        // The only rivals taking bool (not measured yet).
        "bool" => &["glidesort", "radsort"],
        l if l.starts_with("records bool") => &["glidesort", "radsort"],
        // glidesort: Asc / Desc Saw 5M 0.35-0.38 (139 cases).
        l if l.starts_with("records") && l.ends_with(" 8B") => &["radsort", "rdst", "glidesort"],
        // rdst: Unif / Composite 1M-10M 0.59-0.61 (54 cases).
        l if l.starts_with("records") => &["glidesort", "radsort", "rdst"],
        _ => &[],
    }
}

/// `BENCH_SORTS`: the sorts kept (the first one, v2, always: the reference);
/// `best`: std, v1, v2 (stable and unstable for the records) and the best
/// rivals of the type (`best_rivals`).
fn keep_sorts<'a, T>(label: &str, v: Vec<Variant<'a, T>>) -> Vec<Variant<'a, T>> {
    // "v2 new" (the rule under test, `v2::dev::ab`) only with BENCH_AB=1.
    let ab = std::env::var("BENCH_AB").is_ok_and(|v| v == "1");
    let v: Vec<_> = v.into_iter().filter(|x| ab || !x.name.starts_with("v2 new")).collect();
    let list = match std::env::var("BENCH_SORTS") {
        Ok(list) if list.trim() == "best" => {
            format!(
                "std,std unstable,v1,v1 stable,v2 unstable,v2 new,{}",
                best_rivals(label).join(",")
            )
        },
        Ok(list) => list,
        Err(_) => return v,
    };
    v.into_iter()
        .enumerate()
        .filter(|(i, x)| *i == 0 || list.split(',').any(|n| n.trim() == x.name))
        .map(|(_, x)| x)
        .collect()
}

/// Runs `f` with the rule under test on (`v2::dev::ab`).
fn ab_new(f: impl FnOnce()) {
    crate::v2::dev::ab::NEW_RULE.store(true, Relaxed);
    f();
    crate::v2::dev::ab::NEW_RULE.store(false, Relaxed);
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

/// Gives the freed heap memory back to the system before each measurement,
/// so that every sort pays the page faults of its buffers as in an isolated
/// call. Only glibc keeps freed memory (already touched pages, then no
/// fault); Windows pays them on every allocation anyway.
fn trim_heap() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        extern "C" {
            fn malloc_trim(pad: usize) -> i32;
        }
        // SAFETY: glibc's malloc_trim, no precondition.
        unsafe { malloc_trim(0) };
    }
}

/// Median ns per key of each variant on each distribution at `size`. The
/// variants sort copies of the same batch, in an order rotating between
/// repetitions; `ordered(a, b)`: `a` may come before `b`.
/// Arrays sorted per measurement and repetitions (the median of the
/// repetitions is kept) at `size`.
fn iterations<T>(cfg: &Config, size: usize) -> (usize, usize) {
    let bytes = std::mem::size_of::<T>().max(1);
    let count = (BATCH_KEYS / size).min(BATCH_BYTES / (size * bytes)).max(1);
    let at_size =
        cfg.min_sorts.iter().filter(|&&(n, _)| n == size).map(|&(_, k)| k).max().unwrap_or(0);
    let reps = cfg.reps.max(cfg.min_arrays.div_ceil(count)).max(at_size.div_ceil(count));
    (count, reps)
}

/// One distribution: its name and, per sort, the median time in ns per key.
type Row = (String, Vec<f64>);

fn measure<T: Clone>(
    report: &mut Report,
    cfg: &Config,
    label: &str,
    gens: &Gens<T>,
    size: usize,
    variants: &[Variant<T>],
    ordered: fn(&T, &T) -> bool,
) -> Vec<Row> {
    let (count, reps) = iterations::<T>(cfg, size);
    let nv = variants.len();
    let mut rows = Vec::new();
    for (gen, name) in gens.iter() {
        // Per sort, the time of one array in each measurement: the time of
        // the whole batch divided by its number of arrays.
        let mut times: Vec<Vec<f64>> = vec![Vec::new(); nv];
        for rep in 0..reps {
            let batch: Vec<Vec<T>> = (0..count).map(|_| gen(size)).collect();
            for k in 0..nv {
                let v = (k + rep) % nv;
                trim_heap();
                let mut work = batch.clone();
                let start = std::time::Instant::now();
                // A sort that panics loses its measurement, not the run.
                let sorted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for a in work.iter_mut() {
                        (variants[v].sort)(a);
                    }
                }));
                if sorted.is_err() {
                    report.line(&format!(
                        "SORT PANICKED,{},{},{},{}",
                        label,
                        size,
                        short_name(name),
                        variants[v].name
                    ));
                    times[v].push(f64::NAN);
                    continue;
                }
                times[v].push(start.elapsed().as_nanos() as f64 / count as f64);
                if rep == 0 && !work.iter().all(|a| a.windows(2).all(|p| ordered(&p[0], &p[1]))) {
                    report.line(&format!(
                        "CHECK FAILED,{},{},{},{}",
                        label,
                        size,
                        short_name(name),
                        variants[v].name
                    ));
                }
            }
        }
        // The median of the measurements (the mean of the two middle ones
        // for an even count), given per key for the raw lines and the tools.
        let per_key = times
            .iter_mut()
            .map(|t| {
                t.sort_by(|a, b| a.total_cmp(b));
                let m = t.len() / 2;
                let median = if t.len().is_multiple_of(2) { (t[m - 1] + t[m]) / 2.0 } else { t[m] };
                median / size as f64
            })
            .collect();
        rows.push((short_name(name), per_key));
    }
    rows
}

/// One type of one part: raw rows and a summary per size.
fn bench<T: Clone>(
    report: &mut Report,
    cfg: &Config,
    label: &str,
    gens: &Gens<T>,
    variants: &[Variant<T>],
    ordered: fn(&T, &T) -> bool,
    min_size: usize,
) {
    let only: Option<Vec<usize>> = std::env::var("BENCH_SIZES")
        .ok()
        .map(|l| l.split(',').map(|x| x.trim().replace('_', "").parse().unwrap()).collect());
    // `BENCH_SIZES` gives the sizes themselves (any value), else `SIZES`.
    let mut listed = only.unwrap_or_else(|| SIZES.to_vec());
    listed.sort_unstable();
    listed.dedup();
    // Sizes given by `BENCH_SIZES` apply to every type; the default list
    // goes beyond `BIG` for the main types only.
    let given = std::env::var("BENCH_SIZES").is_ok();
    let sizes: Vec<usize> = listed
        .into_iter()
        .filter(|&s| s <= cfg.max && s >= min_size.max(cfg.min))
        .filter(|&s| given || s <= BIG || BIG_LABELS.contains(&label))
        .collect();
    bench_sizes(report, cfg, label, gens, variants, ordered, &sizes);
}

/// `bench` at the given sizes.
fn bench_sizes<T: Clone>(
    report: &mut Report,
    cfg: &Config,
    label: &str,
    gens: &Gens<T>,
    variants: &[Variant<T>],
    ordered: fn(&T, &T) -> bool,
    sizes: &[usize],
) {
    let names: Vec<&str> = variants.iter().map(|v| v.name).collect();
    report.line("");
    report.line(&format!(
        "== {} ({} distributions; ns per key; ratios other / {}, > 1: {} faster)",
        label,
        gens.len(),
        names[0],
        names[0]
    ));
    report.line(&format!("cols,{},size,gen,{}", label, names.join(",")));
    for &size in sizes {
        // The batch, its working copy and a buffer of the same size.
        let live = if cfg.min_arrays == 1 { 3 } else { 4 };
        if size > BIG
            && size.saturating_mul(std::mem::size_of::<T>()).saturating_mul(live) > cfg.memory
        {
            report.line(&format!("skip,{},{},memory budget (BENCH_MEM_MB)", label, size));
            report.text(&format!(
                "{}, {} keys: skipped (more memory than the budget, BENCH_MEM_MB)
",
                label,
                thousands(size)
            ));
            continue;
        }
        let started = std::time::Instant::now();
        let rows = measure(report, cfg, label, gens, size, variants, ordered);
        for (gen, ns) in rows.iter() {
            let ns: Vec<String> = ns.iter().map(|x| format!("{:.3}", x)).collect();
            report.line(&format!("row,{},{},{},{}", label, size, gen, ns.join(",")));
        }
        let plan = iterations::<T>(cfg, size);
        report.line(&format!("iter,{},{},{},{}", label, size, plan.0, plan.1));
        human_table(report, label, size, &names, &rows, plan);
        let first: Vec<f64> = rows.iter().map(|r| r.1[0]).collect();
        report.line(&format!(
            "sum,{},{},{} geo {:.2} ns ({:.0} s)",
            label,
            size,
            names[0],
            geo(&first),
            started.elapsed().as_secs_f64()
        ));
        for (v, name) in names.iter().enumerate().skip(1) {
            let mut ratios: Vec<f64> = rows.iter().map(|r| r.1[v] / r.1[0]).collect();
            let wins = ratios.iter().filter(|&&x| x > 1.0).count();
            ratios.sort_by(|a, b| a.total_cmp(b));
            report.line(&format!(
                "sum,{},{},{} geo {:.2} med {:.2} p10 {:.2} wins {}/{}",
                label,
                size,
                name,
                geo(&ratios),
                ratios[ratios.len() / 2],
                ratios[ratios.len() / 10],
                wins,
                ratios.len()
            ));
        }
    }
}

/// `n` with a space every three digits (10 000 000).
fn thousands(n: usize) -> String {
    let d = n.to_string();
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
        if i > 0 && (d.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// A duration in nanoseconds, three significant digits, in ns, us, ms or s.
fn duration(ns: f64) -> String {
    let (v, unit) = match ns {
        x if x < 1e3 => (x, "ns"),
        x if x < 1e6 => (x / 1e3, "us"),
        x if x < 1e9 => (x / 1e6, "ms"),
        x => (x / 1e9, "s"),
    };
    let digits = if v < 10.0 {
        2
    } else if v < 100.0 {
        1
    } else {
        0
    };
    format!("{:.*} {}", digits, v, unit)
}

/// The table of one type and size for the report: per distribution and
/// sort, the duration of one sort, the ns per key and the ratio to v2.
fn human_table(
    report: &mut Report,
    label: &str,
    size: usize,
    names: &[&str],
    rows: &[Row],
    (count, reps): (usize, usize),
) {
    const CELL: usize = 25;
    let gen_width = rows.iter().map(|r| r.0.len()).max().unwrap_or(0).max(12);
    let arrays = if count == 1 { "1 array".to_string() } else { format!("{} arrays", count) };
    report.text(&format!(
        "{}, arrays of {} keys: each sort timed on {} per measurement, {}",
        label,
        thousands(size),
        arrays,
        if reps == 1 {
            "1 measurement".to_string()
        } else {
            format!("{} measurements, median kept", reps)
        }
    ));
    let mut head = format!("{:<w$}", "distribution", w = gen_width);
    let mut sub = format!("{:<w$}", "", w = gen_width);
    for name in names {
        head += &format!(" | {:<w$}", name, w = CELL);
        sub += &format!(" | {:>9} {:>7} {:>6}", "duration", "ns/key", "ratio");
    }
    report.text(&head);
    report.text(&sub);
    for (gen, ns) in rows {
        let mut line = format!("{:<w$}", gen, w = gen_width);
        for &x in ns {
            line += &format!(" | {:>9} {:>7.2} {:>6.2}", duration(x * size as f64), x, x / ns[0]);
        }
        report.text(&line);
    }
    let mut geo_line = format!("{:<w$}", "geo. mean", w = gen_width);
    let mut med_line = format!("{:<w$}", "median", w = gen_width);
    let mut win_line = format!("{:<w$}", "v2 faster", w = gen_width);
    for v in 0..names.len() {
        let mut keys: Vec<f64> = rows.iter().map(|r| r.1[v]).collect();
        let mut ratios: Vec<f64> = rows.iter().map(|r| r.1[v] / r.1[0]).collect();
        let g = geo(&keys);
        geo_line +=
            &format!(" | {:>9} {:>7.2} {:>6.2}", duration(g * size as f64), g, geo(&ratios));
        keys.sort_by(|a, b| a.total_cmp(b));
        ratios.sort_by(|a, b| a.total_cmp(b));
        let m = keys[keys.len() / 2];
        med_line += &format!(
            " | {:>9} {:>7.2} {:>6.2}",
            duration(m * size as f64),
            m,
            ratios[ratios.len() / 2]
        );
        let wins = rows.iter().filter(|r| r.1[v] > r.1[0]).count();
        let cell = if v == 0 { String::new() } else { format!("{} / {}", wins, rows.len()) };
        win_line += &format!(" | {:>w$}", cell, w = CELL);
    }
    report.text(&geo_line);
    report.text(&med_line);
    report.text(&win_line);
    report.text("");
}

/// The variants of one decision for the `tune` part: its default budget,
/// then forced to L2 / 4, L2 x 4, L2 x 16 and the L3 (when one is reported
/// above the L2), the other decisions untouched.
fn tune_variants<'a, T: 'a>(b: Budget, sort: impl Fn(&mut [T]) + Copy + 'a) -> Vec<Variant<'a, T>> {
    let l2 = l2_bytes();
    let mut sizes: Vec<(&'static str, usize)> =
        vec![("L2 / 4", l2 / 4), ("L2 x 4", l2 * 4), ("L2 x 16", l2 * 16)];
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

/// The operating system and its version (Windows: `ver`; Linux: the
/// distribution and the kernel; macOS: `sw_vers`).
fn os_version() -> String {
    let os = std::env::consts::OS;
    let version = match os {
        "windows" => command("cmd", &["/C", "ver"]),
        "macos" => command("sw_vers", &["-productVersion"]).map(|v| format!("macOS {}", v)),
        _ => {
            let name = std::fs::read_to_string("/etc/os-release").ok().and_then(|t| {
                t.lines()
                    .find_map(|l| l.strip_prefix("PRETTY_NAME="))
                    .map(|v| v.trim_matches('"').to_string())
            });
            let kernel = command("uname", &["-r"]);
            match (name, kernel) {
                (Some(n), Some(k)) => Some(format!("{}, kernel {}", n, k)),
                (n, k) => n.or(k),
            }
        },
    };
    version.map_or_else(|| os.to_string(), |v| format!("{} ({})", os, v))
}

fn command(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program).args(args).output().ok()?;
    // Lossy: some tools answer in the local code page (accents).
    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn cpu_name() -> String {
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        if let Some(line) =
            info.lines().find(|l| l.starts_with("model name") || l.starts_with("Model"))
        {
            return line.split_once(':').map(|x| x.1).unwrap_or("").trim().to_string();
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

/// Details of the CPU beyond its name: vendor, family / model / stepping,
/// physical cores, maximum clock; "unknown" parts left out.
fn cpu_details() -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        let field = |name: &str| {
            info.lines()
                .find(|l| l.split(':').next().is_some_and(|k| k.trim() == name))
                .and_then(|l| l.split_once(':'))
                .map(|x| x.1.trim().to_string())
        };
        if let Some(v) = field("vendor_id").or_else(|| field("CPU implementer")) {
            parts.push(v);
        }
        if let (Some(f), Some(m)) = (field("cpu family"), field("model")) {
            parts.push(format!("family {} model {}", f, m));
        }
        if let Some(st) = field("stepping") {
            parts.push(format!("stepping {}", st));
        }
        // Physical cores: distinct (package, core) pairs.
        let mut cores = std::collections::BTreeSet::new();
        let (mut package, mut core) = (String::new(), String::new());
        for l in info.lines() {
            if let Some((k, v)) = l.split_once(':') {
                match k.trim() {
                    "physical id" => package = v.trim().to_string(),
                    "core id" => {
                        core = v.trim().to_string();
                        cores.insert((package.clone(), core.clone()));
                    },
                    _ => {},
                }
            }
        }
        let _ = core;
        if !cores.is_empty() {
            parts.push(format!("{} physical cores", cores.len()));
        }
        let max = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq");
        if let Some(khz) = max.ok().and_then(|t| t.trim().parse::<u64>().ok()) {
            parts.push(format!("max {:.2} GHz", khz as f64 / 1e6));
        }
    } else if cfg!(windows) {
        let key = r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0";
        let value = |name: &str, kind: &str| {
            command("reg", &["query", key, "/v", name]).and_then(|out| {
                out.lines()
                    .find(|l| l.contains(name))
                    .and_then(|l| l.split(kind).nth(1))
                    .map(|v| v.trim().to_string())
            })
        };
        if let Some(v) = value("VendorIdentifier", "REG_SZ") {
            parts.push(v);
        }
        if let Some(v) = value("Identifier", "REG_SZ") {
            parts.push(v);
        }
        if let Some(mhz) = value("~MHz", "REG_DWORD")
            .and_then(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        {
            parts.push(format!("{:.2} GHz", mhz as f64 / 1e3));
        }
        let ps = "(Get-CimInstance Win32_Processor | Measure-Object NumberOfCores -Sum).Sum";
        if let Some(n) = command("powershell", &["-NoProfile", "-Command", ps]) {
            parts.push(format!("{} physical cores", n));
        }
    } else {
        if let Some(n) = command("sysctl", &["-n", "hw.physicalcpu"]) {
            parts.push(format!("{} physical cores", n));
        }
        if let Some(hz) =
            command("sysctl", &["-n", "hw.cpufrequency_max"]).and_then(|v| v.parse::<u64>().ok())
        {
            parts.push(format!("max {:.2} GHz", hz as f64 / 1e9));
        }
    }
    if parts.is_empty() {
        "unknown".into()
    } else {
        parts.join(", ")
    }
}

/// The power setting the CPU runs under (Linux: the frequency governor;
/// Windows: the active power plan; macOS: low power mode).
fn power_mode() -> String {
    let governor = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    if let Ok(g) = std::fs::read_to_string(governor) {
        return format!("governor {}", g.trim());
    }
    if cfg!(windows) {
        if let Some(out) = command("powercfg", &["/getactivescheme"]) {
            return out.split_once(':').map_or(out.clone(), |x| x.1.trim().to_string());
        }
    }
    if let Some(out) = command("pmset", &["-g"]) {
        if let Some(l) = out.lines().find(|l| l.contains("lowpowermode")) {
            return l.trim().to_string();
        }
    }
    "unknown".into()
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
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
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
    let show = |name: &str, v: Option<usize>| {
        format!("{} {}", name, v.map_or("?".to_string(), |b| format!("{} KiB", b >> 10)))
    };
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
        let out = command("powershell", &[
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])?;
        return out.trim().parse().ok();
    }
    command("sysctl", &["-n", "hw.memsize"])?.trim().parse().ok()
}

fn machine(report: &mut Report, cfg: &Config) {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let reported = reported_l2().map_or("not read".to_string(), |b| format!("{} KiB", b >> 10));
    let lines = [
        format!("cpu,{}", cpu_name()),
        format!("cpu details,{}", cpu_details()),
        format!("power,{}", power_mode()),
        format!("arch,{}", std::env::consts::ARCH),
        format!("os,{}", os_version()),
        format!("threads,{}", threads),
        format!(
            "ram,{}",
            total_ram().map_or("unknown".to_string(), |b| format!(
                "{:.1} GiB",
                b as f64 / (1u64 << 30) as f64
            ))
        ),
        format!("l2,{} KiB used (reported: {})", l2_bytes() >> 10, reported),
        format!("caches,{}", cache_levels()),
        format!("page,{} B", page_bytes()),
        format!("target features,{}", target_features()),
        format!("rustc,{}", command("rustc", &["--version"]).unwrap_or_else(|| "unknown".into())),
        format!(
            "commit,{}",
            command("git", &["describe", "--always", "--dirty"])
                .unwrap_or_else(|| "unknown".into())
        ),
        format!(
            "settings,sizes {} to {} reps {} memory budget {} MiB parts {}",
            cfg.min,
            cfg.max,
            cfg.reps,
            cfg.memory >> 20,
            cfg.parts.join(" ")
        ),
    ];
    report.line("== machine");
    report.text("Machine");
    report.text("-------");
    for l in lines.iter() {
        report.line(l);
        if let Some((k, v)) = l.split_once(',') {
            report.text(&format!("  {:<16} {}", k, v));
        }
    }
}

fn ord_total_f32(a: &f32, b: &f32) -> bool { a.total_cmp(b).is_le() }

fn ord_total_f64(a: &f64, b: &f64) -> bool { a.total_cmp(b).is_le() }

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
    // `BENCH_OUT`, else resultsbenchs/<host>/; `BENCH_NAME`: the bench
    // (bench, bench-type-u32, bench-big...).
    let dir: std::path::PathBuf = std::env::var("BENCH_OUT").map(Into::into).unwrap_or_else(|_| {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resultsbenchs").join(host_name())
    });
    std::fs::create_dir_all(&dir).unwrap();
    let name = std::env::var("BENCH_NAME").unwrap_or_else(|_| "bench-cpu".into());
    let stem = format!("{}-{}-{}", name, host_name(), utc_stamp());
    let path = dir.join(format!("{}.csv", stem));
    let human = dir.join(format!("{}.txt", stem));
    let mut report = Report {
        file: std::fs::File::create(&path).unwrap(),
        human: std::fs::File::create(&human).unwrap(),
    };
    report.text(&format!("voracious_radix_sort {}: {}", env!("CARGO_PKG_VERSION"), name));
    report.text("");
    report.text(EXPLAINED);
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
    report.text(&format!("  {:<16} {}", "core", core));
    report.text("");
    let r = &mut report;

    // Noise: v2 against itself.
    if cfg.part("aa") {
        let u64s = all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators());
        bench(
            r,
            &cfg,
            "aa u64",
            &u64s,
            &[
                variant("v2", |a: &mut [u64]| devour_sort(a)),
                variant("v2 again", |a: &mut [u64]| devour_sort(a)),
            ],
            |a, b| a <= b,
            0,
        );
        bench(
            r,
            &cfg,
            "aa f64",
            &g::float_64::generators_f64(),
            &[
                variant("v2", |a: &mut [f64]| ravenous_sort(a)),
                variant("v2 again", |a: &mut [f64]| ravenous_sort(a)),
            ],
            ord_total_f64,
            0,
        );
    }

    // Integers.
    {
        use rdst::RadixSort as Rdst;
        macro_rules! int {
            ($t:ty, $gens:expr, [$($extra:expr),*]) => {
                if cfg.wants("int", stringify!($t)) {
                    let variants: Vec<Variant<$t>> = vec![
                        variant("v2", |a: &mut [$t]| devour_sort(a)),
                        variant("v2 new", |a: &mut [$t]| ab_new(|| devour_sort(a))),
                        variant("std", |a: &mut [$t]| a.sort_unstable()),
                        variant("v1", |a: &mut [$t]| a.voracious_sort()),
                        variant("glidesort", |a: &mut [$t]| glidesort::sort(a)),
                        $($extra),*
                    ];
                    bench(r, &cfg, stringify!($t), &$gens, &keep_sorts(stringify!($t), variants), |a, b| a <= b, 0);
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
        int!(
            u16,
            all(g::unsigned_u16::generators_u16(), g::narrow_extra::extra_u16::generators()),
            [
                variant("blazing", |a: &mut [u16]| blazing_radix::radix_sort::<u16>(a)),
                variant("radsort", |a: &mut [u16]| radsort::sort(a)),
                variant("rdst", |a: &mut [u16]| a.radix_sort_unstable()),
                variant("turbosort", |a: &mut [u16]| turbosort::sort(a))
            ]
        );
        int!(
            i16,
            all(g::signed_i16::generators_i16(), g::narrow_extra::extra_i16::generators()),
            [
                variant("blazing", |a: &mut [i16]| blazing_radix::radix_sort::<i16>(a)),
                variant("radsort", |a: &mut [i16]| radsort::sort(a))
            ]
        );
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
            variant("v2", |a: &mut [f32]| ravenous_sort(a)),
            variant("std", |a: &mut [f32]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("v1", |a: &mut [f32]| a.voracious_sort()),
            variant("glidesort", |a: &mut [f32]| glidesort::sort_by(a, |x, y| x.total_cmp(y))),
            variant("blazing", |a: &mut [f32]| blazing_radix::radix_sort::<f32>(a)),
            variant("radsort", |a: &mut [f32]| radsort::sort(a)),
            variant("turbosort", |a: &mut [f32]| turbosort::sort(a)),
        ];
        bench(
            r,
            &cfg,
            "f32",
            &all(g::float_32::generators_f32(), g::float_extra::extra_f32::generators()),
            &keep_sorts("f32", variants),
            ord_total_f32,
            0,
        );
    }
    if cfg.wants("float", "f64") {
        let variants: Vec<Variant<f64>> = vec![
            variant("v2", |a: &mut [f64]| ravenous_sort(a)),
            variant("std", |a: &mut [f64]| a.sort_unstable_by(|x, y| x.total_cmp(y))),
            variant("v1", |a: &mut [f64]| a.voracious_sort()),
            variant("glidesort", |a: &mut [f64]| glidesort::sort_by(a, |x, y| x.total_cmp(y))),
            variant("radsort", |a: &mut [f64]| radsort::sort(a)),
            variant("turbosort", |a: &mut [f64]| turbosort::sort(a)),
        ];
        bench(
            r,
            &cfg,
            "f64",
            &all(g::float_64::generators_f64(), g::float_extra::extra_f64::generators()),
            &keep_sorts("f64", variants),
            ord_total_f64,
            0,
        );
    }

    // 128-bit integers.
    {
        use rdst::RadixSort as Rdst;
        macro_rules! wide {
            ($t:ty, $gens:expr) => {
                if cfg.wants("wide", stringify!($t)) {
                    let variants: Vec<Variant<$t>> = vec![
                        variant("v2", |a: &mut [$t]| devour_sort(a)),
                        variant("std", |a: &mut [$t]| a.sort_unstable()),
                        variant("v1", |a: &mut [$t]| a.voracious_sort()),
                        variant("radsort", |a: &mut [$t]| radsort::sort(a)),
                        variant("rdst", |a: &mut [$t]| a.radix_sort_unstable()),
                    ];
                    bench(
                        r,
                        &cfg,
                        stringify!($t),
                        &$gens,
                        &keep_sorts(stringify!($t), variants),
                        |a, b| a <= b,
                        0,
                    );
                }
            };
        }
        wide!(
            u128,
            all(g::unsigned_u128::generators_u128(), g::int128_extra::extra_u128::generators())
        );
        wide!(
            i128,
            all(g::signed_i128::generators_i128(), g::int128_extra::extra_i128::generators())
        );
    }

    // bool arrays: the v2 has no bool sort (bool is a record key only); the
    // v1 boolean sort is the reference.
    if cfg.wants("int", "bool") {
        let variants: Vec<Variant<bool>> = vec![
            variant("v1", |a: &mut [bool]| a.voracious_sort()),
            variant("std", |a: &mut [bool]| a.sort()),
            variant("std unstable", |a: &mut [bool]| a.sort_unstable()),
            variant("glidesort", |a: &mut [bool]| glidesort::sort(a)),
            variant("radsort", |a: &mut [bool]| radsort::sort(a)),
        ];
        bench(
            r,
            &cfg,
            "bool",
            &g::boolean::generators_bool(),
            &keep_sorts("bool", variants),
            |a, b| a <= b,
            0,
        );
    }

    // Records sorted by key (v2 and v1 in both versions: stable, unstable;
    // rdst and std unstable are not stable).
    {
        use rdst::RadixSort as Rdst;
        macro_rules! records {
            ($label:expr, $ty:ty, $gens:expr) => {
                if cfg.wants("records", $label) {
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("v2", |a: &mut [$ty]| by_key::devour_sort_by_key(a, |x| x.key)),
                        variant("std", |a: &mut [$ty]| a.sort_by_key(|x| x.key)),
                        variant("std unstable", |a: &mut [$ty]| a.sort_unstable_by_key(|x| x.key)),
                        variant("std cached", |a: &mut [$ty]| a.sort_by_cached_key(|x| x.key)),
                        variant("glidesort", |a: &mut [$ty]| glidesort::sort_by_key(a, |x| x.key)),
                        variant("radsort", |a: &mut [$ty]| radsort::sort_by_key(a, |x| x.key)),
                        variant("rdst", |a: &mut [$ty]| a.radix_sort_unstable()),
                        variant("turbosort", |a: &mut [$ty]| turbosort::sort_by_key(a, |x| x.key)),
                        variant("v1", |a: &mut [$ty]| a.voracious_sort()),
                        // Like for like: unstable against unstable, stable against stable.
                        variant("v2 unstable", |a: &mut [$ty]| {
                            by_key::devour_sort_unstable_by_key(a, |x| x.key)
                        }),
                        variant("v1 stable", |a: &mut [$ty]| a.voracious_stable_sort()),
                    ];
                    bench(
                        r,
                        &cfg,
                        concat!("records ", $label),
                        &$gens,
                        &keep_sorts(concat!("records ", $label), variants),
                        ordered_by_key!($ty),
                        0,
                    );
                }
            };
        }
        // bool keys: rdst and turbosort take no bool key.
        macro_rules! records_bool {
            ($label:expr, $ty:ty) => {
                if cfg.wants("records", $label) {
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("v2", |a: &mut [$ty]| by_key::devour_sort_by_key(a, |x| x.key)),
                        variant("std", |a: &mut [$ty]| a.sort_by_key(|x| x.key)),
                        variant("std unstable", |a: &mut [$ty]| a.sort_unstable_by_key(|x| x.key)),
                        variant("glidesort", |a: &mut [$ty]| glidesort::sort_by_key(a, |x| x.key)),
                        variant("radsort", |a: &mut [$ty]| radsort::sort_by_key(a, |x| x.key)),
                        variant("v1", |a: &mut [$ty]| a.voracious_sort()),
                        variant("v2 unstable", |a: &mut [$ty]| {
                            by_key::devour_sort_unstable_by_key(a, |x| x.key)
                        }),
                        variant("v1 stable", |a: &mut [$ty]| a.voracious_stable_sort()),
                    ];
                    let gens = wrap(
                        g::boolean::generators_bool(),
                        rec::<bool, { std::mem::size_of::<$ty>() / 4 - 1 }>,
                    );
                    bench(
                        r,
                        &cfg,
                        concat!("records ", $label),
                        &gens,
                        &keep_sorts(concat!("records ", $label), variants),
                        ordered_by_key!($ty),
                        0,
                    );
                }
            };
        }
        records_bool!("bool 8B", Rec<bool, 1>);
        records_bool!("bool 16B", Rec<bool, 3>);
        // 128-bit keys (turbosort takes none), char keys (radsort and
        // glidesort only), float keys (std and glidesort by total_cmp).
        macro_rules! records_with {
            ($label:expr, $ty:ty, $gens:expr, $ordered:expr, [$($extra:expr),* $(,)?]) => {
                if cfg.wants("records", $label) {
                    let mut variants: Vec<Variant<$ty>> = vec![
                        variant("v2", |a: &mut [$ty]| by_key::devour_sort_by_key(a, |x| x.key)),
                        variant("radsort", |a: &mut [$ty]| radsort::sort_by_key(a, |x| x.key)),
                        variant("v1", |a: &mut [$ty]| a.voracious_sort()),
                        variant("v2 unstable", |a: &mut [$ty]| by_key::devour_sort_unstable_by_key(a, |x| x.key)),
                        variant("v1 stable", |a: &mut [$ty]| a.voracious_stable_sort()),
                    ];
                    variants.extend(vec![$($extra),*]);
                    bench(r, &cfg, concat!("records ", $label), &$gens, &keep_sorts(concat!("records ", $label), variants), $ordered, 0);
                }
            };
        }
        macro_rules! records_ord {
            ($label:expr, $ty:ty, $gens:expr, [$($extra:expr),* $(,)?]) => {
                records_with!($label, $ty, $gens, ordered_by_key!($ty), [
                    variant("std", |a: &mut [$ty]| a.sort_by_key(|x| x.key)),
                    variant("std unstable", |a: &mut [$ty]| a.sort_unstable_by_key(|x| x.key)),
                    variant("glidesort", |a: &mut [$ty]| glidesort::sort_by_key(a, |x| x.key)),
                    $($extra),*
                ])
            };
        }
        macro_rules! records_float {
            ($label:expr, $ty:ty, $gens:expr) => {
                records_with!(
                    $label,
                    $ty,
                    $gens,
                    |a: &$ty, b: &$ty| a.key.total_cmp(&b.key).is_le(),
                    [
                        variant("std", |a: &mut [$ty]| a.sort_by(|x, y| x.key.total_cmp(&y.key))),
                        variant("std unstable", |a: &mut [$ty]| a
                            .sort_unstable_by(|x, y| x.key.total_cmp(&y.key))),
                        variant("glidesort", |a: &mut [$ty]| glidesort::sort_by(a, |x, y| x
                            .key
                            .total_cmp(&y.key))),
                        variant("rdst", |a: &mut [$ty]| a.radix_sort_unstable()),
                        variant("turbosort", |a: &mut [$ty]| turbosort::sort_by_key(a, |x| x.key)),
                    ]
                )
            };
        }
        // Every key type in every record size class of the v2 chains (8,
        // 16, 32, 64 bytes and more; 8-byte keys from 16, 16-byte keys from
        // 32).
        records!("u8 8B", Rec<u8, 1>, wrap(keys_u8(), rec::<u8, 1>));
        records!("u8 16B", Rec<u8, 3>, wrap(keys_u8(), rec::<u8, 3>));
        records!("u8 32B", Rec<u8, 7>, wrap(keys_u8(), rec::<u8, 7>));
        records!("u8 64B", Rec<u8, 15>, wrap(keys_u8(), rec::<u8, 15>));
        records!("i8 8B", Rec<i8, 1>, wrap(keys_i8(), rec::<i8, 1>));
        records!("i8 16B", Rec<i8, 3>, wrap(keys_i8(), rec::<i8, 3>));
        records!("i8 32B", Rec<i8, 7>, wrap(keys_i8(), rec::<i8, 7>));
        records!("i8 64B", Rec<i8, 15>, wrap(keys_i8(), rec::<i8, 15>));
        records!("u16 8B", Rec<u16, 1>, wrap(keys_u16(), rec::<u16, 1>));
        records!("u16 16B", Rec<u16, 3>, wrap(keys_u16(), rec::<u16, 3>));
        records!("u16 32B", Rec<u16, 7>, wrap(keys_u16(), rec::<u16, 7>));
        records!("u16 64B", Rec<u16, 15>, wrap(keys_u16(), rec::<u16, 15>));
        records!("i16 8B", Rec<i16, 1>, wrap(keys_i16(), rec::<i16, 1>));
        records!("i16 16B", Rec<i16, 3>, wrap(keys_i16(), rec::<i16, 3>));
        records!("i16 32B", Rec<i16, 7>, wrap(keys_i16(), rec::<i16, 7>));
        records!("i16 64B", Rec<i16, 15>, wrap(keys_i16(), rec::<i16, 15>));
        records!("u32 8B", Rec<u32, 1>, wrap(keys_u32(), rec::<u32, 1>));
        records!("u32 16B", Rec<u32, 3>, wrap(keys_u32(), rec::<u32, 3>));
        records!("u32 32B", Rec<u32, 7>, wrap(keys_u32(), rec::<u32, 7>));
        records!("u32 128B", Rec<u32, 31>, wrap(keys_u32(), rec::<u32, 31>));
        records!("i32 8B", Rec<i32, 1>, wrap(keys_i32(), rec::<i32, 1>));
        records!("i32 16B", Rec<i32, 3>, wrap(keys_i32(), rec::<i32, 3>));
        records!("i32 32B", Rec<i32, 7>, wrap(keys_i32(), rec::<i32, 7>));
        records!("i32 64B", Rec<i32, 15>, wrap(keys_i32(), rec::<i32, 15>));
        records!("u64 16B", Rec<u64, 2>, wrap(keys_u64(), rec::<u64, 2>));
        records!("u64 32B", Rec<u64, 6>, wrap(keys_u64(), rec::<u64, 6>));
        records!("u64 64B", Rec<u64, 14>, wrap(keys_u64(), rec::<u64, 14>));
        records!("i64 16B", Rec<i64, 2>, wrap(keys_i64(), rec::<i64, 2>));
        records!("i64 32B", Rec<i64, 6>, wrap(keys_i64(), rec::<i64, 6>));
        records!("i64 64B", Rec<i64, 14>, wrap(keys_i64(), rec::<i64, 14>));
        records_ord!("u128 32B", Rec<u128, 4>, wrap(keys_u128(), rec::<u128, 4>), [
            variant("rdst", |a: &mut [Rec<u128, 4>]| a.radix_sort_unstable())
        ]);
        records_ord!("u128 64B", Rec<u128, 12>, wrap(keys_u128(), rec::<u128, 12>), [
            variant("rdst", |a: &mut [Rec<u128, 12>]| a.radix_sort_unstable())
        ]);
        records_ord!("i128 32B", Rec<i128, 4>, wrap(keys_i128(), rec::<i128, 4>), [
            variant("rdst", |a: &mut [Rec<i128, 4>]| a.radix_sort_unstable())
        ]);
        records_ord!("i128 64B", Rec<i128, 12>, wrap(keys_i128(), rec::<i128, 12>), [
            variant("rdst", |a: &mut [Rec<i128, 12>]| a.radix_sort_unstable())
        ]);
        records_ord!("char 8B", Rec<char, 1>, wrap(keys_char(), rec::<char, 1>), []);
        records_ord!("char 16B", Rec<char, 3>, wrap(keys_char(), rec::<char, 3>), []);
        records_ord!("char 32B", Rec<char, 7>, wrap(keys_char(), rec::<char, 7>), []);
        records_ord!("char 64B", Rec<char, 15>, wrap(keys_char(), rec::<char, 15>), []);
        records_float!("f32 8B", Rec<f32, 1>, wrap(keys_f32(), rec::<f32, 1>));
        records_float!("f32 16B", Rec<f32, 3>, wrap(keys_f32(), rec::<f32, 3>));
        records_float!("f32 32B", Rec<f32, 7>, wrap(keys_f32(), rec::<f32, 7>));
        records_float!("f32 64B", Rec<f32, 15>, wrap(keys_f32(), rec::<f32, 15>));
        records_float!("f64 16B", Rec<f64, 2>, wrap(keys_f64(), rec::<f64, 2>));
        records_float!("f64 32B", Rec<f64, 6>, wrap(keys_f64(), rec::<f64, 6>));
        records_float!("f64 64B", Rec<f64, 14>, wrap(keys_f64(), rec::<f64, 14>));
    }

    // CPU parameters of the records sort: from which record size the run
    // merges go 4-way (one move per two levels) instead of 2-way (cheaper
    // steps): the default 32 was measured on the Snapdragon X.
    if cfg.part("rparams") {
        use crate::v2::sorts::by_key::FOUR_WAY_MIN;
        let with = |b: usize| move || FOUR_WAY_MIN.store(b, Relaxed);
        macro_rules! rparams {
            ($label:expr, $ty:ty, $gens:expr) => {
                if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == "records")) {
                    let runs: Gens<$ty> = $gens
                        .into_iter()
                        .filter(|(_, n)| {
                            ["Runs", "Saw", "Killer", "Almost", "Pipe"]
                                .iter()
                                .any(|k| n.contains(k))
                        })
                        .collect();
                    let (w16, w32, w64, wnever) = (with(16), with(32), with(64), with(usize::MAX));
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("4-way from 32B", move |a: &mut [$ty]| {
                            w32();
                            by_key::devour_sort_by_key(a, |x| x.key);
                        }),
                        variant("4-way from 16B", move |a: &mut [$ty]| {
                            w16();
                            by_key::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                        variant("4-way from 64B", move |a: &mut [$ty]| {
                            w64();
                            by_key::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                        variant("2-way only", move |a: &mut [$ty]| {
                            wnever();
                            by_key::devour_sort_by_key(a, |x| x.key);
                            FOUR_WAY_MIN.store(32, Relaxed);
                        }),
                    ];
                    bench(
                        r,
                        &cfg,
                        concat!("rparams ", $label),
                        &runs,
                        &variants,
                        ordered_by_key!($ty),
                        100_000,
                    );
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
        if cfg.types.as_ref().is_none_or(|t| t.iter().any(|x| x == "records")) {
            use crate::v2::sorts::by_key::LSD11;
            type R = Rec<u64, 2>;
            let disorder: Gens<R> = wrap(keys_u64(), rec::<u64, 2>)
                .into_iter()
                .filter(|(_, n)| {
                    [
                        "Unif",
                        "Normale",
                        "Int 0..n",
                        "Zipf",
                        "Composite",
                        "Clusters",
                        "Bimodal",
                        "Timestamps",
                        "Aligned",
                        "Prefix",
                    ]
                    .iter()
                    .any(|k| n.contains(k))
                })
                .collect();
            let variants: Vec<Variant<R>> = vec![
                variant("LSD 11 bits", |a: &mut [R]| {
                    LSD11.store(true, Relaxed);
                    by_key::devour_sort_by_key(a, |x| x.key);
                }),
                variant("LSD 8 bits", |a: &mut [R]| {
                    LSD11.store(false, Relaxed);
                    by_key::devour_sort_by_key(a, |x| x.key);
                    LSD11.store(true, Relaxed);
                }),
            ];
            bench(
                r,
                &cfg,
                "rparams lsd digit u64 16B",
                &disorder,
                &variants,
                ordered_by_key!(R),
                1_000_000,
            );
        }
    }

    // One cache-derived decision at a time: its budget forced to a quarter,
    // 4x and 16x the L2 and to the L3, the others left at the L2 (the `l2`
    // part moves them all together). Tells which cache level each decision
    // should follow on this CPU. `BENCH_TUNE`: decisions to run (names of
    // `BUDGETS`), all by default.
    if cfg.part("tune") {
        let wanted: Option<Vec<String>> = std::env::var("BENCH_TUNE")
            .ok()
            .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());
        let runs = |n: &str| {
            ["Runs", "Saw", "Killer", "Almost", "Pipe", "Asc", "Desc", "Front", "Middle", "Organ"]
                .iter()
                .any(|k| n.contains(k))
        };
        // `BUDGET_OVERRIDE` is indexed by the enum order.
        assert!(BUDGETS.iter().enumerate().all(|(i, &(b, _))| b as usize == i));
        for &(b, name) in BUDGETS.iter() {
            if wanted.as_ref().is_some_and(|w| !w.iter().any(|x| x == name)) {
                continue;
            }
            macro_rules! tune {
                ($ty:ty, $tyname:expr, $gens:expr, $sort:expr, $ordered:expr) => {
                    if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == $tyname)) {
                        let variants: Vec<Variant<$ty>> = tune_variants(b, $sort);
                        bench(
                            r,
                            &cfg,
                            &format!("tune {} {}", name, $tyname),
                            &$gens,
                            &variants,
                            $ordered,
                            10_000,
                        );
                    }
                };
            }
            let u32s =
                || all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators());
            let u64s =
                || all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators());
            let only_runs =
                |v: Gens<u64>| -> Gens<u64> { v.into_iter().filter(|(_, n)| runs(n)).collect() };
            match b {
                Budget::Lsd11 | Budget::Roller32 => {
                    tune!(
                        u32,
                        "u32",
                        u32s(),
                        |a: &mut [u32]| devour_sort(a),
                        |a: &u32, b: &u32| a <= b
                    );
                },
                Budget::Roller64 | Budget::AliasInt => {
                    tune!(
                        u64,
                        "u64",
                        u64s(),
                        |a: &mut [u64]| devour_sort(a),
                        |a: &u64, b: &u64| a <= b
                    );
                },
                Budget::CountLanes | Budget::WideCount => {
                    tune!(
                        u32,
                        "u32",
                        u32s(),
                        |a: &mut [u32]| devour_sort(a),
                        |a: &u32, b: &u32| a <= b
                    );
                    tune!(
                        u64,
                        "u64",
                        u64s(),
                        |a: &mut [u64]| devour_sort(a),
                        |a: &u64, b: &u64| a <= b
                    );
                },
                Budget::Roller128 => {
                    let gens = all(
                        g::unsigned_u128::generators_u128(),
                        g::int128_extra::extra_u128::generators(),
                    );
                    tune!(
                        u128,
                        "u128",
                        gens,
                        |a: &mut [u128]| devour_sort(a),
                        |a: &u128, b: &u128| a <= b
                    );
                },
                Budget::RollerF32 => {
                    tune!(
                        f32,
                        "f32",
                        g::float_32::generators_f32(),
                        |a: &mut [f32]| ravenous_sort(a),
                        ord_total_f32
                    );
                },
                Budget::RollerF64 | Budget::AliasFloat => {
                    tune!(
                        f64,
                        "f64",
                        g::float_64::generators_f64(),
                        |a: &mut [f64]| ravenous_sort(a),
                        ord_total_f64
                    );
                },
                Budget::MergeTwoWay | Budget::PingPong => {
                    tune!(
                        u64,
                        "u64",
                        only_runs(u64s()),
                        |a: &mut [u64]| devour_sort(a),
                        |a: &u64, b: &u64| a <= b
                    );
                },
                Budget::Records => {
                    if cfg.types.as_ref().is_none_or(|t| t.iter().any(|x| x == "records")) {
                        type R8 = Rec<u32, 1>;
                        type R16 = Rec<u64, 2>;
                        type R32 = Rec<u64, 6>;
                        let v8: Vec<Variant<R8>> = tune_variants(b, |a: &mut [R8]| {
                            by_key::devour_sort_by_key(a, |x| x.key)
                        });
                        bench(
                            r,
                            &cfg,
                            "tune records 8B",
                            &wrap(keys_u32(), rec::<u32, 1>),
                            &v8,
                            ordered_by_key!(R8),
                            10_000,
                        );
                        let v16: Vec<Variant<R16>> = tune_variants(b, |a: &mut [R16]| {
                            by_key::devour_sort_by_key(a, |x| x.key)
                        });
                        bench(
                            r,
                            &cfg,
                            "tune records 16B",
                            &wrap(keys_u64(), rec::<u64, 2>),
                            &v16,
                            ordered_by_key!(R16),
                            10_000,
                        );
                        let v32: Vec<Variant<R32>> = tune_variants(b, |a: &mut [R32]| {
                            by_key::devour_sort_by_key(a, |x| x.key)
                        });
                        bench(
                            r,
                            &cfg,
                            "tune records 32B",
                            &wrap(keys_u64(), rec::<u64, 6>),
                            &v32,
                            ordered_by_key!(R32),
                            10_000,
                        );
                    }
                },
            }
        }
    }

    // Very large arrays (30M to 1G keys, one thread): the rollercoaster
    // (MSD passes first) against the LSD / DLSD, each forced, on the shapes
    // in disorder. The choice between them depends on the CPU (the LSD was
    // faster at 10M+ on an Intel Core Ultra, the MSD on an AMD Ryzen). One
    // array per measurement.
    if cfg.part("big") {
        use crate::v2::cache::Budget as B;
        // Three arrays live (batch, working copy, buffer): up to 3/4 of the
        // RAM unless BENCH_MEM_MB says otherwise.
        let memory = if std::env::var("BENCH_MEM_MB").is_ok() {
            cfg.memory
        } else {
            total_ram().map_or(cfg.memory, |r| r / 4 * 3)
        };
        let big = Config { min_arrays: 1, memory, ..cfg.clone() };
        // `BENCH_BIG_SIZES`: other sizes (e.g. 1M to 100M, to place the threshold).
        let listed: Option<Vec<usize>> = std::env::var("BENCH_BIG_SIZES")
            .ok()
            .map(|l| l.split(',').map(|x| x.trim().replace('_', "").parse().unwrap()).collect());
        let sizes: Vec<usize> = listed
            .unwrap_or(BIG_SIZES.to_vec())
            .into_iter()
            .filter(|&s| s >= cfg.min && s <= cfg.max)
            .collect();
        let keep = |n: &str| {
            let n = short_name(n);
            [
                "Unif",
                "Normale 20",
                "Normale 30",
                "Zipf",
                "Clusters",
                "Aligned",
                "Prefix",
                "Asc Saw",
                "Timestamps",
                "Bimodal",
            ]
            .iter()
            .any(|k| n.starts_with(k))
        };
        macro_rules! big {
            ($ty:ty, $gens:expr, $budget:expr, $sort:expr, $ordered:expr) => {
                if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == stringify!($ty))) {
                    let gens: Gens<$ty> = $gens.into_iter().filter(|(_, n)| keep(n)).collect();
                    let b = $budget as usize;
                    let variants: Vec<Variant<$ty>> = vec![
                        variant("default", |a: &mut [$ty]| $sort(a)),
                        variant("LSD", move |a: &mut [$ty]| {
                            BUDGET_OVERRIDE[b].store(1 << 50, Relaxed);
                            $sort(a);
                            BUDGET_OVERRIDE[b].store(0, Relaxed);
                        }),
                        variant("rollercoaster", move |a: &mut [$ty]| {
                            BUDGET_OVERRIDE[b].store(1, Relaxed);
                            $sort(a);
                            BUDGET_OVERRIDE[b].store(0, Relaxed);
                        }),
                    ];
                    bench_sizes(
                        r,
                        &big,
                        concat!("big ", stringify!($ty)),
                        &gens,
                        &variants,
                        $ordered,
                        &sizes,
                    );
                }
            };
        }
        big!(
            u32,
            all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators()),
            B::Roller32,
            devour_sort,
            |a: &u32, b: &u32| a <= b
        );
        big!(
            u64,
            all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators()),
            B::Roller64,
            devour_sort,
            |a: &u64, b: &u64| a <= b
        );
        big!(f32, g::float_32::generators_f32(), B::RollerF32, ravenous_sort, ord_total_f32);
        big!(f64, g::float_64::generators_f64(), B::RollerF64, ravenous_sort, ord_total_f64);
    }

    // Thresholds derived from the L2: detected size against half and twice.
    if cfg.part("l2") {
        let l2 = l2_bytes();
        let with_l2 = |bytes: usize| move || L2_OVERRIDE.store(bytes, Relaxed);
        macro_rules! l2 {
            ($t:ty, $gens:expr, $sort:expr, $ordered:expr) => {
                if cfg.types.as_ref().map_or(true, |t| t.iter().any(|x| x == stringify!($t))) {
                    let (half, double, quad, arm) =
                        (with_l2(l2 / 2), with_l2(l2 * 2), with_l2(l2 * 4), with_l2(ARM_L2));
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
                    bench(
                        r,
                        &cfg,
                        concat!("l2 ", stringify!($t)),
                        &$gens,
                        &variants,
                        $ordered,
                        100_000,
                    );
                }
            };
        }
        l2!(
            u32,
            all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators()),
            devour_sort,
            |a: &u32, b: &u32| a <= b
        );
        l2!(
            u64,
            all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators()),
            devour_sort,
            |a: &u64, b: &u64| a <= b
        );
        l2!(f64, g::float_64::generators_f64(), ravenous_sort, ord_total_f64);
        if cfg.types.as_ref().is_none_or(|t| t.iter().any(|x| x == "records")) {
            let (half, double, quad, arm) =
                (with_l2(l2 / 2), with_l2(l2 * 2), with_l2(l2 * 4), with_l2(ARM_L2));
            type R = Rec<u64, 6>;
            let variants: Vec<Variant<R>> = vec![
                variant("detected L2", |a: &mut [R]| by_key::devour_sort_by_key(a, |x| x.key)),
                variant("L2 / 2", move |a: &mut [R]| {
                    half();
                    by_key::devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 x 2", move |a: &mut [R]| {
                    double();
                    by_key::devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 x 4", move |a: &mut [R]| {
                    quad();
                    by_key::devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
                variant("L2 12 MiB", move |a: &mut [R]| {
                    arm();
                    by_key::devour_sort_by_key(a, |x| x.key);
                    L2_OVERRIDE.store(0, Relaxed);
                }),
            ];
            bench(
                r,
                &cfg,
                "l2 records u64 32B",
                &wrap(keys_u64(), rec::<u64, 6>),
                &variants,
                ordered_by_key!(R),
                100_000,
            );
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
        fn GetSystemCpuSetInformation(
            info: *mut u8,
            len: u32,
            returned: *mut u32,
            process: isize,
            flags: u32,
        ) -> i32;
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
        if len == 0
            || unsafe {
                GetSystemCpuSetInformation(buf.as_mut_ptr() as *mut u8, len, &mut len, 0, 0)
            } == 0
        {
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
        let kinds = format!(
            "{} logical processors, {} of the fastest class {} (classes {}-{})",
            sets.len(),
            fast,
            hi,
            lo,
            hi
        );
        if std::env::var("BENCH_PIN").is_ok_and(|v| v == "0") {
            return format!("not pinned (BENCH_PIN=0), current processor {}; {}", current(), kinds);
        }
        if lo == hi {
            return format!(
                "not pinned (all cores alike), current processor {}; {}",
                current(),
                kinds
            );
        }
        // `BENCH_PIN=slow`: the slowest class instead (to measure the gap).
        let class = if std::env::var("BENCH_PIN").is_ok_and(|v| v == "slow") { lo } else { hi };
        // A core of that class other than core 0 (interrupts) when there is one.
        let chosen: Vec<_> = sets.iter().filter(|s| s.3 == class).collect();
        let pick = chosen.iter().find(|s| s.2 != 0).unwrap_or(&chosen[0]);
        // SAFETY: GetCurrentThread returns a pseudo handle; one valid id.
        if unsafe { SetThreadSelectedCpuSets(GetCurrentThread(), &pick.0, 1) } == 0 {
            return format!("not pinned (SetThreadSelectedCpuSets failed); {}", kinds);
        }
        std::thread::yield_now();
        format!(
            "pinned to logical processor {} (core {}, class {}), now on processor {}; {}",
            pick.1,
            pick.2,
            class,
            current(),
            kinds
        )
    }

    pub(super) fn current() -> u32 {
        // SAFETY: no arguments, cannot fail.
        unsafe { GetCurrentProcessorNumber() }
    }
}

#[cfg(not(windows))]
mod pin {
    pub(super) fn pin_fastest() -> String { "not pinned (only on Windows)".into() }
}
