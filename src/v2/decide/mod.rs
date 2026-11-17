//! The rules: which way each array goes, one file per decision, written
//! as tables (the first matching line wins). Sizes are compared with the
//! caches (`cache::budget`); shapes come from samples of the keys (`shape`).
//!
//! - `path`: the radix path every key type goes through (presorted check,
//!   near-sorted repair, detectors, Verge, merges);
//! - `chunks`: the engine of each chunk of the radix path, per key family;
//! - `records`: the thresholds of the records sort (and its chain);
//! - `shape`: ordered keys, tight clusters, read on samples.
//!
//! The chains of the sorts are written with `chain!`: one line per rule.

/// A decision chain: rules tried in order, the first one that sorts the
/// array returns from the function. A rule is a trace label, an optional
/// guard (`if ...`), then either
/// - `do { ... }`: an action that always sorts (its label noted first), or
/// - `try expr`: an attempt returning whether it sorted (the label noted
///   when it did).
///
/// `_` in place of the label: an attempt noting its own labels (a group of
/// detectors). It expands to the plain `if` chain (no indirection).
macro_rules! chain {
    () => {};
    ($label:literal $(if $cond:expr)? => do $body:block; $($rest:tt)*) => {
        if true $(&& $cond)? {
            trace!($label);
            $body;
            return;
        }
        chain!($($rest)*);
    };
    ($label:literal $(if $cond:expr)? => try $e:expr; $($rest:tt)*) => {
        if true $(&& $cond)? && $crate::v2::dev::trace::hit($e, $label) {
            return;
        }
        chain!($($rest)*);
    };
    (_ $(if $cond:expr)? => try $e:expr; $($rest:tt)*) => {
        if true $(&& $cond)? && $e {
            return;
        }
        chain!($($rest)*);
    };
}

pub(crate) mod chunks;
pub(crate) mod path;
pub(crate) mod records;
pub(crate) mod shape;
