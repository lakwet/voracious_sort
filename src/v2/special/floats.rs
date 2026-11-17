//! Float detectors, ideas from blazing-radix (MIT License, Copyright (c)
//! 2026 Max Tikhomirov):
//!
//! - constant array;
//! - integer-valued floats spanning at most 256 integers;
//! - power-of-two grids (`low + i * 2^e`) spanning at most 256 points;
//! - few distinct values (at most `FEW_MAX`, counted in a hash table).
//!
//! A small sample only proposes a case; every value is verified before
//! anything is written, so a failed detection leaves the array untouched.

/// From this size the detectors are tried.
pub(crate) const DETECT_MIN: usize = 129;
/// Most distinct values sorted by counting (`few`), and its hash table size
/// (at most half full).
const FEW_MAX: usize = 256;
const FEW_TABLE_BITS: u32 = 9;
const FEW_TABLE: usize = 1 << FEW_TABLE_BITS;

/// Validation blocks: a short head rejects misleading samples early, then
/// larger blocks amortize the checks.
fn validation_blocks<T>(values: &[T]) -> impl Iterator<Item = &[T]> {
    let (head, tail) = values.split_at(values.len().min(64));
    std::iter::once(head).chain(tail.chunks(1024))
}

/// The lane counters of the range and grid detectors are `u32`, added to
/// `usize` totals at least every `FLUSH` keys: no overflow beyond 4G keys.
const FLUSH: usize = 1 << 30;

/// Adds the 4 lanes of `counts` to `totals` and clears them.
fn flush(counts: &mut [[u32; 256]; 4], totals: &mut [usize; 256]) {
    for b in 0..256 {
        totals[b] += counts[0][b] as usize + counts[1][b] as usize + counts[2][b] as usize + counts[3][b] as usize;
    }
    *counts = [[0; 256]; 4];
}

