//! Sorting elements (structs) by a key, stably.
//!
//! Each key is encoded into an unsigned integer with the same order
//! (`SortKey`). One read of the keys gives their min, max and whether they
//! are already in order. Then:
//! - ascending: nothing to do; strictly descending: reversed (no equal
//!   keys, so reversing keeps the sort stable);
//! - few distinct keys (at most `FEW_MAX`): a stable counting sort, the
//!   keys counted in a hash table, the elements moved once to their place;
//! - else the key and the element's index are packed into one integer,
//!   `(key - min) << index_bits | index`, the index on log2(n) bits (33 from
//!   4.29 billion elements), in a u64 when key and index fit, else a u128;
//!   the packed integers are sorted by `devour_sort2` (equal keys keep their
//!   index order: stable) and the elements gathered in that order.
//! Keys too wide for a u128 with their index fall back to `sort_by`.

use super::devour_sort2::devour_sort2;
use std::mem::MaybeUninit;

/// A key encoded into an unsigned integer with the same order.
pub trait SortKey: Copy {
    fn encode(self) -> u128;
}

macro_rules! key_unsigned {
    ($($t:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                self as u128
            }
        }
    )*};
}
key_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! key_signed {
    ($($t:ty => $u:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                ((self as $u) ^ (1 << (<$u>::BITS - 1))) as u128
            }
        }
    )*};
}
key_signed!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128, isize => usize);

impl SortKey for char {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u32 as u128
    }
}

impl SortKey for bool {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u128
    }
}

// Floats in total order (`total_cmp`): -0.0 before +0.0, NaN not handled.
impl SortKey for f32 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i32) >> 31) as u32) | (1 << 31))) as u128
    }
}

impl SortKey for f64 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i64) >> 63) as u64) | (1 << 63))) as u128
    }
}

/// Tests / benchmarks: switches (bit 1: no presorted check, bit 2: no
/// few-keys count, bit 4: gather through a buffer instead of the in-place
/// permutation, bit 8: no Verge on the elements).
#[cfg(test)]
pub(crate) static BY_KEY_OFF: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(always)]
fn off(bit: usize) -> bool {
    #[cfg(test)]
    {
        BY_KEY_OFF.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
    }
    #[cfg(not(test))]
    {
        let _ = bit;
        false
    }
}

/// Sorts `arr` by `key`, stably.
pub fn devour_sort_by_key<T, K: SortKey, F: Fn(&T) -> K>(arr: &mut [T], key: F) {
    sort_enc(arr, &|x: &T| key(x).encode());
}

/// Direction of a pair of keys: -1 descending, 0 equal, 1 ascending.
#[inline(always)]
fn direction(a: u128, b: u128) -> i8 {
    (b > a) as i8 - (b < a) as i8
}

fn sort_enc<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    // Elements no larger than a packed (key, index) and beyond the cache:
    // the LSD on the elements, its byte histograms taken in this read.
    let size = std::mem::size_of::<T>();
    // Beyond half the L2: the moves of the elements dominate (LSD on small
    // elements, Verge on the elements, both counted in this read).
    let beyond = n.saturating_mul(size) > super::cache_info::l2_bytes() / 2;
    let lsd_regime = lsd_forced() || (size <= 8 && beyond);
    let mut hist = if lsd_regime { vec![[0usize; 256]; 8] } else { Vec::new() };
    let first = enc(&arr[0]);
    let (mut lo, mut hi, mut prev) = (first, first, first);
    let (mut asc, mut desc) = (true, true);
    // Turns: changes of direction between neighbouring pairs (equal keys
    // keep the direction): about 2n/3 on random keys, one per tooth on
    // sawtooth data, one on a pipe organ.
    let (mut turns, mut last) = (0usize, 0i8);
    if lsd_regime {
        let count = |hist: &mut [[usize; 256]], k: u128| {
            for (d, h) in hist.iter_mut().enumerate() {
                h[((k as u64 >> (8 * d)) & 0xFF) as usize] += 1;
            }
        };
        count(&mut hist, first);
        for x in arr[1..].iter() {
            let k = enc(x);
            lo = lo.min(k);
            hi = hi.max(k);
            asc &= prev <= k;
            desc &= prev > k;
            let d = direction(prev, k);
            turns += (d != 0 && d != last && last != 0) as usize;
            last = if d != 0 { d } else { last };
            prev = k;
            count(&mut hist, k);
        }
    } else if beyond {
        for x in arr[1..].iter() {
            let k = enc(x);
            lo = lo.min(k);
            hi = hi.max(k);
            asc &= prev <= k;
            desc &= prev > k;
            let d = direction(prev, k);
            turns += (d != 0 && d != last && last != 0) as usize;
            last = if d != 0 { d } else { last };
            prev = k;
        }
    } else {
        // In cache: the turns are not used (no Verge on the elements).
        for x in arr[1..].iter() {
            let k = enc(x);
            lo = lo.min(k);
            hi = hi.max(k);
            asc &= prev <= k;
            desc &= prev > k;
            prev = k;
        }
    }
    if !off(1) {
        if asc {
            return;
        }
        if desc {
            arr.reverse();
            return;
        }
    }
    if !off(2) && few_keys(arr, enc) {
        return;
    }
    // Few turns: long runs likely; Verge on the elements (runs kept, the
    // gaps sorted, everything merged).
    if !off(8) && beyond && n >= VERGE_MIN && 8 * turns <= n && verge(arr, enc) {
        return;
    }
    let key_bits = u128::BITS - (hi - lo).leading_zeros();
    if lsd_regime && hi <= u64::MAX as u128 {
        lsd(arr, enc, &hist);
        return;
    }
    // In the cache the gather through a buffer is cheaper than scanning the
    // permutation first (8-byte records at 10k-100k: 0.65-0.68 against
    // 0.75-0.77 of the stable std sort); beyond it, the in-place
    // permutation (see `permute`).
    if !off(4) && beyond {
        if !sort_in_place(arr, enc, lo, key_bits) {
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
        }
        return;
    }
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the elements are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buf.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, enc, lo, key_bits) {
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}

