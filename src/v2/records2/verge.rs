//! Verge on the records: runs kept, the gaps sorted, all merged.
//!
//! Unlike Verge on the keys, merging moves the records in memory order
//! (sequential reads and writes) while the packed sort gathers them at
//! random: runs are worth keeping even when shorter or when they interleave
//! unpredictably.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

use super::merge::{merge_asc_desc, merge_all, merge_all_half, merge_all_kway, merge_all_quad, merge_two};
use super::sort_encoded;

/// Shortest array tried for Verge on the elements.
pub(super) const VERGE_MIN: usize = 4096;

/// Exploration knobs (temporary): at most `n / RUN_DIV` segments' worth of
/// run length, runs of at least `RUN_MIN`, tried when `TURN_DIV * turns <=
/// n`, in the cache too when `IN_CACHE`.
pub(crate) static RUN_DIV: AtomicUsize = AtomicUsize::new(64);
pub(crate) static RUN_MIN: AtomicUsize = AtomicUsize::new(64);
pub(crate) static TURN_DIV: AtomicUsize = AtomicUsize::new(4);
pub(crate) static IN_CACHE: AtomicBool = AtomicBool::new(false);
pub(crate) static HALF: AtomicBool = AtomicBool::new(true);
pub(crate) static ORGAN: AtomicBool = AtomicBool::new(true);
/// Predictable merges of many segments: 2-way branchy rounds between the
/// array and a buffer (one move per level) instead of half a buffer.
pub(crate) static BRANCHY_ROUNDS: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACE: AtomicBool = AtomicBool::new(false);
/// Unpredictable merges of records of at most this size: four at a time.
pub(crate) static QUAD_MAX: AtomicUsize = AtomicUsize::new(0);
/// Unpredictable merges of records of at most this size: k-way rounds.
pub(crate) static KWAY_MAX: AtomicUsize = AtomicUsize::new(usize::MAX);
/// k-way rounds for predictable merges of more than two segments too.
pub(crate) static KWAY_BRANCHY: AtomicBool = AtomicBool::new(true);
/// Predictable merges of more than two segments of records of at least
/// this size: k-way rounds too.
pub(crate) static KWAY_BRANCHY_MIN: AtomicUsize = AtomicUsize::new(128);

/// Verge on the elements: runs of at least `max(n / RUN_DIV, RUN_MIN)`
/// keys (ascending, or strictly descending and then reversed: still
/// stable) are kept, the gaps between them sorted, and all the segments
/// merged stably. Returns false (nothing done) when the runs cover less
/// than half the array.
pub(super) fn verge<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    verge_if(arr, enc, false)
}

/// `verge`; with `unpredictable_only`, declined when the first two runs
/// would merge predictably (sawtooth: the packed sort is faster in the
/// cache).
pub(super) fn verge_if<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unpredictable_only: bool) -> bool {
    let t0 = std::time::Instant::now();
    let n = arr.len();
    let long = (n / RUN_DIV.load(Relaxed).max(1)).max(RUN_MIN.load(Relaxed)).max(2);
    // Runs: (start, end, descending).
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    let mut covered = 0;
    let (mut breaks, mut isolated) = (0usize, 0usize);
    let mut i = 0;
    while i + 1 < n {
        let start = i;
        let desc = enc(&arr[i + 1]) < enc(&arr[i]);
        let mut j = i + 1;
        if desc {
            while j < n && enc(&arr[j]) < enc(&arr[j - 1]) {
                j += 1;
            }
            if j < n {
                breaks += 1;
                let (a, b) = (enc(&arr[j - 1]), enc(&arr[j]));
                isolated += ((j >= 2 && b < enc(&arr[j - 2])) || (j + 1 < n && enc(&arr[j + 1]) < a)) as usize;
            }
        } else {
            while j < n && enc(&arr[j]) >= enc(&arr[j - 1]) {
                j += 1;
            }
            // A break of an ascending run: isolated when one of the two
            // keys is out of place alone (the order resumes around it).
            if j < n {
                breaks += 1;
                let (a, b) = (enc(&arr[j - 1]), enc(&arr[j]));
                isolated += ((j >= 2 && b >= enc(&arr[j - 2])) || (j + 1 < n && enc(&arr[j + 1]) >= a)) as usize;
            }
        }
        if j - start >= long {
            runs.push((start, j, desc));
            covered += j - start;
        }
        i = j;
    }
    if unpredictable_only && runs.len() >= 2 && predictable(arr, runs[0], runs[1], enc) {
        return false;
    }
    finish(arr, enc, runs, covered, breaks, isolated, t0)
}

