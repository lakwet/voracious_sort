//! Decision trace of the v2 sorts: for every (type, size, distribution)
//! cell (`cells`), the path the sort takes (presorted, detectors, Verge,
//! engines), read from the `trace!` notes. No timing: a change shows the
//! exact cells it moves, and only those need the sentinel bench. Inputs
//! are seeded per cell: two runs of the same code give the same trace.
//!
//! ```text
//! make trace        # compare with the reference of this CPU
//! make trace-ab     # rule under test (v2::ab) off against on
//! make trace-write  # write the reference
//! ```
//!
//! The reference is `traces/<arch>-l2-<KiB>-big-<KiB>.txt` (the decisions
//! depend on the architecture and on the two cache sizes the rules read).
//! `TRACE_WRITE=1` writes it; `TRACE_AB=1` traces each cell with the rule
//! under test off then on, without reference; otherwise the run lists the
//! cells whose path differs from the reference. The moved cells also go to
//! `traces/moved.txt`, which the sentinel bench reads. `TRACE_MAX` (keys,
//! `10M` accepted) skips the larger sizes; `TRACE_TYPES` keeps some types.

#![cfg(feature = "v2_trace")]

use super::super::v2::ab::NEW_RULE;
use super::super::v2::cache::{l2_bytes, largest_bytes};
use super::super::v2::trace;
use super::cells::{env_list, env_size, parse_key, visit, Visit};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering::Relaxed;

/// The path of every visited cell; with `ab`, both sides of the rule.
struct Tracer {
    ab: bool,
    out: Vec<(String, String, String)>,
}

impl Visit for Tracer {
    fn wants(&self, _: &str) -> bool {
        true
    }
    fn cell<T: Copy + PartialOrd + Send + 'static>(&mut self, key: &str, _: usize, keys: &dyn Fn() -> Vec<T>, sort: fn(&mut [T])) {
        let mut path = |new: bool| {
            NEW_RULE.store(new, Relaxed);
            let mut a = keys();
            trace::take();
            sort(&mut a);
            NEW_RULE.store(false, Relaxed);
            trace::take().join(" > ")
        };
        let old = path(false);
        let new = if self.ab { path(true) } else { old.clone() };
        self.out.push((key.to_string(), old, new));
    }
}

/// `type size name` aligned in columns.
fn show(key: &str) -> String {
    let mut it = key.splitn(3, ' ');
    let (t, n, name) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or(""));
    format!("{:5} {:>9} {:16}", t, n, name)
}

fn file_name() -> String {
    format!("{}-l2-{}-big-{}.txt", std::env::consts::ARCH, l2_bytes() >> 10, largest_bytes() >> 10)
}

#[test]
#[ignore]
fn trace_decisions() {
    let max = env_size("TRACE_MAX").unwrap_or(usize::MAX);
    let types = env_list("TRACE_TYPES");
    let ab = std::env::var("TRACE_AB").is_ok();
    let mut t = Tracer { ab, out: Vec::new() };
    visit(&mut t, types.as_deref(), max);

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("traces");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(file_name());
    if std::env::var("TRACE_WRITE").is_ok() {
        let text: String = t.out.iter().map(|(k, p, _)| format!("{} : {}\n", show(k), p)).collect();
        std::fs::write(&file, &text).unwrap();
        println!("{} cells written to {}", t.out.len(), file.display());
        return;
    }
    // Old side: the reference, or the rule off.
    let reference: BTreeMap<String, String> = if ab {
        t.out.iter().map(|(k, p, _)| (k.clone(), p.clone())).collect()
    } else {
        let text = std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("no reference {} (make trace-write)", file.display()));
        text.lines().filter_map(|l| Some((parse_key(l)?, l.split_once(" : ")?.1.to_string()))).collect()
    };
    let mut moved = String::new();
    let mut count = 0;
    for (k, p, q) in t.out.iter() {
        let new = if ab { q } else { p };
        match reference.get(k) {
            Some(old) if old == new => {}
            Some(old) => {
                count += 1;
                let line = format!("{} : {}  ->  {}", show(k), old, new);
                println!("{}", line);
                moved.push_str(&line);
                moved.push('\n');
            }
            None => println!("{} : (new) {}", show(k), new),
        }
    }
    std::fs::write(dir.join("moved.txt"), &moved).unwrap();
    let against = if ab { "rule under test off -> on".to_string() } else { format!("reference {}", file.display()) };
    println!("{} cells, {} moved ({}); list in traces/moved.txt", t.out.len(), count, against);
}
