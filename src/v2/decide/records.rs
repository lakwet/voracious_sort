//! The thresholds of the records sort (`sorts::by_key::sort_encoded`), whose
//! chain reads (first matching line wins, `trace!` label in brackets):
//!
//! | records    | when                                          | then                         |
//! |------------|-----------------------------------------------|------------------------------|
//! | unstable   | `size >= UNSTABLE_MSD_MIN`, n < `UNSTABLE_PDQ_MAX` | `sort_unstable` [std_unstable_large] |
//! | all        | runs at both ends cover most of it            | middle sorted, merged [ends] |
//! | <= 8 B     | unstable, n >= `UNSTABLE_PDQ_MIN`, many repeats | `sort_unstable` [std_unstable_dups] |
//! |            | a few heavy keys                              | extracted [heavy]            |
//! |            | locally sorted, unpredictable merges          | Verge [verge_runs]           |
//! |            | else                                          | `by_key::small` [small_path] |
//! | > 8 B      | ascending / strictly descending (first read)  | done / reversed              |
//! |            | unstable, large, up to twice the cache, not   | in-place MSD [msd_unstable]  |
//! |            | nearly sorted                                 |                              |
//! |            | unstable, small, n >= `UNSTABLE_PDQ_MIN`, many repeats | `sort_unstable` [unstable_dups] |
//! |            | at most 256 distinct keys                     | counting [few]               |
//! |            | a few heavy keys                              | extracted [heavy]            |
//! |            | in the cache, breaks <= n / `IN_CACHE_BREAKS_DIV` | Verge [verge_few]        |
//! |            | beyond, turns <= n / `TURN_DIV`               | Verge [verge]                |
//! |            | unstable, small, keys over `UNSTABLE_WIDE_BITS` bits | `sort_unstable` [std_unstable_wide] |
//! |            | beyond, few turns or many repeats             | counting <= 4096 keys [few_wide] |
//! |            | in the cache, n >= `IN_CACHE_MIN`, many repeats | counting <= 4096 keys [few_wide_cache] |
//! |            | in the cache, <= 16 B, (key, index) > 64 bits | LSD 11 bits [lsd11_cache]    |
//! |            | beyond, <= `LSD_MAX` B, turns > n / `LSD_TURNS` | LSD on the records [lsd]   |
//! |            | beyond                                        | packed sort, permutation in place [packed_in_place] |
//! |            | else                                          | packed sort via a buffer [packed] |
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
