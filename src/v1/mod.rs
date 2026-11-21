//! Version 1 of voracious (2020-2026), no longer public since 2.0: kept for
//! the multithread sort (`voracious_mt_sort`: peeka sort) and as the
//! reference of the benchmarks and tests.

// The sorts of 1.x not used by the multithread sort, and the re-exports
// below, only reached by the tests and benchmarks.
#![allow(dead_code, unused_imports)]

pub(crate) mod algo;
pub(crate) mod dedicated;
pub(crate) mod sorts;
pub(crate) mod traits;
pub(crate) mod types;

// The items public in 1.x, reached by the v1 files as `super::super::…`
// and by the tests as `crate::v1::…`.
pub(crate) use traits::dispatcher::Dispatcher;
pub(crate) use traits::radix_key::RadixKey;
pub(crate) use traits::radixable::Radixable;
pub(crate) use traits::radixsort::RadixSort;

pub(crate) use sorts::american_flag_sort::american_flag_sort;
pub(crate) use sorts::awardofthesky_sort::awardofthesky_sort;
pub(crate) use sorts::boolean_sort::boolean_sort;
pub(crate) use sorts::comparative_sort::insertion_sort;
pub(crate) use sorts::counting_sort::counting_sort;
pub(crate) use sorts::dlsd_sort::dlsd_radixsort;
pub(crate) use sorts::lsd_sort::lsd_radixsort;
pub(crate) use sorts::lsd_stable_sort::lsd_stable_radixsort;
pub(crate) use sorts::msd_sort::msd_radixsort;
pub(crate) use sorts::msd_stable_sort::msd_stable_radixsort;
#[cfg(feature = "voracious_multithread")] pub(crate) use sorts::peeka_sort::peeka_sort;
pub(crate) use sorts::rollercoaster_sort::rollercoaster_sort;
pub(crate) use sorts::ska_sort::ska_sort;
pub(crate) use sorts::voracious_sort::voracious_sort;
