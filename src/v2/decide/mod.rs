//! The rules: which way each array goes, one file per decision, written
//! as tables (the first matching line wins). Sizes are compared with the
//! caches (`cache::budget`); shapes come from samples of the keys (`shape`).
//!
//! - `chunks`: the engine of each chunk of the radix path, per key family.

pub(crate) mod chunks;
pub(crate) mod shape;
