//! Voracious 2: the new sorts (integers, floats, 128-bit keys, records by
//! key), beside the 1.x library which stays unchanged.

#![allow(dead_code)]

pub(crate) mod by_key;
pub(crate) mod cache;
pub(crate) mod dlsd;
pub(crate) mod float_detect;
pub(crate) mod floats;
pub(crate) mod integers;
pub(crate) mod lsd;
pub(crate) mod merge;
pub(crate) mod near_sorted;
pub(crate) mod roller;
pub(crate) mod small_sort;
pub(crate) mod verge;
pub(crate) mod wide128;
