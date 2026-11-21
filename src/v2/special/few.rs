//! Few distinct integer values, counted in a hash table.

macro_rules! int_few {
    ($t:ty, $uint:ty, $encode:expr, $decode:expr) => {
        /// Few distinct values (at most `FEW_MAX`): a spread sample gives up
        /// as soon as half of it is distinct; else one pass counts every
        /// value in an open-addressing hash table (4 lanes of counters),
        /// giving up beyond `FEW_MAX` values with nothing written; the
        /// distinct values are sorted and the array rewritten (same scheme
        /// as the float `few`).
        pub(crate) fn few(values: &mut [$t], sample: &[$t; SAMPLE]) -> bool {
            let n = values.len();
            if n < FEW_MIN || n > u32::MAX as usize {
                return false;
            }
            let hash = |v: $t| {
                ((v as $uint as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - FEW_TABLE_BITS))
                    as usize
            };
            let s = (n / 32).clamp(16, 256);
            let mut keys = [0 as $t; FEW_TABLE];
            let mut used = [false; FEW_TABLE];
            let mut distinct = 0;
            // Its first keys are the shared spread sample (same positions).
            for i in 0..s as u64 {
                let v = if (i as usize) < SAMPLE {
                    sample[i as usize]
                } else {
                    values[(((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32)
                        as usize]
                };
                let mut h = hash(v);
                while used[h] && keys[h] != v {
                    h = (h + 1) & (FEW_TABLE - 1);
                }
                if !used[h] {
                    used[h] = true;
                    keys[h] = v;
                    distinct += 1;
                    if distinct > s / 2 {
                        return false;
                    }
                }
            }
            // Many values in the sample: a table 4 times larger, so that
            // the probe chains stay short (the time follows the
            // collisions). Below `FEW_BIG_MIN` keys, zeroing it costs more
            // than it saves.
            if distinct > FEW_TABLE / 8 && n >= FEW_BIG_MIN {
                few_count::<{ 4 * FEW_TABLE }>(values)
            } else {
                few_count::<FEW_TABLE>(values)
            }
        }

        /// The counting pass of `few` in a table of `TABLE` slots.
        fn few_count<const TABLE: usize>(values: &mut [$t]) -> bool {
            let n = values.len();
            let bits = TABLE.trailing_zeros();
            let mask = TABLE - 1;
            let hash = |v: $t| {
                ((v as $uint as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - bits)) as usize
            };
            let mut keys = [0 as $t; TABLE];
            let mut used = [false; TABLE];
            let mut counts = [[0u32; TABLE]; 4];
            let mut distinct = 0;
            let mut find = |v: $t, distinct: &mut usize| -> Option<usize> {
                let mut h = hash(v);
                while used[h] && keys[h] != v {
                    h = (h + 1) & mask;
                }
                if !used[h] {
                    if *distinct == FEW_MAX {
                        return None;
                    }
                    used[h] = true;
                    keys[h] = v;
                    *distinct += 1;
                }
                Some(h)
            };
            let mut quads = values.chunks_exact(4);
            for q in &mut quads {
                for lane in 0..4 {
                    match find(q[lane], &mut distinct) {
                        Some(h) => counts[lane][h] += 1,
                        None => return false,
                    }
                }
            }
            for &v in quads.remainder() {
                match find(v, &mut distinct) {
                    Some(h) => counts[0][h] += 1,
                    None => return false,
                }
            }
            let mut found: Vec<($t, usize)> = (0..=mask)
                .filter(|&h| used[h])
                .map(|h| {
                    (
                        keys[h],
                        counts[0][h] as usize
                            + counts[1][h] as usize
                            + counts[2][h] as usize
                            + counts[3][h] as usize,
                    )
                })
                .collect();
            found.sort_unstable_by_key(|&(v, _)| v);
            let mut start = 0;
            for (v, c) in found {
                values[start..start + c].fill(v);
                start += c;
            }
            debug_assert_eq!(start, n);
            true
        }
    };
}