/// The packed (key, index) sort, then the permutation applied in place by
/// following its cycles: an element already at its place is not touched,
/// the others are moved once (each write lands on the slot just read). No
/// buffer of elements. Returns false (nothing moved) when key and index do
/// not fit in a u128.
fn sort_in_place<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, lo: u128, key_bits: u32) -> bool {
    let n = arr.len();
    let index_bits = usize::BITS - n.saturating_sub(1).leading_zeros();
    if key_bits + index_bits <= 64 {
        let mut packed: Vec<u64> = arr.iter().enumerate().map(|(i, x)| (((enc(x) - lo) << index_bits) as u64) | i as u64).collect();
        devour_sort2(&mut packed);
        let mask = (1u64 << index_bits).wrapping_sub(1);
        permute(arr, &mut packed, |p| (p & mask) as usize, |p, i| *p = (*p & !mask) | i as u64);
        true
    } else if key_bits + index_bits <= 128 {
        let mut packed: Vec<u128> = arr.iter().enumerate().map(|(i, x)| ((enc(x) - lo) << index_bits) | i as u128).collect();
        devour_sort2(&mut packed);
        let mask = (1u128 << index_bits).wrapping_sub(1);
        permute(arr, &mut packed, |p| (p & mask) as usize, |p, i| *p = (*p & !mask) | i as u128);
        true
    } else {
        false
    }
}

/// A move within this many slots counts as near.
const NEAR: usize = 64;

/// Puts `arr[index(perm[j])]` at position `j` for every `j`. Few elements
/// moving far (at most a quarter beyond `NEAR` slots): in place, cycle by
/// cycle, an element at its place not touched, near moves read nearly in
/// sequence (nearly sorted data, shifted blocks). Else gathered through a
/// buffer over the span that moves: following a cycle chains its random
/// reads one after another, while the gather issues them independently
/// (random records of 32 bytes at 1M: 103 against 27 ns per key).
fn permute<T, P: Copy>(arr: &mut [T], perm: &mut [P], index: impl Fn(P) -> usize, set: impl Fn(&mut P, usize)) {
    let n = arr.len();
    let (mut far, mut first, mut last) = (0usize, n, 0usize);
    for (j, &p) in perm.iter().enumerate() {
        let src = index(p);
        if src != j {
            far += (src.abs_diff(j) > NEAR) as usize;
            first = first.min(j);
            last = j;
        }
    }
    if first == n {
        return;
    }
    let a = arr.as_mut_ptr();
    if 4 * far > n {
        // The span [first, last] is closed under the permutation (the
        // slots outside it are fixed points).
        let span = last + 1 - first;
        let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(span);
        // SAFETY: every element of the span is read once (a permutation of
        // the span) into the buffer, then the buffer is copied back over the
        // span: each element moved once there and back, none dropped.
        unsafe {
            let b = buf.as_mut_ptr() as *mut T;
            for j in first..=last {
                b.add(j - first).write(std::ptr::read(a.add(index(perm[j]))));
            }
            std::ptr::copy_nonoverlapping(b, a.add(first), span);
        }
        return;
    }
    for start in first..=last {
        if index(perm[start]) == start {
            continue;
        }
        // SAFETY: `perm` is a permutation of 0..n. Along a cycle, the
        // element taken out of `start` is held in `held`; each slot `k`
        // receives the element of `src = index(perm[k])` (still unmoved:
        // a cycle visits each slot once), then is marked done; the cycle
        // closes when `src` comes back to `start`, which receives `held`.
        // Every element is moved exactly once, none dropped or duplicated;
        // no code that can panic runs while `held` is out.
        unsafe {
            let held = std::ptr::read(a.add(start));
            let mut k = start;
            loop {
                let src = index(perm[k]);
                set(&mut perm[k], k);
                if src == start {
                    a.add(k).write(held);
                    break;
                }
                a.add(k).write(std::ptr::read(a.add(src)));
                k = src;
            }
        }
    }
}

