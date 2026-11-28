//! Dev only (feature `ab`): A/B copy of the u32 / i32 chain (`chain32.rs`,
//! regenerated from it at the 2.1.2 refactor); edit here, measured against
//! the library by `examples/ab_u32.rs` (`copy`).

macro_rules! chain32_ab {
    ($name:ident, $t:ty, $c:ident, $ls:ident, $st:ident, $ro:ident) => {
        pub(crate) mod $name {
            use crate::v2::cache::{budget, Budget};
            use crate::v2::decide::path::{
                can_skip_verge, merges_unpredictable, starts_with_random_runs,
            };
            use crate::v2::decide::shape::{globally_ordered, locally_ordered};
            use crate::v2::devour::keys::*;
            use crate::v2::devour::simd32 as simd;
            use crate::v2::devour::DevourSort;
            use crate::v2::engines::counting::exact_count_stack as exact_count;
            use crate::v2::engines::dlsd::$st;
            use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
            use crate::v2::engines::{lsd::$ls, roller::$ro};
            use crate::v2::inspect::presorted::{
                presorted, probe_pairs, repair_nearly_sorted, Unsorted,
            };
            use crate::v2::runs::merge::{self, merge_runs};
            use crate::v2::runs::verge::verge_sort_preprocessing_fast;
            use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
            use crate::v2::special::plateaus::plateaus;
            use crate::v2::special::{spread, DENSE_MIN};

            /// Up to this size, when the direct networks do not apply:
            /// `sort_unstable`.
            const SMALL: usize = 96;

            /// The direct SIMD networks up to this many keys: x86 512 (the
            /// networks' limit); ARM 256 (Surface, 129-256 keys: 27-34 cells won
            /// against the best rival, 4-22 through `short32`; from 300 the
            /// radix path wins).
            const DIRECT_MAX: usize =
                if cfg!(target_arch = "aarch64") { 256 } else { simd::SIMD_MAX };

            /// ARM: the direct networks from this many keys only (below, the
            /// half-empty 128-key network loses to `sort_unstable`).
            const DIRECT_MIN: usize = if cfg!(target_arch = "aarch64") { 80 } else { 0 };

            /// `short32` up to this many keys (Surface and Ryzen: it wins
            /// 150-500 keys, the light chain from 700: 25-35 cells).
            const SHORT32_MAX: usize = 600;

            /// The light chain up to this many keys.
            const LIGHT_MAX: usize = 8192;

            /// Up to this many keys `short32` ends with the short radix sort on
            /// x86 (it beats the DLSD on 22-28 cells at 170-300 keys, loses
            /// 26-30 from 500).
            const SHORT_MAX: usize = 400;

            /// Below this many keys a chunk goes to the DLSD.
            const DLSD_MAX: usize = 262_144;

            /// The rollercoaster from this size (the L2 bytes; for its memory, n instead of 2n).
            fn roller_min() -> usize { budget(Budget::Roller32) }

            // The range, dense and few detectors of this type, in module `detectors`.
            int_detectors!(detectors, $t, u32, $c, $c, super::roller_min, stack);

            /// Sorts `arr`: a sorting network up to `NETWORK_MAX` keys, else the chain.
            pub(crate) fn sort(arr: &mut [$t]) {
                if arr.len() <= NETWORK_MAX {
                    trace!("network");
                    network_sort(bits_of(arr), $c, $c);
                } else {
                    sort_large(arr);
                }
            }

            /// Arrays beyond the sorting network.
            #[inline(never)]
            fn sort_large(arr: &mut [$t]) {
                let n = arr.len();
                chain! {
                    "merge_small" if n <= MERGE_MAX => do { merge_sort(bits_of(arr), $c, $c) };
                }
                let known = match presorted(arr) {
                    Ok(()) => {
                        trace!("presorted");
                        return;
                    },
                    Err(known) => known,
                };
                // A sorted head of half the array: its tail first, then one
                // merge (the whole path).
                let head = known.ascending_prefix >= n / 2;
                if (DIRECT_MIN..=DIRECT_MAX).contains(&n) && !head && simd::available() {
                    direct(arr);
                    return;
                }
                chain! {
                    "std_small" if n <= SMALL => do { arr.sort_unstable() };
                }
                if n <= LIGHT_MAX && !head {
                    if n <= SHORT32_MAX {
                        short32(arr);
                        return;
                    }
                    // Mostly equal spread pairs from 2000 keys (plateaus): the
                    // whole path. The pairs are compared here when `presorted`
                    // did not (it does only when the first key is the larger).
                    let probes = known.probes.unwrap_or_else(|| probe_pairs(arr));
                    if !(n >= 2000 && probes.equal() >= 24) {
                        light(arr, Unsorted { probes: Some(probes), ..known });
                        return;
                    }
                }
                let buffer = std::cell::RefCell::new(Vec::new());
                let sort_chunk =
                    |chunk: &mut [$t]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
                path(arr, known, &detect, &sort_chunk);
            }

            /// The direct SIMD networks: up to 128 keys the range is checked in
            /// the registers (ARM beyond 128: a min / max pass first); within n
            /// values, counted instead.
            fn direct(arr: &mut [$t]) {
                let n = arr.len();
                let r = if cfg!(target_arch = "aarch64") && n > 128 {
                    let (lo, hi) = bits_of(arr)
                        .iter()
                        .fold((u32::MAX, 0u32), |(l, h), &b| (l.min($c(b)), h.max($c(b))));
                    if ((hi - lo) as usize) < n {
                        Some((lo, hi))
                    } else {
                        simd::sort(bits_of(arr), $c(0));
                        None
                    }
                } else {
                    simd::sort_or_range(bits_of(arr), $c(0), n)
                };
                match r {
                    None => {
                        trace!("simd32_direct");
                    },
                    Some((lo, hi)) if hi - lo < 256 => {
                        trace!("simd32_count");
                        count_stack(bits_of(arr), lo, (hi - lo) as usize + 1);
                    },
                    Some((lo, hi)) => {
                        trace!("simd32_count");
                        exact_count(bits_of(arr), lo, (hi - lo) as usize + 1, $c, $c);
                    },
                }
            }

            /// The light chain (`SHORT32_MAX` to `LIGHT_MAX` keys): the
            /// near-sorted repair when nearly all spread pairs go one way, then
            /// few4, range, a dominant value, few, dense on the spread sample;
            /// then the engine (no plateaus, heavy, Verge).
            fn light(arr: &mut [$t], known: Unsorted) {
                let n = arr.len();
                let probes = known.probes.unwrap_or_else(|| probe_pairs(arr));
                // Nearly all pairs in one direction (30 of 32: saw teeth show
                // about 28): the near-sorted repair alone (no Verge: saw teeth
                // and runs cost 1.2-3x on the whole path).
                if (probes.ascending >= 30 || probes.descending >= 30)
                    && repair_nearly_sorted(arr, known).0
                {
                    trace!("light_repaired");
                    return;
                }
                let sample = spread(arr);
                if few4(arr, &sample) {
                    trace!("light_few4");
                    return;
                }
                if detectors::range(arr, &sample) {
                    trace!("light_range");
                    return;
                }
                // One dominant value with a long tail of distinct values
                // (Sparse), on the first 16 sample keys: the majority vote, its
                // count at least 8 and no other value 3 times or more (FakeZipf
                // repeats a second value: left to `few`). Then the std partition
                // (as `detect`). The full `dominant` on the 64 keys cost up to
                // 0.8 ns per key at 250 keys.
                let s16 = &sample[..16];
                let (mut v, mut votes) = (s16[0], 0usize);
                for &x in s16 {
                    if votes == 0 {
                        v = x;
                    }
                    votes = if x == v { votes + 1 } else { votes - 1 };
                }
                let count = |u: $t| s16.iter().filter(|&&x| x == u).count();
                if count(v) >= 8 && s16.iter().all(|&u| u == v || count(u) < 3) {
                    trace!("light_dominant");
                    arr.sort_unstable();
                    return;
                }
                // Below 512 keys `few` only when the first 16 sample keys hold
                // at most 6 values (Heavy 4 shows about 10: few sorts its ~120
                // values at 300 keys, 1.5x slower than the DLSD).
                let mut distinct = 0;
                for i in 0..16 {
                    distinct += !sample[..i].contains(&sample[i]) as usize;
                }
                if (distinct <= 6 || n >= 512) && detectors::few(arr, &sample) {
                    trace!("light_few");
                    return;
                }
                // From `DENSE_MIN` keys a dense range with outliers (Int late
                // out: 2.3x slower without it at 5000 keys), on the sorted
                // sample as `detect`.
                if n >= DENSE_MIN {
                    let mut sorted = sample;
                    sorted.sort_unstable();
                    if detectors::dense(arr, &sorted) {
                        trace!("light_dense");
                        return;
                    }
                }
                sort_chunk(bits_of(arr), &mut Vec::new());
            }

            /// At most 4 distinct values on the sample (Binary, Few 4): each key
            /// compared with them in one pass (no branch per key), the keys
            /// written back from the counts when they are all among them (else
            /// nothing written). Port of the 16-bit `cheap_few`; idea of
            /// blazing-radix (a sample proposes at most four keys, verified by a
            /// full scan before any write).
            fn few4(arr: &mut [$t], sample: &[$t]) -> bool {
                let mut reps = [sample[0]; 4];
                let mut k = 1;
                for &v in &sample[1..] {
                    if !reps[..k].contains(&v) {
                        if k == 4 {
                            return false;
                        }
                        reps[k] = v;
                        k += 1;
                    }
                }
                reps[..k].sort_unstable();
                match k {
                    1 => few4_k::<1>(arr, &reps),
                    2 => few4_k::<2>(arr, &reps),
                    3 => few4_k::<3>(arr, &reps),
                    _ => few4_k::<4>(arr, &reps),
                }
            }

            fn few4_k<const KN: usize>(arr: &mut [$t], reps: &[$t; 4]) -> bool {
                let mut c = [0usize; KN];
                for &x in arr.iter() {
                    for j in 0..KN {
                        c[j] += (x == reps[j]) as usize;
                    }
                }
                if c.iter().sum::<usize>() != arr.len() {
                    return false;
                }
                let mut at = 0;
                for j in 0..KN {
                    arr[at..at + c[j]].fill(reps[j]);
                    at += c[j];
                }
                true
            }

            /// The short sort (up to `SHORT32_MAX` keys), few questions. One
            /// pass: the descents and the bounds (ORs of the distances to the
            /// first key); sorted or reversed (no descent, or all); a range of
            /// at most n values counted; 8 spread probes (at most 4 values, a
            /// dominant one, many duplicates); x86: the SIMD networks, then the
            /// short radix sort up to `SHORT_MAX` keys; else the DLSD.
            fn short32(arr: &mut [$t]) {
                let n = arr.len();
                let (up, down, desc, e0, any_below) = {
                    let bits: &[u32] = bits_of(arr);
                    let e0 = $c(bits[0]);
                    let (up, down, desc, any) = bits.iter().zip(&bits[1..]).fold(
                        (0u32, 0u32, 0u32, 0u32),
                        |(u, dn, d, a_), (&a, &b)| {
                            let (ea, eb) = ($c(a), $c(b));
                            let x = eb.wrapping_sub(e0);
                            let below = ((x as i32) >> 31) as u32;
                            (u | (x & !below), dn | (!x & below), d + (ea > eb) as u32, a_ | below)
                        },
                    );
                    (up, down, desc as usize, e0, any != 0)
                };
                if desc == 0 {
                    trace!("short32_sorted");
                    return;
                }
                if desc == n - 1 {
                    trace!("short32_reversed");
                    arr.reverse();
                    return;
                }
                let tp = 32 - up.leading_zeros();
                let tn = 32 - down.leading_zeros();
                // No key below the first one: the low bound is the first key
                // (`e0 - 1` underflowed at 0 and dropped the bounds: saws, Int
                // 0..n starting at their minimum).
                let below = if !any_below { 0 } else { 1u32 << tn };
                let bounds = if tp < 31 && tn < 31 {
                    match (e0.checked_sub(below), e0.checked_add((1u32 << tp) - 1)) {
                        (Some(l), Some(h)) => Some((l, h)),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some((lo, hi)) = bounds {
                    // Counted up to n values (as blazing-radix); up to 256 on the
                    // stack.
                    if ((hi - lo) as usize) < n {
                        if hi - lo < 256 {
                            trace!("short32_count_stack");
                            count_stack(bits_of(arr), lo, (hi - lo) as usize + 1);
                            return;
                        }
                        trace!("short32_count");
                        exact_count(bits_of(arr), lo, (hi - lo) as usize + 1, $c, $c);
                        return;
                    }
                }
                // 8 spread probes only here (wide keys): Binary, Sparse, Few 4
                // 1.4-2.1x faster at 300-500 keys.
                if probe8(arr) {
                    return;
                }
                // ARM: no networks here (the NEON merges of blocks lose to the
                // DLSD from 200 keys on the Surface).
                if cfg!(not(target_arch = "aarch64")) && n <= simd::SIMD_MAX && simd::available() {
                    trace!("short32_simd");
                    simd::sort(bits_of(arr), $c(0));
                    return;
                }
                if let Some((lo, hi)) = bounds {
                    if cfg!(not(target_arch = "aarch64")) && n <= SHORT_MAX {
                        trace!("short32_short");
                        short_radix(bits_of(arr), lo, hi);
                        return;
                    }
                }
                trace!("short32_dlsd");
                $st::dlsd(bits_of(arr));
            }

            /// Keys within `lo..lo + width` (`width` at most 256), counted in
            /// two tables on the stack (consecutive keys alternate), written
            /// back.
            #[inline(never)]
            fn count_stack(bits: &mut [u32], lo: u32, width: usize) {
                let mut a = [0u32; 256];
                let mut b = [0u32; 256];
                let off = |k: u32| ($c(k).wrapping_sub(lo) & 255) as usize;
                if width <= 4 {
                    // At most 4 values: counted by comparisons (vectorized; two
                    // counters in memory would wait on their previous increment:
                    // Binary at 300-600 keys 2x slower than blazing on the
                    // Surface).
                    let mut c = [0u32; 4];
                    for &k in bits.iter() {
                        let o = $c(k).wrapping_sub(lo);
                        for (j, cj) in c.iter_mut().enumerate() {
                            *cj += (o == j as u32) as u32;
                        }
                    }
                    a[..4].copy_from_slice(&c);
                } else {
                    let (pairs, tail) = bits.as_chunks::<2>();
                    for p in pairs {
                        a[off(p[0])] += 1;
                        b[off(p[1])] += 1;
                    }
                    for &k in tail {
                        a[off(k)] += 1;
                    }
                }
                // Short runs by blocks of 16 (the extra copies overwritten by
                // the next runs), as `exact_count`.
                let n = bits.len();
                let mut at = 0;
                for d in 0..width {
                    let c = (a[d] + b[d]) as usize;
                    if c != 0 {
                        let v = $c(lo + d as u32);
                        let end = at + c;
                        if c <= 64 && end + 16 <= n {
                            let block = [v; 16];
                            let mut p = at;
                            while p < end {
                                bits[p..p + 16].copy_from_slice(&block);
                                p += 16;
                            }
                        } else {
                            bits[at..end].fill(v);
                        }
                        at = end;
                    }
                }
            }

            /// 8 spread probes, given up at the 5th value. One value 5 times or
            /// more and the others once (Sparse): `sort_unstable`. At most 4
            /// values: `few4_early`, else many duplicates (FakeZipf):
            /// `sort_unstable` (its partitions set the keys equal to the pivot
            /// aside at once).
            #[inline(never)]
            fn probe8(arr: &mut [$t]) -> bool {
                let n = arr.len();
                let mut p = [arr[0]; 8];
                for i in 0..8 {
                    p[i] = arr[i * (n - 1) / 7];
                }
                let mut vals = [p[0]; 4];
                let mut cnt = [0usize; 4];
                let mut k = 0;
                for &x in p.iter() {
                    match vals[..k].iter().position(|&v| v == x) {
                        Some(j) => cnt[j] += 1,
                        None if k < 4 => {
                            vals[k] = x;
                            cnt[k] = 1;
                            k += 1;
                        },
                        None => return false,
                    }
                }
                let top = *cnt[..k].iter().max().unwrap();
                if top >= 5 && top + (k - 1) == 8 {
                    trace!("short32_dominant");
                    arr.sort_unstable();
                    return true;
                }
                if few4_early(arr, &p[..]) {
                    trace!("short32_few4");
                    return true;
                }
                trace!("short32_dups");
                arr.sort_unstable();
                true
            }

            /// `few4` giving up at the first block of 64 keys holding a key
            /// outside the representatives (`few4` reads every key first). The 4
            /// comparisons of a key without a `j < k` test (missing values padded
            /// with copies, their counts ignored), u32 counters per block:
            /// vectorized (Few 4 at 300-600 keys was 1.6-1.8x slower than
            /// blazing on the Surface).
            fn few4_early(arr: &mut [$t], sample: &[$t]) -> bool {
                let mut reps = [sample[0]; 4];
                let mut k = 1;
                for &v in &sample[1..] {
                    if !reps[..k].contains(&v) {
                        if k == 4 {
                            return false;
                        }
                        reps[k] = v;
                        k += 1;
                    }
                }
                reps[..k].sort_unstable();
                let mut c = [0usize; 4];
                let pad = reps[0];
                for r in reps[k..].iter_mut() {
                    *r = pad;
                }
                for chunk in arr.chunks(64) {
                    let mut cc = [0u32; 4];
                    for &x in chunk {
                        for j in 0..4 {
                            cc[j] += (x == reps[j]) as u32;
                        }
                    }
                    let hit: u32 = cc[..k].iter().sum();
                    if hit as usize != chunk.len() {
                        return false;
                    }
                    for j in 0..k {
                        c[j] += cc[j] as usize;
                    }
                }
                let mut at = 0;
                for j in 0..k {
                    arr[at..at + c[j]].fill(reps[j]);
                    at += c[j];
                }
                true
            }

            /// The short radix sort of `short32` (x86, up to `SHORT_MAX` keys),
            /// on raw bits whose encoded keys lie in `lo..=hi`: the top
            /// log2(n) + 3 bits of `key - lo` (at most 12), one pass up to 8
            /// bits, else 2 LSD passes, both histograms in one read, buffer and
            /// counters on the stack; then an insertion sort finishes the keys
            /// sharing a prefix (`sort_unstable` beyond 2n moves: clustered
            /// keys). No sample, no allocation (the 16-bit short LSD's approach).
            fn short_radix(bits: &mut [u32], lo: u32, hi: u32) {
                let n = bits.len();
                debug_assert!(n <= SHORT_MAX);
                let range = hi - lo;
                if range == 0 {
                    return;
                }
                let top = 32 - range.leading_zeros();
                let log_n = usize::BITS - n.leading_zeros();
                let p = (log_n + 3).min(top);
                let base = top - p;
                // At most 8 bits: one pass (a counting sort that moves the keys:
                // a range of 256 values at 150 keys, where writing the values
                // back from 256 counters costs 2.5x).
                if p <= 8 {
                    let key = |b: u32| ($c(b).wrapping_sub(lo) >> base) as usize;
                    let mut c = [0u32; 256];
                    let c = &mut c[..1 << p];
                    for &b in bits.iter() {
                        // SAFETY: key < 2^p <= 256.
                        unsafe { *c.get_unchecked_mut(key(b)) += 1 };
                    }
                    let mut sum = 0;
                    for x in c.iter_mut() {
                        let v = *x;
                        *x = sum;
                        sum += v;
                    }
                    let mut store = [0u32; SHORT_MAX];
                    let buf = &mut store[..n];
                    // SAFETY: as above; each head plus its keys written is below n.
                    unsafe {
                        for &b in bits.iter() {
                            let at = c.get_unchecked_mut(key(b));
                            *buf.get_unchecked_mut(*at as usize) = b;
                            *at += 1;
                        }
                    }
                    bits.copy_from_slice(buf);
                    if base > 0 {
                        finish_short(bits, n);
                    }
                    return;
                }
                let w0 = p / 2;
                let m0 = (1u32 << w0) - 1;
                let key = |b: u32| $c(b).wrapping_sub(lo) >> base;
                let (mut c0, mut c1) = ([0u32; 128], [0u32; 128]);
                for &b in bits.iter() {
                    let k = key(b);
                    // SAFETY: k < 2^p, p <= 12, so both digits are below 128.
                    unsafe {
                        *c0.get_unchecked_mut((k & m0) as usize) += 1;
                        *c1.get_unchecked_mut((k >> w0) as usize) += 1;
                    }
                }
                let (mut s0, mut s1) = (0u32, 0u32);
                for d in 0..128 {
                    let (a, b) = (c0[d], c1[d]);
                    c0[d] = s0;
                    c1[d] = s1;
                    s0 += a;
                    s1 += b;
                }
                let mut store = [0u32; SHORT_MAX];
                let buf = &mut store[..n];
                // SAFETY (both passes): the digits are below 128 (above); each
                // head plus the keys of its bucket already written is below n.
                unsafe {
                    for &b in bits.iter() {
                        let d = (key(b) & m0) as usize;
                        let at = c0.get_unchecked_mut(d);
                        *buf.get_unchecked_mut(*at as usize) = b;
                        *at += 1;
                    }
                    for &b in buf.iter() {
                        let d = (key(b) >> w0) as usize;
                        let at = c1.get_unchecked_mut(d);
                        *bits.get_unchecked_mut(*at as usize) = b;
                        *at += 1;
                    }
                }
                if base == 0 {
                    return;
                }
                finish_short(bits, n);
            }

            /// The insertion finish of the short radix sort: a block of 16
            /// keys in order passed in one test; `sort_unstable` beyond 2n moves.
            fn finish_short(bits: &mut [u32], n: usize) {
                let mut moves = 0;
                let (mut i, mut test_at) = (1, 1);
                while i < n {
                    if i >= test_at && i + 16 <= n {
                        let block = &bits[i - 1..i + 16];
                        let mut in_order = true;
                        for p in 0..16 {
                            in_order &= $c(block[p]) <= $c(block[p + 1]);
                        }
                        if in_order {
                            i += 16;
                            continue;
                        }
                        test_at = i + 16;
                    }
                    let v = bits[i];
                    let kv = $c(v);
                    if $c(bits[i - 1]) > kv {
                        let mut j = i;
                        while j > 0 && $c(bits[j - 1]) > kv {
                            bits[j] = bits[j - 1];
                            j -= 1;
                        }
                        bits[j] = v;
                        moves += i - j;
                        if moves > 2 * n {
                            bits.sort_unstable_by_key(|&b| $c(b));
                            return;
                        }
                    }
                    i += 1;
                }
            }

            /// The detectors, on one spread sample (sorted once, only when the detectors
            /// that need it in order are reached). `equal_pairs`: equal pairs among the
            /// probes of the near-sorted repair.
            fn detect(arr: &mut [$t], equal_pairs: usize) -> bool {
                let mut sample = spread(arr);
                chain! {
                    return true;
                    "few4" => try few4(arr, &sample);
                    "range" => try detectors::range(arr, &sample);
                    "plateaus" => try plateaus(arr, equal_pairs);
                    // One dominant value among spread ones, short array: the std
                    // partition settles the keys equal to its pivot in one pass.
                    "dominant" if arr.len() < DENSE_MIN
                        && dominant(&sample, |x, y| x == y)
                        => do { arr.sort_unstable() };
                    "few" => try detectors::few(arr, &sample);
                }
                // Heavy keys and dense ranges: the sample in order.
                if arr.len() < HEAVY_MIN
                    || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y))
                {
                    return false;
                }
                sample.sort_unstable();
                chain! {
                    return true;
                    "heavy" => try heavy_sorted_by(
                        arr,
                        &sample,
                        HEAVY_MIN,
                        &|r: &mut [$t]| sort(r),
                        |x, y| x == y,
                        |x, y| x < y,
                    );
                    "dense" if arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
                }
                false
            }

            /// The engine of one chunk (the whole array, or a gap between Verge's runs).
            #[inline(always)]
            fn sort_chunk(bits: &mut [u32], _: &mut Vec<std::mem::MaybeUninit<u32>>) {
                let n = bits.len();
                // Below `DLSD_MAX` keys the DLSD, except: a global trend under
                // local noise (timestamps with jitter: spread keys in order,
                // neighbours not), the DLSD 1.3-1.9x slower there (Surface
                // 20k-200k); composite keys from 65 536 keys. Both to the LSD.
                // Beyond the cache, the rollercoaster only on ordered keys (the
                // MSD pass then writes in few sequential streams; random keys go
                // 15 to 45 % faster through the LSD from 30M to 1G).
                if (4096..DLSD_MAX).contains(&n) && globally_ordered(bits) && !locally_ordered(bits)
                {
                    trace!("lsd_trend");
                    $ls::lsd(bits);
                } else if (65536..DLSD_MAX).contains(&n) && composite_like(bits) {
                    trace!("lsd_composite");
                    $ls::lsd(bits);
                } else if n < DLSD_MAX {
                    trace!("dlsd");
                    $st::dlsd(bits);
                } else if n >= roller_min() && (locally_ordered(bits) || globally_ordered(bits)) {
                    trace!("roller");
                    $ro::roller(bits);
                } else {
                    trace!("lsd");
                    $ls::lsd(bits);
                }
            }

            /// Composite keys (few high halves spread over the domain, uniform
            /// low halves: a tenant and a value), on 256 spread keys: 8 to 64
            /// distinct high halves spanning at least 16 times their count. The
            /// DLSD sizes its prefix for the tenants and leaves whole tenants to
            /// its finish (Surface 5k-200k: LSD 1.1-1.25x faster).
            fn composite_like(bits: &[u32]) -> bool {
                let step = bits.len() / 256;
                let mut k = [0u32; 256];
                for (i, x) in k.iter_mut().enumerate() {
                    *x = $c(bits[i * step]);
                }
                k.sort_unstable();
                let (mut distinct, mut groups, mut wide) = (1, 0, 0);
                let mut start = 0;
                for i in 1..=256 {
                    if i == 256 || k[i] >> 16 != k[start] >> 16 {
                        if i - start >= 2 {
                            groups += 1;
                            // Low halves spread over the group (uniform values).
                            wide += ((k[i - 1] & 0xFFFF) - (k[start] & 0xFFFF) >= 1 << 14) as usize;
                        }
                        if i < 256 {
                            distinct += 1;
                            start = i;
                        }
                    }
                }
                let span = ((k[255] >> 16) - (k[0] >> 16)) as usize;
                (8..=64).contains(&distinct)
                    && span >= 16 * distinct
                    && groups >= 4
                    && wide * 4 >= groups * 3
            }

            /// The radix path (as `decide::path`); the first matching rule wins.
            fn path(
                arr: &mut [$t],
                known: Unsorted,
                detect: &dyn Fn(&mut [$t], usize) -> bool,
                sort: &dyn Fn(&mut [$t]),
            ) {
                let n = arr.len();
                let p = known.ascending_prefix;
                chain! {
                    // A sorted prefix of at least half the array: the tail alone goes
                    // through the path, then one merge. A short tail: sorted (or
                    // reversed) as it is, else the engine.
                    "tail_first" if tail_first(n, p) => do {
                        let tail = &mut arr[p..];
                        if tail.len() >= TAIL_PATH_MIN {
                            path(tail, Unsorted::default(), detect, sort);
                        } else if presorted(tail).is_err() {
                            sort(tail);
                        }
                        merge_runs(arr, &mut vec![0, p, n]);
                    };
                }
                let simd_rule = merge::simd_rule_applies::<$t>(n);
                let random_runs = !simd_rule && starts_with_random_runs(arr, p);
                let (sorted, equal_pairs) = if random_runs {
                    (false, known.probes.map_or(0, |p| p.equal()))
                } else {
                    repair_nearly_sorted(arr, known)
                };
                if random_runs {
                    trace!("random_runs");
                }
                chain! {
                    "repaired" if sorted => do {};
                    _ => try detect(arr, equal_pairs);
                    "no_verge" if random_runs || can_skip_verge(arr) => do { sort(arr) };
                }
                // Verge: the unsorted chunks are recorded, sorted after the merge rule.
                let base = arr.as_ptr() as usize;
                let chunks: std::cell::RefCell<Vec<(usize, usize)>> =
                    std::cell::RefCell::new(Vec::new());
                let mut separators =
                    verge_sort_preprocessing_fast::<_, false>(arr, 8, &|chunk: &mut [$t], _| {
                        let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<$t>();
                        chunks.borrow_mut().push((start, chunk.len()));
                    });
                let chunks = chunks.into_inner();
                // SIMD merges: merging beats the radix sort up to a few merge levels.
                let segments = separators.len().saturating_sub(1);
                let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
                let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
                let unpredictable = merges_unpredictable(arr, &separators, &chunks, levels);
                let sort_chunks = |arr: &mut [$t]| {
                    for &(start, len) in chunks.iter() {
                        sort(&mut arr[start..start + len]);
                    }
                };
                chain! {
                    "verge_whole" if unpredictable && !simd_merge => do { sort(arr) };
                    "verge_simd_merge" if unpredictable => do {
                        sort_chunks(arr);
                        merge::merge_runs_simd(arr, &mut separators);
                    };
                    "verge_merge" => do {
                        sort_chunks(arr);
                        merge_runs(arr, &mut separators);
                    };
                }
            }

            /// Tails from this length go through the whole path (repair, detectors);
            /// shorter ones go to the engine.
            const TAIL_PATH_MIN: usize = 4096;

            /// A sorted prefix of `p` keys out of `n`: the tail-first path.
            fn tail_first(n: usize, p: usize) -> bool { p >= n / 2 && p < n }
        }
    };
}

chain32_ab!(cu32, u32, id32, wu32, su32, ru32);
chain32_ab!(ci32, i32, flip32, wi32, si32, ri32);
