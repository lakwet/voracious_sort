//! Tests that are neither correctness nor benchmarks: the method's tools
//! (`trace`: decision trace, `make trace` / `trace-ab`; `sentinel`: timings
//! of the moved cells, `make sentinel`; `cells`: the cells they share) and
//! the memory checks under Miri (`miri`: `cargo +nightly miri test miri_`,
//! ordinary fast tests without Miri).

mod cells;
mod miri;
mod sentinel;
mod trace;
