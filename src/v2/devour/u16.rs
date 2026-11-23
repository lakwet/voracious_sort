//! u16: the generic 16-bit chain (`narrow16`, instance `nu16`).

/// Sorts `arr` (see `narrow16`).
pub(crate) fn sort(arr: &mut [u16]) { super::narrow16::nu16::sort(arr) }

/// The detectors of the chain, for the unit tests of `special`.
#[cfg(test)]
pub(crate) use super::narrow16::nu16::detectors;
