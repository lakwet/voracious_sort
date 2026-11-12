//! Histograms of all the digits in one read of the keys.

macro_rules! lsd_histograms {
    ($uint:ty) => {
        #[inline(always)]
        fn digit<const R: usize>(key: $uint, shift: u32) -> usize {
            ((key >> shift) as usize) & (R - 1)
        }

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

        /// Histograms of the `P` digits of `step` bits (`R = 2^step`
        /// buckets) of the encoded keys, in `L` lanes (key `i` counted in
        /// lane `i % L`, so that consecutive keys sharing a digit do not
        /// wait on the same counter); the digit loop is unrolled.
        fn hist_const<C: Counter, const R: usize, const P: usize, const L: usize, const ENC: bool>(
            bits: &[$uint],
            step: u32,
        ) -> Vec<[C; R]> {
            let encode = |v: $uint| if ENC { encode(v) } else { v };
            let mut lanes = vec![[[C::ZERO; R]; P]; L];
            // Digits constant on 16 spread keys: counted by blocks of 8
            // keys sharing the digit (one addition of 8 instead of 8
            // increments waiting on each other); the others key by key.
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
    };
}
