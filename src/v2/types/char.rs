//! char: the whole decision chain of this type, sorted as its code point
//! (the order of `char`; sorting permutes valid values only). Copy of the
//! shared integer chain of 2026-11-17 (as u32); edit here only.
//!
//! 1. at most `NETWORK_MAX` keys: a sorting network; up to `MERGE_MAX`:
//!    blocks and a merge;
//! 2. already sorted, reversed or constant arrays (one scan);
//! 3. up to `SMALL` keys: `sort_unstable`;
//! 4. the radix path (`path`), with the detectors (`detect`) and the engine
//!    of each chunk (`sort_chunk`).

use std::mem::MaybeUninit;

use crate::v2::cache::{budget, Budget};
use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
use crate::v2::decide::shape::{globally_ordered, locally_ordered};
use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
use crate::v2::engines::{lsd::wu32, roller::ru32};
use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs};
use crate::v2::runs::verge::verge_sort_preprocessing_fast;
use crate::v2::sorts::devour::keys::*;
use crate::v2::sorts::devour::DevourSort2;
use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
use crate::v2::special::plateaus::plateaus;
use crate::v2::special::{spread, DENSE_MIN, SAMPLE};

/// Up to this size: `sort_unstable` (in-process A/B, Snapdragon X Elite).
const SMALL: usize = 200;

/// The rollercoaster from this size (the L2 bytes; for its memory, n instead of 2n).
fn roller_min() -> usize { budget(Budget::Roller32) }

// The range, dense and few detectors of this type, in module `detectors`.
int_detectors!(detectors, u32, u32, id32, id32, super::roller_min);

/// Sorts `arr` as its code points (`u32`, same order).
/// Sorts `arr` as its code points (`u32`, same order).
pub(crate) fn sort(arr: &mut [char]) {
    // SAFETY: `char` has the size and alignment of `u32`, every `char` is a
    // valid `u32`, and the sorted slice holds the same values.
    let codes = unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut u32, arr.len()) };
    sort_codes(codes);
}

/// The chain on the code points: a sorting network up to `NETWORK_MAX`,
/// else the rest of the chain.
/// The chain on the code points: a sorting network up to `NETWORK_MAX`,
/// else the rest of the chain.
fn sort_codes(arr: &mut [u32]) {
    if arr.len() <= NETWORK_MAX {
        trace!("network");
        network_sort(bits_of(arr), id32, id32);
    } else {
        sort_large(arr);
    }
}

/// Arrays beyond the sorting network.
#[inline(never)]
fn sort_large(arr: &mut [u32]) {
    let n = arr.len();
    chain! {
        "merge_small" if n <= MERGE_MAX => do { merge_sort(bits_of(arr), id32, id32) };
    }
    let known = match presorted(arr) {
        Ok(()) => {
            trace!("presorted");
            return;
        },
        Err(known) => known,
    };
    chain! {
        "std_small" if n <= SMALL => do { arr.sort_unstable() };
    }
    let buffer = std::cell::RefCell::new(Vec::new());
    let sort_chunk = |chunk: &mut [u32]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
    path(arr, known, &detect, &sort_chunk);
}

/// The detectors, on one spread sample (sorted once, only when the detectors
/// that need it in order are reached). `equal_pairs`: equal pairs among the
/// probes of the near-sorted repair.
fn detect(arr: &mut [u32], equal_pairs: usize) -> bool {
    let mut sample = spread(arr);
    chain! {
        return true;
        "range" => try detectors::range(arr, &sample);
        "plateaus" => try plateaus(arr, equal_pairs);
        // One dominant value among spread ones, short array: the std
        // partition settles the keys equal to its pivot in one pass.
        "dominant" if arr.len() < DENSE_MIN
            && dominant(&sample, |x, y| x == y)
            => do { arr.sort_unstable() };
        "few" => try detectors::few(arr, &sample);
    }
    // Heavy keys and dense ranges: the sample in order.
    if arr.len() < HEAVY_MIN || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y)) {
        return false;
    }
    sample.sort_unstable();
    chain! {
        return true;
        "heavy" => try heavy_sorted_by(
            arr,
            &sample,
            HEAVY_MIN,
            &|r: &mut [u32]| sort_codes(r),
            |x, y| x == y,
            |x, y| x < y,
        );
        "dense" if arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
    }
    false
}

/// The engine of one chunk (the whole array, or a gap between Verge's runs).
#[inline(always)]
fn sort_chunk(bits: &mut [u32], _: &mut Vec<MaybeUninit<u32>>) {
    // The rollercoaster beyond the cache only on ordered keys (the MSD pass
    // then writes in few sequential streams; random keys go 15 to 45 %
    // faster through the LSD from 30M to 1G).
    if bits.len() >= roller_min() && (locally_ordered(bits) || globally_ordered(bits)) {
        trace!("roller");
        ru32::roller(bits);
    } else {
        trace!("lsd");
        wu32::lsd(bits);
    }
}

/// The radix path (copy of `decide::path` of 2026-11-17); the first matching
/// rule wins.
fn path(
    arr: &mut [u32],
    known: Unsorted,
    detect: &dyn Fn(&mut [u32], usize) -> bool,
    sort: &dyn Fn(&mut [u32]),
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
    let simd_rule = merge::simd_rule_applies::<u32>(n);
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
        verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [u32], _| {
            let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<u32>();
            chunks.borrow_mut().push((start, chunk.len()));
        });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
    let sort_chunks = |arr: &mut [u32]| {
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
