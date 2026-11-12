//! Dense range with outliers: a 3-way partition, the inside counted.

macro_rules! int_dense {
    ($t:ty, $uint:ty, $encode:expr, $decode:expr) => {
        /// Most keys in a narrow range, a minority spread (text with a
        /// few rare characters, sparse data, sensors with outliers): when
        /// 3 / 4 of the sample fit in a window of half the counting
        /// limit, a 3-way partition (below / inside / above), the inside
        /// counted, the two minorities sorted recursively: nothing to
        /// merge. Tried after the few-values count (cheaper when it
        /// applies). Gives up (array permuted, unsorted) when less than
        /// half the keys fall inside. Mixed text (85 % ASCII, 10 % Latin,
        /// 3 % CJK, 2 % emoji): 3.0-4.2 to 1.1-2.4 ns per key.
        pub(crate) fn dense(values: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
            let n = values.len();
            if n < DENSE_MIN || n > u32::MAX as usize {
                return false;
            }
            let limit = RANGE_MAX.min(n / RANGE_KEYS_PER_COUNTER) as u64;
            // `sample`: the spread sample, sorted.
            let w = limit as i128 / 2;
            let (mut best, mut bi, mut bj, mut j) = (0, 0, 0, 0);
            for i in 0..SAMPLE {
                j = j.max(i);
                while j + 1 < SAMPLE && (sample[j + 1] as i128 - sample[i] as i128) < w {
                    j += 1;
                }
                if j + 1 - i > best {
                    best = j + 1 - i;
                    bi = i;
                    bj = j;
                }
            }
            if 4 * best < 3 * SAMPLE {
                return false;
            }
            let (slo, shi) = (sample[bi] as i128, sample[bj] as i128);
            let width = (2 * (shi - slo + 1)).max(256).min(limit as i128);
            let lo = (slo - (width - (shi - slo + 1)) / 2).max(<$t>::MIN as i128);
            let hi = (lo + width - 1).min(<$t>::MAX as i128);
            // The inside keys expected from the sample must fill the window
            // as densely as a narrow range (`RANGE_KEYS_PER_COUNTER`):
            // otherwise counting them costs more than a radix sort (u16
            // Gaussian at 20k-30k keys, 85 % of them within n / 2 values:
            // 2.1 against 1.3 ns per key).
            if (best * n) < RANGE_KEYS_PER_COUNTER * (hi - lo + 1) as usize * SAMPLE {
                return false;
            }
            let (lo, hi) = (lo as $t, hi as $t);
            // 3-way partition, [0, a) below, [a, i) inside, [b, n) above
            // (branches: the inside keys are the predicted majority; a
            // branch-free split into a buffer was slower, 1.3 to 2.2 ns
            // per key on mixed text at 1M).
            let (mut a, mut i, mut b) = (0, 0, n);
            while i < b {
                let v = values[i];
                if v < lo {
                    values.swap(a, i);
                    a += 1;
                    i += 1;
                } else if v > hi {
                    b -= 1;
                    values.swap(i, b);
                } else {
                    i += 1;
                }
            }
            if 2 * (b - a) < n {
                return false;
            }
            exact_count(bits_of(&mut values[a..b]), $encode(lo as $uint), (hi as i128 - lo as i128 + 1) as usize, $encode, $decode);
            <$t as DevourSort2>::devour_sort2_impl(&mut values[..a]);
            <$t as DevourSort2>::devour_sort2_impl(&mut values[b..]);
            true
        }
    };
}