macro_rules! float_detect {
    (
        $float:ty, $uint:ty, $sint:ty, $mant:expr, $bias:expr,
        $range:ident, $grid:ident, $few:ident, $constant:ident, $detect:ident
    ) => {
        /// Monotone key of the float bits (total order).
        #[inline(always)]
        fn key(bits: $uint) -> $uint {
            const SIGN: $uint = 1 << (<$uint>::BITS - 1);
            bits ^ ((((bits as $sint) >> (<$uint>::BITS - 1)) as $uint) | SIGN)
        }

        /// Integer-valued floats spanning at most 256 integers: counted by
        /// their low byte, then rewritten. Else the grid detector.
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
            let mut totals = [0usize; 256];
            let mut pending = 0usize;
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
                pending += block.len();
                if pending >= FLUSH {
                    flush(&mut counts, &mut totals);
                    pending = 0;
                }
            }
            flush(&mut counts, &mut totals);
            // The span is at most 256 integers: low bytes are distinct.
            let mut start = 0;
            let mut k = lo;
            loop {
                let c = totals[(k as usize) & 255];
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

        /// Floats on a power-of-two grid of at most 256 points: counted by
        /// grid index, then rewritten.
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
            let mut totals = [0usize; 256];
            let mut pending = 0usize;
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
                pending += block.len();
                if pending >= FLUSH {
                    flush(&mut counts, &mut totals);
                    pending = 0;
                }
            }
            flush(&mut counts, &mut totals);
            let lo = *minima.iter().min().unwrap();
            let hi = *maxima.iter().max().unwrap();
            let mut start = 0;
            for i in lo..=hi {
                let c = totals[(i as usize) & 255];
                values[start..start + c].fill(rebuild(i));
                start += c;
            }
            debug_assert_eq!(start, n);
            true
        }

        /// Constant array: first, middle and last keys equal, then every
        /// key checked; nothing is rewritten.
        fn $constant(values: &[$float]) -> bool {
            let n = values.len();
            let first = values[0].to_bits();
            values[n / 2].to_bits() == first
                && values[n - 1].to_bits() == first
                && validation_blocks(values).all(|block| block.iter().fold(0, |d, v| d | (v.to_bits() ^ first)) == 0)
        }

        /// Few distinct values (at most `FEW_MAX`): a spread sample rejects
        /// arrays mostly distinct; one pass counts every value in a small
        /// open-addressing hash table (4 lanes of counters), giving up beyond
        /// `FEW_MAX` values; the array is rewritten from the sorted values.
        pub(crate) fn $few(values: &mut [$float]) -> bool {
            let n = values.len();
            if n < DETECT_MIN {
                return false;
            }
            let hash = |b: $uint| (((b as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) >> (64 - FEW_TABLE_BITS)) as usize;
            // Sample: distinct values among s spread keys (golden ratio
            // positions), counted in the table; stop as soon as half of
            // them are distinct.
            let s = (n / 32).clamp(16, 256);
            let mut keys = [0 as $uint; FEW_TABLE];
            let mut used = [false; FEW_TABLE];
            let mut distinct = 0;
            for i in 0..s as u64 {
                let pos = ((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32;
                let b = values[pos as usize].to_bits();
                let mut h = hash(b);
                while used[h] && keys[h] != b {
                    h = (h + 1) & (FEW_TABLE - 1);
                }
                if !used[h] {
                    used[h] = true;
                    keys[h] = b;
                    distinct += 1;
                    if distinct > s / 2 {
                        return false;
                    }
                }
            }
            // Counting pass (table restarted: exact counts of every key).
            let mut keys = [0 as $uint; FEW_TABLE];
            let mut used = [false; FEW_TABLE];
            let mut counts = [[0usize; FEW_TABLE]; 4];
            let mut distinct = 0;
            let mut quads = values.chunks_exact(4);
            let mut find = |b: $uint, distinct: &mut usize| -> Option<usize> {
                let mut h = hash(b);
                while used[h] && keys[h] != b {
                    h = (h + 1) & (FEW_TABLE - 1);
                }
                if !used[h] {
                    if *distinct == FEW_MAX {
                        return None;
                    }
                    used[h] = true;
                    keys[h] = b;
                    *distinct += 1;
                }
                Some(h)
            };
            for q in &mut quads {
                for lane in 0..4 {
                    match find(q[lane].to_bits(), &mut distinct) {
                        Some(h) => counts[lane][h] += 1,
                        None => return false,
                    }
                }
            }
            for v in quads.remainder() {
                match find(v.to_bits(), &mut distinct) {
                    Some(h) => counts[0][h] += 1,
                    None => return false,
                }
            }
            // Distinct values in total order, then the rewrite.
            let mut found: Vec<($uint, usize)> = (0..FEW_TABLE)
                .filter(|&h| used[h])
                .map(|h| (keys[h], counts[0][h] + counts[1][h] + counts[2][h] + counts[3][h]))
                .collect();
            found.sort_unstable_by_key(|&(b, _)| key(b));
            let mut start = 0;
            for (b, c) in found {
                values[start..start + c].fill(<$float>::from_bits(b));
                start += c;
            }
            debug_assert_eq!(start, n);
            true
        }

        /// Constant, integer range or grid; whether the array is now sorted.
        /// (`few` runs later, in the radix path, after the repair of nearly
        /// sorted arrays: sorted arrays never pay for its sample.)
        pub(crate) fn $detect(values: &mut [$float]) -> bool {
            values.len() >= DETECT_MIN && ($constant(values) || $range(values))
        }
    };
}

mod w32 {
    use super::*;
    float_detect!(f32, u32, i32, 23, 127, range, grid, few, constant, detect);
}

mod w64 {
    use super::*;
    float_detect!(f64, u64, i64, 52, 1023, range, grid, few, constant, detect);
}

pub(crate) use w32::{detect as detect32, few as few32};
pub(crate) use w64::{detect as detect64, few as few64};
