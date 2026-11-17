//! i128: copy of `inspect::presorted::presorted` (v1 lost by
//! 12-18 % on sorted u128 / i128): the ascending scan is the v1 one, 4
//! pairs at a time with an exit at the first descent.

use crate::v2::inspect::presorted::{probe_pairs, Probes, Unsorted};
use crate::v2::runs::verge::explore_forward_desc;

/// Same as in `inspect::presorted`.
const PROBE_MIN: usize = 2048;

/// See `inspect::presorted::presorted`.
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
    let prefix = forward_asc(arr);
    if prefix == arr.len() {
        Ok(())
    } else {
        Err(Unsorted { ascending_prefix: prefix, probes })
    }
}

/// The v1 ascending exploration from 0 (`algo::verge_sort_heuristic`):
/// first index `j` with `arr[j - 1] > arr[j]`, or `arr.len()`.
#[inline]
fn forward_asc<T: PartialOrd>(arr: &[T]) -> usize {
    let n = arr.len();
    if n < 2 {
        return n;
    }
    let mut i = 0;
    while i + 4 < n {
        let b0 = arr[i] <= arr[i + 1];
        let b1 = arr[i + 1] <= arr[i + 2] && b0;
        let b2 = arr[i + 2] <= arr[i + 3] && b1;
        let b3 = arr[i + 3] <= arr[i + 4] && b2;
        if !b3 {
            return if b2 {
                i + 4
            } else if b1 {
                i + 3
            } else if b0 {
                i + 2
            } else {
                i + 1
            };
        }
        i += 4;
    }
    while i + 1 < n {
        if arr[i] <= arr[i + 1] {
            i += 1;
        } else {
            return i + 1;
        }
    }
    n
}

/// Same as in `inspect::presorted`.
#[inline]
fn all_equal<T: Copy + PartialEq>(arr: &[T], first: T) -> bool {
    let (blocks, rest) = arr.as_chunks::<64>();
    for block in blocks {
        if !block.iter().fold(true, |all, &v| all & (v == first)) {
            return false;
        }
    }
    rest.iter().all(|&v| v == first)
}

#[cfg(test)]
mod tests {
    use super::forward_asc;

    #[test]
    fn forward_asc_matches_scan() {
        for n in 0..40usize {
            for bad in 0..n {
                let mut a: Vec<u128> = (0..n as u128).collect();
                a[bad] = 1000;
                if bad + 1 < n {
                    let want = crate::v2::runs::verge::explore_forward_asc(&a, 0);
                    assert_eq!(forward_asc(&a), want, "n {} bad {}", n, bad);
                }
            }
            let a: Vec<u128> = (0..n as u128).collect();
            assert_eq!(forward_asc(&a), n);
        }
    }
}
