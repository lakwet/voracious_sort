//! In-place MSD radix sort.
//!
//! `msd`: records by a key, unstably (equal keys may come out in any
//! order). What the freedom buys: an in-place MSD radix sort on the records
//! (no buffer, no (key, index) pairs): each pass counts one byte of the
//! key, then moves every record to its bucket by cycles of swaps (American
//! flag sort, McIlroy, Bostic & McIlroy 1993; the cycle loop as in
//! Skarupke's ska_sort 2016; Regions sort, Obeya et al. 2019, runs the same
//! moves in parallel), and recurses in each bucket. Own code.

/// Buckets up to this many records: sorted by comparisons.
const SMALL: usize = 64;

/// Sorts `arr` by its encoded keys by the in-place MSD radix sort, keys
/// already known to lie in `lo..=hi`.
pub(crate) fn msd<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, lo: u128, hi: u128) {
    if hi == lo {
        return;
    }
    if hi - lo <= u64::MAX as u128 {
        let key = |x: &T| (enc(x) - lo) as u64;
        pass(arr, &key, 0, (hi - lo) as u64);
    } else {
        let key = |x: &T| enc(x) - lo;
        pass(arr, &key, 0, hi - lo);
    }
}

/// Key types of the passes: u64 when the range fits.
trait Digits: Copy + Ord + std::ops::BitXor<Output = Self> {
    const ZERO: Self;
    const MAX: Self;
    fn digit(self, shift: u32) -> usize;
    fn bits(self) -> u32;
}

impl Digits for u64 {
    const ZERO: Self = 0;
    const MAX: Self = u64::MAX;
    #[inline(always)]
    fn digit(self, shift: u32) -> usize {
        ((self >> shift) & 0xFF) as usize
    }
    #[inline(always)]
    fn bits(self) -> u32 {
        64 - self.leading_zeros()
    }
}

impl Digits for u128 {
    const ZERO: Self = 0;
    const MAX: Self = u128::MAX;
    #[inline(always)]
    fn digit(self, shift: u32) -> usize {
        ((self >> shift) & 0xFF) as usize
    }
    #[inline(always)]
    fn bits(self) -> u32 {
        128 - self.leading_zeros()
    }
}

/// One MSD pass on the keys of `arr`, known to lie in `lo..=hi` (`lo <
/// hi`): the byte below their common prefix, then each bucket on its own
/// range (the count notes each bucket's smallest and largest key: a bucket
/// of equal keys is done, common bytes are skipped without a pass).
fn pass<T, K: Digits, F: Fn(&T) -> K>(arr: &mut [T], key: &F, lo: K, hi: K) {
    let n = arr.len();
    if n <= SMALL {
        insertion(arr, key);
        return;
    }
    let shift = (lo ^ hi).bits().saturating_sub(8);
    let mut count = [0usize; 256];
    let mut blo = [K::MAX; 256];
    let mut bhi = [K::ZERO; 256];
    for x in arr.iter() {
        let k = key(x);
        let d = k.digit(shift);
        count[d] += 1;
        blo[d] = blo[d].min(k);
        bhi[d] = bhi[d].max(k);
    }
    let mut head = [0usize; 256];
    let mut tail = [0usize; 256];
    let mut sum = 0;
    for b in 0..256 {
        head[b] = sum;
        sum += count[b];
        tail[b] = sum;
    }
    cycles(arr, key, shift, &mut head, &tail);
    let mut start = 0;
    for b in 0..256 {
        let end = tail[b];
        if end - start > 1 && blo[b] != bhi[b] {
            pass(&mut arr[start..end], key, blo[b], bhi[b]);
        }
        start = end;
    }
}

/// Cycles: the record at the head of bucket `b` goes to the head of its own
/// bucket, the record found there comes back, until one belongs. For records
/// the cycles beat the rounds of `permute` (128-byte records 50k-1M, Surface
/// sentinel: rounds 1.08 times slower, up to 1.6).
fn cycles<T, K: Digits, F: Fn(&T) -> K>(
    arr: &mut [T],
    key: &F,
    shift: u32,
    head: &mut [usize; 256],
    tail: &[usize; 256],
) {
    for b in 0..256 {
        while head[b] < tail[b] {
            let i = head[b];
            let mut d = key(&arr[i]).digit(shift);
            while d != b {
                let j = head[d];
                head[d] = j + 1;
                arr.swap(i, j);
                d = key(&arr[i]).digit(shift);
            }
            head[b] = i + 1;
        }
    }
}

