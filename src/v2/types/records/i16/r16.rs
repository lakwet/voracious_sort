//! Records of 9 to 16 bytes sorted by i16 keys: the whole decision chain of
//! this pair (copy of the shared records chain of 2026-11-17; edit here
//! only, see `types`).
//!
//! Records over 8 bytes: runs at both ends, then one read of the keys and
//! the rules below (first matching rule wins, `trace!` label in brackets).
//! "Beyond": the records take more than half the `Records` budget (L2).
//! "Many repeats": fewer than `MANY_DISTINCT` distinct keys in a sample of
//! 512 (`sample_distinct`).

use crate::v2::cache::{budget, Budget};
use crate::v2::sorts::by_key::few::{few_keys, few_keys_wide};
use crate::v2::sorts::by_key::heavy::heavy_keys;
use crate::v2::sorts::by_key::lsd::{lsd, lsd11};
use crate::v2::sorts::by_key::packed::sort_in_place;
use crate::v2::sorts::by_key::verge::{verge, verge_from_breaks, verge_from_turns, verge_if};
use crate::v2::sorts::by_key::{self as bk, Sample, Unstable, DROPS_MAX, LSD11};

/// A sample of 512 keys holds at least this many distinct: far more than
/// 4096 distinct keys in the array (about 481 expected for 4096 random keys).
const MANY_DISTINCT: usize = 500;
/// Runs at both ends tried from this length.
const ENDS_MIN_N: usize = 64;
/// Shortest array tried for Verge on the records.
const VERGE_MIN: usize = 4096;
/// Beyond the cache, Verge tried when `TURN_DIV * turns <= n`.
const TURN_DIV: usize = 4;
/// In the cache, Verge tried on at most n / `IN_CACHE_BREAKS_DIV` breaks of
/// ascending order (long runs: concatenated sorted pieces).
const IN_CACHE_BREAKS_DIV: usize = 64;
/// Beyond the cache, after Verge declined: the counting of up to 4096 keys
/// tried on at most n / `FEW_WIDE_TURN_DIV` turns (blocks of equal keys).
const FEW_WIDE_TURN_DIV: usize = 16;
/// In the cache: the counting of many repeated keys and the 11-bit LSD of
/// wide (key, index) pairs from this many records.
const IN_CACHE_MIN: usize = 32768;
/// In the cache: the 11-bit LSD for records of at most this many bytes.
const IN_CACHE_LSD_MAX: usize = 16;
/// Records of at most this size in disorder beyond the cache: the LSD.
const LSD_MAX: usize = 16;
/// Disorder: more than n / `LSD_TURNS` turns.
const LSD_TURNS: usize = 16;
/// Unstable sort: records from this size may take the in-place MSD.
const UNSTABLE_MSD_MIN: usize = 64;
/// Unstable sort: large records below this many go to the standard
/// library's unstable sort (in place: no buffer to fault in, presorted data
/// found as it goes).
const UNSTABLE_PDQ_MAX: usize = 32768;
/// Unstable sort, small records: the rules toward the standard library's
/// unstable sort from this many (below, the radix sort in the cache is
/// faster: Saw 16 B at 10k 4.2 against 7.8 ns per record).
const UNSTABLE_PDQ_MIN: usize = 65536;
/// Unstable sort of small records in disorder: keys wider than this many
/// bits go to the standard library's unstable sort.
const UNSTABLE_WIDE_BITS: u32 = 40;

/// Sorts `arr` by its encoded keys (`enc`), stably.
pub(crate) fn sort_stable<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) { chain(arr, enc); }

/// Sorts `arr` by its encoded keys, unstably (equal keys in any order):
/// the same chain with its unstable steps on.
pub(crate) fn sort_unstable<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    // Few large records: the standard library's pattern-defeating quicksort
    // (in place: no buffer to fault in, presorted data found as it goes).
    if std::mem::size_of::<T>() >= UNSTABLE_MSD_MIN && arr.len() < UNSTABLE_PDQ_MAX {
        trace!("std_unstable_large");
        arr.sort_unstable_by_key(|x| enc(x));
        return;
    }
    let _unstable = Unstable::enter();
    chain(arr, enc);
}

