//! Low-cardinality detectors for [`float_sort`](super::float_sort), adapted
//! from blazing-radix (MIT License, Copyright (c) 2026 Max Tikhomirov):
//!
//! - integer-valued floats spanning at most 256 integers,
//! - power-of-two grids (`low + i * 2^e`) spanning at most 256 points,
//! - at most 16 distinct values.
//!
//! A small sample only proposes a case. Every value is then verified (exact
//! bit reconstruction / membership) before anything is written, so a failed
//! detection leaves the array untouched. The worst case is a detection that
//! fails late (one extra read pass).

/// From this size the detectors are tried.
pub(crate) const DETECT_MIN: usize = 129;
/// From this size up to 16 distinct values are detected (else at most 4).
const SPARSE_MIN: usize = 4096;

/// Validation blocks: a short head rejects misleading samples early, then
/// larger blocks amortize the checks.
fn validation_blocks<T>(values: &[T]) -> impl Iterator<Item = &[T]> {
    let (head, tail) = values.split_at(values.len().min(64));
    std::iter::once(head).chain(tail.chunks(1024))
}

macro_rules! float_detect {
    (
        $float:ty, $uint:ty, $sint:ty, $mant:expr, $bias:expr,
        $range:ident, $grid:ident, $sparse:ident, $sparse_k:ident, $detect:ident
    ) => {
        /// Monotone key of the float bits (total order).
        #[inline(always)]
        fn key(bits: $uint) -> $uint {
            const SIGN: $uint = 1 << (<$uint>::BITS - 1);
            bits ^ ((((bits as $sint) >> (<$uint>::BITS - 1)) as $uint) | SIGN)
        }

        fn $range(values: &mut [$float]) -> bool {
            let n = values.len();
            let exact = |v: $float| ((v as $sint) as $float).to_bits() == v.to_bits();
            let (mut lo, mut hi) = (values[0] as $sint, values[0] as $sint);
            for &v in values[..16].iter().chain(values.last()).chain(values.get(n / 2)) {
                if !exact(v) {
                    return $grid(values);
                }
                let k = v as $sint;
                lo = lo.min(k);
                hi = hi.max(k);
                if (hi as i128) - (lo as i128) > 255 {
                    return $grid(values);
                }
            }
            let mut counts = [[0u32; 256]; 4];
            let mut minima = [lo; 4];
            let mut maxima = [hi; 4];
            for block in validation_blocks(values) {
                let mut chunks = block.chunks_exact(4);
                let mut ok = true;
                for chunk in &mut chunks {
                    for lane in 0..4 {
                        let v = chunk[lane];
                        let k = v as $sint;
                        ok &= (k as $float).to_bits() == v.to_bits();
                        counts[lane][(k as usize) & 255] += 1;
                        minima[lane] = minima[lane].min(k);
                        maxima[lane] = maxima[lane].max(k);
                    }
                }
                for &v in chunks.remainder() {
                    let k = v as $sint;
                    ok &= (k as $float).to_bits() == v.to_bits();
                    counts[0][(k as usize) & 255] += 1;
                    minima[0] = minima[0].min(k);
                    maxima[0] = maxima[0].max(k);
                }
                lo = *minima.iter().min().unwrap();
                hi = *maxima.iter().max().unwrap();
                if !ok || (hi as i128) - (lo as i128) > 255 {
                    return false;
                }
            }
            // The span is at most 256 integers: low bytes are distinct.
            let mut start = 0;
            let mut k = lo;
            loop {
                let b = (k as usize) & 255;
                let c = (counts[0][b] + counts[1][b] + counts[2][b] + counts[3][b]) as usize;
                values[start..start + c].fill(k as $float);
                start += c;
                if k == hi {
                    break;
                }
                k += 1;
            }
            debug_assert_eq!(start, n);
            true
        }

        fn $grid(values: &mut [$float]) -> bool {
            let n = values.len();
            let sample = || values[..16].iter().chain(values.last()).chain(values.get(n / 2));
            let (mut low, mut high) = (values[0], values[0]);
            for &v in sample() {
                if !v.is_finite() {
                    return false;
                }
                low = low.min(v);
                high = high.max(v);
            }
            let span = high - low;
            if !span.is_finite() || span <= 0.0 {
                return false;
            }
            // Step: a power of two such that the sampled span is < 256 steps.
            let exponent = ((span.to_bits() >> $mant) as i32) - $bias - 7;
            if !(-($bias - 1)..=($bias - 1)).contains(&exponent) {
                return false;
            }
            let step = <$float>::from_bits(((exponent + $bias) as $uint) << $mant);
            let scale = <$float>::from_bits((($bias - exponent) as $uint) << $mant);
            let index = |v: $float| ((v - low) * scale) as i32;
            let rebuild = |i: i32| low + i as $float * step;
            if sample().any(|&v| rebuild(index(v)).to_bits() != v.to_bits()) {
                return false;
            }
            let mut counts = [[0u32; 256]; 4];
            let (mut minima, mut maxima) = ([i32::MAX; 4], [i32::MIN; 4]);
            for block in validation_blocks(values) {
                let mut chunks = block.chunks_exact(4);
                let mut ok = true;
                for chunk in &mut chunks {
                    for lane in 0..4 {
                        let v = chunk[lane];
                        let i = index(v);
                        ok &= rebuild(i).to_bits() == v.to_bits();
                        counts[lane][(i as usize) & 255] += 1;
                        minima[lane] = minima[lane].min(i);
                        maxima[lane] = maxima[lane].max(i);
                    }
                }
                for &v in chunks.remainder() {
                    let i = index(v);
                    ok &= rebuild(i).to_bits() == v.to_bits();
                    counts[0][(i as usize) & 255] += 1;
                    minima[0] = minima[0].min(i);
                    maxima[0] = maxima[0].max(i);
                }
                let lo = *minima.iter().min().unwrap();
                let hi = *maxima.iter().max().unwrap();
                if !ok || i64::from(hi) - i64::from(lo) > 255 {
                    return false;
                }
            }
            let lo = *minima.iter().min().unwrap();
            let hi = *maxima.iter().max().unwrap();
            let mut start = 0;
            for i in lo..=hi {
                let b = (i as usize) & 255;
                let c = (counts[0][b] + counts[1][b] + counts[2][b] + counts[3][b]) as usize;
                values[start..start + c].fill(rebuild(i));
                start += c;
            }
            debug_assert_eq!(start, n);
            true
        }

        fn $sparse_k<const K: usize>(
            values: &mut [$float],
            reps: &[$uint; 16],
            distinct: usize,
        ) -> bool {
            let keys: [$uint; K] = std::array::from_fn(|i| reps[i]);
            let mut counts = [0usize; K];
            for block in validation_blocks(values) {
                let mut all_known = true;
                for v in block {
                    let b = v.to_bits();
                    let mut known = false;
                    for i in 0..K {
                        let m = b == keys[i];
                        counts[i] += usize::from(m);
                        known |= m;
                    }
                    all_known &= known;
                }
                if !all_known {
                    return false;
                }
            }
            let mut start = 0;
            for i in 0..distinct {
                let end = start + counts[i];
                values[start..end].fill(<$float>::from_bits(reps[i]));
                start = end;
            }
            true
        }

        fn $sparse(values: &mut [$float]) -> bool {
            // Beyond 4 values the per-element cost (K comparisons) only pays
            // on larger arrays (same limit as blazing).
            let limit = if values.len() < SPARSE_MIN { 4 } else { 16 };
            let mut reps = [values[0].to_bits(); 16];
            let mut distinct = 1;
            let head = values.len().min(64);
            for v in values[..head].iter().chain(values.last()) {
                let b = v.to_bits();
                if !reps[..distinct].contains(&b) {
                    if distinct == limit {
                        return false;
                    }
                    reps[distinct] = b;
                    distinct += 1;
                }
            }
            if distinct == 1 {
                // Constant array: verify without rewriting it.
                let first = reps[0];
                return validation_blocks(values)
                    .all(|block| block.iter().fold(0, |d, v| d | (v.to_bits() ^ first)) == 0);
            }
            reps[..distinct].sort_unstable_by_key(|&b| key(b));
            // Unused lanes duplicate the first key; they are never written.
            let first = reps[0];
            reps[distinct..].fill(first);
            if distinct <= 4 {
                $sparse_k::<4>(values, &reps, distinct)
            } else if distinct <= 8 {
                $sparse_k::<8>(values, &reps, distinct)
            } else {
                $sparse_k::<16>(values, &reps, distinct)
            }
        }

        /// Tries the detectors; returns true if the array is now sorted.
        pub(crate) fn $detect(values: &mut [$float]) -> bool {
            values.len() >= DETECT_MIN && ($range(values) || $sparse(values))
        }
    };
}

mod w32 {
    use super::*;
    float_detect!(f32, u32, i32, 23, 127, range, grid, sparse, sparse_k, detect);
}

mod w64 {
    use super::*;
    float_detect!(f64, u64, i64, 52, 1023, range, grid, sparse, sparse_k, detect);
}

pub(crate) use w32::detect as detect32;
pub(crate) use w64::detect as detect64;
