//! i16: the generic 16-bit chain (`narrow16`, instance `ni16`): the u16
//! chain with the sign bit flipped wherever the order matters.

use crate::v2::devour::keys::bits_of;

/// Sorts `arr` (see `narrow16`).
pub(crate) fn sort(arr: &mut [i16]) { super::narrow16::ni16::sort(bits_of(arr)) }

/// The detectors of the chain, for the unit tests of `special`.
#[cfg(test)]
pub(crate) use super::narrow16::ni16::detectors;
