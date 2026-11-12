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
/// Below this size the byte histograms take one lane on the stack.
const SHORT_MAX: usize = 4096;

/// Tests / benchmarks: `SHORT_MAX` override (0: none).
#[cfg(test)]
pub(crate) static SHORT_MAX_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(always)]
fn short_max() -> usize {
    #[cfg(test)]
    {
        match SHORT_MAX_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
            0 => SHORT_MAX,
            v => v,
        }
    }
    #[cfg(not(test))]
    {
        SHORT_MAX
    }
}

/// From this size 11-bit digits may be used.
const WIDE_MIN: usize = 8192;
/// Up to this size 11-bit digits may be used on 32-bit keys: while the
/// keys and the buffer fit in half the L2 (786k keys with 12 MiB). Beyond,
/// the 2048 write streams of a scatter pass leave the cache and cost more
/// than the saved pass (u32 random, bytes / 11-bit: 1M-6M 0.70-0.80 on
/// Snapdragon X Elite; at 600k mixed; was 4M keys).
fn wide_max_32() -> usize {
    #[cfg(test)]
    {
        if WIDE_OLD.load(std::sync::atomic::Ordering::Relaxed) {
            return 4_000_000;
        }
    }
    super::cache_info::l2_bytes() / 2 / (2 * 4)
}

/// Tests / benchmarks: the former 11-bit limit (4M keys), A/B.
#[cfg(test)]
pub(crate) static WIDE_OLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Digit layout forced for experiments (`Auto` in production).
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Digits {
    Auto,
    Bytes,
    Wide,
}

