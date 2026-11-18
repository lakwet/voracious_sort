//! f64: the whole decision chain of this type (copy of the shared float
//! chain of 2026-11-17; edit here only, see `types`). Sorted through the
//! encoded bits (`sorts::ravenous`: unsigned order = `total_cmp` order).
//!
//! 1. at most `NETWORK_MAX` keys: a sorting network;
//! 2. NaN set aside (negative first, positive last, by their bits);
//! 3. below `PRESORTED_FIRST_MIN` keys: the float detectors, then up to
//!    `SMALL` keys `sort_unstable` on the encoded keys;
//! 4. else the presorted check first;
//! 5. the radix path (`path`), with the detectors and the engine of each
//!    chunk; -0.0 before +0.0 at the end.

use crate::v2::cache::{budget, Budget};
use crate::v2::decide::path::{can_skip_verge, merges_unpredictable};
use crate::v2::engines::small::{network_sort, NETWORK_MAX};
use crate::v2::engines::{dlsd::d64, roller::r64};
use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs};
use crate::v2::runs::verge::verge_sort_preprocessing_fast;
use crate::v2::sorts::ravenous::f64_bits;
use crate::v2::special::floats::{detect64, few64};
use crate::v2::special::heavy::{dominant, heavy_from_spread, HEAVY_MIN};
use crate::v2::special::{spread, DENSE_MIN};

/// Up to this size: `sort_unstable` on the encoded keys (where it crosses
/// the radix path, median over the distributions, Snapdragon X Elite).
const SMALL: usize = 700;

/// From this size the presorted check and the repair of nearly sorted
/// arrays come before the float detectors (f32 timestamps, a few distinct
/// values nearly in order: 0.67 to 0.15-0.27 ns per key at 10k-3M). Below,
/// the detectors first (1k: sparse 2.8 against 3.6).
const PRESORTED_FIRST_MIN: usize = 4096;

/// The rollercoaster from this size (L2 / 48, where the DLSD's keys and
/// buffer take a third of the L2).
fn roller_min() -> usize { budget(Budget::RollerF64) / 48 }

/// Sorts `arr` in the total order (`total_cmp`): NaNs set aside at the
/// ends first, then a sorting network up to `NETWORK_MAX` keys, else the
/// chain.
/// Sorts `arr` in the total order (`total_cmp`): NaNs set aside at the
/// ends first, then a sorting network up to `NETWORK_MAX` keys, else the
/// chain.
pub(crate) fn sort(arr: &mut [f64]) {
    if arr.len() <= NETWORK_MAX {
        trace!("network");
        network_sort(f64_bits::bits(arr), f64_bits::encode, f64_bits::decode);
    } else {
        let rest = f64_bits::nan_aside(arr);
        if rest.len() <= NETWORK_MAX {
            network_sort(f64_bits::bits(rest), f64_bits::encode, f64_bits::decode);
        } else {
            sort_large(rest);
        }
    }
}

/// Arrays beyond the network (no NaN): detectors, small path, radix path.
#[inline(never)]
fn sort_large(arr: &mut [f64]) {
    let presorted_first = arr.len() >= PRESORTED_FIRST_MIN;
    chain! {
        "float_detect" if !presorted_first => try detect64(arr);
        "std_small" if !presorted_first && arr.len() <= SMALL
            => do { f64_bits::sort_small(f64_bits::bits(arr)) };
    }
    // As for integers: the presorted check first, then the repair of nearly
    // sorted arrays, and only then the detectors (in `detect`).
    let known = if presorted_first {
        match presorted(arr) {
            Ok(()) => {
                trace!("presorted");
                f64_bits::order_signed_zeros(arr);
                return;
            },
            Err(known) => known,
        }
    } else {
        Unsorted::default()
    };
    let buffer = std::cell::RefCell::new(Vec::new());
    let detect = |a: &mut [f64], _| {
        chain! {
            return true;
            "float_detect" if presorted_first => try detect64(a);
            // One dominant value among spread ones, short array: the std
            // partition settles the keys equal to its pivot in one pass.
            "dominant" if a.len() < DENSE_MIN
                && dominant(&spread(a), f64_bits::same)
                => do { f64_bits::sort_small(f64_bits::bits(a)) };
        }
        // One spread sample for both (heavy reads it from `HEAVY_MIN`).
        let sample = (a.len() >= HEAVY_MIN).then(|| spread(a));
        chain! {
            return true;
            "few" => try few64(a, sample.as_ref());
            "heavy" if sample.is_some() => try heavy_from_spread(
                a,
                sample.unwrap(),
                HEAVY_MIN,
                &|r: &mut [f64]| sort(r),
                f64_bits::same,
                f64_bits::less,
            );
        }
        false
    };
    let sort_chunk = |chunk: &mut [f64]| {
        if chunk.len() >= roller_min() {
            trace!("roller");
            r64::roller(f64_bits::bits(chunk));
        } else {
            trace!("dlsd");
            d64::dlsd_with(f64_bits::bits(chunk), &mut buffer.borrow_mut());
        }
    };
    path(arr, known, &detect, &sort_chunk);
    f64_bits::order_signed_zeros(arr);
}

/// The radix path (copy of `decide::path` of 2026-11-17); the first matching
/// rule wins.
fn path(
    arr: &mut [f64],
    known: Unsorted,
    detect: &dyn Fn(&mut [f64], usize) -> bool,
    sort: &dyn Fn(&mut [f64]),
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
    let simd_rule = merge::simd_rule_applies::<f64>(n);
    let random_runs = false;
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
        verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [f64], _| {
            let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<f64>();
            chunks.borrow_mut().push((start, chunk.len()));
        });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
    let sort_chunks = |arr: &mut [f64]| {
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
