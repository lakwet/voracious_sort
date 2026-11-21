//! u8: the whole decision chain of this type (copy of the shared integer
//! chain; edit here only, see `types`).
//!
//! 1. at most `NETWORK_MAX` keys: a sorting network; up to `MERGE_MAX`:
//!    blocks and a merge;
//! 2. up to 2^32 keys: the order check that writes nothing unless the
//!    array is reversed, then the byte counting sort (`engines::bytes`);
//! 3. beyond: the radix path (`path`), with the detectors (`detect`) and the engine
//!    of each chunk (`sort_chunk`).

use std::mem::MaybeUninit;

use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
use crate::v2::engines::bytes;
use crate::v2::engines::lsd::wu8;
use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs};
use crate::v2::runs::verge::verge_sort_preprocessing_fast;
use crate::v2::sorts::devour::keys::*;
use crate::v2::sorts::devour::DevourSort;
use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
use crate::v2::special::plateaus::plateaus;
use crate::v2::special::{spread, DENSE_MIN, SAMPLE};

/// The rollercoaster from this size (never: 8 and 16 bits).
fn roller_min() -> usize { usize::MAX }

// The range, dense and few detectors of this type, in module `detectors`.
int_detectors!(detectors, u8, u8, id8, id8, super::roller_min);

/// Sorts `arr`: a sorting network up to `NETWORK_MAX` keys, else the chain.
pub(crate) fn sort(arr: &mut [u8]) {
    if arr.len() <= NETWORK_MAX {
        trace!("network");
        network_sort(bits_of(arr), id8, id8);
    } else {
        sort_large(arr);
    }
}

/// Arrays beyond the sorting network.
#[inline(never)]
fn sort_large(arr: &mut [u8]) {
    let n = arr.len();
    chain! {
        "merge_small" if n <= MERGE_MAX => do { merge_sort(bits_of(arr), id8, id8) };
    }
    // Beyond the small merge: the order check that writes nothing unless
    // reversed, then the byte counting sort (`engines::bytes`, u32 counters).
    if n <= u32::MAX as usize {
        if bytes::sorted_or_reversed(arr) {
            trace!("presorted");
        } else {
            trace!("count8");
            bytes::count::<0>(bits_of(arr));
        }
        return;
    }
    let known = match presorted(arr) {
        Ok(()) => {
            trace!("presorted");
            return;
        },
        Err(known) => known,
    };
    // Beyond 2^32 keys (counters too narrow): the radix path.
    let buffer = std::cell::RefCell::new(Vec::new());
    let sort_chunk = |chunk: &mut [u8]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
    path(arr, known, &detect, &sort_chunk);
}

/// The detectors, on one spread sample (sorted once, only when the detectors
/// that need it in order are reached). `equal_pairs`: equal pairs among the
/// probes of the near-sorted repair.
fn detect(arr: &mut [u8], equal_pairs: usize) -> bool {
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
            &|r: &mut [u8]| sort(r),
            |x, y| x == y,
            |x, y| x < y,
        );
        "dense" if arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
    }
    false
}

/// The engine of one chunk (the whole array, or a gap between Verge's runs).
#[inline(always)]
fn sort_chunk(bits: &mut [u8], _: &mut Vec<MaybeUninit<u8>>) {
    // 8 and 16 bits: the fused LSD (two passes at most), always.
    trace!("lsd");
    wu8::lsd(bits);
}

/// The radix path (copy of `decide::path`); the first matching
/// rule wins.
fn path(
    arr: &mut [u8],
    known: Unsorted,
    detect: &dyn Fn(&mut [u8], usize) -> bool,
    sort: &dyn Fn(&mut [u8]),
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
    let simd_rule = merge::simd_rule_applies::<u8>(n);
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
        verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [u8], _| {
            let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<u8>();
            chunks.borrow_mut().push((start, chunk.len()));
        });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
    let sort_chunks = |arr: &mut [u8]| {
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
