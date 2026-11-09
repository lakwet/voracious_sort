//! Fused LSD radix sort on float bits (encoding fused into the first scatter
//! pass, decoding into the last one), tuned for single thread. Several ideas
//! come from blazing-radix (MIT License, Copyright (c) 2026 Max Tikhomirov):
//!
//! - one read pass builds the histograms of every digit, with a layout known
//!   at compile time (unrolled digit loop) and several lanes (4 for bytes, 2
//!   for 11-bit digits) so that consecutive equal digits do not serialize on
//!   the same counter (store-to-load dependency);
//! - `u32` counters while `n <= u32::MAX` (half the cache footprint), `usize`
//!   counters beyond;
//! - 11-bit digits (3 passes instead of 4 on 32-bit keys) when the keys vary
//!   on enough bits; digits equal in every key are skipped;
//! - the scatter loop reads 4 keys before writing them, with a fast path when
//!   the 4 keys share their digit; on the top digit (sign + exponent: few,
//!   unbalanced buckets) the 4 positions are resolved inside the block
//!   without branches, so the 4 counter loads are independent;
//! - the scratch buffer is not zero-initialized: the first scatter pass
//!   writes every slot of it before anything reads it.

use std::mem::MaybeUninit;

/// Below this size the encoded keys are sorted with Rust's `sort_unstable`.
const LSD_MIN: usize = 128;
/// From this size 11-bit digits may be used.
const WIDE_MIN: usize = 8192;
/// Up to this size 11-bit digits may be used on 32-bit keys: beyond, the
/// 2048 write streams of a scatter pass cost more than the saved pass
/// (measured on Snapdragon X Elite, to check on x86).
const WIDE_MAX_32: usize = 4_000_000;

/// Digit layout forced for experiments (`Auto` in production).
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Digits {
    Auto,
    Bytes,
    Wide,
}

/// Histogram counter: `u32` for arrays up to `u32::MAX` elements.
trait Counter: Copy + std::ops::AddAssign {
    const ZERO: Self;
    const ONE: Self;
    const FOUR: Self;
    fn get(self) -> usize;
    fn from_usize(v: usize) -> Self;
}

impl Counter for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const FOUR: Self = 4;
    #[inline(always)]
    fn get(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v as u32
    }
}

impl Counter for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const FOUR: Self = 4;
    #[inline(always)]
    fn get(self) -> usize {
        self
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v
    }
}

