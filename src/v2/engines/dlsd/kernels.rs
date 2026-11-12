//! DLSD kernels: unrolled histograms, insertion sort, scatter passes (plain
//! and staged when the bucket starts alias in the cache).

macro_rules! dlsd_kernels {
    ($uint:ty) => {
        /// Histograms of `K` digits of `w` bits (prefix of `key - min` from
        /// bit `base`), two lanes (`t0`, `t1`, `K * r` counters each);
        /// `K` known at compile time: the digit loop is unrolled.
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn hist_k<const K: usize, const RAW: bool>(keys: &[$uint], min: $uint, base: u32, w: u32, mask: $uint, r: usize, t0: &mut [u32], t1: &mut [u32]) {
            let mut pairs = keys.chunks_exact(2);
            for pr in &mut pairs {
                let a = enc::<RAW>(pr[0]).wrapping_sub(min) >> base;
                let b = enc::<RAW>(pr[1]).wrapping_sub(min) >> base;
                for t in 0..K {
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
                for t in 0..K {
                    t0[t * r + ((a >> (t as u32 * w)) & mask) as usize] += 1;
                }
            }
            for i in 0..K * r {
                t0[i] += t1[i];
            }
        }

        /// Insertion sort (small groups).
        fn insertion(a: &mut [$uint]) {
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
        /// line positions of a page (see `cache::crowded`).
        fn aliased(heads: &[u32], n: usize) -> bool {
            super::super::super::cache::crowded((0..heads.len()).map(|b| {
                let start = heads[b] as usize;
                let end = if b + 1 < heads.len() { heads[b + 1] as usize } else { n };
                (start * std::mem::size_of::<$uint>(), (end - start) * std::mem::size_of::<$uint>())
            }))
        }

        /// One stable scatter pass on the digit `((key - min) >> shift) & mask`.
        /// `FIRST` encodes the raw bits read, `LAST` decodes the keys written.
        ///
        /// # Safety
        /// `dst` valid for `src.len()` writes, not overlapping `src`; `heads` the
        /// exclusive prefix sums of this digit over the keys of `src`.
        #[inline(always)]
        unsafe fn scatter<const FIRST: bool, const LAST: bool>(
            src: &[$uint],
            dst: *mut $uint,
            heads: &mut [u32],
            min: $uint,
            shift: u32,
            mask: $uint,
        ) {
            let key = |v: $uint| if FIRST { w::encode_key(v) } else { v };
            let out = |k: $uint| if LAST { w::decode_key(k) } else { k };
            let digit = |k: $uint| ((k.wrapping_sub(min) >> shift) & mask) as usize;
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
            src: &[$uint],
            dst: *mut $uint,
            heads: &mut [u32],
            min: $uint,
            shift: u32,
            mask: $uint,
            staging: &mut [$uint],
            starts: &mut [u32],
        ) {
            let r = mask as usize + 1;
            assert!(staging.len() >= r * WB && starts.len() >= r && heads.len() >= r);
            let key = |v: $uint| if FIRST { w::encode_key(v) } else { v };
            let out = |k: $uint| if LAST { w::decode_key(k) } else { k };
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
    };
}
