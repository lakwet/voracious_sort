//! Sorting records (structs) by a key, stably.
//!
//! Each key is encoded into an unsigned integer of the same order
//! (`SortKey`). One read of the keys gives their range, whether they are
//! already in order and, beyond the cache, how often they change direction
//! (and the byte histograms for small records). Then:
//! - ascending: nothing to do; strictly descending: reversed (still stable);
//! - few distinct keys: a stable counting sort;
//! - beyond the cache, few turns: Verge on the records (long runs kept);
//! - small records beyond the cache: the LSD on the records themselves;
//! - else the packed (key, index) sort, the records moved once (in place
//!   beyond the cache, through a buffer in it).
//!
//! Keys too wide for a u128 with their index fall back to `sort_by`.

mod few;
mod heavy;

mod lsd;
mod merge;
mod packed;
mod permute;
mod verge;
mod verge_small;

pub use super::records::SortKey;

use std::mem::MaybeUninit;

use few::few_keys;
use lsd::lsd;
use packed::{sort_in_place, sort_into};
use verge::{verge, IN_CACHE, TURN_DIV, VERGE_MIN};
pub(crate) use heavy::ONE as HEAVY1;
pub(crate) use merge::{FOUR_WAY_MIN, HALF_KWAY, REUSE, K4M, KWAY2, ORGAN2, PREFAULT, PREFAULT2, TRIM};
#[cfg(test)]
pub(crate) use merge::bench_kernel;
/// Records of at most this size in disorder beyond the cache: the LSD.
pub(crate) static LSD_MAX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(16);
/// Disorder: more than n / LSD_TURNS turns.
pub(crate) static LSD_TURNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(16);
pub(crate) static HEAVY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static WIDE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static LSD11: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static LSD11_SMALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(crate) static FROM_BREAKS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) use verge::{BRANCHY_ROUNDS, KWAY_BRANCHY_MIN, ORGAN, KWAY_BRANCHY, KWAY_MAX, QUAD_MAX, TRACE, HALF, IN_CACHE as VERGE_IN_CACHE, RUN_DIV, RUN_MIN};

/// Sorts `arr` by `key`, stably.
pub fn devour_sort_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    sort_encoded(arr, &|x: &T| key(x).encode());
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

/// Direction of a pair of keys: -1 descending, 0 equal, 1 ascending.
#[inline(always)]
fn direction(a: u128, b: u128) -> i8 {
    (b > a) as i8 - (b < a) as i8
}

/// Reads every key once: range, breaks of ascending order (`down`), and
/// with `TURNS` the turns (changes between a break and a non-break from one
/// pair to the next: about n/2 on random keys, two per break on runs) and
/// the positions of the first `DROPS_MAX` breaks; with `HIST` the byte
/// histograms of the low 64 bits. Two elements per step, two ranges.
#[inline(always)]
fn scan<T, E: Fn(&T) -> u128, const TURNS: bool, const HIST: bool>(arr: &[T], enc: &E, hist: &mut [[usize; 256]], drops: &mut [usize], turn_at: &mut [usize]) -> Scan {
    let count = |hist: &mut [[usize; 256]], k: u128| {
        for (d, h) in hist.iter_mut().enumerate() {
            h[((k as u64 >> (8 * d)) & 0xFF) as usize] += 1;
        }
    };
    let n = arr.len();
    let first = enc(&arr[0]);
    if HIST {
        count(hist, first);
    }
    let (mut lo, mut hi, mut lo2, mut hi2) = (first, first, first, first);
    // `last`: kind of the previous pair (the first pair counts as a turn
    // when it breaks: turns then start with a descending stretch).
    let (mut down, mut turns, mut last) = (0usize, 0usize, false);
    let mut prev = first;
    let mut i = 1;
    // Ascending prefix first (one comparison per key: sorted arrays read
    // as fast as by an is-sorted check); the scan goes on from its end.
    if !HIST {
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
        if HIST {
            count(hist, ka);
            count(hist, kb);
        }
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
        if HIST {
            count(hist, ka);
        }
    }
    Scan { lo: lo.min(lo2), hi: hi.max(hi2), ascending: down == 0, strictly_descending: down == n - 1, turns, drops: down }
}

