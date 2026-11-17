//! Sorting records (structs) by a key, stably or not.
//!
//! Each key is encoded into an unsigned integer of the same order
//! (`SortKey`). One read of the keys gives their range, whether they are
//! already in order and, beyond the cache, where the order breaks and how
//! often it changes direction. The chain of rules, and its thresholds:
//! `decide::records`. Keys too wide for a u128 with their index fall back
//! to `sort_by`.

mod few;
mod heavy;

mod lsd;
mod merge;
mod packed;
mod permute;
mod small;
mod verge;
mod unstable;

mod keys;

pub use keys::SortKey;

use crate::v2::dev::trace::hit;
use std::mem::MaybeUninit;

use few::few_keys;
use lsd::lsd;
use packed::{sort_in_place, sort_into};
use crate::v2::decide::records::*;
use verge::verge;
pub(crate) use merge::FOUR_WAY_MIN;
pub(crate) static LSD11: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Sorts `arr` by `key`, stably.
pub fn devour_sort_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    sort_encoded(arr, &|x: &T| key(x).encode());
}

/// Sorts `arr` by `key`, unstably: equal keys may come out in any order.
/// The stable sort's path, where the freedom pays: the run merges by
/// vector blocks (`simd`), large records in disorder by the in-place MSD
/// (`unstable`: no buffer).
pub fn devour_sort_unstable_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    let enc = |x: &T| key(x).encode();
    // Few large records: the standard library's pattern-defeating quicksort
    // (in place: no buffer to fault in, presorted data found as it goes).
    let size = std::mem::size_of::<T>();
    if size >= UNSTABLE_MSD_MIN && arr.len() < UNSTABLE_PDQ_MAX {
        trace!("std_unstable_large");
        arr.sort_unstable_by_key(|x| enc(x));
        return;
    }
    let _unstable = Unstable::enter();
    sort_encoded(arr, &enc);
}


