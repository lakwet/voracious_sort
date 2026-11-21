//! Sorting records (structs) by a key, stably or not.
//!
//! Each key is encoded into an unsigned integer of the same order
//! (`SortKey`). One read of the keys gives their range, whether they are
//! already in order and, beyond the cache, where the order breaks and how
//! often it changes direction. The chains of rules, one per key type and
//! record size, with their thresholds: `types::records`. Here: the public
//! entries and the pieces the chains share. Keys too wide for a u128 with
//! their index fall back to `sort_by`.

pub(crate) mod few;
pub(crate) mod heavy;

pub(crate) mod lsd;
mod merge;
pub(crate) mod packed;
mod permute;
pub(crate) mod small;
pub(crate) mod verge;

mod keys;

pub use keys::SortKey;

use std::mem::MaybeUninit;

#[allow(unused_imports)] // the CPU bench sets it
pub(crate) use merge::FOUR_WAY_MIN;
use packed::sort_into;
/// The 11-bit LSD of the large-record chains (the CPU bench turns it off
/// to compare).
pub(crate) static LSD11: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Sorts `arr` by `key`, stably.
pub fn devour_sort_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    K::sort_records(arr, &|x: &T| key(x).encode(), false);
}

/// Sorts `arr` by `key`, unstably: equal keys may come out in any order.
/// The stable sort's path, where the freedom pays: the run merges by
/// vector blocks (`simd`), large records in disorder by the in-place MSD
/// (`unstable`: no buffer).
pub fn devour_sort_unstable_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    K::sort_records(arr, &|x: &T| key(x).encode(), true);
}

