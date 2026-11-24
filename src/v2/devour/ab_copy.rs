//! A/B by copy (dev only, feature `ab`): a second instance of the 16-bit
//! chain, called by the measuring examples as `ab_u16_copy` / `ab_i16_copy`.
//!
//! To try a change: copy `narrow16.rs` here as a macro of another name
//! (`narrow16_ab!`), edit the copy only, and instantiate it below instead
//! of `narrow16!`; `make abench-align` then compares it with the library
//! (`ab_u16_gen`) in two code placements. Left as it is, the copy is the
//! same code in its own place: the placement noise of a separate instance.

narrow16!(cu16, u16, 0, id16);
narrow16!(ci16, i16, 0x8000, flip16);