std::thread_local! {
    /// Set while this thread runs the unstable sort.
    static UNSTABLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the running sort may reorder equal keys.
#[inline]
pub(super) fn unstable() -> bool {
    UNSTABLE.with(|u| u.get())
}

/// Sets the unstable mode for its lifetime (restored even on a panic).
struct Unstable(bool);

impl Unstable {
    fn enter() -> Self {
        Unstable(UNSTABLE.with(|u| u.replace(true)))
    }
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
fn nearly_sorted<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E, s: &Scan, drops: &[usize]) -> bool {
    let n = arr.len();
    if s.drops <= DROPS_MAX {
        let k = |i: usize| enc(&arr[i]);
        let isolated = drops[..s.drops]
            .iter()
            .filter(|&&j| (j + 1 < n && j >= 1 && k(j + 1) >= k(j - 1)) || (j >= 2 && k(j) >= k(j - 2)))
            .count();
        return 2 * isolated >= s.drops;
    }
    const W: usize = 64;
    let samples = 256.min(n / W);
    (0..samples).all(|i| {
        let a = i * (n - W) / samples;
        enc(&arr[a]) <= enc(&arr[a + W])
    })
}


/// What one read of the keys learned.
/// Most breaks of ascending order the first read keeps for Verge.
const DROPS_MAX: usize = 1024;

struct Scan {
    lo: u128,
    hi: u128,
    ascending: bool,
    strictly_descending: bool,
    /// Changes of direction between neighbouring pairs (equal keys keep the
    /// direction): about 2n/3 on random keys, one per tooth on sawtooth
    /// data, one on a pipe organ. Counted only if `TURNS`.
    turns: usize,
    /// Breaks of ascending order (`key[j] < key[j - 1]`).
    drops: usize,
}

/// Reads every key once: range, breaks of ascending order (`down`), and
/// with `TURNS` the turns (changes between a break and a non-break from one
/// pair to the next: about n/2 on random keys, two per break on runs) and
/// the positions of the first `DROPS_MAX` breaks. Two elements per step,
/// two ranges.
#[inline(always)]
fn scan<T, E: Fn(&T) -> u128, const TURNS: bool>(arr: &[T], enc: &E, drops: &mut [usize], turn_at: &mut [usize]) -> Scan {
    let n = arr.len();
    let first = enc(&arr[0]);
    let (mut lo, mut hi, mut lo2, mut hi2) = (first, first, first, first);
    // `last`: kind of the previous pair (the first pair counts as a turn
    // when it breaks: turns then start with a descending stretch).
    let (mut down, mut turns, mut last) = (0usize, 0usize, false);
    let mut prev = first;
    let mut i = 1;
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
    hi = prev;
    if i == n {
        return Scan { lo, hi, ascending: true, strictly_descending: n == 1, turns: 0, drops: 0 };
    }
    // Strictly descending from the start: the same, for reversed arrays.
    if i == 1 {
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
            return Scan { lo: p, hi: first, ascending: false, strictly_descending: true, turns: 1, drops: n - 1 };
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
    Scan { lo: lo.min(lo2), hi: hi.max(hi2), ascending: down == 0, strictly_descending: down == n - 1, turns, drops: down }
}

/// Distinct keys among 512 spread over the array: 500 or more when the
/// array likely holds far more than 4096 distinct keys (about 481 expected
/// for 4096 random keys, at most the number of keys when fewer).
fn sample_distinct<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E) -> usize {
    let n = arr.len() as u64;
    let mut sample: Vec<u128> = (0..512u64).map(|i| enc(&arr[((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n >> 32) as usize])).collect();
    sample.sort_unstable();
    sample.dedup();
    sample.len()
}

/// 32 pairs of neighbours spread over the array all in order: long runs
/// likely (random keys pass with odds 2^-32).
fn locally_sorted<T, E: Fn(&T) -> u128>(arr: &[T], enc: &E) -> bool {
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
fn ends<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    let n = arr.len();
    let (p, pdesc) = run_at_start(arr, enc);
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
        sort_encoded(&mut arr[p..s], enc);
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

pub(crate) fn sort_encoded<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    if n >= ENDS_MIN_N && hit(ends(arr, enc), "ends") {
        return;
    }
    // Small records: their own path (`small`), after the rules that pay
    // for them too.
    if std::mem::size_of::<T>() <= SMALL_RECORD {
        // Unstable, many repeated keys: as below for larger records.
        if unstable() && n >= UNSTABLE_PDQ_MIN && sample_distinct(arr, enc) < MANY_DISTINCT {
            trace!("std_unstable_dups");
            arr.sort_unstable_by_key(|x| enc(x));
            return;
        }
        if hit(heavy::heavy_keys(arr, enc), "heavy") {
            return;
        }
        if n >= VERGE_MIN && locally_sorted(arr, enc) && hit(verge::verge_if(arr, enc, true), "verge_runs") {
            return;
        }
        trace!("small_path");
        small::sort_encoded(arr, enc);
        return;
    }
    // Larger records. Beyond half the L2 the moves of the
    // records dominate: Verge on the records (fed by the first read).
    let size = std::mem::size_of::<T>();
    let beyond_cache = n.saturating_mul(size) > crate::v2::cache::budget(crate::v2::cache::Budget::Records) / 2;
    // The MSD's passes read and swap at random across the array: up to
    // twice the cache (beyond, the packed sort moves each record once).
    let msd_ok = unstable() && size >= UNSTABLE_MSD_MIN && n >= VERGE_MIN
        && n.saturating_mul(size) <= 2 * crate::v2::cache::budget(crate::v2::cache::Budget::Records);
    let try_verge = n >= VERGE_MIN && beyond_cache;
    let mut drops = if try_verge || msd_ok { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let mut turn_at = if try_verge || msd_ok { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let s = if try_verge || msd_ok {
        scan::<T, E, true>(arr, enc, &mut drops, &mut turn_at)
    } else {
        scan::<T, E, false>(arr, enc, &mut drops, &mut turn_at)
    };
    if s.ascending {
        trace!("presorted");
        return;
    }
    if s.strictly_descending {
        trace!("reversed");
        arr.reverse();
        return;
    }
    // Unstable, large records: the in-place MSD (each record moved once per
    // byte level, no buffer) unless the stable passes are cheaper: two runs
    // (a pipe organ: one merge) or breaks scattered in ascending order (the
    // repair of nearly sorted data).
    if msd_ok && s.turns > 2 && !nearly_sorted(arr, enc, &s, &drops) {
        trace!("msd_unstable");
        unstable::msd(arr, enc, s.lo, s.hi);
        return;
    }
    // Unstable, small records, many repeated keys: the standard library's
    // unstable sort (the MSD, whose buckets of equal keys end at once, was
    // slower).
    if unstable() && size < UNSTABLE_MSD_MIN && n >= UNSTABLE_PDQ_MIN && sample_distinct(arr, enc) < MANY_DISTINCT {
        trace!("unstable_dups");
        arr.sort_unstable_by_key(|x| enc(x));
        return;
    }
    if hit(few_keys(arr, enc), "few") {
        return;
    }
    if hit(heavy::heavy_keys(arr, enc), "heavy") {
        return;
    }
    // In the cache: Verge only on few breaks of ascending order (long runs:
    // concatenated sorted pieces), the runs found by a second read.
    if !beyond_cache && n >= VERGE_MIN && IN_CACHE_BREAKS_DIV * s.drops <= n && hit(verge::verge_if(arr, enc, true), "verge_few") {
        return;
    }
    // Beyond it, few turns: runs worth more than gathering the records at
    // random.
    if try_verge && TURN_DIV * s.turns <= n
        && hit(if s.drops <= DROPS_MAX {
            verge::verge_from_breaks(arr, enc, &drops[..s.drops])
        } else if s.turns <= DROPS_MAX {
            verge::verge_from_turns(arr, enc, &turn_at[..s.turns])
        } else {
            verge(arr, enc)
        }, "verge")
    {
        return;
    }
    // Unstable, small records in disorder with wide keys (Verge declined):
    // the standard library's unstable sort.
    if unstable() && size < UNSTABLE_MSD_MIN && n >= UNSTABLE_PDQ_MIN
        && u128::BITS - (s.hi - s.lo).leading_zeros() > UNSTABLE_WIDE_BITS
    {
        trace!("std_unstable_wide");
        arr.sort_unstable_by_key(|x| enc(x));
        return;
    }
    // Keys changing direction rarely after Verge declined (blocks of equal
    // keys): up to 4096 distinct keys counted.
    if try_verge
        && (FEW_WIDE_TURN_DIV * s.turns <= n || sample_distinct(arr, enc) < MANY_DISTINCT)
        && hit(few::few_keys_wide(arr, enc), "few_wide")
    {
        return;
    }
    // In the cache too: small records whose (key, index) needs a u128 (its
    // packed sort is slow on keys with many repeats: Zipf, clusters).
    let wide = (u128::BITS - (s.hi - s.lo).leading_zeros()) + (usize::BITS - (n - 1).leading_zeros()) > 64;
    if !beyond_cache && n >= IN_CACHE_MIN && sample_distinct(arr, enc) < MANY_DISTINCT && hit(few::few_keys_wide(arr, enc), "few_wide_cache") {
        return;
    }
    if !beyond_cache && size <= IN_CACHE_LSD_MAX && n >= IN_CACHE_MIN && wide && s.hi <= u64::MAX as u128 {
        trace!("lsd11_cache");
        lsd::lsd11(arr, enc, s.lo as u64);
        return;
    }
    // Records up to LSD_MAX bytes, beyond the cache, keys in disorder (many
    // turns): the LSD on the records too (the packed (key, index) is as
    // large as the record, and its gather reads at random). The byte
    // histograms are taken by one more read, only here.
    if size <= LSD_MAX && beyond_cache && s.hi <= u64::MAX as u128 && LSD_TURNS * s.turns > n {
        trace!("lsd");
        if LSD11.load(std::sync::atomic::Ordering::Relaxed) {
            lsd::lsd11(arr, enc, s.lo as u64);
            return;
        }
        let mut hist = vec![[0usize; 256]; 8];
        for x in arr.iter() {
            let k = enc(x) as u64;
            for (d, h) in hist.iter_mut().enumerate() {
                h[((k >> (8 * d)) & 0xFF) as usize] += 1;
            }
        }
        lsd(arr, enc, &hist);
        return;
    }
    let key_bits = u128::BITS - (s.hi - s.lo).leading_zeros();
    // In the cache the gather through a buffer is cheaper than scanning the
    // permutation first; beyond it, the permutation applied in place.
    if beyond_cache {
        trace!("packed_in_place");
        if !sort_in_place(arr, enc, s.lo, key_bits) {
            trace!("std_fallback");
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
        }
        return;
    }
    trace!("packed");
    let mut buffer: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the records are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buffer.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, enc, s.lo, key_bits) {
            trace!("std_fallback");
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}