/// The chain (first matching rule wins).
fn chain<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let size = std::mem::size_of::<T>();
    let sample = Sample::new();
    // The pieces left by `ends`, `heavy` and Verge: this chain again.
    let again = |a: &mut [T]| chain(a, enc);
    // The run at the start, read by `ends`: not read again by the scan.
    let mut start = (0, false);
    chain! {
        "ends" if n >= ENDS_MIN_N => try bk::ends(arr, enc, &mut start, &again);
    }
    // One read of the keys. Beyond half the L2 the moves of the records
    // dominate: Verge on the records (fed by the read).
    let records = budget(Budget::Records);
    let beyond_cache = n.saturating_mul(size) > records / 2;
    // The MSD's passes read and swap at random across the array: up to
    // twice the cache (beyond, the packed sort moves each record once).
    let msd_ok = bk::unstable()
        && size >= UNSTABLE_MSD_MIN
        && n >= VERGE_MIN
        && n.saturating_mul(size) <= 2 * records;
    let try_verge = n >= VERGE_MIN && beyond_cache;
    let mut drops = if try_verge || msd_ok { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let mut turn_at = if try_verge || msd_ok { vec![0usize; DROPS_MAX + 1] } else { Vec::new() };
    let s = if try_verge || msd_ok {
        bk::scan::<T, E, true>(arr, enc, &mut drops, &mut turn_at, start)
    } else {
        bk::scan::<T, E, false>(arr, enc, &mut drops, &mut turn_at, start)
    };
    let key_bits = u128::BITS - (s.hi - s.lo).leading_zeros();
    // (key, index) needs a u128 (its packed sort is slow on keys with many
    // repeats: Zipf, clusters).
    let wide = key_bits + (usize::BITS - (n - 1).leading_zeros()) > 64;
    let small_unstable = bk::unstable() && size < UNSTABLE_MSD_MIN && n >= UNSTABLE_PDQ_MIN;
    chain! {
        "presorted" if s.ascending => do {};
        "reversed" if s.strictly_descending => do { arr.reverse() };
        // The in-place MSD (each record moved once per byte level, no
        // buffer) unless the stable passes are cheaper: two runs (a pipe
        // organ: one merge) or breaks scattered in ascending order (the
        // repair of nearly sorted data).
        "msd_unstable" if msd_ok && s.turns > 2 && !bk::nearly_sorted(arr, enc, &s, &drops)
            => do { crate::v2::engines::msd::msd(arr, enc, s.lo, s.hi) };
        // The MSD, whose buckets of equal keys end at once, was slower.
        "unstable_dups" if small_unstable
            && bk::sample_distinct(arr, enc, &sample) < MANY_DISTINCT
            => do { arr.sort_unstable_by_key(|x| enc(x)) };
        "few" => try few_keys(arr, enc, &sample);
        "heavy" => try heavy_keys(arr, enc, &sample, &again);
        // In the cache: long runs only (concatenated sorted pieces), found
        // by a second read.
        "verge_few" if !beyond_cache && n >= VERGE_MIN && IN_CACHE_BREAKS_DIV * s.drops <= n
            => try verge_if(arr, enc, true, &again);
        // Beyond it, few turns: runs worth more than gathering the records
        // at random.
        "verge" if try_verge && TURN_DIV * s.turns <= n
            => try if s.drops <= DROPS_MAX {
                verge_from_breaks(arr, enc, &drops[..s.drops], &again)
            } else if s.turns <= DROPS_MAX {
                verge_from_turns(arr, enc, &turn_at[..s.turns], &again)
            } else {
                verge(arr, enc, &again)
            };
        // Wide keys in disorder (Verge declined): comparisons beat the
        // radix passes.
        "std_unstable_wide" if small_unstable && key_bits > UNSTABLE_WIDE_BITS
            => do { arr.sort_unstable_by_key(|x| enc(x)) };
        // Keys changing direction rarely (blocks of equal keys): up to 4096
        // distinct keys counted.
        "few_wide" if try_verge
            && (FEW_WIDE_TURN_DIV * s.turns <= n
                || bk::sample_distinct(arr, enc, &sample) < MANY_DISTINCT)
            => try few_keys_wide(arr, enc);
        "few_wide_cache" if !beyond_cache
            && n >= IN_CACHE_MIN
            && bk::sample_distinct(arr, enc, &sample) < MANY_DISTINCT
            => try few_keys_wide(arr, enc);
        "lsd11_cache" if !beyond_cache
            && size <= IN_CACHE_LSD_MAX
            && n >= IN_CACHE_MIN
            && wide
            && s.hi <= u64::MAX as u128
            => do { lsd11(arr, enc, s.lo as u64) };
        // Keys in disorder beyond the cache: the LSD on the records too (the
        // packed (key, index) is as large as the record, and its gather
        // reads at random). The byte histograms: one more read, only here.
        "lsd" if size <= LSD_MAX
            && beyond_cache
            && s.hi <= u64::MAX as u128
            && LSD_TURNS * s.turns > n
            => do {
            // Written here, not in a function: as a call, the histogram loop
            // came out 8 % slower (r16u64 1M Unif / Timestamps / Clusters,
            // Surface binary A/B).
            if LSD11.load(std::sync::atomic::Ordering::Relaxed) {
                lsd11(arr, enc, s.lo as u64);
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
        };
        // Beyond the cache the permutation applied in place; in it, the
        // gather through a buffer (cheaper than scanning the permutation).
        "packed_in_place" if beyond_cache => do {
            if !sort_in_place(arr, enc, s.lo, key_bits) {
                bk::std_fallback(arr, enc);
            }
        };
        "packed" => do { bk::packed_buffer(arr, enc, s.lo, key_bits) };
    }
}