macro_rules! fused_lsd {
    ($uint:ty, $sint:ty, $name:ident, $wide_bits:expr) => {
        pub(crate) mod $name {
            use super::{Counter, Digits, MaybeUninit, LSD_MIN, WIDE_MAX_32, WIDE_MIN};

            const BITS: u32 = <$uint>::BITS;

            #[inline(always)]
            fn encode(bits: $uint) -> $uint {
                const SIGN: $uint = 1 << (BITS - 1);
                bits ^ ((((bits as $sint) >> (BITS - 1)) as $uint) | SIGN)
            }

            #[inline(always)]
            fn decode(key: $uint) -> $uint {
                const SIGN: $uint = 1 << (BITS - 1);
                key ^ ((key >> (BITS - 1)).wrapping_sub(1) | SIGN)
            }

            #[inline(always)]
            fn digit<const R: usize>(key: $uint, shift: u32) -> usize {
                ((key >> shift) as usize) & (R - 1)
            }

            /// Histograms of the `P` digits of `STEP` bits (`R = 2^STEP`
            /// buckets) of the encoded keys, in `L` lanes (element `i`
            /// counted in lane `i % L`, so that consecutive keys sharing a
            /// digit do not wait on the same counter). The layout is known
            /// at compile time, so the digit loop is unrolled.
            fn hist_const<C: Counter, const R: usize, const P: usize, const L: usize>(
                bits: &[$uint],
                step: u32,
            ) -> Vec<[C; R]> {
                let mut lanes = vec![[[C::ZERO; R]; P]; L];
                let mut chunks = bits.chunks_exact(L);
                for chunk in &mut chunks {
                    for l in 0..L {
                        let key = encode(chunk[l]);
                        let table = &mut lanes[l];
                        for t in 0..P {
                            table[t][digit::<R>(key, step * t as u32)] += C::ONE;
                        }
                    }
                }
                for &value in chunks.remainder() {
                    let key = encode(value);
                    for t in 0..P {
                        lanes[0][t][digit::<R>(key, step * t as u32)] += C::ONE;
                    }
                }
                let mut total = vec![[C::ZERO; R]; P];
                for lane in lanes.iter() {
                    for t in 0..P {
                        for b in 0..R {
                            total[t][b] += lane[t][b];
                        }
                    }
                }
                total
            }

            /// One stable scatter pass. `FIRST` encodes the raw bits read,
            /// `LAST` decodes the keys written.
            ///
            /// # Safety
            /// `dst` must be valid for `src.len()` writes and must not
            /// overlap `src`; `heads` must be the exclusive prefix sums of
            /// the digit at `shift` of exactly the keys of `src`.
            #[inline(always)]
            unsafe fn scatter<C: Counter, const R: usize, const FIRST: bool, const LAST: bool>(
                src: &[$uint],
                dst: *mut $uint,
                heads: &mut [C; R],
                shift: u32,
            ) {
                let key = |v: $uint| if FIRST { encode(v) } else { v };
                let out = |k: $uint| if LAST { decode(k) } else { k };
                let mut blocks = src.chunks_exact(4);
                // SAFETY (all writes): for each bucket, its head plus the
                // number of its keys already written is an unused index below
                // `src.len()` (buckets partition 0..len); `dst` is valid for
                // `src.len()` writes (caller).
                // Branch-free position resolution on the top digit only
                // (sign + exponent for floats: few, unbalanced buckets, so
                // consecutive keys often share a bucket counter).
                if shift as usize + R.trailing_zeros() as usize >= BITS as usize {
                    for block in &mut blocks {
                        let (a, b, c, d) = (key(block[0]), key(block[1]), key(block[2]), key(block[3]));
                        let (da, db) = (digit::<R>(a, shift), digit::<R>(b, shift));
                        let (dc, dd) = (digit::<R>(c, shift), digit::<R>(d, shift));
                        // Positions resolved inside the block: each key adds
                        // the number of previous keys of the block in the
                        // same bucket, so the 4 counter loads are independent
                        // (no store-to-load chain inside the block).
                        let pa = heads[da].get();
                        let pb = heads[db].get() + usize::from(db == da);
                        let pc = heads[dc].get() + usize::from(dc == da) + usize::from(dc == db);
                        let pd = heads[dd].get()
                            + usize::from(dd == da)
                            + usize::from(dd == db)
                            + usize::from(dd == dc);
                        // Stores in block order: for equal digits the last
                        // one (largest position) wins.
                        heads[da] = C::from_usize(pa + 1);
                        heads[db] = C::from_usize(pb + 1);
                        heads[dc] = C::from_usize(pc + 1);
                        heads[dd] = C::from_usize(pd + 1);
                        unsafe {
                            dst.add(pa).write(out(a));
                            dst.add(pb).write(out(b));
                            dst.add(pc).write(out(c));
                            dst.add(pd).write(out(d));
                        }
                    }
                }
                for block in &mut blocks {
                    let (a, b, c, d) = (key(block[0]), key(block[1]), key(block[2]), key(block[3]));
                    let (da, db) = (digit::<R>(a, shift), digit::<R>(b, shift));
                    let (dc, dd) = (digit::<R>(c, shift), digit::<R>(d, shift));
                    if da == dd && da == db && dc == dd {
                        let p = heads[da].get();
                        heads[da] += C::FOUR;
                        unsafe {
                            let w = dst.add(p);
                            w.write(out(a));
                            w.add(1).write(out(b));
                            w.add(2).write(out(c));
                            w.add(3).write(out(d));
                        }
                        continue;
                    }
                    let pa = heads[da].get();
                    heads[da] += C::ONE;
                    let pb = heads[db].get();
                    heads[db] += C::ONE;
                    let pc = heads[dc].get();
                    heads[dc] += C::ONE;
                    let pd = heads[dd].get();
                    heads[dd] += C::ONE;
                    unsafe {
                        dst.add(pa).write(out(a));
                        dst.add(pb).write(out(b));
                        dst.add(pc).write(out(c));
                        dst.add(pd).write(out(d));
                    }
                }
                for &v in blocks.remainder() {
                    let k = key(v);
                    let dk = digit::<R>(k, shift);
                    let p = heads[dk].get();
                    heads[dk] += C::ONE;
                    unsafe { dst.add(p).write(out(k)) };
                }
            }

            fn sort_digits<C: Counter, const R: usize>(
                bits: &mut [$uint],
                all_shifts: &[u32],
                mut tables: Vec<[C; R]>,
            ) {
                let size = bits.len();
                // Skip the digits equal in every key.
                let mut passes: Vec<(u32, usize)> = Vec::with_capacity(all_shifts.len());
                for (t, &shift) in all_shifts.iter().enumerate() {
                    if !tables[t].iter().any(|&c| c.get() == size) {
                        passes.push((shift, t));
                    }
                }
                if passes.is_empty() {
                    return;
                }
                // Exclusive prefix sums.
                for table in tables.iter_mut() {
                    let mut sum = C::ZERO;
                    for count in table.iter_mut() {
                        let mut next = sum;
                        next += *count;
                        *count = sum;
                        sum = next;
                    }
                    assert_eq!(sum.get(), size);
                }
                let mut buffer: Vec<MaybeUninit<$uint>> = Vec::with_capacity(size);
                // SAFETY: `MaybeUninit` needs no initialization.
                unsafe { buffer.set_len(size) };
                let buffer_ptr = buffer.as_mut_ptr() as *mut $uint;
                let last = passes.len() - 1;
                for (pass, &(shift, t)) in passes.iter().enumerate() {
                    let heads = &mut tables[t];
                    // SAFETY: pass 0 reads `bits` and writes every slot of
                    // `buffer` (a scatter writes each of the `size`
                    // positions once), so from pass 1 on the buffer is
                    // initialized and may be read as `[$uint]`. Array and
                    // buffer never overlap. Heads are the prefix sums of this
                    // digit over the same multiset of keys at every pass.
                    unsafe {
                        let (src, dst): (&[$uint], *mut $uint) = if pass % 2 == 0 {
                            (&*bits, buffer_ptr)
                        } else {
                            (std::slice::from_raw_parts(buffer_ptr, size), bits.as_mut_ptr())
                        };
                        match (pass == 0, pass == last) {
                            (true, true) => scatter::<C, R, true, true>(src, dst, heads, shift),
                            (true, false) => scatter::<C, R, true, false>(src, dst, heads, shift),
                            (false, true) => scatter::<C, R, false, true>(src, dst, heads, shift),
                            (false, false) => scatter::<C, R, false, false>(src, dst, heads, shift),
                        }
                    }
                }
                if passes.len() % 2 == 1 {
                    // SAFETY: the buffer is fully initialized (see above).
                    let sorted = unsafe { std::slice::from_raw_parts(buffer_ptr, size) };
                    bits.copy_from_slice(sorted);
                }
            }

            /// Bits varying among 16 spread encoded keys (blazing's sample).
            fn variation(bits: &[$uint]) -> u32 {
                let first = encode(bits[0]);
                bits.iter()
                    .step_by((bits.len() / 16).max(1))
                    .take(16)
                    .fold(0, |acc, &v| acc | (encode(v) ^ first))
                    .count_ones()
            }

            fn sort_with<C: Counter>(bits: &mut [$uint], mode: Digits) {
                let wide = match mode {
                    Digits::Bytes => false,
                    Digits::Wide => true,
                    Digits::Auto => {
                        bits.len() >= WIDE_MIN
                            && (BITS == 64 || bits.len() <= WIDE_MAX_32)
                            && variation(bits) >= $wide_bits
                    }
                };
                if wide {
                    let shifts: Vec<u32> = (0..BITS).step_by(11).collect();
                    let tables = hist_const::<C, 2048, { ((BITS + 10) / 11) as usize }, 2>(bits, 11);
                    sort_digits::<C, 2048>(bits, &shifts, tables);
                } else {
                    let shifts: Vec<u32> = (0..BITS).step_by(8).collect();
                    let tables = hist_const::<C, 256, { (BITS / 8) as usize }, 4>(bits, 8);
                    sort_digits::<C, 256>(bits, &shifts, tables);
                }
            }

            /// Sorts raw float bits in total order.
            pub(crate) fn lsd(bits: &mut [$uint]) {
                lsd_mode(bits, Digits::Auto);
            }

            /// Sorts raw float bits in total order with a forced layout.
            pub(crate) fn lsd_mode(bits: &mut [$uint], mode: Digits) {
                if bits.len() <= LSD_MIN {
                    bits.iter_mut().for_each(|b| *b = encode(*b));
                    bits.sort_unstable();
                    bits.iter_mut().for_each(|b| *b = decode(*b));
                } else if bits.len() <= u32::MAX as usize {
                    sort_with::<u32>(bits, mode);
                } else {
                    sort_with::<usize>(bits, mode);
                }
            }
        }
    };
}

fused_lsd!(u32, i32, w32, 20);
