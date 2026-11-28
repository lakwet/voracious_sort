//! The radix engines: fused LSD, diverting LSD (DLSD) and the in-place
//! rollercoaster MSD, one module per key width and codec; the in-place MSD
//! (`msd`: records by a key, and the bucket permutation shared with the
//! rollercoaster); sorting networks and merges for short arrays (`small`).

#[macro_use]
mod codec;
pub(crate) mod bytes;
pub(crate) mod counter;
pub(crate) mod counting;
pub(crate) mod dlsd;
pub(crate) mod lsd;
pub(crate) mod msd;
pub(crate) mod roller;
#[cfg(feature = "ab")]
pub(crate) mod roller_ab;
pub(crate) mod small;
