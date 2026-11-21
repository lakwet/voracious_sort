//! Shapes read on samples of the keys: ordered keys (the rollercoaster's
//! MSD pass then writes in few streams), tight clusters (the DLSD's prefix
//! would be sized for a few pairs).

use crate::v2::sorts::devour::keys::Key;

/// Sampled neighbours (64 pairs spread over the keys) mostly in one order:
/// sawteeth, runs.
pub(crate) fn locally_ordered<U: Copy + PartialOrd>(bits: &[U]) -> bool {
    const PAIRS: usize = 64;
    let step = bits.len() / (PAIRS + 1);
    let asc = (1..=PAIRS).filter(|&k| bits[k * step] <= bits[k * step + 1]).count();
    let desc = (1..=PAIRS).filter(|&k| bits[k * step] >= bits[k * step + 1]).count();
    asc >= PAIRS * 7 / 8 || desc >= PAIRS * 7 / 8
}

/// Sampled keys (65 spread over the slice) mostly in one order: a global
/// trend under local noise (timestamps with jitter).
pub(crate) fn globally_ordered<U: Copy + PartialOrd>(bits: &[U]) -> bool {
    const PAIRS: usize = 64;
    let step = bits.len() / (PAIRS + 1);
    let asc = (0..PAIRS).filter(|&k| bits[k * step] <= bits[(k + 1) * step]).count();
    let desc = (0..PAIRS).filter(|&k| bits[k * step] >= bits[(k + 1) * step]).count();
    asc >= PAIRS * 7 / 8 || desc >= PAIRS * 7 / 8
}

/// Tight clusters (many groups of nearby keys spread over the whole
/// range): on a sorted spread sample, the bits that separate neighbours
/// are few for most pairs (the cluster) and many for the pairs that fall
/// in the same cluster. The DLSD sizes its prefix on the 95th percentile of
/// those needs, so every key pays the passes that a few pairs need, and the
/// groups left to its finish hold a whole cluster: when the 95th percentile
/// needs more than two 11-bit digits above the median, the LSD is used
/// instead (no other distribution shows more than 7 bits between the two).
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

#[cfg(test)] mod tests;