/// Tests: the LSD on the records forced, whatever their size.
#[cfg(test)]
pub(crate) static FORCE_LSD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn lsd_forced() -> bool {
    #[cfg(test)]
    return FORCE_LSD.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
    false
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

/// Runs at both ends tried from this length.
const ENDS_MIN_N: usize = 64;
pub(crate) static ENDS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static LSD_INCACHE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static HEAVY8: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static VERGE_FEW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub(crate) static SHORT_END: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

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
    let (p, pdesc) = if 8 * p < n && SHORT_END.load(std::sync::atomic::Ordering::Relaxed) { (0, false) } else { (p, pdesc) };
    let (s, sdesc) = if 8 * (n - s) < n && SHORT_END.load(std::sync::atomic::Ordering::Relaxed) { (n, false) } else { (s, sdesc) };
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
    let t_ends = std::time::Instant::now();
    if n >= ENDS_MIN_N && ENDS.load(std::sync::atomic::Ordering::Relaxed) && ends(arr, enc) {
        if verge::TRACE.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("  ends {:.2} ns", t_ends.elapsed().as_nanos() as f64 / n as f64);
        }
        return;
    }
    // Records of at most 8 bytes: the path of `records` (its first read
    // with byte histograms and the LSD on the records stay the fastest).
    if std::mem::size_of::<T>() <= 8 {
        if HEAVY8.load(std::sync::atomic::Ordering::Relaxed) && heavy::heavy_keys(arr, enc) {
            return;
        }
        super::records::sort_encoded(arr, enc);
        return;
    }
    // Beyond half the L2 the moves of the records dominate: the LSD on small
    // records, Verge on the records (both fed by the first read).
    let size = std::mem::size_of::<T>();
    let beyond_cache = n.saturating_mul(size) > super::cache::l2_bytes() / 2;
    let small_records = lsd_forced() || (size <= 8 && beyond_cache);
    let mut hist = if small_records { vec![[0usize; 256]; 8] } else { Vec::new() };
    let try_verge = n >= VERGE_MIN && (beyond_cache || IN_CACHE.load(std::sync::atomic::Ordering::Relaxed));
    let t_scan = std::time::Instant::now();
    let mut drops = if small_records || try_verge { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let mut turn_at = if small_records || try_verge { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let s = if small_records {
        scan::<T, E, true, true>(arr, enc, &mut hist, &mut drops, &mut turn_at)
    } else if try_verge {
        scan::<T, E, true, false>(arr, enc, &mut hist, &mut drops, &mut turn_at)
    } else {
        scan::<T, E, false, false>(arr, enc, &mut hist, &mut drops, &mut turn_at)
    };
    if verge::TRACE.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("  scan {:.2} ns (turns {})", t_scan.elapsed().as_nanos() as f64 / n as f64, s.turns);
    }
    if s.ascending {
        return;
    }
    if s.strictly_descending {
        arr.reverse();
        return;
    }
    let tr = verge::TRACE.load(std::sync::atomic::Ordering::Relaxed);
    if few_keys(arr, enc) {
        if tr {
            eprintln!("  few_keys");
        }
        return;
    }
    if HEAVY.load(std::sync::atomic::Ordering::Relaxed) && heavy::heavy_keys(arr, enc) {
        if tr {
            eprintln!("  heavy_keys");
        }
        return;
    }
    if tr {
        eprintln!("  n {} beyond_cache {} try_verge {} turns {} drops {} bits {} sample {}", n, beyond_cache, try_verge, s.turns, s.drops, u128::BITS - (s.hi - s.lo).leading_zeros(), sample_distinct(arr, enc));
    }
    // Few turns: long runs likely.
    // Small records: the strict rule (long runs, predictable merges);
    // larger ones: runs worth more than gathering the records at random.
    if size <= 8 && beyond_cache && n >= VERGE_MIN && 8 * s.turns <= n && verge_small::verge(arr, enc) {
        return;
    }
    // In the cache: Verge only on few breaks of ascending order (long runs:
    // concatenated sorted pieces), the runs found by a second read.
    if size > 8 && !beyond_cache && n >= VERGE_MIN && 64 * s.drops <= n && VERGE_FEW.load(std::sync::atomic::Ordering::Relaxed) && verge::verge_if(arr, enc, true) {
        return;
    }
    if size > 8 && try_verge && TURN_DIV.load(std::sync::atomic::Ordering::Relaxed) * s.turns <= n
        && (if s.drops <= DROPS_MAX && FROM_BREAKS.load(std::sync::atomic::Ordering::Relaxed) {
            verge::verge_from_breaks(arr, enc, &drops[..s.drops])
        } else if s.turns <= DROPS_MAX && FROM_BREAKS.load(std::sync::atomic::Ordering::Relaxed) {
            verge::verge_from_turns(arr, enc, &turn_at[..s.turns])
        } else {
            verge(arr, enc)
        })
    {
        return;
    }
    if small_records && s.hi <= u64::MAX as u128 {
        if LSD11_SMALL.load(std::sync::atomic::Ordering::Relaxed) {
            lsd::lsd11(arr, enc, s.lo as u64);
        } else {
            lsd(arr, enc, &hist);
        }
        return;
    }
    // Keys changing direction rarely after Verge declined (blocks of equal
    // keys): up to 4096 distinct keys counted.
    if try_verge && WIDE.load(std::sync::atomic::Ordering::Relaxed)
        && (16 * s.turns <= n || (LSD_INCACHE.load(std::sync::atomic::Ordering::Relaxed) && sample_distinct(arr, enc) < 500))
        && few::few_keys_wide(arr, enc)
    {
        return;
    }
    // In the cache too: small records whose (key, index) needs a u128 (its
    // packed sort is slow on keys with many repeats: Zipf, clusters).
    let wide = (u128::BITS - (s.hi - s.lo).leading_zeros()) + (usize::BITS - (n - 1).leading_zeros()) > 64;
    if LSD_INCACHE.load(std::sync::atomic::Ordering::Relaxed) && !beyond_cache && n >= 32768 && sample_distinct(arr, enc) < 500 && few::few_keys_wide(arr, enc) {
        return;
    }
    if LSD_INCACHE.load(std::sync::atomic::Ordering::Relaxed) && !beyond_cache && size <= 16 && n >= 32768 && wide && s.hi <= u64::MAX as u128 {
        lsd::lsd11(arr, enc, s.lo as u64);
        return;
    }
    // Records up to LSD_MAX bytes, beyond the cache, keys in disorder (many
    // turns): the LSD on the records too (the packed (key, index) is as
    // large as the record, and its gather reads at random). The byte
    // histograms are taken by one more read, only here.
    if size <= LSD_MAX.load(std::sync::atomic::Ordering::Relaxed) && beyond_cache && s.hi <= u64::MAX as u128 && LSD_TURNS.load(std::sync::atomic::Ordering::Relaxed) * s.turns > n {
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
        if !sort_in_place(arr, enc, s.lo, key_bits) {
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
        }
        return;
    }
    let mut buffer: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the records are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buffer.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, enc, s.lo, key_bits) {
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}
