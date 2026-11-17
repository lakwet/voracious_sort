//! Rollercoaster: in-place MSD passes on the top bits of `key - min` until
//! the buckets fit in the cache, then the DLSD on each bucket (the leaves).
//! The encoding is fused into the pass that finds the range, the decoding
//! into the leaves. The MSD digit starts at the highest varying bit and its
//! values are grouped into buckets of about `LEAF_KEYS` keys (keys are far
//! from uniform over their bits). Memory: the array plus one leaf-sized
//! buffer.

/// Target bucket size (keys) for the leaves: their DLSD prefix then fits in
/// 2 passes of 12 bits (journal §12).
const LEAF_KEYS: usize = 1 << 15;
/// A bucket up to this many times `LEAF_KEYS` still goes to a leaf sort
/// (cheaper than one more MSD pass over it).
const LEAF_SLACK: usize = 4;
/// Most buckets made by one MSD pass (more write streams slow the in-place
/// permutation down).
const MAX_BUCKETS: usize = 1024;
/// MSD digit width bounds.
const MIN_BITS: u32 = 8;
const MAX_BITS: u32 = 18;
/// log2 of the average number of keys per digit value.
const KEYS_PER_DIGIT_LOG: u32 = 7;

macro_rules! roller_width {
    ($uint:ty, $w:ident, $d:ident, $m:ident) => {
        pub(crate) mod $m {
            use std::mem::MaybeUninit;

            use super::super::dlsd::$d as d;
            use super::super::lsd::$w as w;
            use super::*;

            /// Sorts raw bits.
            pub(crate) fn roller(bits: &mut [$uint]) {
                if bits.len() <= LEAF_KEYS {
                    d::dlsd(bits);
                    return;
                }
                // Encode in place and find the range of the keys.
                let (mut min, mut max) = (<$uint>::MAX, 0);
                for b in bits.iter_mut() {
                    let k = w::encode_key(*b);
                    *b = k;
                    min = min.min(k);
                    max = max.max(k);
                }
                let mut scratch: Vec<MaybeUninit<$uint>> = Vec::new();
                msd(bits, min, max, &mut scratch);
            }

            /// Range of encoded keys.
            fn min_max(keys: &[$uint]) -> ($uint, $uint) {
                keys.iter().fold((<$uint>::MAX, 0), |(lo, hi), &k| (lo.min(k), hi.max(k)))
            }

            /// Sorts encoded keys (all in `min..=max`) and writes them back decoded.
            fn msd(
                keys: &mut [$uint],
                min: $uint,
                max: $uint,
                scratch: &mut Vec<MaybeUninit<$uint>>,
            ) {
                let range = max - min;
                if range == 0 {
                    keys.iter_mut().for_each(|k| *k = w::decode_key(*k));
                    return;
                }
                // `key - min` fits in `top` bits; the digit takes the top `width`.
                let top = <$uint>::BITS - range.leading_zeros();
                let log_n = usize::BITS - keys.len().leading_zeros();
                let width =
                    log_n.saturating_sub(KEYS_PER_DIGIT_LOG).clamp(MIN_BITS, MAX_BITS).min(top);
                let shift = top - width;
                let counts = histogram(keys, min, shift, 1 << width);
                let (map, ends) = group_digits(&counts, keys.len());
                let starts: Vec<usize> =
                    std::iter::once(0).chain(ends[..ends.len() - 1].iter().copied()).collect();
                if ends.len() > 1 {
                    permute(keys, &mut starts.clone(), &ends, min, shift, &map);
                }
                for (&start, &end) in starts.iter().zip(ends.iter()) {
                    let bucket = &mut keys[start..end];
                    if bucket.len() <= LEAF_SLACK * LEAF_KEYS {
                        d::dlsd_encoded(bucket, scratch);
                    } else {
                        let (lo, hi) = min_max(bucket);
                        msd(bucket, lo, hi, scratch);
                    }
                }
            }

            /// Consecutive digit values grouped into buckets of about
            /// `LEAF_KEYS` keys (or more, to stay within `MAX_BUCKETS`; a digit
            /// value holding more keys is a bucket on its own). Returns the
            /// bucket of each digit value and the end of each bucket.
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

            /// In-place permutation by bucket `map[(key - min) >> shift]`
            /// (`msd::permute`).
            fn permute(
                keys: &mut [$uint],
                heads: &mut [usize],
                ends: &[usize],
                min: $uint,
                shift: u32,
                map: &[u32],
            ) {
                let mask = (map.len() - 1) as $uint;
                // SAFETY: the masked digit indexes `map`.
                let digit = |k: &$uint| unsafe {
                    *map.get_unchecked(((k.wrapping_sub(min) >> shift) & mask) as usize) as usize
                };
                super::super::msd::permute(keys, heads, ends, digit);
            }
        }
    };
}

roller_width!(u64, w64, d64, r64);
roller_width!(u32, w32, d32, r32);
roller_width!(u64, wu64, du64, ru64);
roller_width!(u32, wu32, du32, ru32);
roller_width!(u64, wi64, di64, ri64);
roller_width!(u32, wi32, di32, ri32);
roller_width!(u128, wu128, du128, ru128);
roller_width!(u128, wi128, di128, ri128);
