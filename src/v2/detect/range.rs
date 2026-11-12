//! Narrow ranges of integers, sorted by counting: a window that follows
//! the keys, the whole domain, a range wrapping around the end of the
//! domain, or keys at both ends (offset sort); wide ranges when their
//! counters fit in the cache.

macro_rules! int_range {
    ($t:ty, $uint:ty, $encode:expr, $decode:expr, $roller_min:path) => {
        /// Narrow range: values spanning at most `RANGE_MAX` integers and
        /// at most one counter per `RANGE_KEYS_PER_COUNTER` keys are sorted by
        /// counting (one counting pass, then the values written in order).
        /// A spread sample rejects wide ranges first. A sample range narrow
        /// against the limit (ratings, small counters) is counted in one
        /// pass in a window around it (`window_count`); a wider one gets
        /// the exact min / max first, then the count. Measured (Snapdragon
        /// X Elite, against min / max then a 1-lane count): u32 0..100 0.54
        /// to 0.35 ns per key at 1M, Alt16 0.60 to 0.38; one wide outlier
        /// late in narrow keys costs a wasted counting pass (0.58 to 0.75).
        pub(crate) fn range(values: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
            let n = values.len();
            if n < FEW_MIN {
                return false;
            }
            let limit = RANGE_MAX.min(n / RANGE_KEYS_PER_COUNTER) as u64;
            // Whole domain within the limit (8- and 16-bit keys): direct count.
            let domain = n <= u32::MAX as usize && (<$uint>::MAX as u64) < limit;
            // From 8 keys per domain counter on, the whole table is cheap
            // against the keys: counted directly.
            if domain && n as u64 >= (<$uint>::MAX as u64).saturating_add(1).saturating_mul(8) {
                domain_count(bits_of(values), $encode, $decode);
                return true;
            }
            let (mut lo, mut hi) = (values[0], values[0]);
            // Same sample, keys rotated by half the domain (top encoded bit
            // flipped): a range wrapping around the end of the domain
            // (small values and values just below the maximum, e.g. signed
            // offsets stored unsigned) becomes contiguous.
            let rot = |v: $t| $encode(v as $uint) ^ ((1 as $uint) << (<$uint>::BITS - 1));
            let (mut rlo, mut rhi) = (values[0], values[0]);
            for &v in sample.iter() {
                lo = lo.min(v);
                hi = hi.max(v);
                if rot(v) < rot(rlo) {
                    rlo = v;
                }
                if rot(v) > rot(rhi) {
                    rhi = v;
                }
            }
            if domain {
                // Very narrow sample range (in either order): a window of
                // at most 1/32 of the domain table, the whole domain when
                // keys fall outside it (u16 at 131k-300k: 16 values 0.96
                // to 0.48 ns per key, 0..255 0.70 to 0.44, values at both
                // ends 0.67 to 0.37-0.44).
                let span = (hi as i128 - lo as i128) as u64 + 1;
                let rspan = (rot(rhi) - rot(rlo)) as u64 + 1;
                let tiny = limit / 64;
                if span <= tiny && window_count(bits_of(values), lo as $uint, hi as $uint, limit as usize, $encode, $decode) {
                    return true;
                }
                if span > tiny && rspan <= tiny && wrapped_count(values, rlo, rhi, limit as usize) {
                    return true;
                }
                domain_count(bits_of(values), $encode, $decode);
                return true;
            }
            if (hi as i128 - lo as i128) as u64 >= limit {
                let rspan = (rot(rhi) - rot(rlo)) as u64 + 1;
                if n <= u32::MAX as usize && 2 * rspan < limit {
                    return wrapped_count(values, rlo, rhi, limit as usize);
                }
                // Values at both ends of the domain, wider than a window (u64
                // at 1k-1M: 6.7-9.0 to 3.0-4.0 ns per key; u32 3.1 to 2.1).
                if <$uint>::BITS >= 32 && n < $roller_min() && offset_sort(values, lo, hi, rlo, rhi) {
                    return true;
                }
                // Wide range, counted all the same when its counters fit in
                // half the L2 and enough keys share each counter: one read
                // and one write against the radix passes (char over its 21
                // bits at 10M keys: 3.5 to 1.1 ns per key; u64 sawtooth at 3M-10M:
                // 5.4 to 0.9).
                if <$uint>::BITS >= 32 && n <= u32::MAX as usize {
                    let wide = ((l2_bytes() / 8).min(n / WIDE_KEYS_PER_COUNTER)) as u128;
                    if (hi as i128 - lo as i128) as u128 + 1 <= wide {
                        let (lo, hi) = values.iter().fold((lo, hi), |(l, h), &v| (l.min(v), h.max(v)));
                        let span = (hi as i128 - lo as i128) as u128 + 1;
                        if span <= wide {
                            exact_count(bits_of(values), $encode(lo as $uint), span as usize, $encode, $decode);
                            return true;
                        }
                    }
                }
                return false;
            }

            /// Keys at both ends of the domain (small values and values just
            /// below the maximum, wider than a counting window): rotated by
            /// half the domain and shifted to `key - min`, they span fewer
            /// bytes than vary among them; sorted on the remaining bytes,
            /// rotated back into true order and shifted back. From the
            /// sample first, then checked on the exact min / max (one pass,
            /// nothing written when the gain is not there). Only below the
            /// rollercoaster size: its MSD pass already splits the two ends.
            /// Without rotation (a narrow range across a byte boundary) the
            /// DLSD already sizes its digits from a sample: 10-25 % slower.
            fn offset_sort(values: &mut [$t], lo: $t, hi: $t, rlo: $t, rhi: $t) -> bool {
                const TOP: $uint = (1 as $uint) << (<$uint>::BITS - 1);
                let bytes = |x: $uint| (<$uint>::BITS - x.leading_zeros() + 7) / 8;
                let rot = |v: $t| $encode(v as $uint) ^ TOP;
                let (elo, ehi) = ($encode(lo as $uint), $encode(hi as $uint));
                let rspan = rot(rhi).wrapping_sub(rot(rlo));
                if rspan >= ehi - elo || bytes(rspan) + OFFSET_BYTES > bytes(elo ^ ehi) {
                    return false;
                }
                let keys = bits_of(values);
                // Exact min / max of the rotated keys, and the bytes that vary
                // among the keys as they are (what the radix sort would see).
                let e0 = $encode(keys[0]);
                let (mut kmin, mut kmax, mut diff) = (<$uint>::MAX, 0 as $uint, 0 as $uint);
                for &k in keys.iter() {
                    let e = $encode(k);
                    diff |= e ^ e0;
                    kmin = kmin.min(e ^ TOP);
                    kmax = kmax.max(e ^ TOP);
                }
                if bytes(kmax - kmin) + OFFSET_BYTES > bytes(diff) {
                    return false;
                }
                for k in keys.iter_mut() {
                    *k = ($encode(*k) ^ TOP) - kmin;
                }
                <$uint as DevourSort2>::devour_sort2_impl(keys);
                // Rotated order puts the keys of the top half of the encoded
                // domain first; true order puts them last.
                let split = keys.partition_point(|&k| k + kmin < TOP);
                keys.rotate_left(split);
                for k in keys.iter_mut() {
                    *k = $decode((*k + kmin) ^ TOP);
                }
                true
            }

            /// Window count on keys rotated by half the domain, written back
            /// from the rotated value `TOP` on (the keys of the bottom half
            /// of the encoded domain first, their true order).
            fn wrapped_count(values: &mut [$t], rlo: $t, rhi: $t, limit: usize) -> bool {
                const TOP: $uint = (1 as $uint) << (<$uint>::BITS - 1);
                fn renc(k: $uint) -> $uint {
                    $encode(k) ^ TOP
                }
                fn rdec(k: $uint) -> $uint {
                    $decode(k ^ TOP)
                }
                window_count_split(bits_of(values), rlo as $uint, rhi as $uint, limit, renc, rdec, TOP as u64)
            }

            let small = n <= u32::MAX as usize;
            // Sample range narrow against the limit: count in a window
            // around it, in one pass.
            if small && 2 * (hi as i128 - lo as i128 + 1) < limit as i128 {
                return window_count(bits_of(values), lo as $uint, hi as $uint, limit as usize, $encode, $decode);
            }
            let (lo, hi) = values.iter().fold((lo, hi), |(l, h), &v| (l.min(v), h.max(v)));
            let span = (hi as i128 - lo as i128) as u64 + 1;
            if span > limit {
                return false;
            }
            if small {
                exact_count(bits_of(values), $encode(lo as $uint), span as usize, $encode, $decode);
                return true;
            }
            let mut counts = vec![0u32; span as usize];
            // Counts of at most 2^31 keys per block (u32 counters).
            let mut total = vec![0usize; 0];
            let big = n > u32::MAX as usize;
            if big {
                total = vec![0usize; span as usize];
            }
            for block in values.chunks(if big { 1 << 31 } else { usize::MAX }) {
                for &v in block {
                    // SAFETY: lo <= v <= hi, so the offset is below span.
                    unsafe { *counts.get_unchecked_mut((v as i128 - lo as i128) as usize) += 1 };
                }
                if big {
                    for (t, c) in total.iter_mut().zip(counts.iter_mut()) {
                        *t += *c as usize;
                        *c = 0;
                    }
                }
            }
            let mut start = 0;
            for d in 0..span as usize {
                let c = if big { total[d] } else { counts[d] as usize };
                if c != 0 {
                    values[start..start + c].fill((lo as i128 + d as i128) as $t);
                    start += c;
                }
            }
            debug_assert_eq!(start, n);
            true
        }
    };
}
