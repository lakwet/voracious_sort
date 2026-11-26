//! Dev only (feature `ab`): A/B copy of the u32 / i32 chains (`u32.rs`,
//! `i32.rs`); edit here, measured against the library by `examples/ab_u32.rs`
//! (`copy`). Trial: the DLSD as the engine below `dlsd_max()` keys
//! (`AB_DLSD_MAX`, default 262 144); its scratch on the stack up to 4096 keys
//! unless `AB_DLSD_HEAP` is set.

/// Below this many keys a chunk goes to the DLSD (env `AB_DLSD_MAX`).
fn dlsd_max() -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("AB_DLSD_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(262_144)
    })
}

/// Questions switched off (the `copy:MASK` sorts of the example): 1 range,
/// 2 plateaus, 4 dominant, 8 few, 16 heavy, 32 dense, 64 every detector
/// (no sample either), 128 Verge (the engine on the whole array), 256 the
/// near-sorted repair, 512 the light chain switched ON (inverted bit: on(512)
/// false means the light chain runs), 1024 the light chain without its
/// sorted-head rule, 2048 without its dominant test, 4096 (inverted) range
/// and few before the near-sorted repair, 8192 no few4 (at most 4 values on
/// the sample, counted by comparisons), 16384 no tiny path, 32768 the DLSD
/// instead of the short radix sort in the tiny path, 65536 the other pass count
/// in the short radix sort below 512 keys (ARM one pass, x86 two).
pub(crate) static SKIP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The light chain up to this many keys (env `AB_LIGHT_MAX`, default 8192).
fn light_max() -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("AB_LIGHT_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(8192)
    })
}

