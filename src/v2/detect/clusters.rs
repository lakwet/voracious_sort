//! Tight clusters: where the DLSD's prefix would be sized for a few pairs
//! of keys, the LSD is used instead.

use super::super::integers::keys::Key;

/// Tight clusters (many groups of nearby keys spread over the whole
/// range): on a sorted spread sample, the bits that separate neighbours
/// are few for most pairs (the cluster) and many for the pairs that fall
/// in the same cluster. The DLSD sizes its prefix on the 95th percentile of
/// those needs, so every key pays the passes that a few pairs need, and the
/// groups left to its finish hold a whole cluster: when the 95th percentile
/// needs more than two 11-bit digits above the median, the LSD is used
/// instead (u64 Clusters at 10k-600k: 5.8-8.4 to 4.8-6.4 ns per key; every
/// other generator: at most 7 bits between the two).
/// `sample`: sorted spread sample of the keys (encoded or raw: flipping the
/// sign bit of both keys of a pair leaves their XOR unchanged), the one the
/// DLSD sizes its prefix on (`dlsd_unless`).
pub(crate) fn clustered<U: Key>(sample: &[U]) -> bool {
    let mut need: Vec<u32> = Vec::with_capacity(sample.len());
    for p in sample.windows(2) {
        let (a, b) = (p[0].to_u64(), p[1].to_u64());
        if a != b {
            need.push((a ^ b).leading_zeros());
        }
    }
    let m = need.len();
    if m < 8 {
        return false;
    }
    let need = &mut need[..];
    // `need` holds the bits each pair shares from the top (the bits it needs
    // to be told apart, minus one): its 95th percentile against its median.
    let shared_p95 = *need.select_nth_unstable(m * 95 / 100).1;
    let shared_p50 = *need.select_nth_unstable(m / 2).1;
    shared_p95 > shared_p50 + 22
}
