//! Rollercoaster for floats (f64 and f32): in-place MSD passes on the top varying bits of
//! the encoded keys until the buckets fit in cache, then the fused LSD of
//! `float_lsd` on each bucket (decoding fused into its last scatter pass).
//!
//! Compared with the generic `rollercoaster_sort`:
//! - the encoding is fused into the pass that finds the varying bits, and
//!   the decoding into the leaves (no separate encode/decode passes);
//! - the MSD digit is taken on `key - min` (exact min and max of the keys):
//!   it starts at the highest bit of the range, so the bits equal in every
//!   key are skipped and a range crossing an exponent boundary (where many
//!   high bits flip) is not split on that boundary alone; its width is chosen
//!   from the size, so that one pass usually leaves buckets of about `LEAF`
//!   elements;
//! - the leaves use the fused LSD (multi-lane histograms, 11-bit digits,
//!   4-key scatter, cache-aliasing fix) with one scratch buffer reused for
//!   every leaf, instead of a generic DLSD that allocates each time;
//! - no Verge pass of its own (done once by `float_sort`).
//!
//! Memory: the array plus one leaf-sized scratch buffer (the MSD passes are
//! in place), against twice the array for the LSD.



/// Target bucket size (keys) for the leaves. Not a cache size: with 2^15
/// keys the DLSD prefix (log2 n + ~8 bits) fits in 2 passes of 12 bits;
/// larger leaves need a third pass. Measured on Snapdragon X Elite, leaves
/// of 128 KiB to 2 MiB of keys at 4M / 16M keys: best at 32k keys for f64
/// (256 KiB: 4.89 ns vs 5.94 with 2 MiB) and f32 (128 KiB: 3.39 vs 5.04).
const LEAF_KEYS: usize = 1 << 15;
/// A bucket up to this many times `LEAF` (a digit value holding more than
/// `LEAF` keys) still goes to the leaf sort: the DLSD stays efficient up to
/// a few MiB, cheaper than one more MSD pass over the bucket.
const LEAF_SLACK: usize = 4;
/// Most buckets made by one MSD pass: beyond about a thousand write streams
/// the in-place permutation slows down (caches, TLB; at 100M on Snapdragon X
/// Elite: 1.4 ns per key with 100 buckets, 1.8 with 1600, 2.9 with 7300).
const MAX_BUCKETS: usize = 1024;
/// MSD digit width bounds (the digit values are grouped into buckets of
/// about `LEAF` keys).
const MIN_BITS: u32 = 8;
const MAX_BITS: u32 = 18;
/// log2 of the average number of keys per digit value.
const KEYS_PER_DIGIT_LOG: u32 = 7;

/// Tests / benchmarks: leaf size override in keys (0: `LEAF_KEYS`).
#[cfg(test)]
pub(crate) static LEAF_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(always)]
fn leaf_size() -> usize {
    #[cfg(test)]
    {
        match LEAF_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
            0 => LEAF_KEYS,
            v => v,
        }
    }
    #[cfg(not(test))]
    {
        LEAF_KEYS
    }
}

/// Tests: time (ns) per phase and work counters, see `PROF_NAMES`.
#[cfg(test)]
pub(crate) static PROF: [std::sync::atomic::AtomicU64; 8] = [const { std::sync::atomic::AtomicU64::new(0) }; 8];
#[cfg(test)]
pub(crate) const PROF_NAMES: [&str; 8] =
    ["encode", "minmax", "hist", "permute", "leaves", "msd_elems", "leaf_count", "msd_calls"];

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

/// Tests / benchmarks: leaf sort (0 DLSD, 1 fused LSD).
#[cfg(test)]
pub(crate) static LEAF_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Tests / benchmarks: use the block permutation instead of ska sort's swap
/// loop (A/B). Measured on Snapdragon X Elite: equal at 400M on random
/// data (ska 1.67 vs blocks 1.79 ns per key), blocks better on aliased
/// buckets (integer-valued keys, 2.5 vs 3.5), worse on Zipf at 10M (2.05
/// vs 1.35): ska by default, to revisit on x86 and at billions of keys.
#[cfg(test)]
pub(crate) static BLOCK_PERMUTE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn block_permute_enabled() -> bool {
    #[cfg(test)]
    {
        BLOCK_PERMUTE.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        false
    }
}

