//! One decision chain per key type: everything that decides how an array of
//! that type is sorted (small arrays, presorted check, detector order, radix
//! path rules, engine of each chunk, thresholds). A fix for one type is made
//! in its file only, so no other type moves (method of 2026-11-17: copy,
//! route, change the copy only; factor what is shared once every type wins).
//!
//! Shared by all (not decisions): the engines (`engines`), the detectors
//! themselves (`special`), the inspections (`inspect`, `decide::shape`,
//! the helpers of `decide::path`), runs and merges (`runs`), the key
//! encodings (`sorts::devour::keys`, `sorts::ravenous`).
//!
//! Inventory, against the shared chain the files were copied from
//! (2026-11-17):
//!
//! | file        | differences                                          |
//! |-------------|------------------------------------------------------|
//! | u8, i8      | none                                                 |
//! | u16, i16    | none                                                 |
//! | u32, i32    | none                                                 |
//! | u64, i64    | none                                                 |
//! | u128, i128  | none (`presorted128` under test, behind `new_rule`)  |
//! | f32, f64    | none                                                 |
//! | char        | none (its own copy of the u32 chain)                 |
//!
//! `usize` / `isize` go to the type of their width (`sorts::devour`).
//!
//! Records (`records`): one folder per key type (bool, u8 ... i128, f32,
//! f64, char), one chain per record size in it (r1, r2, r4, r8, r16, r32,
//! r64; from the key's own size up). Chains of at most 8 bytes: copies of
//! the small-records path; above: copies of the large-records chain. No
//! differences yet. The pieces they share re-sort through the caller's
//! chain (`ends`, `heavy_keys`, Verge take it as an argument).

pub(crate) mod char;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i128;
pub(crate) mod i16;
pub(crate) mod i32;
pub(crate) mod i64;
pub(crate) mod i8;
pub(crate) mod records;
pub(crate) mod u128;
pub(crate) mod u16;
pub(crate) mod u32;
pub(crate) mod u64;
pub(crate) mod u8;
