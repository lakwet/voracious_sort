//! The DLSD steps: small arrays, min / max, narrowing of 128-bit keys,
//! prefix layout, histograms, scatter passes on the prefix digits, finish.

macro_rules! dlsd_sort {
    ($uint:ty, $alias_min:path) => {
        /// Sorts raw bits.
        pub(crate) fn dlsd(bits: &mut [$uint]) {
            let mut scratch = Vec::new();
            dlsd_with(bits, &mut scratch);
        }

        /// `dlsd` with a reusable scratch buffer.
        pub(crate) fn dlsd_with(bits: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>) {
            if bits.len() > u32::MAX as usize {
                w::lsd(bits);
                return;
            }
            dlsd_rec::<true>(bits, scratch, None);
        }

        /// `dlsd_with`, unless `reject` refuses the sorted spread sample of the
        /// encoded keys that sizes the prefix (the caller's own decision read
        /// on the same sample). Returns whether the keys were sorted.
        pub(crate) fn dlsd_unless<F: FnOnce(&[$uint]) -> bool>(bits: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>, reject: F) -> bool {
            let n = bits.len();
            if n > u32::MAX as usize || n <= SMALL {
                dlsd_with(bits, scratch);
                return true;
            }
            let mut stack = [0 as $uint; STACK_SAMPLE];
            let mut heap: Vec<$uint> = Vec::new();
            let sample = sample_slots(&mut stack, &mut heap, sample_size(n));
            for (i, slot) in sample.iter_mut().enumerate() {
                *slot = w::encode_key(bits[spread_position(i, n)]);
            }
            sample.sort_unstable();
            if reject(sample) {
                return false;
            }
            dlsd_rec::<true>(bits, scratch, Some(sample));
            true
        }

        /// Sorts already encoded keys and writes them back decoded.
        pub(crate) fn dlsd_encoded(keys: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>) {
            if keys.len() > u32::MAX as usize {
                w::lsd_encoded(keys, scratch);
                return;
            }
            dlsd_rec::<false>(keys, scratch, None);
        }

        /// The key of a stored value: encoded when `RAW`.
        #[inline(always)]
        fn enc<const RAW: bool>(v: $uint) -> $uint {
            if RAW {
                w::encode_key(v)
            } else {
                v
            }
        }

        #[inline(always)]
        fn decode_all(keys: &mut [$uint]) {
            keys.iter_mut().for_each(|k| *k = w::decode_key(*k));
        }

        /// The digits of the prefix: `digits` digits of `width` bits, the
        /// prefix being bits `base..top` of `key - min` (exact when `base` is 0).
        #[derive(Clone, Copy)]
        struct Layout {
            width: u32,
            digits: usize,
            base: u32,
            buckets: usize,
            mask: $uint,
        }

        impl Layout {
            /// Digits for a prefix of `prefix` bits under `top`: as few passes
            /// of at most `MAX_DIGIT_BITS` as possible, the prefix split evenly,
            /// at most about n / 16 digit values. 128-bit keys fill the passes
            /// paid for (smaller groups left to the finish).
            fn new(prefix: u32, top: u32, n: usize) -> Self {
                let log_n = usize::BITS - n.leading_zeros();
                let widest = MAX_DIGIT_BITS.min(log_n.saturating_sub(4).max(8));
                let mut bits = prefix;
                if <$uint>::BITS == 128 {
                    bits = (bits.div_ceil(widest) * widest).min(top);
                }
                let passes = bits.div_ceil(MAX_DIGIT_BITS);
                let width = bits.div_ceil(passes).min(log_n.saturating_sub(4).max(8)).max(1);
                let digits = bits.div_ceil(width) as usize;
                let base = top.saturating_sub(digits as u32 * width);
                let buckets = 1usize << width;
                Layout { width, digits, base, buckets, mask: (buckets - 1) as $uint }
            }
        }

        /// `RAW`: `keys` hold raw bits, else encoded keys; output: raw bits.
        /// `pre`: the sorted spread sample of the encoded keys, when taken.
        fn dlsd_rec<const RAW: bool>(keys: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>, pre: Option<&mut [$uint]>) {
            let n = keys.len();
            if n <= small_limit() {
                sort_small::<RAW>(keys);
                return;
            }
            let (min, max) = min_max::<RAW>(keys);
            let range = max - min;
            if range == 0 {
                if !RAW {
                    decode_all(keys);
                }
                return;
            }
            // 128-bit keys within a 64-bit range: sorted as 64- or 32-bit keys.
            if <$uint as WideNarrow>::narrow(keys, min, range, RAW, w::encode_key, w::decode_key) {
                return;
            }
            let top = <$uint>::BITS - range.leading_zeros();
            let layout = Layout::new(prefix_bits::<RAW>(keys, min, top, pre), top, n);
            // Histograms: on the stack when they fit (only the part used is
            // zeroed), else on the heap.
            let mut stack = std::mem::MaybeUninit::<[u32; STACK_COUNTERS]>::uninit();
            let mut heap: Vec<u32> = Vec::new();
            let tables = zeroed_counters(&mut stack, &mut heap, 2 * layout.digits * layout.buckets);
            count_digits::<RAW>(keys, min, &layout, tables);
            let tables = &mut tables[..layout.digits * layout.buckets];
            scatter_passes::<RAW>(keys, scratch, tables, min, &layout);
            if layout.base == 0 {
                return;
            }
            if !finish_by_insertion(keys) {
                finish_groups(keys, scratch, min, layout.base);
            }
        }

        /// Arrays up to this size go to `sort_small`: larger for 128-bit keys
        /// (the fixed costs of a DLSD call outweigh `sort_unstable` there).
        #[inline(always)]
        fn small_limit() -> usize {
            if <$uint>::BITS == 128 {
                SMALL_128
            } else {
                SMALL
            }
        }

        /// Small arrays: insertion sort or `sort_unstable` on the encoded keys;
        /// 128-bit keys within a 64-bit range first narrowed.
        fn sort_small<const RAW: bool>(keys: &mut [$uint]) {
            let n = keys.len();
            if <$uint>::BITS == 128 && n > GROUP_INSERTION {
                let (min, max) = min_max::<RAW>(keys);
                if <$uint as WideNarrow>::narrow(keys, min, max - min, RAW, w::encode_key, w::decode_key) {
                    return;
                }
            }
            if RAW {
                keys.iter_mut().for_each(|k| *k = w::encode_key(*k));
            }
            if n <= GROUP_INSERTION {
                insertion(keys);
            } else {
                keys.sort_unstable();
            }
            decode_all(keys);
        }

        #[inline(always)]
        fn min_max<const RAW: bool>(keys: &[$uint]) -> ($uint, $uint) {
            let (mut min, mut max) = (<$uint>::MAX, 0 as $uint);
            for &v in keys.iter() {
                let k = enc::<RAW>(v);
                min = min.min(k);
                max = max.max(k);
            }
            (min, max)
        }

        /// `len` zeroed counters, on the stack when they fit.
        #[inline(always)]
        fn zeroed_counters<'a>(stack: &'a mut std::mem::MaybeUninit<[u32; STACK_COUNTERS]>, heap: &'a mut Vec<u32>, len: usize) -> &'a mut [u32] {
            if len <= STACK_COUNTERS {
                // SAFETY: the first `len` counters are zeroed before the slice
                // over them is formed.
                unsafe {
                    let ptr = stack.as_mut_ptr() as *mut u32;
                    std::ptr::write_bytes(ptr, 0, len);
                    std::slice::from_raw_parts_mut(ptr, len)
                }
            } else {
                heap.resize(len, 0);
                heap
            }
        }

        /// Histograms of the prefix digits in one read, two lanes (`tables`
        /// holds both: `digits * buckets` counters each; summed into the first).
        /// Up to 4 digits the digit loop is unrolled (`hist_k`).
        #[inline(always)]
        fn count_digits<const RAW: bool>(keys: &[$uint], min: $uint, layout: &Layout, tables: &mut [u32]) {
            let Layout { width, digits, base, buckets, mask } = *layout;
            let (t0, t1) = tables.split_at_mut(digits * buckets);
            match digits {
                1 => hist_k::<1, RAW>(keys, min, base, width, mask, buckets, t0, t1),
                2 => hist_k::<2, RAW>(keys, min, base, width, mask, buckets, t0, t1),
                3 => hist_k::<3, RAW>(keys, min, base, width, mask, buckets, t0, t1),
                4 => hist_k::<4, RAW>(keys, min, base, width, mask, buckets, t0, t1),
                _ => {
                    let mut pairs = keys.chunks_exact(2);
                    for pair in &mut pairs {
                        let a = enc::<RAW>(pair[0]).wrapping_sub(min) >> base;
                        let b = enc::<RAW>(pair[1]).wrapping_sub(min) >> base;
                        for t in 0..digits {
                            let s = t as u32 * width;
                            // SAFETY: masked digits index a table of `buckets` counters.
                            unsafe {
                                *t0.get_unchecked_mut(t * buckets + ((a >> s) & mask) as usize) += 1;
                                *t1.get_unchecked_mut(t * buckets + ((b >> s) & mask) as usize) += 1;
                            }
                        }
                    }
                    for &v in pairs.remainder() {
                        let a = enc::<RAW>(v).wrapping_sub(min) >> base;
                        for t in 0..digits {
                            t0[t * buckets + ((a >> (t as u32 * width)) & mask) as usize] += 1;
                        }
                    }
                    for i in 0..digits * buckets {
                        t0[i] += t1[i];
                    }
                }
            }
        }

        /// The scatter passes on the prefix digits that vary (a digit equal in
        /// every key has all of them in the bucket of the first key), keys and
        /// buffer alternating; a pass whose bucket starts alias in the cache is
        /// staged. Decodes in the last pass when the prefix is exact.
        fn scatter_passes<const RAW: bool>(keys: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>, tables: &mut [u32], min: $uint, layout: &Layout) {
            let n = keys.len();
            let Layout { width, digits, base, buckets, mask } = *layout;
            let exact = base == 0;
            let first_prefix = enc::<RAW>(keys[0]).wrapping_sub(min) >> base;
            let mut varying = [0usize; 64];
            let mut passes = 0;
            for t in 0..digits {
                if tables[t * buckets + ((first_prefix >> (t as u32 * width)) & mask) as usize] as usize != n {
                    varying[passes] = t;
                    passes += 1;
                }
            }
            let varying = &varying[..passes];
            if scratch.len() < n {
                scratch.reserve(n - scratch.len());
                // SAFETY: `MaybeUninit` needs no initialization.
                unsafe { scratch.set_len(n) };
            }
            let buf = scratch.as_mut_ptr() as *mut $uint;
            // The tables become their exclusive prefix sums (the bucket heads).
            let check_alias = n >= $alias_min() / std::mem::size_of::<$uint>();
            let mut staged = [false; 64];
            for (i, &t) in varying.iter().enumerate() {
                let table = &mut tables[t * buckets..(t + 1) * buckets];
                let mut sum = 0u32;
                for c in table.iter_mut() {
                    let v = *c;
                    *c = sum;
                    sum += v;
                }
                staged[i] = check_alias && aliased(table, n);
            }
            let mut staging: Vec<$uint> = Vec::new();
            let mut starts: Vec<u32> = Vec::new();
            let last = passes - 1;
            for (i, &t) in varying.iter().enumerate() {
                let shift = base + t as u32 * width;
                let heads = &mut tables[t * buckets..(t + 1) * buckets];
                if staged[i] && staging.len() < heads.len() * WB {
                    staging = vec![0; heads.len() * WB];
                    starts = vec![0; heads.len()];
                }
                // SAFETY: pass `i` reads `keys` (even) or the buffer (odd, fully
                // written by pass `i - 1`) and writes each of the `n` slots of the
                // other once (heads are the prefix sums of this digit over the same
                // keys); they do not overlap.
                unsafe {
                    let (src, dst): (&[$uint], *mut $uint) = if i % 2 == 0 {
                        (&*keys, buf)
                    } else {
                        (std::slice::from_raw_parts(buf, n), keys.as_mut_ptr())
                    };
                    macro_rules! pass {
                        ($first:expr, $decode:expr) => {
                            if staged[i] {
                                scatter_staged::<$first, $decode>(src, dst, heads, min, shift, mask, &mut staging, &mut starts)
                            } else {
                                scatter::<$first, $decode>(src, dst, heads, min, shift, mask)
                            }
                        };
                    }
                    match (RAW && i == 0, exact && i == last) {
                        (true, true) => pass!(true, true),
                        (true, false) => pass!(true, false),
                        (false, true) => pass!(false, true),
                        (false, false) => pass!(false, false),
                    }
                }
            }
            if passes % 2 == 1 {
                // SAFETY: the buffer was fully written by the last pass.
                keys.copy_from_slice(unsafe { std::slice::from_raw_parts(buf, n) });
            }
        }

        /// Finish of an inexact prefix: the keys are sorted up to groups of equal
        /// prefix, mostly single keys or small groups. An insertion sort over the
        /// whole array then mostly takes the well predicted "in place" branch; a
        /// block of 16 keys already in order is passed in one vectorized test.
        /// Gives up (returns false) beyond `INSERTION_BUDGET` moves per key;
        /// returns true once every key is in place, decoded.
        fn finish_by_insertion(keys: &mut [$uint]) -> bool {
            let n = keys.len();
            let budget = INSERTION_BUDGET * n;
            let mut moves = 0;
            let mut i = 1;
            // Blocks are tested from this index on (a failed test lets the
            // scalar loop through the whole block first).
            let mut test_at = 1;
            while i < n {
                if i >= test_at && i + 16 <= n {
                    let block = &keys[i - 1..i + 16];
                    let mut in_order = true;
                    for p in 0..16 {
                        in_order &= block[p] <= block[p + 1];
                    }
                    if in_order {
                        i += 16;
                        continue;
                    }
                    test_at = i + 16;
                }
                let v = keys[i];
                if keys[i - 1] > v {
                    let mut j = i;
                    while j > 0 && keys[j - 1] > v {
                        keys[j] = keys[j - 1];
                        j -= 1;
                    }
                    keys[j] = v;
                    moves += i - j;
                    if moves > budget {
                        return false;
                    }
                }
                i += 1;
            }
            decode_all(keys);
            true
        }

        /// Groups of equal prefix finished one by one: insertion sort, or the
        /// DLSD again at the scale of the group (its own min and max).
        fn finish_groups(keys: &mut [$uint], scratch: &mut Vec<MaybeUninit<$uint>>, min: $uint, low: u32) {
            let n = keys.len();
            let prefix = |k: $uint| k.wrapping_sub(min) >> low;
            let mut i = 0;
            while i < n {
                let group = prefix(keys[i]);
                let mut j = i + 1;
                while j < n && prefix(keys[j]) == group {
                    j += 1;
                }
                if j == i + 1 {
                    keys[i] = w::decode_key(keys[i]);
                } else if j - i <= GROUP_INSERTION {
                    let g = &mut keys[i..j];
                    insertion(g);
                    decode_all(g);
                } else {
                    dlsd_rec::<false>(&mut keys[i..j], scratch, None);
                }
                i = j;
            }
        }
    };
}
