//! Cold measure (`BENCH_COLD=1`, the default of `make bench-type`): in real
//! life an array is not sorted again and again while it sits in the caches.
//!
//! ```text
//! warm:  batch of arrays, then per sort: copy the batch, sort sort sort ...
//! cold:  arrays of KEYS keys in all (beyond the caches), then per
//!        repetition and per sort (order rotating, the copy alone as one
//!        more slot): for each array, a fresh copy, sorted once;
//!        time per array = (sorts - copies alone) / arrays
//! ```
//!
//! The same measure as the work benches (`make abench-u16`, `abench-i16`),
//! so the published numbers and the work ones can be compared.

use std::time::Instant;

/// `BENCH_COLD=1`: the cold measure.
pub(crate) fn enabled() -> bool {
    std::env::var("BENCH_COLD").is_ok_and(|v| v == "1")
}

/// Keys of all the arrays of one measurement: 12 M, as the work benches.
const KEYS: usize = 12_000_000;
/// Most bytes of them (large records).
const BYTES: usize = 256 << 20;
/// From this size an array is copied into a buffer kept between
/// repetitions (a fresh allocation of a large array measures the system's
/// page faults more than the sort; the work benches do the same).
const FRESH_BELOW: usize = 1_000_000;

/// Arrays per measurement at `size`.
pub(crate) fn arrays<T>(size: usize) -> usize {
    let bytes = std::mem::size_of::<T>().max(1);
    (KEYS / size).min(BYTES / (size * bytes)).max(1)
}

/// Copies kept between repetitions (large arrays), empty below FRESH_BELOW.
pub(crate) fn buffers<T: Clone>(arrays: &[Vec<T>]) -> Vec<Vec<T>> {
    if arrays.first().is_some_and(|a| a.len() >= FRESH_BELOW) {
        arrays.to_vec()
    } else {
        Vec::new()
    }
}

/// ns of copying then sorting every array once (`sort` = None: the copies
/// alone).
pub(crate) fn time<T: Clone>(
    arrays: &[Vec<T>],
    work: &mut [Vec<T>],
    sort: Option<&dyn Fn(&mut [T])>,
) -> f64 {
    let start = Instant::now();
    if work.is_empty() {
        for a in arrays {
            let mut w = a.clone();
            if let Some(s) = sort {
                s(&mut w);
            }
            std::hint::black_box(&w);
        }
    } else {
        for (w, a) in work.iter_mut().zip(arrays) {
            w.clone_from_slice(a);
            if let Some(s) = sort {
                s(w);
            }
            std::hint::black_box(&w);
        }
    }
    start.elapsed().as_nanos() as f64
}
