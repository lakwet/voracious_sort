//! The rule under test. A change is written `if ab::new_rule() { new } else
//! { old }`; the decision trace (`TRACE_AB=1`) lists the cells whose path
//! it moves and the sentinel bench times both sides of those cells in one
//! process. Before the commit the winner stays and the switch goes.

#![allow(dead_code)] // idle between two rules under test

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// Rule under test switched on (trace-ab, sentinel).
pub(crate) static NEW_RULE: AtomicBool = AtomicBool::new(false);

#[inline(always)]
/// Whether the rule under test is on.
pub(crate) fn new_rule() -> bool { NEW_RULE.load(Relaxed) }
