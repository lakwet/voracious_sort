//! Already sorted arrays: sorted, reversed or constant ones found in one
//! scan (idea from turbosort's presorted check), then the repair of nearly
//! sorted ones. What a failed scan learned is passed on.

use super::near_sorted::{repair_ascending, repair_ascending_from};
use super::verge::{explore_forward_asc, explore_forward_desc};

/// From this size the direction of a descending-ended array is first
/// checked on spread pairs (below, a direct scan costs less).
const PROBE_MIN: usize = 2048;
/// Most spread pairs against a direction for a nearly sorted array.
const AGAINST_MAX: usize = 8;

/// Directions of 32 spread adjacent pairs (the others are equal).
#[derive(Clone, Copy)]
pub(crate) struct Probes {
    pub(crate) ascending: usize,
    pub(crate) descending: usize,
}

impl Probes {
    /// Equal pairs among the 32.
    #[inline]
    pub(crate) fn equal(self) -> usize {
        32 - self.ascending - self.descending
    }
}

/// What the presorted check learned about an unsorted array.
#[derive(Clone, Copy, Default)]
pub(crate) struct Unsorted {
    /// Length of the prefix known to be ascending (0: unknown).
    pub(crate) ascending_prefix: usize,
    /// The spread pairs, when compared.
    pub(crate) probes: Option<Probes>,
}

/// Compares the 32 spread adjacent pairs (at least 64 keys).
#[inline]
pub(crate) fn probe_pairs<T: PartialOrd>(arr: &[T]) -> Probes {
    let step = (arr.len() - 1) / 32;
    let (mut ascending, mut descending) = (0, 0);
    for k in 0..32 {
        let i = k * step;
        if arr[i] < arr[i + 1] {
            ascending += 1;
        } else if arr[i] > arr[i + 1] {
            descending += 1;
        }
    }
    Probes { ascending, descending }
}

/// Sorts `arr` if it is constant, ascending or descending (reversed): one
/// scan in the direction its ends suggest. Spread probes are avoided before
/// long ascending scans (read just before, they slow it down from ~1M keys);
/// a descending array is reversed first, then scanned ascending, so that a
/// failed scan still leaves a known ascending prefix.
#[inline]
pub(crate) fn presorted<T: Copy + PartialOrd>(arr: &mut [T]) -> Result<(), Unsorted> {
    let n = arr.len();
    let (first, last) = (arr[0], arr[n - 1]);
    if first == last && first == arr[n / 2] && all_equal(arr, first) {
        return Ok(());
    }
    if first <= last {
        return scan_ascending(arr, None);
    }
    if n < PROBE_MIN {
        if explore_forward_desc(arr, 0) == n {
            arr.reverse();
            return Ok(());
        }
        return Err(Unsorted::default());
    }
    let probes = probe_pairs(arr);
    if probes.ascending > 0 {
        return Err(Unsorted { ascending_prefix: 0, probes: Some(probes) });
    }
    arr.reverse();
    let reversed = Probes { ascending: probes.descending, descending: probes.ascending };
    scan_ascending(arr, Some(reversed))
}

#[inline(always)]
fn scan_ascending<T: PartialOrd>(arr: &[T], probes: Option<Probes>) -> Result<(), Unsorted> {
    let prefix = explore_forward_asc(arr, 0);
    if prefix == arr.len() {
        Ok(())
    } else {
        Err(Unsorted { ascending_prefix: prefix, probes })
    }
}

/// Whether every key equals `first`: blocks of 64 compared without early
/// exit inside a block (vectorized), an exit between blocks.
#[inline]
fn all_equal<T: Copy + PartialEq>(arr: &[T], first: T) -> bool {
    let mut blocks = arr.chunks_exact(64);
    for block in &mut blocks {
        if !block.iter().fold(true, |all, &v| all & (v == first)) {
            return false;
        }
    }
    blocks.remainder().iter().all(|&v| v == first)
}

/// Repairs `arr` if nearly sorted in either direction (a few misplaced
/// elements), as seen on the 32 spread pairs; returns whether it is now
/// sorted, and the number of equal pairs (read by the plateaus detector).
/// Below 64 keys: no repair.
pub(crate) fn repair_nearly_sorted<T: Copy + PartialOrd>(arr: &mut [T], known: Unsorted) -> (bool, usize) {
    let n = arr.len();
    if n < 64 {
        return (false, 0);
    }
    let probes = known.probes.unwrap_or_else(|| probe_pairs(arr));
    let Probes { ascending, descending } = probes;
    // Equal pairs fit both directions: the unequal pairs decide, or the ends.
    let mostly_descending =
        ascending <= AGAINST_MAX && (descending > ascending || (descending == ascending && arr[0] > arr[n - 1]));
    if mostly_descending {
        arr.reverse();
        return (repair_ascending(arr), probes.equal());
    }
    if descending > AGAINST_MAX {
        return (false, probes.equal());
    }
    (repair_ascending_from(arr, known.ascending_prefix), probes.equal())
}
