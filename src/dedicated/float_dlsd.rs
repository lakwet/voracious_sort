//! Diverting LSD (DLSD) on f64 bits, built to run inside the cache (the
//! leaves of the float rollercoaster, or whole arrays that fit in cache).
//!
//! A stable LSD sorts only the top bits of `key - min` (min and max of the
//! encoded keys found first): about `log2(n) + DLSD_EXTRA_BITS` bits, enough
//! to tell most keys apart. The array is then sorted up to groups of keys
//! sharing that prefix; one sequential pass finishes them (insertion sort
//! for small groups, the DLSD again, at the scale of the group, for large
//! ones: skewed data) and decodes the keys. When the prefix holds every
//! varying bit, the LSD is exact and the decoding is fused into its last
//! pass.
//!
//! Engineering taken from the fused LSD (`float_lsd`): encoding fused into
//! the first read, histograms of every digit in one read with 2 lanes, the
//! scatter reads 4 keys before writing them with a fast path when they share
//! their digit, and a staged scatter when the bucket starts alias in the
//! cache.

use std::mem::MaybeUninit;

use super::float_lsd::w64;

/// Up to this size: `sort_unstable` on the encoded keys.
const SMALL: usize = 64;
/// Widest digit (4096 buckets: their write streams still fit in L1/L2).
const MAX_DIGIT_BITS: u32 = 12;
/// Sample for the prefix length: n / `SAMPLE_DIV` keys (at least
/// `SAMPLE_MIN`). A sample proportional to n keeps seeing pairs of keys
/// from the same tight cluster however many clusters there are: the share
/// of such neighbouring pairs is about S * c / (2 n) for clusters of c keys,
/// above the 5 % the 95th percentile needs from c = 26; smaller groups are
/// finished cheaply anyway. Sorting it costs ~0.1 ns per key.
const SAMPLE_DIV: usize = 256;
const SAMPLE_MIN: usize = 16;
/// Moves per key allowed to the final insertion sort before the groups of
/// equal prefix are finished one by one (8 measured worse: Prices).
const INSERTION_BUDGET: usize = 2;
/// Groups of equal prefix up to this size are finished by insertion sort.
const GROUP_INSERTION: usize = 16;
/// From this size the bucket starts are checked for cache aliasing. Below,
/// the array and its buffer stay in L2, where aliased write streams cost
/// little and the staged scatter costs more than it saves (measured on
/// Snapdragon X Elite, 12 MiB L2: Sqrt 12.0 vs 7.2 ns at 1M; at 4M staging
/// is needed, sorted integer sequences 14 vs 23 ns).
const ALIAS_MIN: usize = 1 << 21;
/// Elements per staging slot (one 64-byte cache line).
const WB: usize = 8;

/// Tests: profile counters (ns or counts), see `PROF_NAMES`.
#[cfg(test)]
pub(crate) static PROF: [std::sync::atomic::AtomicU64; 9] = [const { std::sync::atomic::AtomicU64::new(0) }; 9];
#[cfg(test)]
pub(crate) const PROF_NAMES: [&str; 9] =
    ["minmax", "hist", "scatter", "staged", "fixup", "calls", "passes", "staged_n", "q_bits"];

macro_rules! prof {
    ($i:expr, $e:expr) => {{
        #[cfg(test)]
        let t = std::time::Instant::now();
        let r = $e;
        #[cfg(test)]
        PROF[$i].fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        r
    }};
}

#[inline(always)]
#[allow(unused_variables)]
fn count(i: usize, v: u64) {
    #[cfg(test)]
    PROF[i].fetch_add(v, std::sync::atomic::Ordering::Relaxed);
}

