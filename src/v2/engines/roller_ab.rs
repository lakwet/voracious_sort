//! Dev only (feature `ab`): the rollercoaster with out-of-place MSD passes
//! (copy of `roller.rs`). One buffer of the array's size for the whole
//! sort: an MSD pass scatters the keys into it by bucket; each bucket is
//! copied back to its place in the array (it fits in the cache) and sorted
//! there by the DLSD (the copy of `dlsd/sort_ab.rs`), whose own buffer is
//! the part of the big one the bucket has just left; a bucket still too
//! large goes through one more MSD pass, array and buffer swapped. No other
//! allocation, nothing zeroed.

/// Target bucket size (keys) for the leaves (as `roller.rs`).
const LEAF_KEYS: usize = 1 << 15;
/// A bucket up to this many times `LEAF_KEYS` still goes to a leaf sort.
const LEAF_SLACK: usize = 4;
/// Most buckets made by one MSD pass (write streams of the scatter).
const MAX_BUCKETS: usize = 1024;
/// MSD digit width bounds.
const MIN_BITS: u32 = 8;
const MAX_BITS: u32 = 18;
/// log2 of the average number of keys per digit value.
const KEYS_PER_DIGIT_LOG: u32 = 7;

macro_rules! roller_oop {
    ($uint:ty, $w:ident, $d:ident, $m:ident) => {
        pub(crate) mod $m {
            use std::mem::MaybeUninit;

            use super::super::dlsd::$d as d;
            use super::super::lsd::$w as w;
            use super::*;

            /// Sorts raw bits.
            pub(crate) fn roller(bits: &mut [$uint]) {
                let n = bits.len();
                if n <= LEAF_KEYS {
                    d::dlsd(bits);
                    return;
                }
                let (mut min, mut max) = (<$uint>::MAX, 0);
                for b in bits.iter_mut() {
                    let k = w::encode_key(*b);
                    *b = k;
                    min = min.min(k);
                    max = max.max(k);
                }
                let mut buf: Vec<MaybeUninit<$uint>> = Vec::with_capacity(n);
                // SAFETY: `MaybeUninit` needs no initialization.
                unsafe { buf.set_len(n) };
                msd(bits, &mut buf, min, max);
            }

            /// Range of encoded keys.
            fn min_max(keys: &[$uint]) -> ($uint, $uint) {
                keys.iter().fold((<$uint>::MAX, 0), |(lo, hi), &k| (lo.min(k), hi.max(k)))
            }

            /// Sorts the encoded keys of `keys` (all in `min..=max`) and
            /// writes them back decoded, in `keys`; `buf` (as long) is free
            /// space.
            fn msd(keys: &mut [$uint], buf: &mut [MaybeUninit<$uint>], min: $uint, max: $uint) {
                let n = keys.len();
                let range = max - min;
                if range == 0 {
                    keys.iter_mut().for_each(|k| *k = w::decode_key(*k));
                    return;
                }
                let top = <$uint>::BITS - range.leading_zeros();
                let log_n = usize::BITS - n.leading_zeros();
                let width =
                    log_n.saturating_sub(KEYS_PER_DIGIT_LOG).clamp(MIN_BITS, MAX_BITS).min(top);
                let shift = top - width;
                let counts = histogram(keys, min, shift, 1 << width);
                let (map, ends) = group_digits(&counts, n);
                if ends.len() == 1 {
                    d::dlsd_encoded_in(keys, buf);
                    return;
                }
                let mut heads: Vec<usize> =
                    std::iter::once(0).chain(ends[..ends.len() - 1].iter().copied()).collect();
                let starts = heads.clone();
                scatter(keys, buf, &mut heads, min, shift, &map);
                let kp = keys.as_mut_ptr();
                let bp = buf.as_mut_ptr() as *mut $uint;
                for (&start, &end) in starts.iter().zip(ends.iter()) {
                    let len = end - start;
                    if len == 0 {
                        continue;
                    }
                    // SAFETY: both slices lie in their own (disjoint) array,
                    // `start..end` within `0..n`; the scatter wrote every slot
                    // of `buf`.
                    let (a, b) = unsafe {
                        (
                            std::slice::from_raw_parts_mut(kp.add(start), len),
                            std::slice::from_raw_parts_mut(bp.add(start), len),
                        )
                    };
                    if len <= LEAF_SLACK * LEAF_KEYS {
                        a.copy_from_slice(b);
                        // SAFETY: the bucket's slots of `buf` are free now.
                        let space = unsafe {
                            std::slice::from_raw_parts_mut(
                                b.as_mut_ptr() as *mut MaybeUninit<$uint>,
                                len,
                            )
                        };
                        d::dlsd_encoded_in(a, space);
                    } else {
                        // Sorted where it is (in `buf`), the array's slots as
                        // its free space, then copied back.
                        let (lo, hi) = min_max(b);
                        // SAFETY: the array's slots of the bucket are free.
                        let space = unsafe {
                            std::slice::from_raw_parts_mut(
                                a.as_mut_ptr() as *mut MaybeUninit<$uint>,
                                len,
                            )
                        };
                        msd(b, space, lo, hi);
                        a.copy_from_slice(b);
                    }
                }
            }

            /// Consecutive digit values grouped into buckets (as `roller.rs`).
            fn group_digits(counts: &[usize], n: usize) -> (Vec<u32>, Vec<usize>) {
                let target = LEAF_KEYS.max(n.div_ceil(MAX_BUCKETS));
                let mut map = vec![0u32; counts.len()];
                let mut ends: Vec<usize> = Vec::new();
                let (mut sum, mut open) = (0usize, 0usize);
                for (digit, &c) in counts.iter().enumerate() {
                    if open > 0 && open + c > target {
                        ends.push(sum);
                        open = 0;
                    }
                    map[digit] = ends.len() as u32;
                    sum += c;
                    open += c;
                }
                ends.push(sum);
                (map, ends)
            }

            /// Counts of the digit `(key - min) >> shift` in 4 lanes.
            fn histogram(keys: &[$uint], min: $uint, shift: u32, r: usize) -> Vec<usize> {
                let mask = (r - 1) as $uint;
                let mut total = vec![0usize; r];
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

            /// Stable out-of-place scatter of `keys` into `buf` by bucket
            /// `map[(key - min) >> shift]`, 4 keys read before their stores.
            fn scatter(
                keys: &[$uint],
                buf: &mut [MaybeUninit<$uint>],
                heads: &mut [usize],
                min: $uint,
                shift: u32,
                map: &[u32],
            ) {
                let mask = (map.len() - 1) as $uint;
                // SAFETY: the masked digit indexes `map`.
                let bucket = |k: $uint| unsafe {
                    *map.get_unchecked(((k.wrapping_sub(min) >> shift) & mask) as usize) as usize
                };
                let dst = buf.as_mut_ptr() as *mut $uint;
                let mut quads = keys.chunks_exact(4);
                // SAFETY (all writes): each bucket's head plus its keys already
                // written is below its end; the buckets partition 0..n.
                unsafe {
                    for q in &mut quads {
                        let (a, b, c, e) = (q[0], q[1], q[2], q[3]);
                        let (ba, bb, bc, be) = (bucket(a), bucket(b), bucket(c), bucket(e));
                        let pa = *heads.get_unchecked(ba);
                        *heads.get_unchecked_mut(ba) = pa + 1;
                        let pb = *heads.get_unchecked(bb);
                        *heads.get_unchecked_mut(bb) = pb + 1;
                        let pc = *heads.get_unchecked(bc);
                        *heads.get_unchecked_mut(bc) = pc + 1;
                        let pe = *heads.get_unchecked(be);
                        *heads.get_unchecked_mut(be) = pe + 1;
                        dst.add(pa).write(a);
                        dst.add(pb).write(b);
                        dst.add(pc).write(c);
                        dst.add(pe).write(e);
                    }
                    for &k in quads.remainder() {
                        let bk = bucket(k);
                        let p = *heads.get_unchecked(bk);
                        *heads.get_unchecked_mut(bk) = p + 1;
                        dst.add(p).write(k);
                    }
                }
            }
        }
    };
}

roller_oop!(u32, wu32, su32, ou32);
roller_oop!(u32, wi32, si32, oi32);
