//! Decision trace (feature `v2_trace`, for the tests): each decision point
//! of the sorts notes a short label; the trace test reads them back per
//! input, so a rule change shows which (type, size, distribution) cells
//! change path, without timing anything. Without the feature `trace!`
//! expands to nothing.

/// Notes `$step` (a `&'static str`) when the feature `v2_trace` is on.
macro_rules! trace {
    ($step:expr) => {
        #[cfg(feature = "v2_trace")]
        $crate::v2::dev::trace::note($step);
    };
}

#[cfg(feature = "v2_trace")]
thread_local! {
    static STEPS: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(feature = "v2_trace")]
/// Records one decision step of the current sort.
pub(crate) fn note(step: &'static str) {
    STEPS.with(|s| s.borrow_mut().push(step));
}

/// The labels noted since the last call, each once, in order of first note.
#[cfg(feature = "v2_trace")]
pub(crate) fn take() -> Vec<&'static str> {
    let steps = STEPS.with(|s| std::mem::take(&mut *s.borrow_mut()));
    let mut seen = Vec::new();
    for s in steps {
        if !seen.contains(&s) {
            seen.push(s);
        }
    }
    seen
}

/// `fired`, noting `step` when true (detectors).
#[inline(always)]
pub(crate) fn hit(fired: bool, _step: &'static str) -> bool {
    if fired {
        trace!(_step);
    }
    fired
}
