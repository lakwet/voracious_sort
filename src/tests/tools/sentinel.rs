//! Sentinel bench: the rule under test (`v2::dev::ab`) off against on, in one
//! process, on the cells the decision trace saw move plus a fixed set of
//! control cells (one per type family, size band and shape). The controls
//! give the noise of the machine: a change is judged on the moved cells
//! against that noise, in minutes instead of a full bench.
//!
//! ```text
//! make trace-ab   # writes traces/moved.txt
//! make sentinel   # times those cells and the controls
//! ```
//!
//! Settings: `SENT_CELLS` (cell list, default `traces/moved.txt`; lines of
//! a trace or `type size name`), `SENT_CONTROLS=0` (no controls),
//! `SENT_MAX` (largest size, `10M` accepted), `SENT_REPS` (measurements per
//! side, default 5, the median kept). Results also go to
//! `traces/sentinel-last.txt`.

use crate::v2::dev::ab::NEW_RULE;
use super::cells::{env_size, parse_key, visit, Visit};
use std::collections::BTreeSet;
use std::sync::atomic::Ordering::Relaxed;

/// Control cells: these types (one per family), sizes (in the L2, around
/// the caches, beyond) and shapes, where they exist.
const CONTROL_TYPES: [&str; 8] = ["u32", "i64", "f32", "f64", "u16", "u128", "r16u64", "ru16u64"];
const CONTROL_SIZES: [usize; 3] = [10_000, 1_000_000, 30_000_000];
const CONTROL_SHAPES: [&str; 10] = [
    "Unif",
    "Few 16",
    "Almost Asc",
    "Asc Saw",
    "Normale 20",
    "Runs 64",
    "Timestamps",
    "Clusters",
    "Gaussian",
    "Near sorted",
];

/// Keys sorted per measurement at least (several arrays for small sizes).
const BATCH_KEYS: usize = 4_000_000;

struct Sentinel {
    moved: BTreeSet<String>,
    controls: bool,
    reps: usize,
    /// key, moved, old ns per key, new ns per key.
    rows: Vec<(String, bool, f64, f64)>,
}

fn is_control(key: &str) -> bool {
    let mut it = key.splitn(3, ' ');
    let (t, n, name) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or(""));
    CONTROL_TYPES.contains(&t)
        && n.parse().is_ok_and(|n: usize| CONTROL_SIZES.contains(&n))
        && CONTROL_SHAPES.contains(&name)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

impl Visit for Sentinel {
    fn wants(&self, key: &str) -> bool {
        self.moved.contains(key) || (self.controls && is_control(key))
    }
    fn cell<T: Copy + PartialOrd + Send + 'static>(
        &mut self,
        key: &str,
        n: usize,
        keys: &dyn Fn() -> Vec<T>,
        sort: fn(&mut [T]),
    ) {
        let a = keys();
        let count = (BATCH_KEYS / n.max(1)).max(1);
        let mut times = [Vec::new(), Vec::new()];
        for rep in 0..self.reps {
            for k in 0..2 {
                let side = (k + rep) % 2;
                let mut work: Vec<Vec<T>> = (0..count).map(|_| a.clone()).collect();
                NEW_RULE.store(side == 1, Relaxed);
                let start = std::time::Instant::now();
                for w in work.iter_mut() {
                    sort(w);
                }
                times[side].push(start.elapsed().as_nanos() as f64 / (count * n) as f64);
                NEW_RULE.store(false, Relaxed);
                assert!(
                    work[0].windows(2).all(|p| p[1].partial_cmp(&p[0]) != Some(std::cmp::Ordering::Less)),
                    "not sorted: {} (rule {})",
                    key,
                    side
                );
            }
        }
        let [old, new] = times;
        let (old, new) = (median(old), median(new));
        let moved = self.moved.contains(key);
        println!(
            "{:42} {:>5} old {:7.2} new {:7.2} new/old {:5.2}",
            key,
            if moved { "moved" } else { "ctrl" },
            old,
            new,
            new / old
        );
        self.rows.push((key.to_string(), moved, old, new));
    }
}

fn geo(r: &[f64]) -> f64 {
    (r.iter().map(|x| x.ln()).sum::<f64>() / r.len().max(1) as f64).exp()
}

#[test]
#[ignore]
fn sentinel() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let list = std::env::var("SENT_CELLS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| root.join("traces").join("moved.txt"));
    let moved: BTreeSet<String> = std::fs::read_to_string(&list)
        .map(|t| t.lines().filter_map(parse_key).collect())
        .unwrap_or_default();
    let controls = std::env::var("SENT_CONTROLS").map_or(true, |v| v != "0");
    let reps = std::env::var("SENT_REPS").ok().and_then(|s| s.parse().ok()).unwrap_or(5);
    let max = env_size("SENT_MAX").unwrap_or(usize::MAX);
    println!(
        "{} moved cells from {}, controls {}",
        moved.len(),
        list.display(),
        if controls { "on" } else { "off" }
    );
    let mut s = Sentinel { moved, controls, reps, rows: Vec::new() };
    visit(&mut s, None, max);

    let mut text = String::new();
    for (k, m, o, n) in s.rows.iter() {
        text.push_str(&format!(
            "{:42} {:>5} old {:7.2} new {:7.2} new/old {:5.2}\n",
            k,
            if *m { "moved" } else { "ctrl" },
            o,
            n,
            n / o
        ));
    }
    // Noise: the spread of new/old on the controls the rule does not move.
    let ctrl: Vec<f64> = s.rows.iter().filter(|r| !r.1).map(|r| r.3 / r.2).collect();
    let mov: Vec<f64> = s.rows.iter().filter(|r| r.1).map(|r| r.3 / r.2).collect();
    let spread = ctrl.iter().map(|r| (r.ln()).abs()).fold(0f64, f64::max).exp();
    let mut summary = format!(
        "controls: {} cells, geo new/old {:.3}, widest {:.3} (noise)\n",
        ctrl.len(),
        geo(&ctrl),
        spread
    );
    if !mov.is_empty() {
        let losses = mov.iter().filter(|&&r| r > spread.max(1.05)).count();
        let wins = mov.iter().filter(|&&r| r < 1.0 / spread.max(1.05)).count();
        summary.push_str(&format!(
            "moved: {} cells, geo new/old {:.3}, {} faster and {} slower beyond the noise\n",
            mov.len(),
            geo(&mov),
            wins,
            losses
        ));
        let mut worst: Vec<&(String, bool, f64, f64)> = s.rows.iter().filter(|r| r.1).collect();
        worst.sort_by(|a, b| (b.3 / b.2).total_cmp(&(a.3 / a.2)));
        for r in worst.iter().take(5) {
            summary.push_str(&format!("  worst: {} new/old {:.2}\n", r.0, r.3 / r.2));
        }
    }
    print!("{}", summary);
    text.push_str(&summary);
    std::fs::write(root.join("traces").join("sentinel-last.txt"), text).unwrap();
}
