//! Performance tests (ignored or printing timings, not correctness):
//! `bench_cpu` (the CPU benchmark of `make benchcpu`, `benchbilan`...),
//! `speed_sort` (v1 speed tests, `make bench <name>`), their helpers.

mod bench_cpu;
mod test_helpers;
#[rustfmt::skip] mod speed_sort;
// #[rustfmt::skip] mod speed_sort_ded;
