//! Heavy hitters: a few values holding a large share of the keys.

use super::{spread, DENSE_MIN, SAMPLE};

/// Heavy hitters: a few values holding a large share of the keys (sparse
/// data full of zeros, default values, a few popular items among many
/// others). From a sorted sample of `SAMPLE` keys, the values seen at
/// least `SAMPLE / 8` times (at most 4) are taken when together they
/// hold a quarter of the sample. One branch-free pass then moves the other
/// keys to the front (in place, in order) and counts each heavy value; only
/// those other keys are sorted (`sort`), and the runs of heavy values are
/// inserted back from the end (block moves and fills). Wrong guesses (the
/// sample over-represents them) still sort correctly, at the cost of the
/// pass. Same idea as the equality buckets of IPS4o (Axtmann et al., 2017).
/// `eq` and `lt`: the equality and order of the keys (floats: same bits and
/// `total_cmp`, so that -0.0 and +0.0 stay apart).
pub(crate) fn heavy_by<T: Copy, E: Fn(T, T) -> bool + Copy, L: Fn(T, T) -> bool + Copy>(arr: &mut [T], sort: &dyn Fn(&mut [T]), eq: E, lt: L) -> bool {
    heavy_from(arr, DENSE_MIN, sort, eq, lt)
}

/// `heavy_by` from `min` keys (wider keys: each comparison and move costs
/// more, the extraction pays off on shorter arrays).
pub(crate) fn heavy_from<T: Copy, E: Fn(T, T) -> bool + Copy, L: Fn(T, T) -> bool + Copy>(arr: &mut [T], min: usize, sort: &dyn Fn(&mut [T]), eq: E, lt: L) -> bool {
    if arr.len() < min {
        return false;
    }
    let mut sample = spread(arr);
    sample.sort_unstable_by(|a, b| {
        if lt(*a, *b) {
            std::cmp::Ordering::Less
        } else if lt(*b, *a) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    heavy_sorted_by(arr, &sample, min, sort, eq, lt)
}

/// `heavy_by` on a spread sample already sorted (by `lt`).
pub(crate) fn heavy_sorted_by<T: Copy, E: Fn(T, T) -> bool + Copy, L: Fn(T, T) -> bool + Copy>(arr: &mut [T], sample: &[T; SAMPLE], min: usize, sort: &dyn Fn(&mut [T]), eq: E, lt: L) -> bool {
    let n = arr.len();
    if n < min {
        return false;
    }
    let mut values = [arr[0]; 4];
    let (mut h, mut held, mut i) = (0, 0, 0);
    while i < SAMPLE {
        let mut j = i + 1;
        while j < SAMPLE && eq(sample[j], sample[i]) {
            j += 1;
        }
        if j - i >= SAMPLE / 8 && h < 4 {
            values[h] = sample[i];
            h += 1;
            held += j - i;
        }
        i = j;
    }
    if 4 * held < SAMPLE {
        return false;
    }
    let (rest, counts) = match h {
        1 => compact::<T, E, 1>(arr, values, eq),
        2 => compact::<T, E, 2>(arr, values, eq),
        3 => compact::<T, E, 3>(arr, values, eq),
        _ => compact::<T, E, 4>(arr, values, eq),
    };
    sort(&mut arr[..rest]);
    // Heavy values in ascending order (the sample was sorted), inserted from
    // the largest: the keys above it move to the end, then its run.
    let (mut end, mut dst) = (rest, n);
    for v in (0..h).rev() {
        let value = values[v];
        let p = arr[..end].partition_point(|x| lt(*x, value));
        arr.copy_within(p..end, dst - (end - p));
        dst -= end - p;
        arr[dst - counts[v]..dst].fill(value);
        dst -= counts[v];
        end = p;
    }
    debug_assert_eq!(dst, end);
    true
}

/// Moves the keys not among `values[..H]` to the front (in order, in place)
/// and counts each value: one pass without branches.
#[inline]
fn compact<T: Copy, E: Fn(T, T) -> bool, const H: usize>(arr: &mut [T], values: [T; 4], eq: E) -> (usize, [usize; 4]) {
    let mut counts = [0usize; 4];
    let mut j = 0;
    for i in 0..arr.len() {
        let k = arr[i];
        let mut hit = false;
        for v in 0..H {
            let e = eq(k, values[v]);
            counts[v] += e as usize;
            hit |= e;
        }
        arr[j] = k;
        j += (!hit) as usize;
    }
    (j, counts)
}
