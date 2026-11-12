//! Scatter passes: each key written to the next slot of its bucket. When the
//! bucket starts crowd on a few cache sets (aliasing), keys are staged by
//! bucket in cache lines first.

macro_rules! lsd_scatters {
    ($uint:ty) => {
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
            super::super::super::cache::crowded((0..R).map(|b| {
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
    };
}
