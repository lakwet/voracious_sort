//! Version 2: the radix sorts with all the improvements, beside the 1.x
//! library.
//!
//! - `integers`, `floats`, `records`: the sorts (entry points);
//! - `pipeline`: the steps shared by every key type (presorted check,
//!   near-sorted repair, Verge, merges);
//! - `detect`: distributions sorted faster than by a radix sort;
//! - `engines`: the radix engines (fused LSD, DLSD, rollercoaster);
//! - `small`: sorting networks and merges for short arrays;
//! - `cache`: cache sizes detected at run time;
//! - `trace`: the decision trace (feature `v2_trace`); `ab`: the rule
//!   under test (trace and sentinel bench compare both sides).

#![allow(dead_code)]

#[macro_use]
pub(crate) mod trace;
pub(crate) mod ab;
pub(crate) mod cache;
#[macro_use]
pub(crate) mod detect;
pub(crate) mod engines;
pub(crate) mod floats;
pub(crate) mod integers;
pub(crate) mod pipeline;
pub(crate) mod records;
pub(crate) mod records2;
pub(crate) mod small;