/// Tests: time (ns) of the 3 phases of `permute_blocks`.
#[cfg(test)]
pub(crate) static PHASES: [std::sync::atomic::AtomicU64; 3] = [const { std::sync::atomic::AtomicU64::new(0) }; 3];

macro_rules! roller_width {
    ($uint:ty, $w:ident, $d:ident, $m:ident) => {
        #[allow(dead_code)]
        pub(crate) mod $m {
            use std::mem::MaybeUninit;

            use super::*;
            use super::super::float_dlsd::$d as d;
            use super::super::float_lsd::$w as w;
            #[cfg(test)]
            use super::super::float_lsd::Digits;

            /// Sorts raw float bits in total order.
            pub(crate) fn roller(bits: &mut [$uint]) {
                let leaf = leaf_size();
                if bits.len() <= leaf {
                    d::dlsd(bits);
                    return;
                }
                // Encode in place and find the range of the keys.
                let (mut min, mut max) = (<$uint>::MAX, 0);
                prof!(0, for b in bits.iter_mut() {
                    let k = w::encode_key(*b);
                    *b = k;
                    min = min.min(k);
                    max = max.max(k);
                });
                let mut scratch: Vec<MaybeUninit<$uint>> = Vec::new();
                msd(bits, min, max, leaf, &mut scratch);
            }

            /// Range of encoded keys.
            fn min_max(keys: &[$uint]) -> ($uint, $uint) {
                keys.iter().fold((<$uint>::MAX, 0), |(lo, hi), &k| (lo.min(k), hi.max(k)))
            }

            /// Sorts encoded keys (all in `min..=max`) and writes them back decoded.
            fn msd(keys: &mut [$uint], min: $uint, max: $uint, leaf: usize, scratch: &mut Vec<MaybeUninit<$uint>>) {
                let range = max - min;
                if range == 0 {
                    keys.iter_mut().for_each(|k| *k = w::decode_key(*k));
                    return;
                }
                // `key - min` fits in `top` bits; the digit takes the top `w` of them.
                let top = <$uint>::BITS - range.leading_zeros();
                // About `KEYS_PER_DIGIT` keys per digit value: the histogram stays cheap
                // next to the pass on small buckets.
                let log_n = usize::BITS - keys.len().leading_zeros();
                let w = log_n.saturating_sub(KEYS_PER_DIGIT_LOG).clamp(MIN_BITS, MAX_BITS).min(top);
                let shift = top - w;
                let r = 1usize << w;

                count(5, keys.len() as u64);
                count(7, 1);
                let counts = prof!(2, histogram(keys, min, shift, r));
                // Consecutive digit values grouped into buckets of about `leaf` keys
                // (a digit value holding more keys is a bucket on its own): float keys
                // are far from uniform over their bits, fixed digits would leave a few
                // huge buckets to split again.
                // Bucket size aimed at: `leaf`, or larger when that would make more than
                // `MAX_BUCKETS` write streams (very large arrays: one more MSD level).
                let target = leaf.max(keys.len().div_ceil(MAX_BUCKETS));
                let mut map = vec![0u32; r];
                let mut ends: Vec<usize> = Vec::new();
                let mut sum = 0usize;
                let mut open = 0usize;
                for d in 0..r {
                    let c = counts[d];
                    if open > 0 && open + c > target {
                        ends.push(sum);
                        open = 0;
                    }
                    map[d] = ends.len() as u32;
                    sum += c;
                    open += c;
                }
                ends.push(sum);
                let m = ends.len();
                let mut heads = vec![0usize; m];
                for b in 1..m {
                    heads[b] = ends[b - 1];
                }
                let starts = heads.clone();
                if m > 1 {
                    if block_permute_enabled() {
                        prof!(3, permute_blocks(keys, &starts, &ends, min, shift, &map));
                    } else {
                        prof!(3, permute(keys, &mut heads, &ends, min, shift, &map));
                    }
                }

                for b in 0..m {
                    let bucket = &mut keys[starts[b]..ends[b]];
                    if bucket.len() <= LEAF_SLACK * leaf {
                        count(6, 1);
                        prof!(4, leaf_sort(bucket, scratch));
                    } else {
                        let (lo, hi) = prof!(1, min_max(bucket));
                        msd(bucket, lo, hi, leaf, scratch);
                    }
                }
            }

            /// Counts of the digit `(key - min) >> shift` (`r` buckets), in 4 lanes so
            /// that consecutive keys of the same bucket do not wait on the same counter.
            fn histogram(keys: &[$uint], min: $uint, shift: u32, r: usize) -> Vec<usize> {
                let mask = (r - 1) as $uint;
                let mut total = vec![0usize; r];
                // Blocks of at most 2^31 keys: a u32 lane counter cannot overflow.
                for block in keys.chunks(1 << 31) {
                    let mut lanes = vec![0u32; 4 * r];
                    let (l0, rest) = lanes.split_at_mut(r);
                    let (l1, rest) = rest.split_at_mut(r);
                    let (l2, l3) = rest.split_at_mut(r);
                    let mut quads = block.chunks_exact(4);
                    for q in &mut quads {
                        l0[((q[0].wrapping_sub(min) >> shift) & mask) as usize] += 1;
                        l1[((q[1].wrapping_sub(min) >> shift) & mask) as usize] += 1;
                        l2[((q[2].wrapping_sub(min) >> shift) & mask) as usize] += 1;
                        l3[((q[3].wrapping_sub(min) >> shift) & mask) as usize] += 1;
                    }
                    for &k in quads.remainder() {
                        l0[((k.wrapping_sub(min) >> shift) & mask) as usize] += 1;
                    }
                    for b in 0..r {
                        total[b] += (l0[b] + l1[b] + l2[b] + l3[b]) as usize;
                    }
                }
                total
            }

            /// In-place permutation by bucket `map[(key - min) >> shift]` (ska sort's
            /// swap loop):
            /// rounds over the unsorted part of every bucket, each key swapped to the
            /// next free slot of its bucket, until every bucket is complete. The
            /// largest bucket is never scanned: it is complete when the others are.
            fn permute(keys: &mut [$uint], heads: &mut [usize], ends: &[usize], min: $uint, shift: u32, map: &[u32]) {
                let mask = (map.len() - 1) as $uint;
                // SAFETY: the masked digit indexes `map`.
                let digit = |k: $uint| unsafe { *map.get_unchecked(((k.wrapping_sub(min) >> shift) & mask) as usize) as usize };
                let mut todo: Vec<usize> = (0..heads.len()).filter(|&b| ends[b] > heads[b]).collect();
                if let Some(pos) = (0..todo.len()).max_by_key(|&i| ends[todo[i]] - heads[todo[i]]) {
                    todo.swap_remove(pos);
                }
                let p = keys.as_mut_ptr();
                // SAFETY: every index read or written is a slot of a bucket region,
                // below `keys.len()`: `o` runs over `heads[b]..ends[b]` and the head of a
                // bucket stays below its end while keys of that bucket remain to place
                // (the histogram counted exactly the keys of each bucket).
                while !todo.is_empty() {
                    todo.retain(|&b| {
                        let (start, end) = (heads[b], ends[b]);
                        let mut o = start;
                        unsafe {
                            while o + 4 <= end {
                                let d0 = digit(*p.add(o));
                                let d1 = digit(*p.add(o + 1));
                                let d2 = digit(*p.add(o + 2));
                                let d3 = digit(*p.add(o + 3));
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
                                let d = digit(*p.add(o));
                                std::ptr::swap(p.add(o), p.add(heads[d]));
                                heads[d] += 1;
                                o += 1;
                            }
                        }
                        heads[b] < ends[b]
                    });
                }
            }

            /// Leaf sort: the DLSD (`float_dlsd`) on the encoded keys of a bucket;
            /// tests can switch to the fused LSD (`LEAF_MODE` 1) for A/B comparisons.
            #[inline(always)]
            fn leaf_sort(bucket: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>) {
                #[cfg(test)]
                if LEAF_MODE.load(std::sync::atomic::Ordering::Relaxed) == 1 {
                    return w::lsd_encoded_mode(bucket, scratch, Digits::Auto);
                }
                d::dlsd_encoded(bucket, scratch);
            }


            /// Keys per block of the block permutation (512 bytes).
            const BLOCK: usize = 512 / std::mem::size_of::<$uint>();

            /// In-place permutation by bucket `map[(key - min) >> shift]` with blocks
            /// (the scheme of IPS4o / IPS2Ra, single thread). Buckets are the ranges
            /// `starts[b]..ends[b]` (from the histogram).
            ///
            /// 1. Classification: one sequential read; each key goes to a small buffer
            ///    of its bucket, a full buffer is written back as a block at the front
            ///    of the array (never past the keys already read).
            /// 2. Block permutation: the region of bucket `b` is cut at multiples of
            ///    `BLOCK` (`d[b]` = `starts[b]` rounded up); the full blocks are moved
            ///    to the regions of their buckets, a block at a time.
            /// 3. Cleanup: the unaligned edges of each bucket are filled with the
            ///    blocks that spilled over its end and the keys left in its buffer.
            ///
            /// Memory: `BLOCK` keys per bucket plus 3 blocks.
            fn permute_blocks(keys: &mut [$uint], starts: &[usize], ends: &[usize], min: $uint, shift: u32, map: &[u32]) {
                let n = keys.len();
                let m = starts.len();
                let mask = (map.len() - 1) as $uint;
                // SAFETY: the masked digit indexes `map`.
                let bucket = |k: $uint| unsafe { *map.get_unchecked(((k.wrapping_sub(min) >> shift) & mask) as usize) as usize };
                let p = keys.as_mut_ptr();

                #[cfg(test)]
                let t1 = std::time::Instant::now();
                // 1. Classification.
                let mut buf = vec![0 as $uint; m * BLOCK];
                let mut lens = vec![0usize; m];
                let mut w = 0usize;
                // SAFETY: bucket indices are below `m` (built by `msd`), buffer slots
                // below `BLOCK`; a block is written at `w..w + BLOCK` where `w + BLOCK`
                // = keys read - keys buffered <= keys read, so only keys already read
                // are overwritten.
                unsafe {
                    let bp = buf.as_mut_ptr();
                    let lp = lens.as_mut_ptr();
                    let push = |k: $uint, b: usize, w: &mut usize| {
                        let l = *lp.add(b);
                        *bp.add(b * BLOCK + l) = k;
                        if l + 1 == BLOCK {
                            std::ptr::copy_nonoverlapping(bp.add(b * BLOCK), p.add(*w), BLOCK);
                            *w += BLOCK;
                            *lp.add(b) = 0;
                        } else {
                            *lp.add(b) = l + 1;
                        }
                    };
                    let mut i = 0;
                    while i + 4 <= n {
                        let (k0, k1, k2, k3) = (*p.add(i), *p.add(i + 1), *p.add(i + 2), *p.add(i + 3));
                        let (b0, b1, b2, b3) = (bucket(k0), bucket(k1), bucket(k2), bucket(k3));
                        if b0 == b1 && b0 == b2 && b0 == b3 && *lp.add(b0) + 4 <= BLOCK {
                            // Same bucket and room for 4: one length update.
                            let l = *lp.add(b0);
                            let q = bp.add(b0 * BLOCK + l);
                            q.write(k0);
                            q.add(1).write(k1);
                            q.add(2).write(k2);
                            q.add(3).write(k3);
                            if l + 4 == BLOCK {
                                std::ptr::copy_nonoverlapping(bp.add(b0 * BLOCK), p.add(w), BLOCK);
                                w += BLOCK;
                                *lp.add(b0) = 0;
                            } else {
                                *lp.add(b0) = l + 4;
                            }
                        } else {
                            push(k0, b0, &mut w);
                            push(k1, b1, &mut w);
                            push(k2, b2, &mut w);
                            push(k3, b3, &mut w);
                        }
                        i += 4;
                    }
                    while i < n {
                        let k = *p.add(i);
                        push(k, bucket(k), &mut w);
                        i += 1;
                    }
                }

                #[cfg(test)]
                let t2 = std::time::Instant::now();
                #[cfg(test)]
                PHASES[0].fetch_add((t2 - t1).as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
                // 2. Block permutation. Region of bucket b: d[b]..d[b + 1]. In it,
                // wp[b]..rp[b] holds blocks not yet processed, d[b]..wp[b] blocks of b
                // in place, rp[b]..d[b + 1] free slots. A bucket's full blocks fit in
                // its region (rounding up is monotone), except that the last region
                // may end past `n`: a block written across `n` keeps its part past `n`
                // in `overflow`.
                let align = |x: usize| x.div_ceil(BLOCK) * BLOCK;
                let mut d: Vec<usize> = starts.iter().map(|&s| align(s)).collect();
                d.push(align(n));
                let mut wp: Vec<usize> = d[..m].to_vec();
                let mut rp: Vec<usize> = (0..m).map(|b| w.clamp(d[b], d[b + 1])).collect();
                // Two block buffers used in turn: `bufs[c]` holds the block in hand.
                let mut bufs = [[0 as $uint; BLOCK]; 2];
                let mut c = 0usize;
                let mut overflow: Vec<$uint> = Vec::new();
                // SAFETY: processed slots `wp..rp` are whole blocks below `w <= n`;
                // free slots may cross `n`, handled through `overflow`.
                unsafe {
                    for b in 0..m {
                        loop {
                            // Blocks already in their bucket's region stay.
                            while wp[b] < rp[b] && bucket(*p.add(wp[b])) == b {
                                wp[b] += BLOCK;
                            }
                            if wp[b] >= rp[b] {
                                break;
                            }
                            rp[b] -= BLOCK;
                            std::ptr::copy_nonoverlapping(p.add(rp[b]), bufs[c].as_mut_ptr(), BLOCK);
                            loop {
                                let t = bucket(bufs[c][0]);
                                while wp[t] < rp[t] && bucket(*p.add(wp[t])) == t {
                                    wp[t] += BLOCK;
                                }
                                let slot = wp[t];
                                wp[t] += BLOCK;
                                if slot < rp[t] {
                                    // An unprocessed block: swap and go on with it.
                                    std::ptr::copy_nonoverlapping(p.add(slot), bufs[1 - c].as_mut_ptr(), BLOCK);
                                    std::ptr::copy_nonoverlapping(bufs[c].as_ptr(), p.add(slot), BLOCK);
                                    c = 1 - c;
                                } else {
                                    // A free slot.
                                    if slot + BLOCK <= n {
                                        std::ptr::copy_nonoverlapping(bufs[c].as_ptr(), p.add(slot), BLOCK);
                                    } else {
                                        let inside = n - slot;
                                        std::ptr::copy_nonoverlapping(bufs[c].as_ptr(), p.add(slot), inside);
                                        overflow.extend_from_slice(&bufs[c][inside..]);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }

                #[cfg(test)]
                let t3 = std::time::Instant::now();
                #[cfg(test)]
                PHASES[1].fetch_add((t3 - t2).as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
                // 3. Cleanup, left to right: the holes of bucket b (its head before
                // d[b], its tail after its blocks) take the keys of its blocks spilled
                // past its end, then the keys of its buffer. The spill of bucket b lies
                // in the head of the next buckets, which are filled after it.
                for b in 0..m {
                    let (s, e) = (starts[b], ends[b]);
                    let blocks_start = d[b];
                    let blocks_end = wp[b];
                    let head_end = blocks_start.min(e);
                    // Tail: after its blocks (empty when its blocks reach or pass e).
                    let tail_start = blocks_end.min(e);
                    // Sources: spill (blocks of b past e), then the buffer.
                    let spill_start = blocks_start.max(e);
                    let spill_end = blocks_end.max(spill_start);
                    let mut src: Vec<$uint> = Vec::new();
                    if spill_end > spill_start {
                        let inside_end = spill_end.min(n);
                        if inside_end > spill_start {
                            src.extend_from_slice(&keys[spill_start..inside_end]);
                        }
                        if spill_end > n {
                            src.extend_from_slice(&overflow);
                        }
                    }
                    src.extend_from_slice(&buf[b * BLOCK..b * BLOCK + lens[b]]);
                    let holes = (head_end - s) + (e - tail_start);
                    assert_eq!(src.len(), holes, "block permutation: bucket {}", b);
                    let (h, t) = src.split_at(head_end - s);
                    keys[s..head_end].copy_from_slice(h);
                    keys[tail_start..e].copy_from_slice(t);
                }
                #[cfg(test)]
                PHASES[2].fetch_add(t3.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
            }
        }
    };
}

roller_width!(u64, w64, d64, r64);
roller_width!(u32, w32, d32, r32);
pub(crate) use r64::roller as roller64;
#[allow(unused_imports)]
pub(crate) use r32::roller as roller32;
