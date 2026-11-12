//! Prefix length of the DLSD, from a sorted spread sample: the bits that a
//! pair of unequal neighbours share, plus one, at the 95th percentile, plus
//! log2(n / S) (neighbours in the array are n / S times closer than in the
//! sample). This sees correlated digits (tight clusters) and isolated
//! extremes; equal keys need no prefix and are ignored.

macro_rules! dlsd_prefix {
    ($uint:ty) => {
        /// Sample size: n / `SAMPLE_DIV` keys, at least `SAMPLE_MIN`.
        #[inline(always)]
        fn sample_size(n: usize) -> usize {
            (n / SAMPLE_DIV).max(SAMPLE_MIN).min(n)
        }

        /// Position of the `i`-th sampled key: frac(i x golden ratio) x n,
        /// spread like a stride but in resonance with no period of the data.
        #[inline(always)]
        fn spread_position(i: usize, n: usize) -> usize {
            ((((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32) as usize
        }

        /// `len` slots for the sample, on the stack when they fit.
        #[inline(always)]
        fn sample_slots<'a>(stack: &'a mut [$uint; STACK_SAMPLE], heap: &'a mut Vec<$uint>, len: usize) -> &'a mut [$uint] {
            if len <= STACK_SAMPLE {
                &mut stack[..len]
            } else {
                heap.resize(len, 0);
                heap
            }
        }

        /// Prefix length (bits of `key - min` from the top of the range, at
        /// most `top`). `pre`: the sorted sample of the encoded keys, if taken.
        fn prefix_bits<const RAW: bool>(keys: &[$uint], min: $uint, top: u32, pre: Option<&mut [$uint]>) -> u32 {
            let n = keys.len();
            let mut stack = [0 as $uint; STACK_SAMPLE];
            let mut heap: Vec<$uint> = Vec::new();
            let sample: &mut [$uint] = match pre {
                // Sorted encoded keys, all >= min: `key - min` keeps the order.
                Some(sample) => {
                    sample.iter_mut().for_each(|k| *k = k.wrapping_sub(min));
                    sample
                }
                None => {
                    let sample = sample_slots(&mut stack, &mut heap, sample_size(n));
                    for (i, slot) in sample.iter_mut().enumerate() {
                        *slot = enc::<RAW>(keys[spread_position(i, n)]).wrapping_sub(min);
                    }
                    sample.sort_unstable();
                    sample
                }
            };
            prefix_from_sample(sample, n, top)
        }

        /// Prefix length from the sorted sample of `key - min`.
        fn prefix_from_sample(sample: &mut [$uint], n: usize, top: u32) -> u32 {
            let s = sample.len();
            // Bits shared from the top of the range, plus one: bits that separate.
            let mut stack = [0u32; STACK_SAMPLE];
            let mut heap: Vec<u32> = Vec::new();
            let need_all: &mut [u32] = if s <= STACK_SAMPLE {
                &mut stack[..s]
            } else {
                heap.resize(s, 0);
                &mut heap
            };
            let mut m = 0;
            for pair in sample.windows(2) {
                if pair[0] != pair[1] {
                    need_all[m] = ((pair[0] ^ pair[1]).leading_zeros() + 1).saturating_sub(<$uint>::BITS - top);
                    m += 1;
                }
            }
            let need = &mut need_all[..m];
            let log_n = usize::BITS - n.leading_zeros();
            if need.is_empty() {
                // Equal keys only: a short prefix, the finish handles the groups.
                return (log_n + 3).min(top);
            }
            let at = (need.len() * 95 / 100).min(need.len() - 1);
            // A selection, or the maximum when the rank is the last one.
            let p95 = if at == need.len() - 1 {
                *need.iter().max().unwrap()
            } else {
                *need.select_nth_unstable(at).1
            };
            let spread = (usize::BITS - (n / s).leading_zeros()).saturating_sub(1);
            let bits = (p95 + spread).clamp(1, top);
            // 128-bit keys sharing most of their bits with their neighbours
            // (clusters, far-apart modes): one top digit instead of 6 to 11
            // passes; each group is then sorted on its own, narrower range.
            if <$uint>::BITS == 128 && bits > 64 {
                return MAX_DIGIT_BITS.min(top);
            }
            bits
        }
    };
}
