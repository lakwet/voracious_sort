//! Long runs of equal keys.

/// Long runs of equal keys (values held for a while, repeated blocks): when
/// at least 31 of 32 spread adjacent pairs are equal, one scan lists the
/// runs (giving up beyond n / 32 of them, nothing written), the (value,
/// length) pairs are sorted and the array rewritten: one read and one write
/// instead of a sort of n keys.
/// `equal`: how many of the 32 spread adjacent pairs of the near-sorted
/// repair are equal.
pub(crate) fn plateaus<T: Copy + Ord>(arr: &mut [T], equal: usize) -> bool {
    let n = arr.len();
    if n < 1024 || equal < 31 {
        return false;
    }
    let max = n / 32;
    let mut runs: Vec<(T, usize)> = Vec::with_capacity(max.min(4096));
    let mut start = 0;
    while start < n {
        let v = arr[start];
        let mut end = start + 1;
        // Blocks of 16 compared without exit inside (vectorized).
        while end + 16 <= n && arr[end..end + 16].iter().fold(true, |acc, &x| acc & (x == v)) {
            end += 16;
        }
        while end < n && arr[end] == v {
            end += 1;
        }
        if runs.len() == max {
            return false;
        }
        runs.push((v, end - start));
        start = end;
    }
    runs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut pos = 0;
    for &(v, len) in runs.iter() {
        arr[pos..pos + len].fill(v);
        pos += len;
    }
    true
}