/// Verge from the breaks of ascending order found by the first read
/// (positions `j` with `key[j] < key[j - 1]`, in order): the runs are the
/// stretches between them, no second read of the keys (only the
/// neighbours of each break, for the near-sorted test).
pub(super) fn verge_from_breaks<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, drops: &[usize]) -> bool {
    let t0 = std::time::Instant::now();
    let n = arr.len();
    let long = (n / RUN_DIV.load(Relaxed).max(1)).max(RUN_MIN.load(Relaxed)).max(2);
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    let mut covered = 0;
    let mut isolated = 0;
    let mut start = 0;
    for (k, &j) in drops.iter().chain(std::iter::once(&n)).enumerate() {
        if j - start >= long {
            runs.push((start, j, false));
            covered += j - start;
        }
        if k < drops.len() {
            let (a, b) = (enc(&arr[j - 1]), enc(&arr[j]));
            isolated += ((j >= 2 && b >= enc(&arr[j - 2])) || (j + 1 < n && enc(&arr[j + 1]) >= a)) as usize;
        }
        start = j;
    }
    finish(arr, enc, runs, covered, drops.len(), isolated, t0)
}

/// Verge from the turns found by the first read: positions `p` where the
/// pair (p - 1, p) is of another kind (break of ascending order or not)
/// than the pair before; the first stretch is descending when the list
/// starts at 1 (the first pair breaks). Element `p` starts a new run: a
/// stretch of breaks gives a strictly descending run, a stretch without an
/// ascending one; a lone break in ascending order (or the reverse) is
/// tested for the near-sorted rule.
pub(super) fn verge_from_turns<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, turn_at: &[usize]) -> bool {
    let t0 = std::time::Instant::now();
    let n = arr.len();
    let long = (n / RUN_DIV.load(Relaxed).max(1)).max(RUN_MIN.load(Relaxed)).max(2);
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    let (mut covered, mut breaks, mut isolated) = (0, 0, 0);
    // The first stretch: descending when the first pair breaks.
    let (mut start, mut desc, from) = if turn_at.first() == Some(&1) { (0, true, 1) } else { (0, false, 0) };
    let mut prev_len = 0;
    for &p in turn_at[from..].iter().chain(std::iter::once(&n)) {
        let len = p - start;
        // The run of a stretch starts one key early (the turn pair is of
        // its kind) and leaves its last key to the next one: a lone key
        // between two runs joins the next run instead of making a segment.
        let (rs, re) = (start.saturating_sub(1), if p < n { p - 1 } else { n });
        if re - rs >= long {
            runs.push((rs, re, desc));
            covered += re - rs;
        }
        // A stretch of one pair between two of the other kind: a lone break.
        if len == 1 && start > 0 && p < n && prev_len > 1 {
            breaks += 1;
            let j = start;
            let (a, b) = (enc(&arr[j - 1]), enc(&arr[j]));
            let lone = if desc {
                (j >= 2 && b >= enc(&arr[j - 2])) || (j + 1 < n && enc(&arr[j + 1]) >= a)
            } else {
                (j >= 2 && b < enc(&arr[j - 2])) || (j + 1 < n && enc(&arr[j + 1]) < a)
            };
            isolated += lone as usize;
        }
        prev_len = len;
        start = p;
        desc = !desc;
    }
    finish(arr, enc, runs, covered, breaks, isolated, t0)
}

