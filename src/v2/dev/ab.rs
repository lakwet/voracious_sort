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
pub(crate) fn new_rule() -> bool {
    NEW_RULE.load(Relaxed)
}

/// The merge of k runs under test (kwaymerge hybrid in `merge_runs`):
/// `AB_MERGE=1` always on (tests), `AB_MERGE=ab` follows the rule under test
/// (`NEW_RULE`, toggled by the sentinel bench), else off.
pub(crate) fn new_merge() -> bool {
    match merge_mode() {
        1 | 3 => true,
        2 | 4 => new_rule(),
        _ => false,
    }
}

/// `AB_MERGE` read once: 1 / ab the hybrid; 3 / ab2 the hybrid and the
/// unpredictable merges merged too (no whole radix sort instead).
fn merge_mode() -> u8 {
    static MODE: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("AB_MERGE").as_deref() {
        Ok("1") => 1,
        Ok("ab") => 2,
        Ok("3") => 3,
        Ok("ab2") => 4,
        _ => 0,
    })
}

/// Whether unpredictable merges are merged anyway (`AB_MERGE=3` / `ab2`).
pub(crate) fn merge_unpredictable_too() -> bool {
    match merge_mode() {
        3 => true,
        4 => new_rule(),
        _ => false,
    }
}
