//! u32: the 32-bit chain (`chain32`, instance `cu32`).

/// Sorts `arr` (see `chain32`).
pub(crate) fn sort(arr: &mut [u32]) { super::chain32::cu32::sort(arr) }

/// The detectors of the chain, for the unit tests of `special`.
#[cfg(test)]
pub(crate) use super::chain32::cu32::detectors;
