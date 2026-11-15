//! Sorting records by a key, unstably: equal keys may come out in any
//! order. What the freedom buys: an in-place MSD radix sort on the records
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
    // Cycles: the record at the head of bucket `b` goes to the head of its
    // own bucket, the record found there comes back, until one belongs.
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
    let mut start = 0;
    for b in 0..256 {
        let end = tail[b];
        if end - start > 1 && blo[b] != bhi[b] {
            pass(&mut arr[start..end], key, blo[b], bhi[b]);
        }
        start = end;
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
