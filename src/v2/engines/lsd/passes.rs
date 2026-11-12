//! The LSD itself: digit layout chosen (bytes or 11 bits), digits equal in
//! every key skipped, a single varying digit counted without scatter, the
//! passes alternating between the keys and a buffer.

macro_rules! lsd_passes {
    ($uint:ty) => {
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
            if np == 1 {
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
                if R == 256 && size >= ALIAS_MIN && aliased::<C, R>(heads, size) {
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

        /// Sorts with 11-bit digits when they save a pass and stay in the
        /// cache (and their passes are not aliased), else bytes.
        fn sort_with<C: Counter, const ENC: bool>(bits: &mut [$uint], buffer: &mut Vec<MaybeUninit<$uint>>) {
            let wide = match () {
                // Encoded input (leaves of the rollercoaster, whose top
                // bits are equal): exact count of the passes.
                () if !ENC => {
                    bits.len() >= WIDE_MIN && (BITS == 64 || bits.len() <= wide_max_32()) && {
                        let first = bits[0];
                        let diff = bits.iter().fold(0, |acc: $uint, &k| acc | (k ^ first));
                        let passes = |step: u32| {
                            (0..BITS).step_by(step as usize).filter(|&s| (diff >> s) & ((1 << step) - 1) != 0).count()
                        };
                        passes(11) < passes(8)
                    }
                }
                () => {
                    bits.len() >= WIDE_MIN
                        && (BITS == 64 || bits.len() <= wide_max_32())
                        && variation(bits) >= WIDE_BITS
                }
            };
            if wide {
                let shifts: [u32; ((BITS + 10) / 11) as usize] = std::array::from_fn(|t| 11 * t as u32);
                let mut tables = hist_const::<C, 2048, { ((BITS + 10) / 11) as usize }, 2, ENC>(bits, 11);
                // The aliasing fix exists for byte digits only: aliased
                // 11-bit passes fall back to bytes.
                if !wide_aliased(&tables, bits.len()) {
                    sort_digits::<C, 2048, ENC>(bits, &shifts, &mut tables, buffer);
                    return;
                }
            }
            {
                let shifts: [u32; (BITS / 8) as usize] = std::array::from_fn(|t| 8 * t as u32);
                if bits.len() < SHORT_MAX {
                    // Short arrays: one lane on the stack (4 lanes on the
                    // heap cost more than the passes there).
                    let mut tables = hist_stack::<C, { (BITS / 8) as usize }, ENC>(bits);
                    sort_digits::<C, 256, ENC>(bits, &shifts, &mut tables, buffer);
                } else {
                    let mut tables = hist_const::<C, 256, { (BITS / 8) as usize }, 4, ENC>(bits, 8);
                    sort_digits::<C, 256, ENC>(bits, &shifts, &mut tables, buffer);
                }
            }
        }

        /// Sorts raw bits (in the order of the codec).
        pub(crate) fn lsd(bits: &mut [$uint]) {
            if bits.len() <= LSD_MIN {
                bits.iter_mut().for_each(|b| *b = encode(*b));
                bits.sort_unstable();
                bits.iter_mut().for_each(|b| *b = decode(*b));
            } else if bits.len() <= u32::MAX as usize {
                sort_with::<u32, true>(bits, &mut Vec::new());
            } else {
                sort_with::<usize, true>(bits, &mut Vec::new());
            }
        }

        /// Sorts already encoded keys and writes them back decoded,
        /// `buffer` as scratch space (grown when needed, kept).
        pub(crate) fn lsd_encoded(keys: &mut [$uint], buffer: &mut Vec<MaybeUninit<$uint>>) {
            if keys.len() <= LSD_MIN {
                keys.sort_unstable();
                keys.iter_mut().for_each(|b| *b = decode(*b));
            } else if keys.len() <= u32::MAX as usize {
                sort_with::<u32, false>(keys, buffer);
            } else {
                sort_with::<usize, false>(keys, buffer);
            }
        }

        /// Raw bits to keys whose unsigned order is the wanted order.
        #[inline(always)]
        pub(crate) fn encode_key(bits: $uint) -> $uint {
            encode(bits)
        }

        /// Inverse of `encode_key`.
        #[inline(always)]
        pub(crate) fn decode_key(key: $uint) -> $uint {
            decode(key)
        }
    };
}