/// Insertion sort by key (small buckets).
fn insertion<T, K: Digits, F: Fn(&T) -> K>(arr: &mut [T], key: &F) {
    for i in 1..arr.len() {
        let k = key(&arr[i]);
        let mut j = i;
        while j > 0 && key(&arr[j - 1]) > k {
            j -= 1;
        }
        if j < i {
            arr[j..=i].rotate_right(1);
        }
    }
}

/// In-place permutation of `arr` into buckets (ska sort's swap loop): bucket
/// `b` spans `heads[b]..ends[b]` (its first free slot to its end), `digit`
/// gives the bucket of an element. Rounds over the unsorted part of every
/// bucket, each element swapped to the next free slot of its bucket. The
/// largest bucket is never scanned: it is complete when the others are.
/// Used by the rollercoaster (`roller`) on keys; records take `cycles`.
#[inline(always)]
pub(crate) fn permute<T, D: Fn(&T) -> usize>(
    arr: &mut [T],
    heads: &mut [usize],
    ends: &[usize],
    digit: D,
) {
    let mut todo: Vec<usize> = (0..heads.len()).filter(|&b| ends[b] > heads[b]).collect();
    if let Some(pos) = (0..todo.len()).max_by_key(|&i| ends[todo[i]] - heads[todo[i]]) {
        todo.swap_remove(pos);
    }
    let p = arr.as_mut_ptr();
    // SAFETY: every index read or written is a slot of a bucket region,
    // below `arr.len()`: `o` runs over `heads[b]..ends[b]` and the head of a
    // bucket stays below its end while elements of that bucket remain to
    // place (the caller counted exactly the elements of each bucket).
    while !todo.is_empty() {
        todo.retain(|&b| {
            let (start, end) = (heads[b], ends[b]);
            let mut o = start;
            unsafe {
                while o + 4 <= end {
                    let d0 = digit(&*p.add(o));
                    let d1 = digit(&*p.add(o + 1));
                    let d2 = digit(&*p.add(o + 2));
                    let d3 = digit(&*p.add(o + 3));
                    if d0 == d1 && d0 == d2 && d0 == d3 {
                        // Same bucket: one counter update instead of a chain
                        // of 4 dependent ones (same swaps as below).
                        let i = heads[d0];
                        heads[d0] += 4;
                        std::ptr::swap(p.add(o), p.add(i));
                        std::ptr::swap(p.add(o + 1), p.add(i + 1));
                        std::ptr::swap(p.add(o + 2), p.add(i + 2));
                        std::ptr::swap(p.add(o + 3), p.add(i + 3));
                        o += 4;
                        continue;
                    }
                    let i0 = heads[d0];
                    heads[d0] += 1;
                    let i1 = heads[d1];
                    heads[d1] += 1;
                    let i2 = heads[d2];
                    heads[d2] += 1;
                    let i3 = heads[d3];
                    heads[d3] += 1;
                    std::ptr::swap(p.add(o), p.add(i0));
                    std::ptr::swap(p.add(o + 1), p.add(i1));
                    std::ptr::swap(p.add(o + 2), p.add(i2));
                    std::ptr::swap(p.add(o + 3), p.add(i3));
                    o += 4;
                }
                while o < end {
                    let d = digit(&*p.add(o));
                    std::ptr::swap(p.add(o), p.add(heads[d]));
                    heads[d] += 1;
                    o += 1;
                }
            }
            heads[b] < ends[b]
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::testing::*;

    /// Records (key, position) by their key: in key order, none lost. Keys
    /// of 64 bits (range within u64) and of 128 bits (wider range).
    #[test]
    fn records() {
        for_each_case::<u64>(|shape, keys| {
            let a: Vec<(u64, u32)> = keys.iter().enumerate().map(|(i, &k)| (k, i as u32)).collect();
            check(&format!("u64 {}", shape), a, |x: &(u64, u32)| x.0 as u128);
        });
        for_each_case::<u128>(|shape, keys| {
            let a: Vec<(u128, u32)> =
                keys.iter().enumerate().map(|(i, &k)| (k, i as u32)).collect();
            check(&format!("u128 {}", shape), a, |x: &(u128, u32)| x.0);
        });
    }

    fn check<T: Copy + Ord, E: Fn(&T) -> u128>(what: &str, a: Vec<T>, enc: E) {
        if a.is_empty() {
            return;
        }
        let lo = a.iter().map(&enc).min().unwrap();
        let hi = a.iter().map(&enc).max().unwrap();
        let mut got = a.clone();
        msd(&mut got, &enc, lo, hi);
        assert!(
            got.windows(2).all(|w| enc(&w[0]) <= enc(&w[1])),
            "{} n={}: not sorted",
            what,
            a.len()
        );
        assert!(same_values(&got, &a), "{} n={}: records lost", what, a.len());
    }
}