/// The rest of Verge once the runs are known.
fn finish<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, runs: Vec<(usize, usize, bool)>, covered: usize, breaks: usize, isolated: usize, t0: std::time::Instant) -> bool {
    let n = arr.len();
    let t1 = t0.elapsed().as_nanos() as f64 / n as f64;
    // Nearly sorted (most breaks isolated): the packed sort with the
    // permutation applied in place moves almost nothing; merging the
    // pieces would move everything.
    if 2 * covered < n || (breaks >= 8 && 4 * isolated >= 3 * breaks) {
        return false;
    }
    // Pipe organ (one ascending run, then one strictly descending run to
    // the end): no reversal in place, the descending run copied reversed
    // into the buffer and merged from the back (1.5 moves instead of 2).
    if runs.len() == 2 && runs[0] == (0, runs[1].0, false) && runs[1].1 == n && runs[1].2 && ORGAN.load(Relaxed) {
        merge_asc_desc(arr, runs[1].0, enc);
        return true;
    }
    // Segments: the runs and the gaps between them, in order.
    let mut bounds = vec![0];
    for &(s, e, desc) in runs.iter() {
        let at = *bounds.last().unwrap();
        if s > at {
            sort_encoded(&mut arr[at..s], enc);
            bounds.push(s);
        }
        if desc {
            arr[s..e].reverse();
        }
        bounds.push(e);
    }
    let at = *bounds.last().unwrap();
    if at < n {
        sort_encoded(&mut arr[at..], enc);
        bounds.push(n);
    }
    let t2 = t0.elapsed().as_nanos() as f64 / n as f64;
    // Predictable merges (the first two segments, now ascending; the rule
    // of the keys' Verge): branches; else branch-free.
    let branchy = bounds.len() >= 3 && predictable(arr, (bounds[0], bounds[1], false), (bounds[1], bounds[2], false), enc);
    if bounds.len() == 3 {
        merge_two(arr, bounds[1], enc, branchy);
    } else if !branchy && super::merge::HALF_KWAY.load(Relaxed) {
        merge_all_half(arr, &bounds, enc, false);
    } else if (!branchy || std::mem::size_of::<T>() >= KWAY_BRANCHY_MIN.load(Relaxed)) && std::mem::size_of::<T>() <= KWAY_MAX.load(Relaxed) {
        merge_all_kway(arr, &bounds, enc, false);
    } else if BRANCHY_ROUNDS.load(Relaxed) && super::merge::KWAY2.load(Relaxed) {
        merge_all_kway(arr, &bounds, enc, true);
    } else if !branchy && std::mem::size_of::<T>() <= QUAD_MAX.load(Relaxed) {
        merge_all_quad(arr, &bounds, enc);
    } else if HALF.load(Relaxed) {
        merge_all_half(arr, &bounds, enc, branchy);
    } else {
        merge_all(arr, &bounds, enc, branchy);
    }
    if TRACE.load(Relaxed) {
        eprintln!("verge n {} runs {} segs {} branchy {} | detect {:.2} gaps {:.2} merge {:.2} ns", n, runs.len(), bounds.len() - 1, branchy, t1, t2 - t1, t0.elapsed().as_nanos() as f64 / n as f64 - t2);
    }
    true
}

/// Whether merging two runs would be predictable (same rule as
/// `predictable_merge`): on the first 64 steps of their merge (runs read in
/// increasing order), at most 16 switches between the two runs, or a
/// pattern periodic with a period of at most 16 (pipe organs alternate one
/// by one).
pub(super) fn predictable<T, E: Fn(&T) -> u128>(arr: &[T], a: (usize, usize, bool), b: (usize, usize, bool), enc: &E) -> bool {
    let at = |r: (usize, usize, bool), i: usize| if r.2 { enc(&arr[r.1 - 1 - i]) } else { enc(&arr[r.0 + i]) };
    let (la, lb) = (a.1 - a.0, b.1 - b.0);
    let (mut i, mut j, mut bits) = (0, 0, 0u64);
    for step in 0..64 {
        if i == la || j == lb {
            return true;
        }
        let left = at(a, i) <= at(b, j);
        bits |= (left as u64) << step;
        if left {
            i += 1;
        } else {
            j += 1;
        }
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}
