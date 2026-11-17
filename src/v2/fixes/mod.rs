//! Fixes of the regressions against the v1, one copy per losing case: the
//! code serving the case is copied here, only that case is routed to the
//! copy, and only the copy is changed (the winning cases keep the original).
//! Once every case is fixed, the copies are compared and factored.
//!
//! Inventory (case, copy of, change, measured gain):
//! - u128 / i128 already sorted: `presorted128`, copy of
//!   `inspect::presorted::presorted`; ascending scan by 4 pairs with an early
//!   exit (the v1 scan) instead of blocks of 32. Gain: pending.

pub(crate) mod presorted128;
