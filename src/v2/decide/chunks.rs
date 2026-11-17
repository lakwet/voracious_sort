//! The engine of each chunk of the radix path (the whole array, or a gap
//! between the runs Verge keeps), per key family. Sizes in keys, from the
//! cache budgets (`cache::budget`); the first matching line wins.
//!
//! | keys            | chunk of n keys                  | engine                   |
//! |-----------------|----------------------------------|--------------------------|
//! | 8, 16 bits      | any                              | fused LSD                |
//! | 32-bit integers | n < Roller32                     | fused LSD                |
//! |                 | keys ordered locally or globally | rollercoaster            |
//! |                 | else                             | fused LSD                |
//! | f32             | n < RollerF32 / 12               | fused LSD                |
//! |                 | else                             | rollercoaster            |
//! | 64-bit integers | n < `DENSE_MIN`                  | DLSD                     |
//! |                 | n < Roller64 / 12                | DLSD, LSD on clusters    |
//! |                 | else                             | rollercoaster            |
//! | f64             | n < RollerF64 / 48               | DLSD                     |
//! |                 | else                             | rollercoaster            |
//! | 128 bits        | n < Roller128 / 24               | DLSD                     |
//! |                 | else                             | rollercoaster            |
//!
//! Measurements: journal §12 (rollercoaster leaves), §99-§100 (32 bits
//! beyond the cache), the cluster rule in `shape::clustered`.

use super::shape::{globally_ordered, locally_ordered};
use crate::v2::cache::{budget, Budget};
use crate::v2::special::DENSE_MIN;

/// What sorts a chunk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Engine {
    /// The fused LSD (`engines::lsd`).
    Lsd,
    /// The diverting LSD (`engines::dlsd`).
    Dlsd,
    /// The DLSD, unless its sample shows tight clusters: then the LSD.
    DlsdOrLsdOnClusters,
    /// In-place MSD passes down to the cache, then the DLSD (`engines::roller`).
    Roller,
}

/// 8- and 16-bit keys: the LSD (two passes at most), always.
#[inline]
pub(crate) fn narrow<U>(_: &[U]) -> Engine {
    Engine::Lsd
}

/// 32-bit integers (and `char`): the rollercoaster beyond `roller_min_32`
/// only on ordered keys (the MSD pass then writes in few sequential
/// streams; random keys go 15 to 45 % faster through the LSD from 30M to
/// 1G).
#[inline]
pub(crate) fn int32<U: Copy + PartialOrd>(bits: &[U]) -> Engine {
    if bits.len() >= roller_min_32() && (locally_ordered(bits) || globally_ordered(bits)) {
        Engine::Roller
    } else {
        Engine::Lsd
    }
}

/// f32: the rollercoaster from `roller_min_f32`.
#[inline]
pub(crate) fn f32(n: usize) -> Engine {
    if n >= roller_min_f32() {
        Engine::Roller
    } else {
        Engine::Lsd
    }
}

/// 64-bit integers.
#[inline]
pub(crate) fn int64(n: usize) -> Engine {
    if n >= roller_min_64() {
        Engine::Roller
    } else if n >= DENSE_MIN {
        Engine::DlsdOrLsdOnClusters
    } else {
        Engine::Dlsd
    }
}

/// f64: the rollercoaster from `roller_min_f64`.
#[inline]
pub(crate) fn f64(n: usize) -> Engine {
    if n >= roller_min_f64() {
        Engine::Roller
    } else {
        Engine::Dlsd
    }
}

/// 128-bit integers: the rollercoaster from `roller_min_128`.
#[inline]
pub(crate) fn wide(n: usize) -> Engine {
    if n >= roller_min_128() {
        Engine::Roller
    } else {
        Engine::Dlsd
    }
}

/// 32-bit integers: the L2 bytes (for its memory, n instead of 2n;
/// rollercoaster / LSD 0.94-1.05 from 1M to 10M). The offset sort of the
/// range detector stays below it.
pub(crate) fn roller_min_32() -> usize {
    budget(Budget::Roller32)
}

/// 64-bit integers: L2 / 12 (rollercoaster / DLSD 1.00 there, 0.82-0.93 at
/// 4M-10M).
pub(crate) fn roller_min_64() -> usize {
    budget(Budget::Roller64) / 12
}

/// 8 and 16 bits: never.
pub(crate) fn no_roller() -> usize {
    usize::MAX
}

/// f32: L2 / 12.
fn roller_min_f32() -> usize {
    budget(Budget::RollerF32) / 12
}

/// f64: L2 / 48, where the DLSD's keys and buffer take a third of the L2.
fn roller_min_f64() -> usize {
    budget(Budget::RollerF64) / 48
}

/// 128 bits: L2 / 24 (keys of 16 bytes, as the 64-bit threshold L2 / 12).
fn roller_min_128() -> usize {
    budget(Budget::Roller128) / 24
}