thread_local! {
    /// Range and few already asked on this path (mask bit 4096).
    static EARLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `sort_unstable` up to this many keys (env `AB_SMALL`, default 96; the
/// type's own `SMALL` with `AB_SMALL=0`).
fn small_max(own: usize) -> usize {
    static M: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let m = *M.get_or_init(|| std::env::var("AB_SMALL").ok().and_then(|v| v.parse().ok()).unwrap_or(96));
    if m == 0 { own } else { m }
}

/// Whether the question `bit` is asked.
#[inline(always)]
fn on(bit: u32) -> bool { SKIP.load(std::sync::atomic::Ordering::Relaxed) & bit == 0 }

/// The DLSD with its scratch on the heap (env `AB_DLSD_HEAP`).
fn dlsd_heap() -> bool {
    static H: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *H.get_or_init(|| std::env::var("AB_DLSD_HEAP").is_ok())
}

macro_rules! chain32_ab {
    ($name:ident, $t:ty, $c:ident, $small:expr, $ls:ident, $dl:ident, $st:ident, $ro:ident) => {
        pub(crate) mod $name {
        use super::{dlsd_heap, dlsd_max, light_max, on, small_max, EARLY};
        use crate::v2::engines::dlsd::$st;
        use std::mem::MaybeUninit;
        use crate::v2::cache::{budget, Budget};
        use crate::v2::decide::path::{can_skip_verge, merges_unpredictable, starts_with_random_runs};
        use crate::v2::decide::shape::{globally_ordered, locally_ordered};
        use crate::v2::devour::keys::*;
        use crate::v2::devour::DevourSort;
        use crate::v2::engines::counting::exact_count;
        use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
        use crate::v2::engines::{dlsd::$dl, lsd::$ls, roller::$ro};
        use crate::v2::inspect::presorted::{
            presorted, probe_pairs, repair_nearly_sorted, Probes, Unsorted,
        };
        use crate::v2::runs::merge::{self, merge_runs};
        use crate::v2::runs::verge::verge_sort_preprocessing_fast;
        use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
        use crate::v2::special::plateaus::plateaus;
        use crate::v2::special::{spread, DENSE_MIN, SAMPLE};

        /// Up to this size: `sort_unstable` (measured where it crosses the radix
        /// path).
        const SMALL: usize = $small;

        /// The rollercoaster from this size (the L2 bytes; for its memory, n instead of 2n).
        fn roller_min() -> usize { budget(Budget::Roller32) }

        // The range, dense and few detectors of this type, in module `detectors`.
        int_detectors!(detectors, $t, u32, $c, $c, super::roller_min);

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
            chain! {
                "std_small" if n <= small_max(SMALL) => do { arr.sort_unstable() };
            }
            // Trial (mask bit 512): the light chain up to `light_max()` keys:
            // range and few on the spread sample, then the engine (no
            // plateaus, dominant, heavy, dense, near-sorted repair, Verge).
            // A sorted head of half the array (its tail first, then one
            // merge), or mostly equal spread pairs from 2000 keys (plateaus):
            // the whole path. The pairs are compared here when `presorted`
            // did not (it does only when the first key is the larger).
            let head = known.ascending_prefix >= n / 2;
            let light = !on(512) && n <= light_max() && !(on(1024) && head);
            // Trial (bit 16384 off): the tiny path up to `TINY_MAX` keys.
            // Bit 8388608: the tiny path up to 1100 keys.
            // From `TINY_END` to `TINY_MAX` keys, only when 18 spread keys show
            // at most 4 values (its count by comparisons: Binary, Few 4 1.3-1.6x
            // faster than the light chain; bit 268435456: tried, not kept: slower at 1000).
            let few_probed = || {
                let mut v = [arr[0]; 18];
                for i in 0..18 {
                    v[i] = arr[i * (n - 1) / 17];
                }
                (0..18).filter(|&i| !v[..i].contains(&v[i])).count() <= 4
            };
            if light
                && on(16384)
                && (n <= if on(8388608) { TINY_END } else { TINY_MAX }
                    || (n <= TINY_MAX && !on(268435456) && few_probed()))
            {
                tiny(arr, known);
                return;
            }
            let probes = if light {
                known.probes.unwrap_or_else(|| probe_pairs(arr))
            } else {
                Probes { ascending: 0, descending: 0 }
            };
            if light && !(on(1024) && n >= 2000 && probes.equal() >= 24) {
                // Nearly all pairs in one direction (30 of 32: saw teeth show
                // about 28): the near-sorted repair alone (no Verge: saw
                // teeth and runs cost 1.2-3x on the whole path).
                if on(1024) && (probes.ascending >= 30 || probes.descending >= 30) {
                    let known = Unsorted { probes: Some(probes), ..known };
                    if repair_nearly_sorted(arr, known).0 {
                        trace!("light_repaired");
                        return;
                    }
                }
                let sample = spread(arr);
                if on(8192) && few4(arr, &sample) {
                    trace!("light_few4");
                    return;
                }
                if on(1) && detectors::range(arr, &sample) {
                    trace!("light_range");
                    return;
                }
                // One dominant value with a long tail of distinct values
                // (Sparse), on the first 16 sample keys: the majority vote,
                // its count at least 8 and no other value 3 times or more
                // (FakeZipf repeats a second value: left to `few`). Then
                // the std partition (as `detect`). The full `dominant` on
                // the 64 keys cost up to 0.8 ns per key at 250 keys.
                if on(2048) {
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
                }
                // Below 512 keys `few` only when the first 16 sample keys
                // hold at most 6 values (Heavy 4 shows about 10: few sorts
                // its ~120 values at 300 keys, 1.5x slower than the DLSD; it
                // rejects many values itself, on
                // 16 keys there, but after zeroing its table: 3-4 % at
                // 250-300 keys; from 512 keys it accepts more values).
                let mut distinct = 0;
                for i in 0..16 {
                    distinct += !sample[..i].contains(&sample[i]) as usize;
                }
                if on(8) && (distinct <= 6 || n >= 512) && detectors::few(arr, &sample) {
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
                return;
            }
            let buffer = std::cell::RefCell::new(Vec::new());
            let sort_chunk = |chunk: &mut [$t]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
            path(arr, known, &detect, &sort_chunk);
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

        /// Up to this many keys the light chain asks its questions on 18
        /// probed keys (the 16-bit short sort's probes).
        const TINY_MAX: usize = 1100;

        /// The tiny path up to this many keys (Surface and Ryzen: it wins
        /// 150-500 keys, the light chain from 700: 25-35 cells).
        const TINY_END: usize = 600;

        /// Up to this many keys the tiny path ends with the short radix sort
        /// (Surface: it beats the DLSD on 22-28 cells at 170-300 keys, loses
        /// 26-30 from 500).
        const SHORT_MAX: usize = 400;

        /// Merges the sorted head `bits[..p]` and the sorted tail `bits[p..]`
        /// (at most half of at most `TINY_MAX` keys) from the end, the tail
        /// copied to the stack (`merge_runs` allocates its bounds and its
        /// buffer: Pipe organ at 150 keys 1.9x slower than blazing); stable.
        /// Nothing to do when the head's last key is not above the tail's
        /// first. Off (bit 536870912 on): only Pipe organ at 150-300 keys gains,
        /// `merge_runs` (which trims the runs first) wins on Front, Middle.
        fn merge_tail_small(bits: &mut [u32], p: usize) {
            let n = bits.len();
            let m = n - p;
            debug_assert!(m <= p && n <= TINY_MAX);
            if p == 0 || m == 0 || $c(bits[p - 1]) <= $c(bits[p]) {
                return;
            }
            let mut buf = [0u32; TINY_MAX / 2 + 1];
            buf[..m].copy_from_slice(&bits[p..]);
            let (mut i, mut j, mut k) = (p, m, n);
            // SAFETY: i <= p, j <= m, k = i + j <= n: every index below is in
            // bounds; the head is read at i - 1 < k - 1 before k - 1 is written.
            unsafe {
                // Without a branch per key (selects; the indices move by 0 or
                // 1): interleaved runs mispredicted one branch in two (Front
                // 4x slower).
                while j > 0 && i > 0 {
                    let (a, b) = (*bits.get_unchecked(i - 1), *buf.get_unchecked(j - 1));
                    k -= 1;
                    let head = ($c(a) > $c(b)) as usize;
                    *bits.get_unchecked_mut(k) = if head == 1 { a } else { b };
                    i -= head;
                    j -= 1 - head;
                }
            }
            bits[..j].copy_from_slice(&buf[..j]);
        }

        /// 18 probes spread over the array (bit 524288: the first 16, the
        /// middle and last ones; neighbours alike, Sqrt, passed for a
        /// dominant value).
        #[inline]
        fn probes18(arr: &[$t]) -> [$t; 18] {
            let n = arr.len();
            let mut s = [arr[0]; 18];
            if on(524288) {
                for i in 0..18 {
                    s[i] = arr[i * (n - 1) / 17];
                }
            } else {
                s[..16].copy_from_slice(&arr[..16]);
                s[16] = arr[n / 2];
                s[17] = arr[n - 1];
            }
            s
        }

        /// The tiny path's questions on its probes: one dominant value
        /// (sorted by `sort_unstable`), at most 4 values (counted by
        /// comparisons). Whether `arr` is now sorted.
        #[inline]
        fn probe_questions(arr: &mut [$t], s: &[$t; 18]) -> bool {
            // One dominant value with a long tail (Sparse): the vote, at
            // least 7 of 18, no other value 3 times; before few4, which
            // Sparse's 18 probes pass (its count pass then fails).
            let (mut v, mut votes) = (s[0], 0usize);
            for &x in s.iter() {
                if votes == 0 {
                    v = x;
                }
                votes = if x == v { votes + 1 } else { votes - 1 };
            }
            let count = |u: $t| s.iter().filter(|&&x| x == u).count();
            if count(v) >= 7 && s.iter().all(|&u| u == v || count(u) < 3) {
                trace!("tiny_dominant");
                arr.sort_unstable();
                return true;
            }
            if few4(arr, s) {
                trace!("tiny_few4");
                return true;
            }
            false
        }

        /// Most values `count_small` counts.
        const COUNT_SMALL_MAX: usize = 256;

        /// Counting sort of raw bits whose encoded keys lie in
        /// `lo..lo + span` (`span` at most `COUNT_SMALL_MAX`), counters on
        /// the stack: 4 tables when keys outnumber values (repeated keys do
        /// not wait on one counter), the values written back without a
        /// branch per counter (two writes each, the next value overwriting
        /// them; a loop only beyond 2 keys): counters of 0, 1 or 2 keys in
        /// random order mispredicted one branch each.
        #[inline]
        fn count_small(bits: &mut [u32], lo: u32, span: usize) {
            const M: usize = COUNT_SMALL_MAX;
            debug_assert!(span <= M);
            let n = bits.len();
            let mut c = [[0u32; M]; 4];
            let key = |b: u32| $c(b).wrapping_sub(lo) as usize & (M - 1);
            if n > span {
                let (quads, rest) = bits.as_chunks::<4>();
                for q in quads {
                    for j in 0..4 {
                        // SAFETY: masked below M.
                        unsafe { *c.get_unchecked_mut(j).get_unchecked_mut(key(q[j])) += 1 };
                    }
                }
                for &b in rest {
                    unsafe { *c[0].get_unchecked_mut(key(b)) += 1 };
                }
                for d in 0..span {
                    c[0][d] += c[1][d] + c[2][d] + c[3][d];
                }
            } else {
                for &b in bits.iter() {
                    unsafe { *c[0].get_unchecked_mut(key(b)) += 1 };
                }
            }
            let p = bits.as_mut_ptr();
            let mut at = 0;
            for d in 0..span {
                let k = c[0][d] as usize;
                let v = $c(lo.wrapping_add(d as u32));
                // SAFETY: the counts add up to n; the two writes stay below n.
                unsafe {
                    if at + 2 <= n {
                        *p.add(at) = v;
                        *p.add(at + 1) = v;
                        for j in 2..k {
                            *p.add(at + j) = v;
                        }
                    } else {
                        for j in 0..k {
                            *p.add(at + j) = v;
                        }
                    }
                }
                at += k;
            }
            debug_assert_eq!(at, n);
        }

        /// The short radix sort's digits for `n` keys spanning `range`:
        /// (one pass, prefix bits, bits below the prefix). ARM: one pass of
        /// log2(n) + 1 bits (at most 10), the insertion finish settling the
        /// collisions (Surface: 20-26 cells won at 150-400 keys); x86: two
        /// passes of log2(n) + 3 bits (Ryzen: one pass loses 19-28). Bit
        /// 65536 swaps the choice (A/B).
        #[inline(always)]
        fn short_layout(n: usize, range: u32) -> (bool, u32, u32) {
            let top = 32 - range.leading_zeros();
            let log_n = usize::BITS - n.leading_zeros();
            let one_here = cfg!(target_arch = "aarch64");
            let one = (one_here != !on(65536)) && log_n + 1 <= 10;
            let p = if one { (log_n + 1).min(top) } else { (log_n + 3).min(top) };
            (one, p, top - p)
        }

        /// Whether the 18 probes fall in at most 12 of 64 equal slices of the
        /// range, with more distinct values than that (spread keys fill about
        /// 16; the short radix splits the range
        /// evenly, clustered keys leave most buckets empty and the insertion
        /// finish long: Bimodal, Exponential, Clusters).
        #[inline]
        fn clustered(mut s: [$t; 18], lo: u32, hi: u32, n: usize) -> bool {
            let range = hi - lo;
            let shift = (32 - range.leading_zeros()).saturating_sub(6);
            let b: &[u32] = bits_of(&mut s);
            let mut seen = 0u64;
            for &x in b.iter() {
                seen |= 1u64 << (($c(x).wrapping_sub(lo) >> shift) & 63);
            }
            let _ = n;
            // Distinct probes well above the slices they fill: clusters (few
            // values fill few slices too, but each its own: Few 16, Heavy 4
            // went to the DLSD, 1.3-1.4x slower at 150 keys on the Ryzen).
            let slices = seen.count_ones() as usize;
            let mut distinct = 0;
            for i in 0..18 {
                distinct += !b[..i].contains(&b[i]) as usize;
            }
            slices <= 12 && distinct >= slices + 4
        }

        /// The short radix sort of the tiny path (up to `TINY_MAX` keys),
        /// on raw bits whose encoded minimum and maximum are known: the top
        /// log2(n) + 3 bits of `key - min` (at most 14) by 2 LSD passes of
        /// at most 7 bits, both histograms in one read, buffer and counters
        /// on the stack; then an insertion sort finishes the keys sharing a
        /// prefix (`sort_unstable` beyond 2n moves: clustered keys). No
        /// sample, no allocation (the 16-bit short LSD's approach).
        fn short_radix(bits: &mut [u32], lo: u32, hi: u32) {
            let n = bits.len();
            debug_assert!(n <= TINY_MAX);
            let range = hi - lo;
            if range == 0 {
                return;
            }
            let (one, p, base) = short_layout(n, range);
            // At most 8 bits: one pass (a counting sort that moves the keys:
            // a range of 256 values at 150 keys, where writing the values
            // back from 256 counters costs 2.5x).
            if p <= 10 && (one || p <= 8) {
                let key = |b: u32| ($c(b).wrapping_sub(lo) >> base) as usize;
                let mut c = [0u32; 1024];
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
                let mut store = [0u32; TINY_MAX];
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
                // SAFETY: k < 2^p, p <= 14, so both digits are below 128.
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
            let mut store = [0u32; TINY_MAX];
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

        /// The tiny path. One inspection pass gives the bits that vary and
        /// the descents: nearly sorted (few descents, or nearly
        /// all) tries the near-sorted repair, a narrow range is counted
        /// directly; the other questions on 18 probed keys (the first 16,
        /// the middle one, the last one); else the DLSD, given the minimum
        /// and maximum (no pass of its own for them).
        fn tiny(arr: &mut [$t], known: Unsorted) {
            let n = arr.len();
            let s = probes18(arr);
            // The questions on the probes first (bit 1073741824: after the
            // inspection pass, which cost Binary, Few 4 1.3-1.5x).
            let early = on(1073741824);
            if early && probe_questions(arr, &s) {
                return;
            }
            // One inspection pass: the descents, and the distances to the
            // first key ORed apart above and below it (no min / max: u32
            // min / max have no SSE2 instruction and, emulated, made the pass
            // 20 % of the sort at 1000 keys on the Ryzen; an OR of the keys
            // themselves missed the narrow ranges across 0x8000_0000, i32
            // keys around 0).
            let (up, down, var, desc, e0) = {
                let bits: &[u32] = bits_of(arr);
                let e0 = $c(bits[0]);
                let (up, down, var, desc) = bits.iter().zip(&bits[1..]).fold(
                    (0u32, 0u32, 0u32, 0u32),
                    |(u, dn, v, d), (&a, &b)| {
                        let (ea, eb) = ($c(a), $c(b));
                        let x = eb.wrapping_sub(e0);
                        let below = ((x as i32) >> 31) as u32;
                        (u | (x & !below), dn | (!x & below), v | (eb ^ e0), d + (ea > eb) as u32)
                    },
                );
                (up, down, var, desc as usize, e0)
            };
            // Two bounds of the keys. By the distances to the first key,
            // [e0 - 2^tn, e0 + 2^tp), valid only when it stays in the domain
            // (else a key more than 2^31 above would pass for one below:
            // Bimodal); i32 keys around 0 need it. Else by the bits that
            // vary: the first key's bits above them, always valid.
            let tp = 32 - up.leading_zeros();
            let tn = 32 - down.leading_zeros();
            let tv = 32 - var.leading_zeros();
            let by_distance = if tp < 31 && tn < 31 {
                match (e0.checked_sub(1u32 << tn), e0.checked_add((1u32 << tp) - 1)) {
                    (Some(l), Some(h)) => Some((l, h)),
                    _ => None,
                }
            } else {
                None
            };
            let by_bits = if tv == 32 {
                (0, u32::MAX)
            } else {
                let low = (1u32 << tv) - 1;
                (e0 & !low, e0 | low)
            };
            let (lo, hi) = match by_distance {
                Some((l, h)) if h - l < by_bits.1 - by_bits.0 => (l, h),
                _ => by_bits,
            };
            let t = if hi - lo == u32::MAX { 32 } else { 0 };
            if desc <= 32 || desc + 33 >= n {
                let probes = known.probes.unwrap_or_else(|| probe_pairs(arr));
                if (probes.ascending >= 30 || probes.descending >= 30)
                    && repair_nearly_sorted(arr, Unsorted { probes: Some(probes), ..known }).0
                {
                    trace!("tiny_repaired");
                    return;
                }
            }
            if !early && probe_questions(arr, &s) {
                return;
            }
            // A range of at most one counter per 2 keys, or of 256 values
            // (Small1 at 300 keys): counted. After few4 (Binary: the
            // comparisons beat the count, 0.30 against 0.62 ns per key).
            // A bound already narrow: the exact minimum and maximum (one
            // more pass, only there; the bounds above are ORs, up to twice
            // the true span: i32 Small1 around 0 showed 512 values for 256).
            // Bounds already within one counter per 2 keys: counted on them,
            // no exact min / max pass (bit 67108864: that pass first; Int
            // 0..100, Saw 1.2-1.4x slower than the light chain's range).
            if on(67108864) && t < 32 && ((hi - lo) as usize) < n / 2 {
                trace!("tiny_count");
                exact_count(bits_of(arr), lo, (hi - lo) as usize + 1, $c, $c);
                return;
            }
            let (lo, hi) = if t < 32 && hi - lo <= 65_535 {
                let bits: &[u32] = bits_of(arr);
                bits.iter().fold((u32::MAX, 0u32), |(l, h), &b| (l.min($c(b)), h.max($c(b))))
            } else {
                (lo, hi)
            };
            let span = (hi - lo) as usize + 1;
            // At most 256 values: one pass of the short radix sort (moves the
            // keys); else up to one counter per 2 keys: counted, the values
            // written back from the counts.
            // Bit 131072: the old order (the short radix first up to 256
            // values: 1.5-2.2x slower at 500-1000 keys, Int 0..100, RootDup,
            // Alt16, Saw, on the Surface).
            if on(131072) && t < 32 && span <= n / 2 {
                trace!("tiny_count");
                exact_count(bits_of(arr), lo, span, $c, $c);
                return;
            }
            // Bit 134217728: up to 256 values counted (`count_small`)
            // instead of one pass of the short radix.
            if !on(134217728) && t < 32 && span <= COUNT_SMALL_MAX {
                trace!("tiny_count");
                count_small(bits_of(arr), lo, span);
                return;
            }
            if t < 32 && span <= 256 {
                trace!("tiny_short1");
                short_radix(bits_of(arr), lo, hi);
                return;
            }
            if t < 32 && span <= n / 2 {
                trace!("tiny_count");
                exact_count(bits_of(arr), lo, span, $c, $c);
                return;
            }
            let mut distinct = 0;
            for i in 0..16 {
                distinct += !s[..i].contains(&s[i]) as usize;
            }
            if distinct <= 6 || n >= 512 {
                // A real spread sample (bit 33554432: the 18 probes repeated,
                // which never trip `few`'s early exit on a half-distinct
                // sample: its full pass then failed late on Heavy 4 and Zipf,
                // 1.7-2x slower at 1000 keys).
                let mut sample = [s[0]; SAMPLE];
                if on(33554432) {
                    for i in 0..SAMPLE {
                        sample[i] = arr[i * (n - 1) / (SAMPLE - 1)];
                    }
                } else {
                    for i in 0..SAMPLE {
                        sample[i] = s[i % 18];
                    }
                }
                if detectors::few(arr, &sample) {
                    trace!("tiny_few");
                    return;
                }
            }
            // The short radix sort up to `SHORT_MAX` keys (bit 32768: never),
            // else the DLSD (from 500 keys its digits beat 2 passes of
            // 6-7 bits: 5-9 % on random keys).
            // Clustered keys (Bimodal, Exponential, Clusters): the DLSD,
            // whose prefix comes from a sample (bit 262144: no such test).
            // ARM: never for a wide range (Surface: the DLSD 25-35 % faster
            // on Unif from 200 keys; bit 1048576 swaps).
            let wide = cfg!(target_arch = "aarch64") == !on(1048576);
            if wide && on(32768) && n <= SHORT_MAX && !(on(262144) && clustered(s, lo, hi, n)) {
                trace!("tiny_short");
                short_radix(bits_of(arr), lo, hi);
            } else {
                trace!("tiny_dlsd");
                // The inspection's bounds hold every key (sometimes twice
                // the span: one more prefix bit at most): no min / max pass
                // of the DLSD (bit 4194304: its own pass).
                if on(4194304) {
                    $st::dlsd_known(bits_of(arr), lo, hi);
                } else {
                    $st::dlsd(bits_of(arr));
                }
            }
        }

        /// The detectors, on one spread sample (sorted once, only when the detectors
        /// that need it in order are reached). `equal_pairs`: equal pairs among the
        /// probes of the near-sorted repair.
        fn detect(arr: &mut [$t], equal_pairs: usize) -> bool {
            if !on(64) {
                return false;
            }
            let mut sample = spread(arr);
            chain! {
                return true;
                "few4" if on(8192) => try few4(arr, &sample);
                "range" if on(1) && !EARLY.with(|e| e.get()) => try detectors::range(arr, &sample);
                "plateaus" if on(2) => try plateaus(arr, equal_pairs);
                // One dominant value among spread ones, short array: the std
                // partition settles the keys equal to its pivot in one pass.
                "dominant" if on(4) && arr.len() < DENSE_MIN
                    && dominant(&sample, |x, y| x == y)
                    => do { arr.sort_unstable() };
                "few" if on(8) && !EARLY.with(|e| e.get()) => try detectors::few(arr, &sample);
            }
            // Heavy keys and dense ranges: the sample in order.
            if arr.len() < HEAVY_MIN || (arr.len() < DENSE_MIN && !may_be_heavy(&sample, |x, y| x == y)) {
                return false;
            }
            sample.sort_unstable();
            chain! {
                return true;
                "heavy" if on(16) => try heavy_sorted_by(
                    arr,
                    &sample,
                    HEAVY_MIN,
                    &|r: &mut [$t]| sort(r),
                    |x, y| x == y,
                    |x, y| x < y,
                );
                "dense" if on(32) && arr.len() >= DENSE_MIN => try detectors::dense(arr, &sample);
            }
            false
        }

        /// The engine of one chunk (the whole array, or a gap between Verge's runs).
        #[inline(always)]
        fn sort_chunk(bits: &mut [u32], _: &mut Vec<MaybeUninit<u32>>) {
            // The rollercoaster beyond the cache only on ordered keys (the MSD pass
            // then writes in few sequential streams; random keys go 15 to 45 %
            // faster through the LSD from 30M to 1G).
            if bits.len() < dlsd_max() {
                trace!("dlsd");
                if dlsd_heap() {
                    $dl::dlsd(bits);
                } else {
                    $st::dlsd(bits);
                }
            } else if bits.len() >= roller_min() && (locally_ordered(bits) || globally_ordered(bits)) {
                trace!("roller");
                $ro::roller(bits);
            } else {
                trace!("lsd");
                $ls::lsd(bits);
            }
        }

        /// The radix path (copy of `decide::path`); the first matching
        /// rule wins.
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
                    if !on(536870912) && n <= TINY_MAX {
                        merge_tail_small(bits_of(arr), p);
                    } else {
                        merge_runs(arr, &mut vec![0, p, n]);
                    }
                };
            }
            // Trial (mask bit 4096): range and few on the sample before the
            // near-sorted repair when the spread pairs show no order (Binary:
            // the repair costs 7-14 %); `detect` then skips them.
            let early = !on(4096)
                && !known.probes.is_some_and(|p| p.ascending >= 28 || p.descending >= 28);
            if early {
                let sample = spread(arr);
                if on(1) && detectors::range(arr, &sample) {
                    trace!("early_range");
                    return;
                }
                if on(8) && detectors::few(arr, &sample) {
                    trace!("early_few");
                    return;
                }
            }
            EARLY.with(|e| e.set(early));
            let simd_rule = merge::simd_rule_applies::<$t>(n);
            let random_runs = !simd_rule && starts_with_random_runs(arr, p);
            let (sorted, equal_pairs) = if random_runs || !on(256) {
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
                "no_verge" if !on(128) || random_runs || can_skip_verge(arr) => do { sort(arr) };
            }
            // Verge: the unsorted chunks are recorded, sorted after the merge rule.
            let base = arr.as_ptr() as usize;
            let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
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

chain32_ab!(cu32, u32, id32, 200, wu32, du32, su32, ru32);
chain32_ab!(ci32, i32, flip32, 150, wi32, di32, si32, ri32);
