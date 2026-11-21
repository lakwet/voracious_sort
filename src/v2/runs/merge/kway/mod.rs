//! The merge of k runs (ported from the kwaymerge test bench): a
//! powersort tree over the runs, each node merged home / into the buffer /
//! by the rotation of three zones / in place, chosen by a cost in moves
//! that reads the runs' ends; buffer n/2. Its kernels: four chains with
//! rounds for the rests and per-architecture steps, hole merges by pieces
//! through `merge_out`, exponential galloping.

mod inplace;
mod kernel;
mod rotation;
mod tree;

pub(crate) use tree::hybrid;
