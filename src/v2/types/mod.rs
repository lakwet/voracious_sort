//! One decision chain per key type: everything that decides how an array of
//! that type is sorted (small arrays, presorted check, detector order, radix
//! path rules, engine of each chunk, thresholds). A fix for one type is made
//! in its file only, so no other type moves; a type that needs a changed
//! piece of shared code gets a folder with its copy (`u128/presorted.rs`).
//! The method: copy, route, change the copy only; factor what is shared
//! once every type is done.
//!
//! Shared by all (not decisions): the engines (`engines`), the detectors
//! themselves (`special`), the inspections (`inspect`, `decide::shape`,
//! the helpers of `decide::path`), runs and merges (`runs`), the key
//! encodings (`sorts::devour::keys`, `sorts::ravenous`).
//!
//! Inventory, against the shared chain the files were copied from:
//!
//! | file        | differences                                             |
//! |-------------|---------------------------------------------------------|
//! | u8, i8      | byte counting sort after the order check                |
//! | u16, i16    | no `dominant` (slower there)                            |
//! | u32, i32    | none                                                    |
//! | u64, i64    | none                                                    |
//! | u128, i128  | own `presorted` (v1 scan by 4 pairs)                    |
//! | f32, f64    | none                                                    |
//! | char        | none (its own copy of the u32 chain)                    |
//! | bool        | copy of the v1 boolean sort (count, two fills)          |
//!
//! `usize` / `isize` go to the type of their width (`sorts::devour`).
//!
//! Records (`records`): one folder per key type (bool, u8 ... i128, f32,
//! f64, char), one chain per record size in it (r1, r2, r4, r8, r16, r32,
//! r64; from the key's own size up). Chains of at most 8 bytes: copies of
//! the small-records path; above: copies of the large-records chain. No
//! differences yet. The pieces they share re-sort through the caller's
//! chain (`ends`, `heavy_keys`, Verge take it as an argument).

pub(crate) mod bool;
pub(crate) mod char;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i128;
pub(crate) mod i16;
pub(crate) mod i32;
pub(crate) mod i64;
pub(crate) mod i8;
#[macro_use]
pub(crate) mod narrow16;
pub(crate) mod records;
pub(crate) mod u128;
pub(crate) mod u16;
pub(crate) mod u32;
pub(crate) mod u64;
pub(crate) mod u8;

#[cfg(test)] mod tests;
