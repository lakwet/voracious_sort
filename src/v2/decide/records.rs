//! The thresholds of the records sort (`sorts::by_key::sort_encoded`), whose
//! chain reads (first matching rule wins, `trace!` label in brackets):
//!
//! 0. unstable, records of `UNSTABLE_MSD_MIN` bytes or more, fewer than
//!    `UNSTABLE_PDQ_MAX`: `sort_unstable` [std_unstable_large];
//! 1. runs at both ends covering most of the array: the middle sorted, all
//!    merged [ends];
//! 2. records of at most `SMALL_RECORD` bytes: unstable with many repeats,
//!    `sort_unstable` [std_unstable_dups]; a few heavy keys, extracted
//!    [heavy]; locally sorted with unpredictable merges, Verge [verge_runs];
//!    else their own path `by_key::small` [small_path];
//!
//! larger records, after one read of the keys:
//!
//! 3. ascending: done; strictly descending: reversed [presorted, reversed];
//! 4. unstable, large, up to twice the cache, not nearly sorted: the
//!    in-place MSD [msd_unstable];
//! 5. unstable, small, from `UNSTABLE_PDQ_MIN`, many repeats:
//!    `sort_unstable` [unstable_dups];
//! 6. at most 256 distinct keys: counting [few];
//! 7. a few heavy keys: extracted [heavy];
//! 8. in the cache, at most n / `IN_CACHE_BREAKS_DIV` breaks: Verge
//!    [verge_few];
//! 9. beyond, at most n / `TURN_DIV` turns: Verge [verge];
//! 10. unstable, small, keys over `UNSTABLE_WIDE_BITS` bits: `sort_unstable`
//!     [std_unstable_wide];
//! 11. beyond, few turns or many repeats: counting up to 4096 keys
//!     [few_wide];
//! 12. in the cache, from `IN_CACHE_MIN`, many repeats: counting up to 4096
//!     keys [few_wide_cache];
//! 13. in the cache, up to `IN_CACHE_LSD_MAX` bytes, (key, index) over 64
//!     bits: the 11-bit LSD [lsd11_cache];
//! 14. beyond, up to `LSD_MAX` bytes, more than n / `LSD_TURNS` turns: the
//!     LSD on the records [lsd];
//! 15. beyond: the packed (key, index) sort, permutation in place
//!     [packed_in_place];
//! 16. else: the packed sort through a buffer [packed].
//!
//! "Beyond": the records take more than half the `Records` budget (L2).
//! "Many repeats": fewer than `MANY_DISTINCT` distinct keys in a sample of
//! 512 (`sample_distinct`).

/// Records of at most this many bytes go to `by_key::small` (its first read
/// with byte histograms and the LSD on the records stay the fastest).
pub(crate) const SMALL_RECORD: usize = 8;

/// A sample of 512 keys holds at least this many distinct: far more than
/// 4096 distinct keys in the array (about 481 expected for 4096 random keys).
pub(crate) const MANY_DISTINCT: usize = 500;

/// Runs at both ends tried from this length.
pub(crate) const ENDS_MIN_N: usize = 64;

/// Shortest array tried for Verge on the records.
pub(crate) const VERGE_MIN: usize = 4096;

/// Beyond the cache, Verge tried when `TURN_DIV * turns <= n`.
pub(crate) const TURN_DIV: usize = 4;

/// In the cache, Verge tried on at most n / `IN_CACHE_BREAKS_DIV` breaks of
/// ascending order (long runs: concatenated sorted pieces).
pub(crate) const IN_CACHE_BREAKS_DIV: usize = 64;

/// Beyond the cache, after Verge declined: the counting of up to 4096 keys
/// tried on at most n / `FEW_WIDE_TURN_DIV` turns (blocks of equal keys).
pub(crate) const FEW_WIDE_TURN_DIV: usize = 16;

/// In the cache: the counting of many repeated keys and the 11-bit LSD of
/// wide (key, index) pairs from this many records.
pub(crate) const IN_CACHE_MIN: usize = 32768;

/// In the cache: the 11-bit LSD for records of at most this many bytes.
pub(crate) const IN_CACHE_LSD_MAX: usize = 16;

/// Records of at most this size in disorder beyond the cache: the LSD.
pub(crate) const LSD_MAX: usize = 16;

/// Disorder: more than n / `LSD_TURNS` turns.
pub(crate) const LSD_TURNS: usize = 16;

/// Unstable sort: records from this size may take the in-place MSD.
pub(crate) const UNSTABLE_MSD_MIN: usize = 64;

/// Unstable sort: large records below this many go to the standard
/// library's unstable sort (in place: no buffer to fault in, presorted
/// data found as it goes).
pub(crate) const UNSTABLE_PDQ_MAX: usize = 32768;

/// Unstable sort, small records: the rules toward the standard library's
/// unstable sort from this many (below, the radix sort in the cache is
/// faster: Saw 16 B at 10k 4.2 against 7.8 ns per record).
pub(crate) const UNSTABLE_PDQ_MIN: usize = 65536;

/// Unstable sort of small records in disorder: keys wider than this many
/// bits go to the standard library's unstable sort (the radix passes cost
/// more than its comparisons).
pub(crate) const UNSTABLE_WIDE_BITS: u32 = 40;
