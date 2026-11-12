//! Whether a merge would be predictable (merge or radix sort, journal §8).

/// Whether a branchy merge of the sorted runs `run[..mid]` and `run[mid..]`
/// is predictable, from its first 64 steps: few switches between the runs
/// (long stretches), or a sequence of choices (left or right run) periodic
/// with a period of at most 16 (runs holding the same values alternate). A
/// branch predictor learns such patterns (about 1 ns per element and per
/// merge round measured); on irregular interleavings (random runs) it
/// mispredicts about one step in two (about 3.5-4 ns per element and per
/// round, as much as a whole radix sort).
pub(crate) fn predictable_merge<T: Copy + PartialOrd>(run: &[T], mid: usize) -> bool {
    let (mut i, mut j) = (0, mid);
    let mut bits = 0u64;
    let mut steps = 0;
    while steps < 64 && i < mid && j < run.len() {
        let take_left = run[i] <= run[j];
        bits |= u64::from(take_left) << steps;
        if take_left {
            i += 1;
        } else {
            j += 1;
        }
        steps += 1;
    }
    if steps < 64 {
        return true;
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}