std::thread_local! {
    /// Set while this thread runs the unstable sort.
    static UNSTABLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the running sort may reorder equal keys.
#[inline]
pub(crate) fn unstable() -> bool { UNSTABLE.with(|u| u.get()) }

/// Sets the unstable mode for its lifetime (restored even on a panic).
pub(crate) struct Unstable(bool);

impl Unstable {
    /// Marks the current sort unstable until dropped (restores the previous
    /// state).
    pub(crate) fn enter() -> Self { Unstable(UNSTABLE.with(|u| u.replace(true))) }
}

impl Drop for Unstable {
    fn drop(&mut self) {
        let was = self.0;
        UNSTABLE.with(|u| u.set(was));
    }
}

/// Ascending order broken only locally (the stable passes repair it for
/// little; the MSD would move every record): few breaks, most of them a
/// single key out of place (the keys around it in order: random swaps in
/// sorted data, not the long runs of a sawtooth), or keys in order at a
/// distance of 64 on a sample (local disorder: timestamps with jitter).
pub(crate) fn nearly_sorted<T, E: Fn(&T) -> u128>(
    arr: &[T],
    enc: &E,
    s: &Scan,
    drops: &[usize],
) -> bool {
    let n = arr.len();
    if s.drops <= DROPS_MAX {
        let k = |i: usize| enc(&arr[i]);
        let isolated = drops[..s.drops]
            .iter()
            .filter(|&&j| {
                (j + 1 < n && j >= 1 && k(j + 1) >= k(j - 1)) || (j >= 2 && k(j) >= k(j - 2))
            })
            .count();
        return 2 * isolated >= s.drops;
    }
    // The distance between the two keys compared in each sample.
    const W: usize = 64;
    let samples = 256.min(n / W);
    (0..samples).all(|i| {
        let a = i * (n - W) / samples;
        enc(&arr[a]) <= enc(&arr[a + W])
    })
}

/// What one read of the keys learned.
/// Most breaks of ascending order the first read keeps for Verge.
pub(crate) const DROPS_MAX: usize = 1024;

/// What one read of the keys learned: their range, whether they are in
/// order, the turns and the breaks of ascending order.
pub(crate) struct Scan {
    pub(crate) lo: u128,
    pub(crate) hi: u128,
    pub(crate) ascending: bool,
    pub(crate) strictly_descending: bool,
    /// Changes of direction between neighbouring pairs (equal keys keep the
    /// direction): about 2n/3 on random keys, one per tooth on sawtooth
    /// data, one on a pipe organ. Counted only if `TURNS`.
    pub(crate) turns: usize,
    /// Breaks of ascending order (`key[j] < key[j - 1]`).
    pub(crate) drops: usize,
}

/// Reads every key once: range, breaks of ascending order (`down`), and
/// with `TURNS` the turns (changes between a break and a non-break from one
/// pair to the next: about n/2 on random keys, two per break on runs) and
/// the positions of the first `DROPS_MAX` breaks. Two elements per step,
/// two ranges.
#[inline(always)]
pub(crate) fn scan<T, E: Fn(&T) -> u128, const TURNS: bool>(
    arr: &[T],
    enc: &E,
    drops: &mut [usize],
    turn_at: &mut [usize],
    start: (usize, bool),
) -> Scan {
    let n = arr.len();
    let first = enc(&arr[0]);
    let (mut lo, mut lo2, mut hi2) = (first, first, first);
    // `last`: kind of the previous pair (the first pair counts as a turn
    // when it breaks: turns then start with a descending stretch).
    let (mut down, mut turns, mut last) = (0usize, 0usize, false);
    let mut prev = first;
    let mut i = 1;
    // `start`: the run at the start already read (by `ends`): ascending of
    // that length, or descending (shorter than the array).
    let (known, known_desc) = start;
    if !known_desc && known > 1 {
        i = known;
        prev = enc(&arr[i - 1]);
    }
    // Ascending prefix first (one comparison per key: sorted arrays read
    // as fast as by an is-sorted check); the scan goes on from its end.
    while i < n {
        let k = enc(&arr[i]);
        if k < prev {
            break;
        }
        prev = k;
        i += 1;
    }
    // The prefix's largest key; `lo` stays its first.
    let mut hi = prev;
    if i == n {
        return Scan { lo, hi, ascending: true, strictly_descending: n == 1, turns: 0, drops: 0 };
    }
    // Strictly descending from the start: the same, for reversed arrays.
    if i == 1 && !known_desc {
        let mut j = 1;
        let mut p = first;
        while j < n {
            let k = enc(&arr[j]);
            if k >= p {
                break;
            }
            p = k;
            j += 1;
        }
        if j == n {
            return Scan {
                lo: p,
                hi: first,
                ascending: false,
                strictly_descending: true,
                turns: 1,
                drops: n - 1,
            };
        }
    }
    while i + 1 < n {
        let (ka, kb) = (enc(&arr[i]), enc(&arr[i + 1]));
        lo = lo.min(ka);
        hi = hi.max(ka);
        lo2 = lo2.min(kb);
        hi2 = hi2.max(kb);
        let (da, db) = (ka < prev, kb < ka);
        if TURNS {
            // Stored without a branch: random keys break half the time.
            // SAFETY: `drops` holds DROPS_MAX + 1 slots.
            unsafe { *drops.get_unchecked_mut(down.min(DROPS_MAX)) = i };
            let c = down + da as usize;
            unsafe { *drops.get_unchecked_mut(c.min(DROPS_MAX)) = i + 1 };
            // Turns: pair `i` (keys i - 1, i) of another kind than pair
            // `i - 1`; the first `DROPS_MAX` positions kept.
            // SAFETY: `turn_at` holds DROPS_MAX + 1 slots.
            unsafe { *turn_at.get_unchecked_mut(turns.min(DROPS_MAX)) = i };
            let t = turns + (da != last) as usize;
            unsafe { *turn_at.get_unchecked_mut(t.min(DROPS_MAX)) = i + 1 };
            turns = t + (db != da) as usize;
            last = db;
        }
        down += da as usize + db as usize;
        prev = kb;
        i += 2;
    }
    if i < n {
        let ka = enc(&arr[i]);
        lo = lo.min(ka);
        hi = hi.max(ka);
        let da = ka < prev;
        if TURNS {
            // SAFETY: as above.
            unsafe { *drops.get_unchecked_mut(down.min(DROPS_MAX)) = i };
            unsafe { *turn_at.get_unchecked_mut(turns.min(DROPS_MAX)) = i };
            turns += (da != last) as usize;
        }
        down += da as usize;
    }
    Scan {
        lo: lo.min(lo2),
        hi: hi.max(hi2),
        ascending: down == 0,
        strictly_descending: down == n - 1,
        turns,
        drops: down,
    }
}

/// Distinct keys among 512 spread over the array: 500 or more when the
/// array likely holds far more than 4096 distinct keys (about 481 expected
/// for 4096 random keys, at most the number of keys when fewer).
pub(crate) fn sample_distinct<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E, sample: &Sample) -> usize {
    if let Some(d) = sample.distinct.get() {
        return d;
    }
    let mut keys = sample.first(arr, enc, 512);
    keys.sort_unstable();
    keys.dedup();
    sample.distinct.set(Some(keys.len()));
    keys.len()
}

/// Encoded keys at the positions frac(i x golden ratio) x n, read as the
/// tries of the chain ask for them, once per array: `few_keys` reads the
/// first up to 256 (stopping early), `heavy_keys` the first 256,
/// `sample_distinct` 512. Valid while nothing moved: the tries give up
/// before moving anything.
pub(crate) struct Sample {
    keys: std::cell::RefCell<Vec<u128>>,
    distinct: std::cell::Cell<Option<usize>>,
}

impl Sample {
    pub(crate) fn new() -> Self {
        Sample { keys: std::cell::RefCell::new(Vec::new()), distinct: std::cell::Cell::new(None) }
    }

