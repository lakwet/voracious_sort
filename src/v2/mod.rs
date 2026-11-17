//! Version 2: the radix sorts with all the improvements, beside the 1.x
//! library.
//!
//! - `sorts`: the sorts (entry points): `devour` (integers, `char`),
//!   `ravenous` (floats), `by_key` (records);
//! - `pipeline`: the radix path shared by every key type (presorted check,
//!   near-sorted repair, detectors, Verge, merges);
//! - `inspect`: already sorted, nearly sorted (and its repair);
//! - `runs`: Verge (long runs kept) and the merges;
//! - `special`: distributions sorted faster than by a radix sort
//!   (narrow ranges, few values, heavy hitters, dense ranges, plateaus);
//! - `engines`: the radix engines (fused LSD, DLSD, rollercoaster);
//! - `engines`: also the sorting networks and merges for short arrays
//!   (`small`);
//! - `cache`: cache sizes detected at run time;
//! - `dev`: the decision trace (feature `v2_trace`) and the switch of the
//!   rule under test (trace and sentinel bench compare both sides).

// Outside the tests nothing calls v2 yet (the public API points to the 1.x
// sorts until 2.0): dead code is only reported in test builds.
#![cfg_attr(not(test), allow(dead_code))]

#[macro_use]
pub(crate) mod dev;
pub(crate) mod cache;
#[macro_use]
pub(crate) mod special;
pub(crate) mod engines;
pub(crate) mod inspect;
pub(crate) mod pipeline;
pub(crate) mod runs;
pub(crate) mod sorts;
