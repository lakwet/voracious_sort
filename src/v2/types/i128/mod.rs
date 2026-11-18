//! i128: the whole decision chain of this type (copy of the shared 128-bit
//! chain of 2026-11-17; edit here only, see `types`). Narrowing, few values
//! and the 128-bit engines: `sorts::devour::wide128`.
//!
//! 1. up to `SMALL` keys: `sort_unstable`;
//! 2. already sorted, reversed or constant arrays (one scan);
//! 3. keys narrowed to 64 or 32 bits when they fit;
//! 4. the radix path (`path`): few values, heavy hitters, then the DLSD on
//!    128-bit keys, the rollercoaster above the cache.

use crate::v2::cache::{budget, Budget};
use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
use crate::v2::dev::ab::new_rule;
mod presorted;
use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs};
use crate::v2::runs::verge::verge_sort_preprocessing_fast;
use crate::v2::sorts::devour::wide128::{bits_of, few, narrow, Key128};
use crate::v2::special::heavy::heavy_from_spread;
use crate::v2::special::spread;

/// Up to this size: `sort_unstable`.
const SMALL: usize = 64;
/// From this size the heavy hitters are extracted (4096 for narrower keys:
/// 16-byte keys cost more to compare and move, the extraction pays off on
/// shorter arrays; Sparse 90 % at 1k-3k: 6.6-7.7 ns per key without it).
const HEAVY_MIN: usize = 512;

/// The rollercoaster from this size (L2 / 24: keys of 16 bytes, as the
/// 64-bit threshold L2 / 12).
fn roller_min() -> usize { budget(Budget::Roller128) / 24 }

/// Sorts `arr`: `sort_unstable` up to `SMALL` keys, the presorted check,
/// the narrowing to 64 or 32 bits when the range allows, else the radix
/// path (few values, heavy keys; DLSD, or the rollercoaster beyond the
/// cache).
pub(crate) fn sort(arr: &mut [i128]) {
    let n = arr.len();
    chain! {
        "std_small" if n <= SMALL => do { arr.sort_unstable() };
    }
    let checked = if new_rule() { self::presorted::presorted(arr) } else { presorted(arr) };
    let known = match checked {
        Ok(()) => {
            trace!("presorted");
            return;
        },
        Err(known) => known,
    };
    // A long sorted prefix goes to the tail-first path without the
    // narrowing scan.
    chain! {
        "narrow128" if !tail_first(n, known.ascending_prefix) => try narrow(arr);
    }
    path(
        arr,
        known,
        &|a: &mut [i128], _| {
            // One spread sample for both (heavy reads it from `HEAVY_MIN`).
            let sample = (a.len() >= HEAVY_MIN).then(|| spread(a));
            chain! {
                return true;
                "few" => try few(a, sample.as_ref());
                "heavy" if sample.is_some() => try heavy_from_spread(
                    a,
                    sample.unwrap(),
                    HEAVY_MIN,
                    &|r: &mut [i128]| sort(r),
                    |x, y| x == y,
                    |x, y| x < y,
                );
            }
            false
        },
        &|chunk: &mut [i128]| {
            if chunk.len() >= roller_min() {
                trace!("roller");
                <i128 as Key128>::roller(bits_of(chunk));
            } else {
                trace!("dlsd");
                <i128 as Key128>::dlsd(bits_of(chunk));
            }
        },
    );
}

/// The radix path (copy of `decide::path` of 2026-11-17); the first matching
/// rule wins.
fn path(
    arr: &mut [i128],
    known: Unsorted,
    detect: &dyn Fn(&mut [i128], usize) -> bool,
    sort: &dyn Fn(&mut [i128]),
) {
    let n = arr.len();
    let p = known.ascending_prefix;
    chain! {
        // A sorted prefix of at least half the array: the tail alone goes
        // through the path, then one merge. A short tail: sorted (or
        // reversed) as it is, else the engine.
        "tail_first" if tail_first(n, p) => do {
            let tail = &mut arr[p..];
            if tail.len() >= TAIL_PATH_MIN {
                path(tail, Unsorted::default(), detect, sort);
            } else if presorted(tail).is_err() {
                sort(tail);
            }
            merge_runs(arr, &mut vec![0, p, n]);
        };
    }
    let simd_rule = merge::simd_rule_applies::<i128>(n);
    let random_runs = !simd_rule && starts_with_random_runs(arr, p);
    let (sorted, equal_pairs) = if random_runs {
        (false, known.probes.map_or(0, |p| p.equal()))
    } else {
        repair_nearly_sorted(arr, known)
    };
    if random_runs {
        trace!("random_runs");
    }
    chain! {
        "repaired" if sorted => do {};
        _ => try detect(arr, equal_pairs);
        "no_verge" if random_runs || can_skip_verge(arr) => do { sort(arr) };
    }
    // Verge: the unsorted chunks are recorded, sorted after the merge rule.
    let base = arr.as_ptr() as usize;
    let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    let mut separators =
        verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [i128], _| {
            let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<i128>();
            chunks.borrow_mut().push((start, chunk.len()));
        });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
    let sort_chunks = |arr: &mut [i128]| {
        for &(start, len) in chunks.iter() {
            sort(&mut arr[start..start + len]);
        }
    };
    chain! {
        "verge_whole" if unpredictable && !simd_merge => do { sort(arr) };
        "verge_simd_merge" if unpredictable => do {
            sort_chunks(arr);
            merge::merge_runs_simd(arr, &mut separators);
        };
        "verge_merge" => do {
            sort_chunks(arr);
            merge_runs(arr, &mut separators);
        };
    }
}

/// Tails from this length go through the whole path (repair, detectors);
/// shorter ones go to the engine.
const TAIL_PATH_MIN: usize = 4096;

/// A sorted prefix of `p` keys out of `n`: the tail-first path.
fn tail_first(n: usize, p: usize) -> bool { p >= n / 2 && p < n }