    /// Key `i` of the sample.
    #[inline]
    pub(crate) fn at<T, E: Fn(&T) -> u128>(&self, arr: &[T], enc: &E, i: usize) -> u128 {
        let mut keys = self.keys.borrow_mut();
        let n = arr.len() as u64;
        while keys.len() <= i {
            let j = keys.len() as u64;
            let pos = (((j.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n) >> 32) as usize;
            keys.push(enc(&arr[pos]));
        }
        keys[i]
    }

    /// A copy of the first `k` keys.
    pub(crate) fn first<T, E: Fn(&T) -> u128>(&self, arr: &[T], enc: &E, k: usize) -> Vec<u128> {
        if k > 0 {
            self.at(arr, enc, k - 1);
        }
        self.keys.borrow()[..k].to_vec()
    }
}

/// 32 pairs of neighbours spread over the array all in order: long runs
/// likely (random keys pass with odds 2^-32).
pub(crate) fn locally_sorted<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E) -> bool {
    let n = arr.len();
    (0..32).all(|i| {
        let j = (2 * i + 1) * (n - 1) / 64;
        enc(&arr[j]) <= enc(&arr[j + 1])
    })
}

/// Length of the run at the start: ascending, or strictly descending
/// (reversible stably); `true` when descending.
fn run_at_start<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E) -> (usize, bool) {
    let n = arr.len();
    let mut prev = enc(&arr[0]);
    let mut i = 1;
    let desc = enc(&arr[1]) < prev;
    if desc {
        while i < n {
            let k = enc(&arr[i]);
            if k >= prev {
                break;
            }
            prev = k;
            i += 1;
        }
    } else {
        while i < n {
            let k = enc(&arr[i]);
            if k < prev {
                break;
            }
            prev = k;
            i += 1;
        }
    }
    (i, desc)
}

/// Start of the run at the end (not below `floor`): ascending, or strictly
/// descending; `true` when descending (and longer than one key).
fn run_at_end<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E, floor: usize) -> (usize, bool) {
    let n = arr.len();
    let mut s = n - 1;
    let mut next = enc(&arr[s]);
    let desc = n - 1 > floor && next < enc(&arr[n - 2]);
    while s > floor {
        let k = enc(&arr[s - 1]);
        if desc != (next < k) || (desc && next == k) {
            break;
        }
        next = k;
        s -= 1;
    }
    (s, desc && s < n - 1)
}

/// Runs at both ends (sorted data with keys out of place at the start, the
/// end or one spot, a random tail, a pipe organ): when the run at the start
/// and the run at the end cover at least 3/4 of the array, only the rest is
/// sorted, then merged with them (the merges move the shorter side and the
/// keys out of place). Sorted and strictly descending arrays end here too.
/// `sort`: the chain of the caller, for the middle.
pub(crate) fn ends<T, E: Fn(&T) -> u128>(
    arr: &mut [T],
    enc: &E,
    start: &mut (usize, bool),
    sort: &dyn Fn(&mut [T]),
) -> bool {
    let n = arr.len();
    let (p, pdesc) = run_at_start(arr, enc);
    *start = (p, pdesc);
    if p == n {
        if pdesc {
            arr.reverse();
        }
        return true;
    }
    let (s, sdesc) = run_at_end(arr, enc, p);
    if 4 * (p + n - s) < 3 * n {
        return false;
    }
    // A short run at one end joins the middle (merging it apart would move
    // the whole array once more).
    let (p, pdesc) = if 8 * p < n { (0, false) } else { (p, pdesc) };
    let (s, sdesc) = if 8 * (n - s) < n { (n, false) } else { (s, sdesc) };
    // Pipe organ: no middle, ascending then descending.
    if s == p && !pdesc && sdesc {
        merge::merge_asc_desc(arr, p, enc);
        return true;
    }
    if pdesc {
        arr[..p].reverse();
    }
    if sdesc {
        arr[s..].reverse();
    }
    if s > p {
        sort(&mut arr[p..s]);
        if s < n {
            let b = verge::predictable(arr, (p, s, false), (s, n, false), enc);
            merge::merge_two(&mut arr[p..], s - p, enc, b);
        }
    }
    if p > 0 {
        let b = verge::predictable(arr, (0, p, false), (p, n, false), enc);
        merge::merge_two(arr, p, enc, b);
    }
    true
}

/// The packed (key, index) sort through a buffer.
pub(crate) fn packed_buffer<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, lo: u128, key_bits: u32) {
    let n = arr.len();
    let mut buffer: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the records are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buffer.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, enc, lo, key_bits) {
            std_fallback(arr, enc);
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}

/// The packed sort declined (keys too wide): the standard library's.
pub(crate) fn std_fallback<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    trace!("std_fallback");
    arr.sort_by_key(|x| enc(x));
}

#[cfg(test)] mod tests;
