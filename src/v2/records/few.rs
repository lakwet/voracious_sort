//! Few distinct keys: a stable counting sort.

use std::mem::MaybeUninit;

/// Most distinct keys counted, and the hash table size (at most half full).
const FEW_MAX: usize = 256;
const FEW_TABLE_BITS: u32 = 9;
const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;

/// Few distinct keys: a spread sample gives up as soon as half of it is
/// distinct; else one pass counts the keys in a hash table (giving up
/// beyond `FEW_MAX`, nothing moved), the distinct keys are sorted, and one
/// pass moves every element to its key's next slot in a buffer (stable),
/// copied back.
pub(super) fn few_keys<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
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

/// Up to `WIDE_MAX` distinct keys (tried only when the keys change
/// direction rarely, e.g. long blocks of equal keys): the same stable
/// counting sort with a larger table on the heap; consecutive equal keys
/// skip the lookup (the previous slot kept). Gives up beyond `WIDE_MAX`
/// distinct keys, nothing moved.
pub(super) fn few_keys_wide<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    const WIDE_MAX: usize = 4096;
    const BITS: u32 = 13;
    const TABLE: usize = 1 << BITS;
    let n = arr.len();
    let hash = |e: u128| (((e as u64) ^ ((e >> 64) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - BITS)) as usize;
    let mut keys = vec![0u128; TABLE];
    let mut used = vec![false; TABLE];
    let mut counts = vec![0usize; TABLE];
    let mut distinct = 0;
    let (mut last_key, mut last_slot) = (enc(&arr[0]), usize::MAX);
    for x in arr.iter() {
        let e = enc(x);
        if e != last_key || last_slot == usize::MAX {
            let mut h = hash(e);
            while used[h] && keys[h] != e {
                h = (h + 1) & (TABLE - 1);
            }
            if !used[h] {
                used[h] = true;
                keys[h] = e;
                distinct += 1;
                if distinct > WIDE_MAX {
                    return false;
                }
            }
            last_key = e;
            last_slot = h;
        }
        counts[last_slot] += 1;
    }
    let mut order: Vec<usize> = (0..TABLE).filter(|&h| counts[h] != 0).collect();
    order.sort_unstable_by_key(|&h| keys[h]);
    let mut heads = vec![0usize; TABLE];
    let mut start = 0;
    for &h in order.iter() {
        heads[h] = start;
        start += counts[h];
    }
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    super::merge::prefault(&mut buf);
    // SAFETY: as `few_keys`: each element moved once into the buffer at its
    // key's next slot (the runs cover 0..n), then the buffer copied back.
    unsafe {
        let src = arr.as_ptr();
        let dst = buf.as_mut_ptr() as *mut T;
        let (mut last_key, mut last_slot) = (enc(&*src), usize::MAX);
        for i in 0..n {
            let x = &*src.add(i);
            let e = enc(x);
            if e != last_key || last_slot == usize::MAX {
                let mut h = hash(e);
                while keys[h] != e || !used[h] {
                    h = (h + 1) & (TABLE - 1);
                }
                last_key = e;
                last_slot = h;
            }
            dst.add(heads[last_slot]).write(std::ptr::read(x));
            heads[last_slot] += 1;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
    true
}