/// Tests / benchmarks: disable the cache-aliasing fix (A/B comparisons).
#[cfg(test)]
pub(crate) static NO_ALIAS_FIX: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn alias_fix_enabled() -> bool {
    #[cfg(test)]
    {
        !NO_ALIAS_FIX.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
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

/// The map from the stored bits to keys whose unsigned order is the wanted
/// order (`encode`), and back (`decode`): floats (total order: flip every
/// bit of negative values, the sign bit of the others), unsigned integers
/// (identity), signed integers (flip the sign bit).
macro_rules! key_codec {
    (float, $uint:ty, $sint:ty) => {
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
    };
    (unsigned, $uint:ty, $sint:ty) => {
        #[inline(always)]
        fn encode(bits: $uint) -> $uint {
            bits
        }

        #[inline(always)]
        fn decode(key: $uint) -> $uint {
            key
        }
    };
    (signed, $uint:ty, $sint:ty) => {
        #[inline(always)]
        fn encode(bits: $uint) -> $uint {
            bits ^ (1 << (BITS - 1))
        }

        #[inline(always)]
        fn decode(key: $uint) -> $uint {
            key ^ (1 << (BITS - 1))
        }
    };
}

macro_rules! fused_lsd {
    ($uint:ty, $sint:ty, $name:ident, $wide_bits:expr, $codec:ident) => {
        #[allow(dead_code)]
        pub(crate) mod $name {
            use super::{alias_fix_enabled, short_max, wide_max_32, Counter, Digits, MaybeUninit, LSD_MIN, WIDE_MIN};

            const BITS: u32 = <$uint>::BITS;

            key_codec!($codec, $uint, $sint);

            #[inline(always)]
            fn digit<const R: usize>(key: $uint, shift: u32) -> usize {
                ((key >> shift) as usize) & (R - 1)
            }

            /// Histograms of the `P` digits of `STEP` bits (`R = 2^STEP`
            /// buckets) of the encoded keys, in `L` lanes (element `i`
            /// counted in lane `i % L`, so that consecutive keys sharing a
            /// digit do not wait on the same counter). The layout is known
            /// at compile time, so the digit loop is unrolled.
            /// Byte histograms of the `P` bytes of the keys, on the stack, in
            /// 2 lanes (consecutive keys alternate, so that the near-constant
            /// top byte of floats does not chain every increment on one
            /// counter), for short arrays.
            #[inline(always)]
            fn hist_stack<C: Counter, const P: usize, const ENC: bool>(bits: &[$uint]) -> [[C; 256]; P] {
                let mut a = [[C::ZERO; 256]; P];
                let mut b = [[C::ZERO; 256]; P];
                let mut pairs = bits.chunks_exact(2);
                for pr in &mut pairs {
                    let (x, y) = if ENC { (encode(pr[0]), encode(pr[1])) } else { (pr[0], pr[1]) };
                    for t in 0..P {
                        a[t][digit::<256>(x, 8 * t as u32)] += C::ONE;
                        b[t][digit::<256>(y, 8 * t as u32)] += C::ONE;
                    }
                }
                for &v in pairs.remainder() {
                    let x = if ENC { encode(v) } else { v };
                    for t in 0..P {
                        a[t][digit::<256>(x, 8 * t as u32)] += C::ONE;
                    }
                }
                for t in 0..P {
                    for d in 0..256 {
                        a[t][d] += b[t][d];
                    }
                }
                a
            }

            fn hist_const<C: Counter, const R: usize, const P: usize, const L: usize, const ENC: bool>(
                bits: &[$uint],
                step: u32,
            ) -> Vec<[C; R]> {
                let encode = |v: $uint| if ENC { encode(v) } else { v };
                let mut lanes = vec![[[C::ZERO; R]; P]; L];
                // Digits constant on 16 spread keys: tested for a shared digit
                // by block of 8 keys (one addition of 8 instead of 8
                // increments of the same counter, each waiting for the
                // previous one); the others are counted key by key (a test
                // failing at random would be mispredicted). Without such a
                // digit, the plain loop below. u32 Aligned 2^16 (two
                // constant low bytes) at 100k: 2.38 -> 1.52 ns per key.
                let first = encode(bits[0]);
                let diff = bits
                    .iter()
                    .step_by((bits.len() / 16).max(1))
                    .take(16)
                    .fold(0, |acc: $uint, &v| acc | (encode(v) ^ first));
                let flat: [bool; P] = std::array::from_fn(|t| digit::<R>(diff, step * t as u32) == 0);
                if L <= 8 && 8 % L == 0 && flat.iter().any(|&f| f) {
                    let eight = {
                        let mut e = C::ZERO;
                        for _ in 0..8 {
                            e += C::ONE;
                        }
                        e
                    };
                    let mut blocks = bits.chunks_exact(8);
                    for b in &mut blocks {
                        let keys: [$uint; 8] = std::array::from_fn(|i| encode(b[i]));
                        for t in 0..P {
                            let d: [usize; 8] = std::array::from_fn(|i| digit::<R>(keys[i], step * t as u32));
                            if !flat[t] {
                                for i in 0..8 {
                                    lanes[i % L][t][d[i]] += C::ONE;
                                }
                                continue;
                            }
                            let mut same = true;
                            for i in 1..8 {
                                same &= d[i] == d[0];
                            }
                            if same {
                                lanes[0][t][d[0]] += eight;
                            } else {
                                for i in 0..8 {
                                    lanes[i % L][t][d[i]] += C::ONE;
                                }
                            }
                        }
                    }
                    for &value in blocks.remainder() {
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
                    return total;
                }
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

            #[inline(always)]
            fn single_digit_enabled() -> bool {
                #[cfg(test)]
                {
                    !super::SINGLE_DIGIT_OFF.load(std::sync::atomic::Ordering::Relaxed)
                }
                #[cfg(not(test))]
                {
                    true
                }
            }

            /// From this size the bucket starts are checked for cache aliasing
            /// (below, the written data fits in L1 and conflicts are cheap;
            /// machine dependent, measured on ARM only). Byte digits only: the
            /// staging lines of 2048 buckets would not fit in L1 themselves.
            const ALIAS_MIN: usize = 32_768;

            /// Elements per staging slot: one 64-byte cache line.
            const WB: usize = 64 / std::mem::size_of::<$uint>();

            /// Whether the starts of the buckets of a pass (exclusive prefix
            /// sums `heads`) crowd on a few positions inside a 4 KiB page.
            /// Integer-valued or grid floats give bucket sizes that are
            /// multiples of powers of two, so up to ~150 of the 256 write
            /// streams then map to the same cache sets and evict each other
            /// (a scatter pass measured 5-12x slower). Criterion: one of the
            /// 64 line positions holds more than 16 bucket starts and more
            /// than 4 times the average (random data: at most ~11; the few
            /// buckets of a top digit never reach it). The 16 relates to the
            /// cache associativity: machine dependent, measured on ARM only.
            fn aliased<C: Counter, const R: usize>(heads: &[C; R], size: usize) -> bool {
                crate::dedicated::cache_info::crowded((0..R).map(|b| {
                    let start = heads[b].get();
                    let end = if b + 1 < R { heads[b + 1].get() } else { size };
                    (start * std::mem::size_of::<$uint>(), (end - start) * std::mem::size_of::<$uint>())
                }))
            }

            /// Scatter pass through per-bucket staging lines: keys are
            /// gathered by bucket in a small contiguous buffer and written to
            /// the destination a cache line at a time, so the scattered
            /// destination lines never need to stay resident together (used
            /// when `aliased`).
            ///
            /// # Safety
            /// Same contract as `scatter`.
            #[inline(always)]
            unsafe fn scatter_staged<C: Counter, const R: usize, const FIRST: bool, const LAST: bool>(
                src: &[$uint],
                dst: *mut $uint,
                heads: &mut [C; R],
                shift: u32,
                staging: &mut [$uint],
                starts: &mut [usize],
            ) {
                assert!(staging.len() >= R * WB && starts.len() >= R);
                let key = |v: $uint| if FIRST { encode(v) } else { v };
                let out = |k: $uint| if LAST { decode(k) } else { k };
                for b in 0..R {
                    starts[b] = heads[b].get();
                }
                let stage = staging.as_mut_ptr();
                // A key going to position `p` is staged at slot `p % WB` of
                // its bucket; when slot `WB - 1` is written, the line (from
                // the bucket start at most) is copied to the destination.
                // SAFETY (all accesses): `b < R` and slots `< WB` stay inside
                // `staging`; positions `from..to` belong to bucket `b` (heads
                // are the prefix sums of this digit over the keys of `src`)
                // and `dst` is valid for `src.len()` writes (caller).
                for &v in src.iter() {
                    let k = key(v);
                    let b = digit::<R>(k, shift);
                    let p = heads[b].get();
                    heads[b] += C::ONE;
                    let slot = p & (WB - 1);
                    unsafe { stage.add(b * WB + slot).write(out(k)) };
                    if slot == WB - 1 {
                        let line = p + 1 - WB;
                        if line >= starts[b] {
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    stage.add(b * WB) as *const [$uint; WB],
                                    dst.add(line) as *mut [$uint; WB],
                                    1,
                                )
                            };
                        } else {
                            let from = starts[b];
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    stage.add(b * WB + (from & (WB - 1))),
                                    dst.add(from),
                                    p + 1 - from,
                                )
                            };
                        }
                    }
                }
                for b in 0..R {
                    let end = heads[b].get();
                    let from = (end & !(WB - 1)).max(starts[b]);
                    if end > from {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                stage.add(b * WB + (from & (WB - 1))),
                                dst.add(from),
                                end - from,
                            )
                        };
                    }
                }
            }

            /// `ENC`: the input holds raw bits (encoded by the first pass);
            /// otherwise it holds encoded keys. The output is always raw
            /// bits. `buffer` is the scratch space, grown when needed.
            fn sort_digits<C: Counter, const R: usize, const ENC: bool>(
                bits: &mut [$uint],
                all_shifts: &[u32],
                tables: &mut [[C; R]],
                buffer: &mut Vec<MaybeUninit<$uint>>,
            ) {
                let size = bits.len();
                // Skip the digits equal in every key: then the bucket of the
                // first key holds all of them (one lookup per digit, not a
                // scan of the table). No allocation (at most 16 digits:
                // bytes of a 128-bit key).
                let first = if ENC { encode(bits[0]) } else { bits[0] };
                let mut pass_store = [(0u32, 0usize); 16];
                let mut np = 0;
                for (t, &shift) in all_shifts.iter().enumerate() {
                    if tables[t][digit::<R>(first, shift)].get() != size {
                        pass_store[np] = (shift, t);
                        np += 1;
                    }
                }
                let passes = &pass_store[..np];
                if passes.is_empty() {
                    if !ENC {
                        bits.iter_mut().for_each(|b| *b = decode(*b));
                    }
                    return;
                }
                // One digit varying, the others equal in every key: each key
                // is the first one with that digit replaced, so the counts
                // alone give the output (no scatter, no copy back): e.g.
                // 16-bit multiples of 256, or keys within one aligned byte.
                if np == 1 && single_digit_enabled() {
                    let (shift, t) = passes[0];
                    let rest = first & !(((R - 1) as $uint) << shift);
                    let mut start = 0;
                    for b in 0..R {
                        let c = tables[t][b].get();
                        if c != 0 {
                            let v = decode(rest | ((b as $uint) << shift));
                            let end = start + c;
                            if c <= 64 && end + 16 <= size {
                                // Short runs by fixed blocks of 16 (the extra
                                // copies are overwritten by the next runs):
                                // a fill call per run costs more.
                                let block = [v; 16];
                                let mut p = start;
                                while p < end {
                                    bits[p..p + 16].copy_from_slice(&block);
                                    p += 16;
                                }
                            } else {
                                bits[start..end].fill(v);
                            }
                            start = end;
                        }
                    }
                    debug_assert_eq!(start, size);
                    return;
                }
                // Exclusive prefix sums (of the digits sorted only).
                for &(_, t) in passes.iter() {
                    let table = &mut tables[t];
                    let mut sum = C::ZERO;
                    for count in table.iter_mut() {
                        let mut next = sum;
                        next += *count;
                        *count = sum;
                        sum = next;
                    }
                    assert_eq!(sum.get(), size);
                }
                if buffer.len() < size {
                    buffer.reserve(size - buffer.len());
                    // SAFETY: `MaybeUninit` needs no initialization.
                    unsafe { buffer.set_len(size) };
                }
                let buffer_ptr = buffer.as_mut_ptr() as *mut $uint;
                let last = passes.len() - 1;
                let mut staging: Vec<$uint> = Vec::new();
                let mut starts: Vec<usize> = Vec::new();
                for (pass, &(shift, t)) in passes.iter().enumerate() {
                    let heads = &mut tables[t];
                    if R == 256 && size >= ALIAS_MIN && alias_fix_enabled() && aliased::<C, R>(heads, size) {
                        if staging.is_empty() {
                            staging = vec![0; R * WB];
                            starts = vec![0; R];
                        }
                        // SAFETY: same as below.
                        unsafe {
                            let (src, dst): (&[$uint], *mut $uint) = if pass % 2 == 0 {
                                (&*bits, buffer_ptr)
                            } else {
                                (std::slice::from_raw_parts(buffer_ptr, size), bits.as_mut_ptr())
                            };
                            let (st, sb) = (&mut staging[..], &mut starts[..]);
                            match (ENC && pass == 0, pass == last) {
                                (true, true) => scatter_staged::<C, R, true, true>(src, dst, heads, shift, st, sb),
                                (true, false) => scatter_staged::<C, R, true, false>(src, dst, heads, shift, st, sb),
                                (false, true) => scatter_staged::<C, R, false, true>(src, dst, heads, shift, st, sb),
                                (false, false) => scatter_staged::<C, R, false, false>(src, dst, heads, shift, st, sb),
                            }
                        }
                        continue;
                    }
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
                        match (ENC && pass == 0, pass == last) {
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
                // Also used on encoded keys: `encode` then flips either all
                // bits or the top one only, so the count barely changes.
                let first = encode(bits[0]);
                bits.iter()
                    .step_by((bits.len() / 16).max(1))
                    .take(16)
                    .fold(0, |acc, &v| acc | (encode(v) ^ first))
                    .count_ones()
            }

            /// Diagnostic (tests only): times in ns per element of each step
            /// of the LSD on `bits`, sorted in place (no digit skipped):
            /// histogram pass, allocation, each scatter pass, final copy;
            /// plus two floors measured on the same data: a copy into fresh
            /// memory (page faults included) and a sequential read + write.
            /// Generate the data on the calling thread: data just written by
            /// another core makes the first passes look slow up to ~1M.
            #[cfg(test)]
            #[allow(dead_code)]
            pub(crate) fn profile(bits: &mut [$uint], wide: bool) -> Vec<(String, f64)> {
                use std::time::Instant;
                let size = bits.len();
                let per = |t: Instant| t.elapsed().as_nanos() as f64 / size as f64;
                let mut res = Vec::new();
                let mut scratch: Vec<$uint> = vec![0; size];
                // Floors.
                let t = Instant::now();
                scratch.copy_from_slice(bits);
                res.push(("floor_memcpy".to_string(), per(t)));
                // Sequential read + encode/decode + write (no random write).
                let t = Instant::now();
                for (d, &s) in scratch.iter_mut().zip(bits.iter()) {
                    *d = decode(encode(s));
                }
                res.push(("floor_seq_rw".to_string(), per(t)));
                std::hint::black_box(&scratch);

                let shifts: Vec<u32> =
                    if wide { (0..BITS).step_by(11).collect() } else { (0..BITS).step_by(8).collect() };
                macro_rules! run {
                    ($r:expr) => {{
                        let t = Instant::now();
                        let mut tables = if $r == 2048 {
                            hist_const::<u32, $r, { ((BITS + 10) / 11) as usize }, 2, true>(bits, 11)
                        } else {
                            hist_const::<u32, $r, { (BITS / 8) as usize }, 4, true>(bits, 8)
                        };
                        res.push(("histogram".to_string(), per(t)));
                        for table in tables.iter_mut() {
                            let mut sum = 0u32;
                            for c in table.iter_mut() {
                                let n = sum + *c;
                                *c = sum;
                                sum = n;
                            }
                        }
                        let t = Instant::now();
                        let mut buffer: Vec<MaybeUninit<$uint>> = Vec::with_capacity(size);
                        unsafe { buffer.set_len(size) };
                        let bp = buffer.as_mut_ptr() as *mut $uint;
                        res.push(("alloc".to_string(), per(t)));
                        let last = shifts.len() - 1;
                        for (pass, &shift) in shifts.iter().enumerate() {
                            let heads = &mut tables[pass];
                            let t = Instant::now();
                            // SAFETY: as in `sort_digits` (no digit skipped).
                            unsafe {
                                let (src, dst): (&[$uint], *mut $uint) = if pass % 2 == 0 {
                                    (&*bits, bp)
                                } else {
                                    (std::slice::from_raw_parts(bp, size), bits.as_mut_ptr())
                                };
                                if $r == 256 && alias_fix_enabled() && aliased::<u32, $r>(heads, size) {
                                    let mut st = vec![0 as $uint; $r * WB];
                                    let mut sb = vec![0usize; $r];
                                    match (pass == 0, pass == last) {
                                        (true, _) => scatter_staged::<u32, $r, true, false>(src, dst, heads, shift, &mut st, &mut sb),
                                        (false, true) => scatter_staged::<u32, $r, false, true>(src, dst, heads, shift, &mut st, &mut sb),
                                        (false, false) => scatter_staged::<u32, $r, false, false>(src, dst, heads, shift, &mut st, &mut sb),
                                    }
                                } else {
                                    match (pass == 0, pass == last) {
                                        (true, _) => scatter::<u32, $r, true, false>(src, dst, heads, shift),
                                        (false, true) => scatter::<u32, $r, false, true>(src, dst, heads, shift),
                                        (false, false) => scatter::<u32, $r, false, false>(src, dst, heads, shift),
                                    }
                                }
                            }
                            res.push((format!("pass{}", pass), per(t)));
                        }
                        let t = Instant::now();
                        if shifts.len() % 2 == 1 {
                            let sorted = unsafe { std::slice::from_raw_parts(bp, size) };
                            bits.copy_from_slice(sorted);
                        }
                        res.push(("copy".to_string(), per(t)));
                    }};
                }
                if wide {
                    run!(2048);
                } else {
                    run!(256);
                }
                res
            }

            /// Whether one of the passes of the 11-bit histograms `tables`
            /// (counts) would scatter to aliased bucket starts.
            fn wide_aliased<C: Counter>(tables: &[[C; 2048]], size: usize) -> bool {
                tables.iter().any(|table| {
                    if size < ALIAS_MIN || table.iter().any(|&c| c.get() == size) {
                        return false;
                    }
                    let mut heads = [C::ZERO; 2048];
                    let mut sum = C::ZERO;
                    for b in 0..2048 {
                        heads[b] = sum;
                        sum += table[b];
                    }
                    aliased::<C, 2048>(&heads, size)
                })
            }

            fn sort_with<C: Counter, const ENC: bool>(
                bits: &mut [$uint],
                mode: Digits,
                buffer: &mut Vec<MaybeUninit<$uint>>,
            ) {
                let wide = match mode {
                    Digits::Bytes => false,
                    Digits::Wide => true,
                    // Encoded input (leaves of the float rollercoaster, whose
                    // top bits are equal): exact count of the passes.
                    Digits::Auto if !ENC => {
                        bits.len() >= WIDE_MIN && (BITS == 64 || bits.len() <= wide_max_32()) && {
                            let first = bits[0];
                            let diff = bits.iter().fold(0, |acc: $uint, &k| acc | (k ^ first));
                            let passes = |step: u32| {
                                (0..BITS).step_by(step as usize).filter(|&s| (diff >> s) & ((1 << step) - 1) != 0).count()
                            };
                            passes(11) < passes(8)
                        }
                    }
                    Digits::Auto => {
                        bits.len() >= WIDE_MIN
                            && (BITS == 64 || bits.len() <= wide_max_32())
                            && variation(bits) >= $wide_bits
                    }
                };
                if wide {
                    let shifts: [u32; ((BITS + 10) / 11) as usize] = std::array::from_fn(|t| 11 * t as u32);
                    let mut tables = hist_const::<C, 2048, { ((BITS + 10) / 11) as usize }, 2, ENC>(bits, 11);
                    // Layout chosen: the cache-aliasing fix exists for byte
                    // digits only, so aliased 11-bit passes (integer-valued
                    // or grid keys, permutations, timestamps) fall back to
                    // bytes. Raw u32 input, 2M keys: permutation of 0..n
                    // 13.5 -> 7.5 ns per key, timestamps 10.2 -> 7.6.
                    if mode != Digits::Auto || !wide_aliased(&tables, bits.len()) {
                        sort_digits::<C, 2048, ENC>(bits, &shifts, &mut tables, buffer);
                        return;
                    }
                }
                {
                    let shifts: [u32; (BITS / 8) as usize] = std::array::from_fn(|t| 8 * t as u32);
                    if bits.len() < short_max() {
                        // Short arrays: one lane on the stack. The 4 lanes on the
                        // heap (allocated, zeroed and summed on each call: 4096
                        // counters for f32, 8192 for f64) cost more than the
                        // passes below a few thousand keys.
                        let mut tables = hist_stack::<C, { (BITS / 8) as usize }, ENC>(bits);
                        sort_digits::<C, 256, ENC>(bits, &shifts, &mut tables, buffer);
                    } else {
                        let mut tables = hist_const::<C, 256, { (BITS / 8) as usize }, 4, ENC>(bits, 8);
                        sort_digits::<C, 256, ENC>(bits, &shifts, &mut tables, buffer);
                    }
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
                    sort_with::<u32, true>(bits, mode, &mut Vec::new());
                } else {
                    sort_with::<usize, true>(bits, mode, &mut Vec::new());
                }
            }

            /// Sorts already encoded keys and writes them back decoded (raw
            /// bits in total order), using `buffer` as scratch space (grown
            /// when needed, kept for the next call).
            #[allow(dead_code)]
            pub(crate) fn lsd_encoded(keys: &mut [$uint], buffer: &mut Vec<MaybeUninit<$uint>>) {
                lsd_encoded_mode(keys, buffer, Digits::Auto);
            }

            /// `lsd_encoded` with a forced digit layout.
            #[allow(dead_code)]
            pub(crate) fn lsd_encoded_mode(keys: &mut [$uint], buffer: &mut Vec<MaybeUninit<$uint>>, mode: Digits) {
                if keys.len() <= LSD_MIN {
                    keys.sort_unstable();
                    keys.iter_mut().for_each(|b| *b = decode(*b));
                } else if keys.len() <= u32::MAX as usize {
                    sort_with::<u32, false>(keys, mode, buffer);
                } else {
                    sort_with::<usize, false>(keys, mode, buffer);
                }
            }

            /// Encodes raw bits into keys whose unsigned order is the total
            /// order of the floats.
            #[inline(always)]
            #[allow(dead_code)]
            pub(crate) fn encode_key(bits: $uint) -> $uint {
                encode(bits)
            }

            /// Inverse of `encode_key`.
            #[inline(always)]
            #[allow(dead_code)]
            pub(crate) fn decode_key(key: $uint) -> $uint {
                decode(key)
            }
        }
    };
}

/// Tests / benchmarks: no counting shortcut when one digit varies.
#[cfg(test)]
pub(crate) static SINGLE_DIGIT_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fused_lsd!(u32, i32, w32, 20, float);
fused_lsd!(u64, i64, w64, 44, float);
fused_lsd!(u32, i32, wu32, 20, unsigned);
fused_lsd!(u64, i64, wu64, 44, unsigned);
fused_lsd!(u32, i32, wi32, 20, signed);
fused_lsd!(u64, i64, wi64, 44, signed);
fused_lsd!(u128, i128, wu128, 88, unsigned);
fused_lsd!(u128, i128, wi128, 88, signed);
// 8- and 16-bit integers (devour_sort): byte digits only (never wide).
fused_lsd!(u16, i16, wu16, 17, unsigned);
fused_lsd!(u16, i16, wi16, 17, signed);
fused_lsd!(u8, i8, wu8, 9, unsigned);
fused_lsd!(u8, i8, wi8, 9, signed);
