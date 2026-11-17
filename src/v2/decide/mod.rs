//! The rules: which way each array goes, one file per decision, written
//! as tables (the first matching line wins). Sizes are compared with the
//! caches (`cache::budget`); shapes come from samples of the keys (`shape`).
//!
//! - `path`: the radix path every key type goes through (presorted check,
//!   near-sorted repair, detectors, Verge, merges);
//! - `chunks`: the engine of each chunk of the radix path, per key family;
//! - `shape`: ordered keys, tight clusters, read on samples.

pub(crate) mod chunks;
pub(crate) mod path;
pub(crate) mod shape;