/// Tests / benchmarks: the LSD on the elements forced (1) or the packed
/// sort forced (2); 0: the rule.
#[cfg(test)]
pub(crate) static BY_KEY_ENGINE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(always)]
fn lsd_forced() -> bool {
    #[cfg(test)]
    {
        BY_KEY_ENGINE.load(std::sync::atomic::Ordering::Relaxed) == 1
    }
    #[cfg(not(test))]
    {
        false
    }
}

/// Stable LSD on the elements themselves: byte digits of the encoded key
/// (histograms `hist` taken in the first read), the digits equal in every
/// key skipped, each pass moving the elements between the array and a
/// buffer (the key read again from each element).
fn lsd<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, hist: &[[usize; 256]]) {
    let n = arr.len();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    let mut in_buf = false;
    // SAFETY: each pass moves every element once from one side to the
    // other (the heads cover 0..n exactly); at the end the elements are
    // copied back if they ended in the buffer. Nothing is dropped twice
    // (the buffer is MaybeUninit); `enc` was already called on every
    // element without panicking.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        for (d, h) in hist.iter().enumerate() {
            if h.iter().any(|&c| c == n) {
                continue;
            }
            let mut heads = [0usize; 256];
            let mut s = 0;
            for (hd, &c) in heads.iter_mut().zip(h.iter()) {
                *hd = s;
                s += c;
            }
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            for i in 0..n {
                let x = &*src.add(i);
                let digit = ((enc(x) as u64 >> (8 * d)) & 0xFF) as usize;
                dst.add(heads[digit]).write(std::ptr::read(x));
                heads[digit] += 1;
            }
            in_buf = !in_buf;
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
}

/// Moves the `n` elements at `src` to `dst` in key order (stable): the
/// (key - lo, index) pairs packed in one integer and sorted. Returns false
/// (nothing moved) when key and index do not fit in a u128.
///
/// # Safety
/// `src` and `dst` point to `n` elements, do not overlap; the elements of
/// `src` are moved out (bitwise) and must be treated as moved by the caller.
unsafe fn sort_into<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, n: usize, enc: &E, lo: u128, key_bits: u32) -> bool {
    let index_bits = usize::BITS - n.saturating_sub(1).leading_zeros();
    if key_bits + index_bits <= 64 {
        let mut packed: Vec<u64> = (0..n).map(|i| (((enc(&*src.add(i)) - lo) << index_bits) as u64) | i as u64).collect();
        devour_sort2(&mut packed);
        let mask = (1u64 << index_bits).wrapping_sub(1);
        for (j, &p) in packed.iter().enumerate() {
            dst.add(j).write(std::ptr::read(src.add((p & mask) as usize)));
        }
        true
    } else if key_bits + index_bits <= 128 {
        let mut packed: Vec<u128> = (0..n).map(|i| ((enc(&*src.add(i)) - lo) << index_bits) | i as u128).collect();
        devour_sort2(&mut packed);
        let mask = (1u128 << index_bits).wrapping_sub(1);
        for (j, &p) in packed.iter().enumerate() {
            dst.add(j).write(std::ptr::read(src.add((p & mask) as usize)));
        }
        true
    } else {
        false
    }
}

/// Most distinct keys counted, and the hash table size (at most half full).
const FEW_MAX: usize = 256;
const FEW_TABLE_BITS: u32 = 9;
const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;

