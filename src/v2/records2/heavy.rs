//! Heavy keys: a few keys holding much of the array (sparse data, heavy
//! hitters) set apart, the rest sorted, all put back in key order.

use std::mem::MaybeUninit;

/// Most heavy keys, and the share of the sample each must hold (1/10).
const HEAVY_MAX: usize = 8;
const SAMPLE: usize = 256;

/// Heavy keys: a spread sample of 256 keys names the keys holding at least
/// a tenth of it (at most 8); when they hold at least 40 % of the sample,
/// one read counts them (giving up below 30 %, nothing moved). Then one
/// pass moves every record into a buffer (the other records first, each
/// heavy key's records in a block, all in their order: stable), the other
/// records are sorted, and one pass moves everything back in key order.
/// Two moves per record plus the sort of the rest.
pub(super) fn heavy_keys<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) -> bool {
    let n = arr.len();
    if n < 4096 {
        return false;
    }
    // Pre-test: 16 keys spread over the array; keys holding 40 % of it make
    // a repeat among them all but certain, distinct keys almost never.
    let mut few = [0u128; 16];
    for (i, k) in few.iter_mut().enumerate() {
        *k = enc(&arr[(i * 2 + 1) * n / 32]);
    }
    let mut repeat = false;
    for i in 0..16 {
        for j in i + 1..16 {
            repeat |= few[i] == few[j];
        }
    }
    if !repeat {
        return false;
    }
    let mut sample: Vec<u128> = (0..SAMPLE as u64)
        .map(|i| enc(&arr[((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64 >> 32) as usize]))
        .collect();
    sample.sort_unstable();
    let mut heavy: Vec<u128> = Vec::new();
    let mut held = 0;
    let mut i = 0;
    while i < SAMPLE {
        let j = i + sample[i..].partition_point(|&k| k == sample[i]);
        if (j - i) * 10 >= SAMPLE && heavy.len() < HEAVY_MAX {
            heavy.push(sample[i]);
            held += j - i;
        }
        i = j;
    }
    if heavy.is_empty() || held * 10 < SAMPLE * 4 {
        return false;
    }
    let h = heavy.len();
    let class = |k: u128| -> usize {
        let mut c = h;
        for (j, &v) in heavy.iter().enumerate() {
            c = if k == v { j } else { c };
        }
        c
    };
    // Counts: heavy keys 0..h (in key order), the others at h.
    let mut counts = [0usize; HEAVY_MAX + 1];
    for x in arr.iter() {
        counts[class(enc(x))] += 1;
    }
    let others = counts[h];
    if (n - others) * 10 < n * 3 {
        return false;
    }
    if h == 1 && ONE.load(std::sync::atomic::Ordering::Relaxed) {
        one_heavy(arr, enc, heavy[0], others);
        return true;
    }
    // Buffer: the others first, then each heavy key's block.
    let mut heads = [0usize; HEAVY_MAX + 1];
    let mut at = others;
    for j in 0..h {
        heads[j] = at;
        at += counts[j];
    }
    heads[h] = 0;
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    super::merge::prefault(&mut buf);
    // SAFETY: every record is moved once into the buffer (the heads cover
    // 0..n exactly), the others sorted there as a slice of initialised
    // records, then every record moved once back into the array (the
    // pieces cover 0..n exactly). `enc` already ran on every record.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        for i in 0..n {
            let c = class(enc(&*a.add(i)));
            b.add(heads[c]).write(std::ptr::read(a.add(i)));
            heads[c] += 1;
        }
        let rest = std::slice::from_raw_parts_mut(b, others);
        super::sort_encoded(rest, enc);
        // Back: the others below each heavy key, then its block.
        let (mut o, mut d) = (0usize, 0usize);
        let mut block = others;
        for j in 0..h {
            let below = o + rest[o..].partition_point(|x| enc(x) < heavy[j]);
            std::ptr::copy_nonoverlapping(b.add(o), a.add(d), below - o);
            d += below - o;
            o = below;
            std::ptr::copy_nonoverlapping(b.add(block), a.add(d), counts[j]);
            d += counts[j];
            block += counts[j];
        }
        std::ptr::copy_nonoverlapping(b.add(o), a.add(d), others - o);
    }
    true
}

/// One heavy key: its records compacted in place (forward, in order), the
/// others moved to a buffer of their size, sorted there, then the block
/// moved to its place and the others put around it.
pub(crate) static ONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn one_heavy<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, key: u128, others: usize) {
    let n = arr.len();
    let c = n - others;
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(others + 1);
    super::merge::prefault(&mut buf);
    // SAFETY: each record is written both to the array at `w <= i` (kept
    // when of the heavy key) and to the buffer at `o` (kept otherwise); the
    // array slot `w` was already read. At the end the array holds the `c`
    // heavy records in order in 0..c, the buffer the others. The block is
    // moved (memmove) to its final place, the others copied around it:
    // every record ends once in the array. `enc` already ran on all.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        let (mut w, mut o) = (0usize, 0usize);
        for i in 0..n {
            let x = std::ptr::read(a.add(i));
            let is = enc(&x) == key;
            b.add(o).write(std::ptr::read(&x));
            a.add(w).write(x);
            w += is as usize;
            o += !is as usize;
        }
        let rest = std::slice::from_raw_parts_mut(b, others);
        super::sort_encoded(rest, enc);
        let below = rest.partition_point(|x| enc(x) < key);
        std::ptr::copy(a, a.add(below), c);
        std::ptr::copy_nonoverlapping(b, a, below);
        std::ptr::copy_nonoverlapping(b.add(below), a.add(below + c), others - below);
    }
}
