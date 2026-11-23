//! Records of at most 8 bytes, every key type: the whole decision chain
//! (the keys arrive encoded as u128 by `SortKey`, in the same order).
//!
//! Records of at most 8 bytes: runs at both ends, many repeats (unstable),
//! heavy keys, Verge on locally sorted keys, then their own path
//! (`small_path`: one read with the byte histograms, the LSD on the records).

use crate::v2::famished::heavy::heavy_keys;
use crate::v2::famished::small::lsd::lsd;
use crate::v2::famished::small::verge::verge;
use crate::v2::famished::small::{scan, Scan};
use crate::v2::famished::verge::verge_if;
use crate::v2::famished::{
    self as bk, few::few_keys, packed::sort_in_place, Sample, Unstable,
};

/// Runs at both ends tried from this length.
const ENDS_MIN_N: usize = 64;
/// A sample of 512 keys holds at least this many distinct: far more than
/// 4096 distinct keys in the array (about 481 expected for 4096 random keys).
const MANY_DISTINCT: usize = 500;
/// Shortest array tried for Verge on the records.
const VERGE_MIN: usize = 4096;
/// Unstable sort: the rules toward the standard library's unstable sort from
/// this many (below, the radix sort in the cache is faster).
const UNSTABLE_PDQ_MIN: usize = 65536;
/// Shortest array tried for Verge in the small path.
const SMALL_VERGE_MIN: usize = 4096;

/// Sorts `arr` by its encoded keys (`enc`), stably.
pub(crate) fn sort_stable<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) { chain(arr, enc); }

/// Sorts `arr` by its encoded keys, unstably (equal keys in any order):
/// the same chain with its unstable steps on.
pub(crate) fn sort_unstable<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let _unstable = Unstable::enter();
    chain(arr, enc);
}

/// The chain (first matching rule wins).
fn chain<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let sample = Sample::new();
    // The pieces left by `ends`, `heavy` and Verge: this chain again.
    let again = |a: &mut [T]| chain(a, enc);
    let mut start = (0, false);
    chain! {
        "ends" if n >= ENDS_MIN_N => try bk::ends(arr, enc, &mut start, &again);
        "std_unstable_dups" if bk::unstable()
            && n >= UNSTABLE_PDQ_MIN
            && bk::sample_distinct(arr, enc, &sample) < MANY_DISTINCT
            => do { arr.sort_unstable_by_key(|x| enc(x)) };
        "heavy" => try heavy_keys(arr, enc, &sample, &again);
        "verge_runs" if n >= VERGE_MIN && bk::locally_sorted(arr, enc)
            => try verge_if(arr, enc, true, &again);
        "small_path" => do { small_path(arr, enc, &sample) };
    }
}

/// The small records' own path, after one read of the keys. `sample`: the
/// key sample of the chain (same array, nothing moved since).
fn small_path<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, sample: &Sample) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    // Beyond half the L2 the moves of the records dominate: the LSD on the
    // records, Verge on the records (both fed by the first read).
    let size = std::mem::size_of::<T>();
    let records = crate::v2::cache::budget(crate::v2::cache::Budget::Records);
    let beyond_cache = n.saturating_mul(size) > records / 2;
    let mut hist = if beyond_cache { vec![[0usize; 256]; 8] } else { Vec::new() };
    let s: Scan = if beyond_cache {
        scan::<T, E, true, true>(arr, enc, &mut hist)
    } else {
        scan::<T, E, false, false>(arr, enc, &mut hist)
    };
    let key_bits = u128::BITS - (s.hi - s.lo).leading_zeros();
    // Verge's gaps: this path again (fresh sample: the array moved).
    let again = |a: &mut [T]| small_path(a, enc, &Sample::new());
    chain! {
        "presorted" if s.ascending => do {};
        "reversed" if s.strictly_descending => do { arr.reverse() };
        "few" => try few_keys(arr, enc, sample);
        // Few turns: long runs likely.
        "verge" if beyond_cache && n >= SMALL_VERGE_MIN && 8 * s.turns <= n
            => try verge(arr, enc, &again);
        "lsd" if beyond_cache && s.hi <= u64::MAX as u128 => do { lsd(arr, enc, &hist) };
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