/// Few distinct keys: a spread sample gives up as soon as half of it is
/// distinct; else one pass counts the keys in a hash table (giving up
/// beyond `FEW_MAX`, nothing moved), the distinct keys are sorted, and one
/// pass moves every element to its key's next slot in a buffer (stable),
/// copied back.
fn few_keys<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    let n = arr.len();
    if n < 128 {
        return false;
    }
    let hash = |e: u128| (((e as u64) ^ ((e >> 64) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - FEW_TABLE_BITS)) as usize;
    let mut keys = [0u128; FEW_TABLE];
    let mut used = [false; FEW_TABLE];
    let find = |e: u128, keys: &mut [u128; FEW_TABLE], used: &mut [bool; FEW_TABLE]| -> (usize, bool) {
        let mut h = hash(e);
        while used[h] && keys[h] != e {
            h = (h + 1) & (FEW_TABLE - 1);
        }
        let new = !used[h];
        if new {
            used[h] = true;
            keys[h] = e;
        }
        (h, new)
    };
    let s = (n / 32).clamp(16, 256);
    let mut distinct = 0;
    for i in 0..s as u64 {
        let e = enc(&arr[((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64 >> 32) as usize]);
        if find(e, &mut keys, &mut used).1 {
            distinct += 1;
            if distinct > s / 2 {
                return false;
            }
        }
    }
    let mut counts = [0usize; FEW_TABLE];
    for x in arr.iter() {
        let (h, new) = find(enc(x), &mut keys, &mut used);
        if new {
            distinct += 1;
            if distinct > FEW_MAX {
                return false;
            }
        }
        counts[h] += 1;
    }
    // Slots in key order, then the start of each slot's run.
    let mut order: Vec<usize> = (0..FEW_TABLE).filter(|&h| counts[h] != 0).collect();
    order.sort_unstable_by_key(|&h| keys[h]);
    let mut heads = [0usize; FEW_TABLE];
    let mut start = 0;
    for &h in order.iter() {
        heads[h] = start;
        start += counts[h];
    }
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: each element is moved once into the buffer at its key's next
    // slot (the runs cover 0..n exactly), then the buffer is copied back
    // over the array. The lookups only find keys already in the table.
    unsafe {
        let src = arr.as_ptr();
        let dst = buf.as_mut_ptr() as *mut T;
        for i in 0..n {
            let x = &*src.add(i);
            let (h, _) = find(enc(x), &mut keys, &mut used);
            dst.add(heads[h]).write(std::ptr::read(x));
            heads[h] += 1;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
    true
}

/// Shortest array tried for Verge on the elements.
const VERGE_MIN: usize = 4096;

/// Verge on the elements: runs of at least n / log2(n) keys (ascending, or
/// strictly descending and then reversed: still stable) are kept, the gaps
/// between them sorted, and all the segments merged stably. Returns false
/// (nothing done) without such a run.
fn verge<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    let n = arr.len();
    let long = super::super::algo::verge_sort_heuristic::compute_big_enough_run(n).max(2);
    // Runs: (start, end, descending).
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    let mut i = 0;
    while i + 1 < n {
        let start = i;
        let desc = enc(&arr[i + 1]) < enc(&arr[i]);
        let mut j = i + 1;
        if desc {
            while j < n && enc(&arr[j]) < enc(&arr[j - 1]) {
                j += 1;
            }
        } else {
            while j < n && enc(&arr[j]) >= enc(&arr[j - 1]) {
                j += 1;
            }
        }
        if j - start >= long {
            runs.push((start, j, desc));
        }
        i = j;
    }
    if runs.is_empty() {
        return false;
    }
    // Two long runs interleaving unpredictably (sorted random sources):
    // merging costs more than sorting by key (the merge rule of the radix
    // path, on the first 64 steps of their merge).
    if runs.len() >= 2 && !predictable(arr, runs[0], runs[1], enc) {
        return false;
    }
    // Segments: the runs and the gaps between them, in order.
    let mut bounds = vec![0];
    for &(s, e, desc) in runs.iter() {
        let at = *bounds.last().unwrap();
        if s > at {
            sort_enc(&mut arr[at..s], enc);
            bounds.push(s);
        }
        if desc {
            arr[s..e].reverse();
        }
        bounds.push(e);
    }
    let at = *bounds.last().unwrap();
    if at < n {
        sort_enc(&mut arr[at..], enc);
        bounds.push(n);
    }
    if bounds.len() == 3 {
        merge_two(arr, bounds[1], enc);
    } else {
        merge_all(arr, &bounds, enc);
    }
    true
}

/// Whether merging two runs would be predictable (same rule as
/// `predictable_merge`): on the first 64 steps of their merge (runs read in
/// increasing order), at most 16 switches between the two runs, or a
/// pattern periodic with a period of at most 16 (pipe organs alternate one
/// by one).
fn predictable<T, E: Fn(&T) -> u128>(arr: &[T], a: (usize, usize, bool), b: (usize, usize, bool), enc: &E) -> bool {
    let at = |r: (usize, usize, bool), i: usize| if r.2 { enc(&arr[r.1 - 1 - i]) } else { enc(&arr[r.0 + i]) };
    let (la, lb) = (a.1 - a.0, b.1 - b.0);
    let (mut i, mut j, mut bits) = (0, 0, 0u64);
    for step in 0..64 {
        if i == la || j == lb {
            return true;
        }
        let left = at(a, i) <= at(b, j);
        bits |= (left as u64) << step;
        if left {
            i += 1;
        } else {
            j += 1;
        }
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer the size of the
/// shorter one: each element moved once (the shorter run copied out, the
/// merge writing from the end or the start into the array).
fn merge_two<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E) {
    let n = arr.len();
    if m == 0 || m == n || enc(&arr[m - 1]) <= enc(&arr[m]) {
        return;
    }
    let a = arr.as_mut_ptr();
    // SAFETY: the shorter run is moved out to the buffer; the merge then
    // fills the array from the side where the moved-out run was, never
    // overtaking the unread elements of the other run (the write position
    // is always at least as far from that side as the read position), and
    // every element is moved once; nothing is dropped or duplicated. `enc`
    // already ran on every element.
    unsafe {
        if n - m <= m {
            // Right run out; merge from the end.
            let len = n - m;
            let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a.add(m), b, len);
            let (mut i, mut j, mut k) = (m, len, n);
            while i > 0 && j > 0 {
                // Stable: on equal keys the right element goes last.
                let take_left = enc(&*a.add(i - 1)) > enc(&*b.add(j - 1));
                k -= 1;
                if take_left {
                    i -= 1;
                    a.add(k).write(std::ptr::read(a.add(i)));
                } else {
                    j -= 1;
                    a.add(k).write(std::ptr::read(b.add(j)));
                }
            }
            std::ptr::copy_nonoverlapping(b, a.add(k - j), j);
        } else {
            // Left run out; merge from the start.
            let len = m;
            let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a, b, len);
            let (mut i, mut j, mut k) = (0, m, 0);
            while i < len && j < n {
                // Stable: on equal keys the left element goes first.
                let take_right = enc(&*a.add(j)) < enc(&*b.add(i));
                if take_right {
                    a.add(k).write(std::ptr::read(a.add(j)));
                    j += 1;
                } else {
                    a.add(k).write(std::ptr::read(b.add(i)));
                    i += 1;
                }
                k += 1;
            }
            std::ptr::copy_nonoverlapping(b.add(i), a.add(k), len - i);
        }
    }
}

/// Stable merge of the sorted segments `bounds[i]..bounds[i + 1]`, pairs of
/// neighbours per round, alternating between the array and a buffer.
fn merge_all<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E) {
    let n = arr.len();
    if bounds.len() <= 2 {
        return;
    }
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    let mut in_buf = false;
    // SAFETY: each round moves every element exactly once from one side to
    // the other (each segment pair merged or copied into the same range);
    // at the end the elements are copied back if they are in the buffer.
    // Nothing is dropped or duplicated; `enc` already ran on every element.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        while bounds.len() > 2 {
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            let mut next = vec![0];
            let mut i = 0;
            while i + 2 < bounds.len() {
                let (s, m, e) = (bounds[i], bounds[i + 1], bounds[i + 2]);
                merge_into(src, dst, s, m, e, enc);
                next.push(e);
                i += 2;
            }
            if i + 1 < bounds.len() {
                let (s, e) = (bounds[i], bounds[i + 1]);
                std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
                next.push(e);
            }
            bounds = next;
            in_buf = !in_buf;
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
}

/// Stable merge of `src[s..m]` and `src[m..e]` into `dst[s..e]`.
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
unsafe fn merge_into<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m > s && m < e && enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let (mut i, mut j, mut k) = (s, m, s);
    while i < m && j < e {
        let right = enc(&*src.add(j)) < enc(&*src.add(i));
        let from = if right { j } else { i };
        dst.add(k).write(std::ptr::read(src.add(from)));
        j += right as usize;
        i += (!right) as usize;
        k += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(i), dst.add(k), m - i);
    k += m - i;
    std::ptr::copy_nonoverlapping(src.add(j), dst.add(k), e - j);
}
