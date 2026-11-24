//! Version 2: the radix sorts of 2.0.
//!
//! How a sort goes: `api` (the `RadixSort` trait) or `sorts` (the free
//! functions) call the chain of the key type in `types` (one file per type,
//! one per key type and record size for the records: each chain can change
//! without touching the others). A chain is a list of rules tried in order:
//! small arrays first, then `inspect` (already sorted?), `special` (a
//! distribution with a faster method?), `runs` (long sorted runs to keep and
//! merge?), and the `decide` rules pick the `engines` for the rest. The
//! thresholds come from `cache`, the cache sizes read at run time.
//!
//! - `api`: the `RadixSort` trait (`voracious_sort`, `_by_key`...);
//! - `sorts`: the entry points: `devour` (integers, `char`), `ravenous`
//!   (floats), `by_key` (records);
//! - `decide`: the rules, as tables: the radix path every key type goes
//!   through (`path`), the engine of each chunk (`chunks`), the shapes read
//!   on samples (`shape`);
//! - `inspect`: already sorted, nearly sorted (and its repair);
//! - `special`: distributions sorted faster than by a radix sort (narrow
//!   ranges, few values, heavy hitters, dense ranges, plateaus);
//! - `runs`: Verge (long runs kept) and the merges;
//! - `engines`: the radix engines (fused LSD, DLSD, rollercoaster), the
//!   counting sort, the sorting networks and merges of short arrays;
//! - `cache`: cache sizes detected at run time;
//! - `dev`: the decision trace (feature `v2_trace`) and the switch of the
//!   rule under test (trace and sentinel bench compare both sides).

pub(crate) mod api;
#[macro_use]
pub(crate) mod dev;
#[macro_use]
pub(crate) mod special;
pub(crate) mod cache;
#[macro_use]
pub(crate) mod decide;
pub(crate) mod devour;
pub(crate) mod engines;
pub(crate) mod famished;
pub(crate) mod inspect;
pub(crate) mod ravenous;
pub(crate) mod runs;
#[cfg(test)] pub(crate) mod testing;
