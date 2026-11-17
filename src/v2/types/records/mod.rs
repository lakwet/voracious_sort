//! Records: one folder per key type, one chain per record size in it
//! (`r4`: 3 to 4 bytes... `r64`: 33 bytes and more; the smallest class of a
//! key also takes records smaller than the key). Reached through
//! `SortKey::sort_records` (`sorts::by_key`).

pub(crate) mod bool;
pub(crate) mod char;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i128;
pub(crate) mod i16;
pub(crate) mod i32;
pub(crate) mod i64;
pub(crate) mod i8;
pub(crate) mod u128;
pub(crate) mod u16;
pub(crate) mod u32;
pub(crate) mod u64;
pub(crate) mod u8;
