//! The rule under test. A change is written `if ab::new_rule() { new } else
//! { old }`; the decision trace (`TRACE_AB=1`) lists the cells whose path
//! it moves and the sentinel bench times both sides of those cells in one
//! process. Before the commit the winner stays and the switch goes.

#![allow(dead_code)] // idle between two rules under test

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering::Relaxed};

/// Rule under test switched on (trace-ab, sentinel).
pub(crate) static NEW_RULE: AtomicBool = AtomicBool::new(false);

#[inline(always)]
/// Whether the rule under test is on.
pub(crate) fn new_rule() -> bool {
    NEW_RULE.load(Relaxed)
}

/// Variant of the rule under test (bits the rule reads; 0 by default), to
/// compare its pieces in one process (bench columns `v2 new ...`).
pub(crate) static VARIANT: AtomicU8 = AtomicU8::new(0);

#[inline(always)]
/// The variant of the rule under test.
pub(crate) fn variant() -> u8 {
    VARIANT.load(Relaxed)
}
