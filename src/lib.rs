//! # Voracious sort
//!
//! Radix sorts for every primitive type (`bool`, `char`, `u8`, `u16`, `u32`,
//! `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32`,
//! `f64`) and for any element sorted by a key.
//!
//! Version 2.0 is a rewrite of the single-thread sort and a breaking change
//! (see `RELEASES.md`); version 1.2.2 stays available. The multithread sort
//! (`voracious_mt_sort`, feature `voracious_multithread`) is still the one
//! of 1.x, not reworked yet.
//!
//! # Primitive types
//!
//! ```
//! use voracious_radix_sort::RadixSort;
//!
//! let mut v = vec![10i32, -3, 7, 0, -42];
//! v.voracious_sort();
//! assert_eq!(v, vec![-42, -3, 0, 7, 10]);
//!
//! // Floats: the order of `f64::total_cmp` (NaNs of both signs, -0.0 < +0.0).
//! let mut f = vec![2.5f64, -1.0, f64::INFINITY, -0.0, 0.0];
//! f.voracious_sort();
//! assert_eq!(f.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
//!            [-1.0, -0.0, 0.0, 2.5, f64::INFINITY].iter().map(|x: &f64| x.to_bits()).collect::<Vec<_>>());
//! ```
//!
//! # Any element, by a key
//!
//! ```
//! use voracious_radix_sort::RadixSort;
//!
//! #[derive(Clone, Copy, Debug, PartialEq)]
//! struct Order { price: f32, id: u32 }
//!
//! let mut orders = vec![Order { price: 9.5, id: 1 }, Order { price: 1.25, id: 2 }, Order { price: 9.5, id: 3 }];
//! // Stable: equal keys keep their order.
//! orders.voracious_sort_by_key(|o| o.price);
//! assert_eq!(orders.iter().map(|o| o.id).collect::<Vec<_>>(), vec![2, 1, 3]);
//! // Unstable: equal keys in any order.
//! orders.voracious_sort_unstable_by_key(|o| o.price);
//! assert_eq!(orders[0].id, 2);
//! ```
//!
//! # A key type of your own
//!
//! Encode it into an unsigned integer of the same order:
//!
//! ```
//! use voracious_radix_sort::{RadixSort, SortKey};
//!
//! #[derive(Clone, Copy)]
//! struct Date { year: u16, month: u8, day: u8 }
//!
//! impl SortKey for Date {
//!     fn encode(self) -> u128 {
//!         ((self.year as u128) << 16) | ((self.month as u128) << 8) | self.day as u128
//!     }
//! }
//!
//! let mut days = vec![Date { year: 2026, month: 11, day: 21 }, Date { year: 2020, month: 3, day: 22 }];
//! days.voracious_sort_by_key(|d| *d);
//! assert_eq!(days[0].year, 2020);
//! ```
//!
//! # Memory
//!
//! Some paths use a buffer of the size of the array or half of it, allocated
//! by the call and freed at its end; small arrays and many large ones are
//! sorted in place, but the path depends on the data: plan for a buffer as
//! large as the array. A strictly in-place sort will be added later.

// The thread-local flags are initialised in `const` blocks already; clippy
// 1.99 still reports them (false positive of missing_const_for_thread_local).
#![allow(clippy::missing_const_for_thread_local)]

#[cfg(feature = "voracious_multithread")]
#[cfg(test)]
mod generators;
#[cfg(feature = "voracious_multithread")]
#[cfg(test)]
mod tests;
// The 1.x code: only the multithread sort (peeka) still runs on it, and the
// tests and benchmarks (built with the same feature) compare against it.
#[cfg(feature = "voracious_multithread")]
mod v1;
mod v2;

pub use v2::api::{RadixSort, Sortable};
pub use v2::devour::{devour_sort, DevourSort};
pub use v2::famished::{famished_sort_by_key, famished_sort_unstable_by_key, SortKey};
pub use v2::ravenous::{ravenous_sort, RavenousSort};

/// Dev only (feature `ab`, the measuring examples): the cache sizes read on
/// this machine (L2, largest level), in bytes.
#[cfg(feature = "ab")]
#[doc(hidden)]
pub fn ab_cache() -> (usize, usize) { (v2::cache::l2_bytes(), v2::cache::largest_bytes()) }

/// Dev only (feature `ab`): the u16 chain (`narrow16`).
#[cfg(feature = "ab")]
#[doc(hidden)]
pub fn ab_u16_gen(keys: &mut [u16]) { v2::devour::narrow16::nu16::sort(keys) }

/// Dev only (feature `ab`): the i16 chain (`narrow16`).
#[cfg(feature = "ab")]
#[doc(hidden)]
pub fn ab_i16_gen(keys: &mut [i16]) { v2::devour::i16::sort(keys) }

/// Dev only (feature `ab`): the u16 chain of the A/B copy (`ab_copy`).
#[cfg(feature = "ab")]
#[doc(hidden)]
pub fn ab_u16_copy(keys: &mut [u16]) { v2::devour::ab_copy::cu16::sort(keys) }

/// Dev only (feature `ab`): the i16 chain of the A/B copy (`ab_copy`).
#[cfg(feature = "ab")]
#[doc(hidden)]
pub fn ab_i16_copy(keys: &mut [i16]) {
    v2::devour::ab_copy::ci16::sort(v2::devour::keys::bits_of(keys))
}

/// Dev only (features `ab` and `v2_trace`): the decision labels noted
/// since the last call.
#[cfg(all(feature = "ab", feature = "v2_trace"))]
#[doc(hidden)]
pub fn ab_trace_take() -> Vec<&'static str> { v2::dev::trace::take() }
