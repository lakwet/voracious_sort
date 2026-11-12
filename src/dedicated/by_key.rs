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
//! - Arrays of elements larger than the cache first go through one MSD pass
//!   on the top bits of the key, which moves the elements in streams into
//!   buckets that fit in the cache; each bucket is then sorted as above, its
//!   gather reading the cache instead of random places in memory.
//! Keys too wide for a u128 with their index fall back to `sort_by`.

use super::cache_info::l2_bytes;
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
/// few-keys count, bit 4: no MSD pass).
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
    let n = arr.len();
    if n < 2 {
        return;
    }
    let enc = |x: &T| key(x).encode();
    let first = enc(&arr[0]);
    let (mut lo, mut hi, mut prev) = (first, first, first);
    let (mut asc, mut desc) = (true, true);
    for x in arr[1..].iter() {
        let k = enc(x);
        lo = lo.min(k);
        hi = hi.max(k);
        asc &= prev <= k;
        desc &= prev > k;
        prev = k;
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
    if !off(2) && few_keys(arr, &enc) {
        return;
    }
    let key_bits = u128::BITS - (hi - lo).leading_zeros();
    let bytes = n.saturating_mul(std::mem::size_of::<T>());
    if !off(4) && key_bits > MSD_BITS && bytes > l2_bytes() / 2 {
        msd(arr, &enc, lo, key_bits, bytes);
        return;
    }
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the elements are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buf.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, &enc, lo, key_bits) {
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}

/// Digit of the MSD pass at most this many bits.
const MSD_BITS: u32 = 12;

/// One MSD pass on the top `bits` of `key - lo` (buckets of about a quarter
/// of the L2), moving the elements into a buffer in streams; then each
/// bucket sorted from the buffer straight into its place in `arr`.
fn msd<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, lo: u128, key_bits: u32, bytes: usize) {
    let n = arr.len();
    let want = (bytes / (l2_bytes() / 4).max(1)).next_power_of_two().max(2);
    let bits = (want.trailing_zeros()).clamp(1, MSD_BITS).min(key_bits);
    let shift = key_bits - bits;
    let digit = |x: &T| ((enc(x) - lo) >> shift) as usize;
    let mut counts = vec![0usize; 1 << bits];
    for x in arr.iter() {
        counts[digit(x)] += 1;
    }
    let mut heads = vec![0usize; 1 << bits];
    let mut s = 0;
    for (h, &c) in heads.iter_mut().zip(counts.iter()) {
        *h = s;
        s += c;
    }
    let starts = heads.clone();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: every element is moved once into the buffer at its bucket's
    // next slot (a stable scatter: the heads cover 0..n exactly), then each
    // bucket is moved back into the same range of `arr` by `sort_into`
    // (every element once). Nothing is dropped or duplicated; no code that
    // can panic runs between the moves except `enc`, called on keys that
    // were already encoded without panicking in the read before.
    unsafe {
        let src = arr.as_ptr();
        let dst = buf.as_mut_ptr() as *mut T;
        for i in 0..n {
            let x = &*src.add(i);
            let d = digit(x);
            dst.add(heads[d]).write(std::ptr::read(x));
            heads[d] += 1;
        }
        let out = arr.as_mut_ptr();
        for d in 0..1 << bits {
            let (s, c) = (starts[d], counts[d]);
            if c == 0 {
                continue;
            }
            let bucket = dst.add(s);
            let (mut blo, mut bhi) = (u128::MAX, 0u128);
            for i in 0..c {
                let k = enc(&*bucket.add(i));
                blo = blo.min(k);
                bhi = bhi.max(k);
            }
            let bk = u128::BITS - (bhi - blo).leading_zeros();
            if !sort_into(bucket, out.add(s), c, enc, blo, bk) {
                // Too wide to pack: back in place, then a comparison sort.
                std::ptr::copy_nonoverlapping(bucket, out.add(s), c);
                std::slice::from_raw_parts_mut(out.add(s), c).sort_by(|a, b| enc(a).cmp(&enc(b)));
            }
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