/// Tests: disable the staged scatter (A/B).
#[cfg(test)]
pub(crate) static NO_STAGE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn stage_enabled() -> bool {
    #[cfg(test)]
    {
        !NO_STAGE.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Sorts raw f64 bits in total order.
pub(crate) fn dlsd64(bits: &mut [u64]) {
    let mut scratch = Vec::new();
    dlsd64_with(bits, &mut scratch);
}

/// `dlsd64` with a reusable scratch buffer.
pub(crate) fn dlsd64_with(bits: &mut [u64], scratch: &mut Vec<MaybeUninit<u64>>) {
    if bits.len() > u32::MAX as usize {
        w64::lsd(bits);
        return;
    }
    dlsd_rec::<true>(bits, scratch);
}

/// Sorts already encoded keys and writes them back decoded.
pub(crate) fn dlsd64_encoded(keys: &mut [u64], scratch: &mut Vec<MaybeUninit<u64>>) {
    if keys.len() > u32::MAX as usize {
        w64::lsd_encoded(keys, scratch);
        return;
    }
    dlsd_rec::<false>(keys, scratch);
}

#[inline(always)]
fn enc<const RAW: bool>(v: u64) -> u64 {
    if RAW {
        w64::encode_key(v)
    } else {
        v
    }
}

/// `RAW`: `keys` holds raw bits, else encoded keys. Output: raw bits.
fn dlsd_rec<const RAW: bool>(keys: &mut [u64], scratch: &mut Vec<MaybeUninit<u64>>) {
    let n = keys.len();
    if n <= SMALL {
        if RAW {
            keys.iter_mut().for_each(|k| *k = w64::encode_key(*k));
        }
        if n <= GROUP_INSERTION {
            insertion(keys);
        } else {
            keys.sort_unstable();
        }
        keys.iter_mut().for_each(|k| *k = w64::decode_key(*k));
        return;
    }
    count(5, 1);
    let (mut min, mut max) = (u64::MAX, 0u64);
    prof!(0, for &v in keys.iter() {
        let k = enc::<RAW>(v);
        min = min.min(k);
        max = max.max(k);
    });
    let range = max - min;
    if range == 0 {
        if !RAW {
            keys.iter_mut().for_each(|k| *k = w64::decode_key(*k));
        }
        return;
    }
    let top = 64 - range.leading_zeros();
    let log_n = usize::BITS - n.leading_zeros();
    // Prefix length: bits needed to tell apart most neighbouring keys,
    // estimated on a spread sample (see `prefix_bits`).
    let bits = prefix_bits::<RAW>(keys, min, top);
    let passes = bits.div_ceil(MAX_DIGIT_BITS);
    // Digit width: the prefix split evenly, at most ~n/16 digit values.
    let w = bits.div_ceil(passes).min(log_n.saturating_sub(4).max(8)).max(1);
    let k = bits.div_ceil(w) as usize;
    let base = top.saturating_sub(k as u32 * w);
    let r = 1usize << w;
    let mask = (r - 1) as u64;

    // Histograms of the prefix digits, one read, 2 lanes.
    let mut tables = vec![0u32; 2 * k * r];
    #[cfg(test)]
    let th = std::time::Instant::now();
    {
        let (t0, t1) = tables.split_at_mut(k * r);
        let mut pairs = keys.chunks_exact(2);
        for pr in &mut pairs {
            let a = enc::<RAW>(pr[0]).wrapping_sub(min) >> base;
            let b = enc::<RAW>(pr[1]).wrapping_sub(min) >> base;
            for t in 0..k {
                let s = t as u32 * w;
                // SAFETY: masked digits index a table of `r` counters.
                unsafe {
                    *t0.get_unchecked_mut(t * r + ((a >> s) & mask) as usize) += 1;
                    *t1.get_unchecked_mut(t * r + ((b >> s) & mask) as usize) += 1;
                }
            }
        }
        for &v in pairs.remainder() {
            let a = enc::<RAW>(v).wrapping_sub(min) >> base;
            for t in 0..k {
                t0[t * r + ((a >> (t as u32 * w)) & mask) as usize] += 1;
            }
        }
        for i in 0..k * r {
            t0[i] += t1[i];
        }
        tables.truncate(k * r);
    }
    #[cfg(test)]
    PROF[1].fetch_add(th.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    let (q, p) = (k, k);
    // The prefix is bits `low..top` of `key - min`; exact when `low == 0`.
    let low = base;
    let exact = low == 0;
    count(8, (top - low) as u64);
    // Passes whose digit varies (the top one always does).
    let todo: Vec<usize> = (k - q..p).filter(|&t| !tables[t * r..(t + 1) * r].iter().any(|&c| c as usize == n)).collect();

    if scratch.len() < n {
        scratch.reserve(n - scratch.len());
        // SAFETY: `MaybeUninit` needs no initialization.
        unsafe { scratch.set_len(n) };
    }
    let buf = scratch.as_mut_ptr() as *mut u64;
    // Scatter passes: (shift, width, exclusive prefix sums, staged). A pass
    // whose bucket starts alias in the cache is staged.
    let mut ops: Vec<(u32, u32, Vec<u32>, bool)> = Vec::with_capacity(todo.len() + 2);
    let prefix_sums = |counts: &[u32]| {
        let mut sum = 0u32;
        counts
            .iter()
            .map(|&c| {
                let h = sum;
                sum += c;
                h
            })
            .collect::<Vec<u32>>()
    };
    for &t in todo.iter() {
        let counts = &tables[t * r..(t + 1) * r];
        let shift = base + t as u32 * w;
        let heads = prefix_sums(counts);
        if n < ALIAS_MIN || !stage_enabled() || !aliased(&heads, n) {
            ops.push((shift, w, heads, false));
            continue;
        }
        ops.push((shift, w, heads, true));
    }
    let mut staging: Vec<u64> = Vec::new();
    let mut starts: Vec<u32> = Vec::new();
    let last = ops.len() - 1;
    for (i, (shift, wd, heads, staged)) in ops.iter_mut().enumerate() {
        let (shift, staged) = (*shift, *staged);
        let mask = (1u64 << *wd) - 1;
        let first = RAW && i == 0;
        let decode = exact && i == last;
        count(6, 1);
        count(7, staged as u64);
        #[cfg(test)]
        let ts = std::time::Instant::now();
        if staged && staging.len() < heads.len() * WB {
            staging = vec![0; heads.len() * WB];
            starts = vec![0; heads.len()];
        }
        // SAFETY: pass `i` reads `keys` (even) or the buffer (odd, fully
        // written by pass `i - 1`) and writes each of the `n` slots of the
        // other once (heads are the prefix sums of this digit over the same
        // keys); they do not overlap.
        unsafe {
            let (src, dst): (&[u64], *mut u64) = if i % 2 == 0 {
                (&*keys, buf)
            } else {
                (std::slice::from_raw_parts(buf, n), keys.as_mut_ptr())
            };
            macro_rules! go {
                ($f:expr, $d:expr) => {
                    if staged {
                        scatter_staged::<$f, $d>(src, dst, heads, min, shift, mask, &mut staging, &mut starts)
                    } else {
                        scatter::<$f, $d>(src, dst, heads, min, shift, mask)
                    }
                };
            }
            match (first, decode) {
                (true, true) => go!(true, true),
                (true, false) => go!(true, false),
                (false, true) => go!(false, true),
                (false, false) => go!(false, false),
            }
        }
        #[cfg(test)]
        PROF[if staged { 3 } else { 2 }].fetch_add(ts.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }
    if ops.len() % 2 == 1 {
        // SAFETY: the buffer was fully written by the last pass.
        keys.copy_from_slice(unsafe { std::slice::from_raw_parts(buf, n) });
    }
    if exact {
        return;
    }
    #[cfg(test)]
    let tf = std::time::Instant::now();
    // Finish: the keys are sorted up to groups of equal prefix, mostly
    // single keys or small groups, often of equal keys. An insertion sort
    // over the whole array then mostly takes the "already in place" branch
    // (well predicted; equal keys never move). It cannot move a key out of
    // its group, so if it exceeds its budget of moves (large unsorted
    // groups), the group pass below finishes from where it stopped.
    let budget = INSERTION_BUDGET * n;
    let mut moves = 0;
    let mut i = 1;
    while i < n {
        let v = keys[i];
        if keys[i - 1] > v {
            let mut j = i;
            while j > 0 && keys[j - 1] > v {
                keys[j] = keys[j - 1];
                j -= 1;
            }
            keys[j] = v;
            moves += i - j;
            if moves > budget {
                break;
            }
        }
        i += 1;
    }
    if i >= n {
        keys.iter_mut().for_each(|k| *k = w64::decode_key(*k));
        #[cfg(test)]
        PROF[4].fetch_add(tf.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    // Groups of equal prefix: insertion sort or the DLSD at their scale.
    let prefix = |k: u64| k.wrapping_sub(min) >> low;
    let mut i = 0;
    while i < n {
        let pi = prefix(keys[i]);
        let mut j = i + 1;
        while j < n && prefix(keys[j]) == pi {
            j += 1;
        }
        if j == i + 1 {
            keys[i] = w64::decode_key(keys[i]);
        } else if j - i <= GROUP_INSERTION {
            let g = &mut keys[i..j];
            insertion(g);
            g.iter_mut().for_each(|k| *k = w64::decode_key(*k));
        } else {
            dlsd_rec::<false>(&mut keys[i..j], scratch);
        }
        i = j;
    }
    #[cfg(test)]
    PROF[4].fetch_add(tf.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
}

/// Prefix length (bits of `key - min` from the top of the range, at most
/// `top`) for the DLSD: on a sorted spread sample of `S` keys, the bits a
/// pair of unequal neighbours share, plus one, at the 95th percentile, plus
/// log2(n / S) (neighbours in the array are n / S times closer than in the
/// sample). Unlike the entropy of each digit taken alone, this sees
/// correlated digits (tight clusters: the top bits name the cluster, the
/// keys of a cluster differ much lower) and isolated extremes. Equal keys
/// need no prefix and are ignored.
fn prefix_bits<const RAW: bool>(keys: &[u64], min: u64, top: u32) -> u32 {
    let n = keys.len();
    let s = (n / SAMPLE_DIV).max(SAMPLE_MIN).min(n);
    // Positions frac(i * golden ratio) * n: spread like a stride but in
    // resonance with no period of the data (a stride of n / S on periodic
    // data, e.g. sawtooth teeth, samples near-duplicates and asks for every
    // bit).
    const PHI: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut sample: Vec<u64> = (0..s as u64)
        .map(|i| {
            let pos = ((i.wrapping_mul(PHI) >> 32) * n as u64) >> 32;
            enc::<RAW>(keys[pos as usize]).wrapping_sub(min)
        })
        .collect();
    sample.sort_unstable();
    // Bits shared from the top of the range, plus one: bits that separate.
    let mut need: Vec<u32> = sample
        .windows(2)
        .filter(|p| p[0] != p[1])
        .map(|p| ((p[0] ^ p[1]).leading_zeros() + 1).saturating_sub(64 - top))
        .collect();
    let log_n = usize::BITS - n.leading_zeros();
    if need.is_empty() {
        // Sample of equal keys (very few distinct values): a short prefix,
        // the finish handles the equal groups for free.
        return (log_n + 3).min(top);
    }
    need.sort_unstable();
    let p95 = need[(need.len() * 95 / 100).min(need.len() - 1)];
    let spread = (usize::BITS - (n / s).leading_zeros()).saturating_sub(1);
    (p95 + spread).clamp(1, top)
}

fn insertion(a: &mut [u64]) {
    for i in 1..a.len() {
        let v = a[i];
        let mut j = i;
        while j > 0 && a[j - 1] > v {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = v;
    }
}

/// Whether the bucket starts (`heads`, exclusive prefix sums) crowd on a few
/// line positions inside a 4 KiB page (same criterion as `float_lsd`).
fn aliased(heads: &[u32], n: usize) -> bool {
    let mut classes = [0u32; 64];
    let mut nonempty = 0u32;
    for b in 0..heads.len() {
        let start = heads[b] as usize;
        let end = if b + 1 < heads.len() { heads[b + 1] as usize } else { n };
        if end > start {
            classes[(start * 8 % 4096) / 64] += 1;
            nonempty += 1;
        }
    }
    let max = *classes.iter().max().unwrap();
    max > 16 && max > 4 * nonempty.div_ceil(64)
}

/// One stable scatter pass on the digit `((key - min) >> shift) & mask`.
/// `FIRST` encodes the raw bits read, `LAST` decodes the keys written.
///
/// # Safety
/// `dst` valid for `src.len()` writes, not overlapping `src`; `heads` the
/// exclusive prefix sums of this digit over the keys of `src`.
#[inline(always)]
unsafe fn scatter<const FIRST: bool, const LAST: bool>(
    src: &[u64],
    dst: *mut u64,
    heads: &mut [u32],
    min: u64,
    shift: u32,
    mask: u64,
) {
    let key = |v: u64| if FIRST { w64::encode_key(v) } else { v };
    let out = |k: u64| if LAST { w64::decode_key(k) } else { k };
    let digit = |k: u64| ((k.wrapping_sub(min) >> shift) & mask) as usize;
    let mut blocks = src.chunks_exact(4);
    // SAFETY (all accesses): masked digits index `heads` (`mask + 1`
    // counters); each head plus the keys of its bucket already written is
    // an unused position below `src.len()`.
    unsafe {
        for q in &mut blocks {
            let (a, b, c, d) = (key(q[0]), key(q[1]), key(q[2]), key(q[3]));
            let (da, db, dc, dd) = (digit(a), digit(b), digit(c), digit(d));
            if da == db && da == dc && da == dd {
                let p = *heads.get_unchecked(da) as usize;
                *heads.get_unchecked_mut(da) += 4;
                let w = dst.add(p);
                w.write(out(a));
                w.add(1).write(out(b));
                w.add(2).write(out(c));
                w.add(3).write(out(d));
                continue;
            }
            let pa = *heads.get_unchecked(da) as usize;
            *heads.get_unchecked_mut(da) += 1;
            let pb = *heads.get_unchecked(db) as usize;
            *heads.get_unchecked_mut(db) += 1;
            let pc = *heads.get_unchecked(dc) as usize;
            *heads.get_unchecked_mut(dc) += 1;
            let pd = *heads.get_unchecked(dd) as usize;
            *heads.get_unchecked_mut(dd) += 1;
            dst.add(pa).write(out(a));
            dst.add(pb).write(out(b));
            dst.add(pc).write(out(c));
            dst.add(pd).write(out(d));
        }
        for &v in blocks.remainder() {
            let k = key(v);
            let dk = digit(k);
            let p = *heads.get_unchecked(dk) as usize;
            *heads.get_unchecked_mut(dk) += 1;
            dst.add(p).write(out(k));
        }
    }
}

/// Scatter pass through per-bucket staging lines (written to `dst` one
/// cache line at a time), used when the bucket starts alias.
///
/// # Safety
/// Same contract as `scatter`; `staging` holds `(mask + 1) * WB` slots and
/// `starts` `mask + 1`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn scatter_staged<const FIRST: bool, const LAST: bool>(
    src: &[u64],
    dst: *mut u64,
    heads: &mut [u32],
    min: u64,
    shift: u32,
    mask: u64,
    staging: &mut [u64],
    starts: &mut [u32],
) {
    let r = mask as usize + 1;
    assert!(staging.len() >= r * WB && starts.len() >= r && heads.len() >= r);
    let key = |v: u64| if FIRST { w64::encode_key(v) } else { v };
    let out = |k: u64| if LAST { w64::decode_key(k) } else { k };
    starts[..r].copy_from_slice(&heads[..r]);
    let stage = staging.as_mut_ptr();
    // SAFETY (all accesses): buckets `< r` and slots `< WB` stay inside
    // `staging`; positions `from..to` belong to bucket `b` and are below
    // `src.len()`.
    unsafe {
        for &v in src.iter() {
            let k = key(v);
            let b = ((k.wrapping_sub(min) >> shift) & mask) as usize;
            let p = *heads.get_unchecked(b) as usize;
            *heads.get_unchecked_mut(b) += 1;
            let slot = p & (WB - 1);
            stage.add(b * WB + slot).write(out(k));
            if slot == WB - 1 {
                let line = p + 1 - WB;
                let from = *starts.get_unchecked(b) as usize;
                if line >= from {
                    std::ptr::copy_nonoverlapping(stage.add(b * WB), dst.add(line), WB);
                } else {
                    std::ptr::copy_nonoverlapping(stage.add(b * WB + (from & (WB - 1))), dst.add(from), p + 1 - from);
                }
            }
        }
        for b in 0..r {
            let end = heads[b] as usize;
            let from = (end & !(WB - 1)).max(starts[b] as usize);
            if end > from {
                std::ptr::copy_nonoverlapping(stage.add(b * WB + (from & (WB - 1))), dst.add(from), end - from);
            }
        }
    }
}
