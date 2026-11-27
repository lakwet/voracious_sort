//! u16 and i16: one decision chain for both. The keys are handled as their 16 bits;
//! `F` is xored onto them wherever their order matters (comparisons, the
//! digits counted, the values written from a digit): 0 for u16, the sign
//! bit for i16 (its order is that of the bits with the sign bit flipped).
//! Equalities and moves of whole keys need nothing.

macro_rules! narrow16 {
    ($m:ident, $k:ty, $f:expr, $enc:ident) => {
pub(crate) mod $m {
use std::mem::MaybeUninit;

use crate::v2::decide::path::{merges_unpredictable, starts_with_random_runs};
use crate::v2::engines::small::{merge_sort, network_sort, MERGE_MAX, NETWORK_MAX};
use crate::v2::inspect::presorted::{presorted, repair_nearly_sorted, Unsorted};
use crate::v2::runs::merge::{self, merge_runs};
use crate::v2::runs::verge::verge_sort_preprocessing_fast;
use crate::v2::devour::keys::*;
use crate::v2::devour::DevourSort;
use crate::v2::special::heavy::{dominant, heavy_sorted_by, may_be_heavy, HEAVY_MIN};
use crate::v2::special::{spread, SAMPLE};

/// The key type (the slices sorted are its bits).
type K = $k;
/// Xored onto the bits for their order.
const F: u16 = $f;

/// The order key of the bits `x`.
#[inline(always)]
fn ord(x: u16) -> u16 { x ^ F }

/// The bits as keys of type `K` (for the shared code that compares them).
#[inline(always)]
fn keys_of(a: &mut [u16]) -> &mut [K] {
    // SAFETY: K is u16 or i16, same size and alignment, every pattern valid.
    unsafe { std::slice::from_raw_parts_mut(a.as_mut_ptr() as *mut K, a.len()) }
}

// The range and few detectors of this type, in module `detectors`.
int_detectors!(detectors, $k, u16, $enc, $enc, super::roller_min, partial);

/// The rollercoaster from this size (never: 8 and 16 bits).
fn roller_min() -> usize { usize::MAX }

/// Sorts the bits `arr` of keys `K`: a sorting network up to `NETWORK_MAX`
/// keys, blocks and a merge up to `MERGE_MAX`, the short sort up to
/// `SHORT_SORT_MAX`, the light chain up to `LIGHT_MAX`, else the chain.
pub(crate) fn sort(arr: &mut [u16]) {
    let n = arr.len();
    if n <= NETWORK_MAX {
        trace!("network");
        network_sort(arr, $enc, $enc);
    } else if n <= MERGE_MAX {
        trace!("merge_small");
        merge_sort(arr, $enc, $enc);
    } else if n <= SHORT_SORT_MAX {
        short_sort(arr);
    } else if n <= LIGHT_MAX {
        light_chain(arr);
    } else {
        chain2(arr);
    }
}

/// Up to this size the short sort (its LSD's buffer on the stack).
const SHORT_SORT_MAX: usize = crate::v2::engines::lsd::short16::STACK_MAX;

/// Up to this size the light chain: against blazing-radix from 5000 to
/// 65 000 keys it loses 20-26 cells of 310 where the chain loses 31-48
/// (Zen 2, Haswell, Surface); beyond, the domain count of the chain (one
/// key per counter) wins Clusters by 1.7-2.2 times.
const LIGHT_MAX: usize = crate::v2::engines::lsd::short16::SHORT_MAX;

/// Short arrays (`MERGE_MAX` to `SHORT_SORT_MAX` keys): the chain's
/// questions, those that mostly fail made cheap or dropped (ideas:
/// blazing-radix `prepare`). One order scan (`run_win`); below `TINY_MAX`
/// probes on 18 keys, beyond them the sample's detectors; then the LSD,
/// through the path only after a sorted head of half the array (Pipe
/// organ): no near-sorted repair, no Verge. Against blazing-radix from 300
/// to 5500 keys: 33 cells lost of 279 where the chain lost 93 (Zen 2),
/// 62 where it lost 79 (Surface).
fn short_sort(arr: &mut [u16]) {
    let n = arr.len();
    // All keys equal: one OR of differences, read only when the first, the
    // middle and the last key agree (Zen 2: Equal 2.2 times faster than the
    // order scan at 5000 keys).
    let first = arr[0];
    if first == arr[n - 1] && first == arr[n / 2] && arr.iter().fold(0, |d, &v| d | (v ^ first)) == 0 {
        trace!("presorted");
        return;
    }
    // One scan: the ascending head (all of it: sorted, or all equal), else
    // descending all through when the first key is above the last (equal
    // keys may open a descending array).
    let p = run_win::<false>(arr);
    if p == n {
        trace!("presorted");
        return;
    }
    if ord(arr[0]) > ord(arr[n - 1]) && run_win::<true>(arr) == n {
        trace!("presorted");
        arr.reverse();
        return;
    }
    let done = if n >= TINY_MAX {
        light_detectors(arr)
    } else if cfg!(target_arch = "aarch64") {
        short_probe(arr)
    } else {
        cheap_probe(arr)
    };
    if done {
        return;
    }
    if tail_first(n, p) {
        let known = Unsorted { ascending_prefix: p, probes: None };
        path(arr, known, &|_: &mut [u16], _: usize| false, &|c: &mut [u16]| light_lsd(c));
    } else {
        light_lsd(arr);
    }
}

/// From `SHORT_SORT_MAX` to `LIGHT_MAX` keys: the short sort's detectors
/// and engine, with the chain's order checks (all keys equal; on x86 the
/// shared scan, faster there on long sorted runs) and its whole path
/// (near-sorted repair, runs, Verge).
fn light_chain(arr: &mut [u16]) {
    let n = arr.len();
    let (first, last) = (arr[0], arr[n - 1]);
    if first == last && first == arr[n / 2] && arr.iter().fold(0, |d, &v| d | (v ^ first)) == 0 {
        trace!("presorted");
        return;
    }
    let mut known = if cfg!(target_arch = "aarch64") {
        let p = run_win::<false>(arr);
        if p == n {
            trace!("presorted");
            return;
        }
        if ord(first) > ord(last) && run_win::<true>(arr) == n {
            trace!("presorted");
            arr.reverse();
            return;
        }
        Unsorted { ascending_prefix: p, probes: None }
    } else {
        match presorted(keys_of(arr)) {
            Ok(()) => {
                trace!("presorted");
                return;
            },
            Err(known) => known,
        }
    };
    if known.ascending_prefix == 0 {
        known.ascending_prefix = scan16::<false>(arr);
    }
    if light_detectors(arr) {
        return;
    }
    path(arr, known, &|_: &mut [u16], _: usize| false, &|c: &mut [u16]| light_lsd(c));
}

/// The engine of the short sort and the light chain: on ARM the chunk LSD
/// (16-bit counters from 600 keys), on x86 the short LSD.
#[inline(always)]
fn light_lsd(arr: &mut [u16]) {
    if cfg!(target_arch = "aarch64") {
        sort_chunk(arr, &mut Vec::new())
    } else {
        trace!("lsd");
        crate::v2::engines::lsd::short16::lsd_short::<F>(arr)
    }
}

/// Length of the ascending run from the start (`DESC`: descending), by
/// blocks of 16 pairs from two overlapping reads, no branch inside a block
/// (Zen 2: Sorted, Saws 20-35 % faster than blazing-radix from 1000 keys),
/// the exact end searched in the failing block.
#[inline(always)]
fn run_win<const DESC: bool>(a: &[u16]) -> usize {
    let n = a.len();
    let bad = |x: u16, y: u16| if DESC { ord(x) < ord(y) } else { ord(x) > ord(y) };
    let mut i = 0;
    while i + 17 <= n {
        let x = a[i..].first_chunk::<16>().unwrap();
        let y = a[i + 1..].first_chunk::<16>().unwrap();
        let mut any = false;
        for j in 0..16 {
            any |= bad(x[j], y[j]);
        }
        if any {
            break;
        }
        i += 16;
    }
    while i + 1 < n && !bad(a[i], a[i + 1]) {
        i += 1;
    }
    i + 1
}

/// The detectors of the short sort from `TINY_MAX` keys and of the light
/// chain, on the spread sample: sorts and returns `true` when one fires.
/// 8 spread picks counted over the sample (one vectorized compare each)
/// replace the chain's branchy questions: a repeated pick rules out random
/// keys, three picks seen once rule out few values, and a majority value
/// is among them (Zen 2 at 1500 keys: Zipf, Clusters, Heavy 4 0.2 ns per
/// key faster).
fn light_detectors(arr: &mut [u16]) -> bool {
    let sample = spread(arr);
    if few_compare(arr, &sample) {
        trace!("few_cmp");
        return true;
    }
    if window256(arr, &sample) {
        trace!("window256");
        return true;
    }
    if sample.iter().all(|&v| v & 255 == sample[0] & 255) && flat_byte16::<u32, true>(arr) {
        trace!("flat_byte");
        return true;
    }
    let picks: [u16; 8] = std::array::from_fn(|k| sample[k * (SAMPLE / 8)]);
    let counts = picks.map(|u| sample.iter().map(|&x| (x == u) as u16).sum::<u16>() as usize);
    let singles = counts.iter().filter(|&&c| c == 1).count();
    if singles == 8 && looks_random(arr, &sample) {
        trace!("random_sample");
        light_lsd(arr);
        return true;
    }
    if detectors::range(keys_of(arr), &sample.map(|v| v as K)) {
        trace!("range");
        return true;
    }
    // One value holding half of the sample, the others under an eighth.
    let heavy = match (0..8).find(|&k| counts[k] >= SAMPLE / 2) {
        Some(k) => (0..8).all(|j| picks[j] == picks[k] || counts[j] < SAMPLE / 8),
        None => false,
    };
    if heavy {
        let mut sorted = sample;
        merge_sort(&mut sorted[..], $enc, $enc);
        if heavy_sorted_by(
            arr,
            &sorted,
            TINY_MAX,
            &|r: &mut [u16]| sort(r),
            |x, y| x == y,
            |x, y| ord(x) < ord(y),
        ) {
            trace!("heavy_first");
            return true;
        }
    }
    if singles < 3 && few_light(arr, &sample) {
        trace!("few");
        return true;
    }
    false
}

/// `few16` on the spread sample alone, its estimate without a branch:
/// three maps of 2048 hashed bits (values seen once, twice, three times)
/// give seen, f1 and f2 for the Chao1 estimate (the hash table's probes
/// mispredicted on Heavy 4: 0.45 ns per key at 1500). Few values only when
/// the sample's keys are mostly repeated (few-valued inputs: at most 5
/// seen once in 64; Heavy 4, 4 heavy values and random keys: 17-29, where
/// the estimate alone let some arrays count and fail).
#[inline(never)]
fn few_light(values: &mut [u16], sample: &[u16; SAMPLE]) -> bool {
    use crate::v2::special::{FEW_MAX, FEW_TABLE};
    let mut m = [[0u64; 32]; 3];
    for &v in sample.iter() {
        let h = ((v as u32).wrapping_mul(0x9E37_79B1) >> 21) as usize;
        let (w, b) = (h >> 6, 1u64 << (h & 63));
        let (s1, s2) = (m[0][w] & b, m[1][w] & b);
        m[2][w] |= s2;
        m[1][w] |= s1;
        m[0][w] |= b;
    }
    let (mut distinct, mut f1, mut f2) = (0usize, 0usize, 0usize);
    for w in 0..32 {
        distinct += m[0][w].count_ones() as usize;
        f1 += (m[0][w] & !m[1][w]).count_ones() as usize;
        f2 += (m[1][w] & !m[2][w]).count_ones() as usize;
    }
    if f1 > SAMPLE / 8
        || distinct > SAMPLE / 2
        || distinct + f1 * f1.saturating_sub(1) / (2 * (f2 + 1)) > FEW_MAX
    {
        trace!("few_estimate");
        return false;
    }
    detectors::few_count::<FEW_TABLE>(keys_of(values))
}

/// The 18 probed keys: the first 16, the middle one, the last one.
#[inline(always)]
fn probe18(a: &[u16]) -> [u16; 18] {
    let n = a.len();
    let mut s = [0u16; 18];
    s[..16].copy_from_slice(&a[..16]);
    s[16] = a[n / 2];
    s[17] = a[n - 1];
    s
}

/// The short sort's probes below `TINY_MAX` keys on ARM (`tiny_probe`'s,
/// read without a branch per key; Surface at 100 keys: 44 to 21 ns): on the
/// 18 probed keys, the range in order and rotated, the bits that vary; each
/// key's count among them only when the first five repeat one (random
/// keys: nearly never); then at most 4 values counted, a range of 256
/// values, the high byte alone, one value holding 7 of the 18.
fn short_probe(arr: &mut [u16]) -> bool {
    let s = probe18(arr);
    let mut same = false;
    for i in 0..5 {
        for j in i + 1..5 {
            same |= s[i] == s[j];
        }
    }
    let counts = if same { s.map(|u| s.iter().map(|&x| (x == u) as u8).sum::<u8>()) } else { [1; 18] };
    // Distinct values: each adds 1/c over its c keys.
    let distinct: f32 = if same { counts.iter().map(|&c| 1.0 / c as f32).sum() } else { 18.0 };
    if distinct < 4.5 {
        let mut reps = [s[0]; 4];
        let mut k = 1;
        for &v in &s[1..] {
            if !reps[..k].contains(&v) {
                reps[k] = v;
                k += 1;
            }
        }
        reps[..k].sort_unstable_by_key(|&v| ord(v));
        let done = match k {
            1 => few_compare_k::<1>(arr, &reps),
            2 => few_compare_k::<2>(arr, &reps),
            3 => few_compare_k::<3>(arr, &reps),
            _ => few_compare_k::<4>(arr, &reps),
        };
        if done {
            trace!("tiny_few");
            return true;
        }
    }
    const R: u16 = 1 << 15;
    let (mut lo, mut hi, mut rlo, mut rhi, mut varies) = (u16::MAX, 0u16, u16::MAX, 0u16, 0u16);
    for &v in &s {
        lo = lo.min(ord(v));
        hi = hi.max(ord(v));
        rlo = rlo.min(ord(v) ^ R);
        rhi = rhi.max(ord(v) ^ R);
        varies |= v ^ s[0];
    }
    if hi - lo <= 255 && tiny_range::<0, u16>(arr) {
        trace!("tiny_range");
        return true;
    }
    if rhi - rlo <= 255 && tiny_range::<R, u16>(arr) {
        trace!("tiny_range_rot");
        return true;
    }
    if varies & 0xFF == 0 && tiny_high(arr) {
        trace!("tiny_high");
        return true;
    }
    if counts.iter().any(|&c| c >= 7) && tiny_dominant(arr) {
        trace!("tiny_dominant");
        return true;
    }
    false
}

/// The short sort's probes below `TINY_MAX` keys on x86 (blazing-radix
/// `prepare`'s questions, each giving up at once; Zen 2 at 300 keys, Unif
/// 6 % faster than `tiny_probe`): on the 18 probed keys, at most 4 values,
/// one byte varying, a range of 256 values (straight or across the middle
/// of the order); each checked on every key before anything is written.
fn cheap_probe(a: &mut [u16]) -> bool {
    let s = probe18(a);
    cheap_few(a, &s) || cheap_one_byte(a, &s) || cheap_range::<0>(a, &s) || cheap_range::<0x8000>(a, &s)
}

/// At most 4 values among the probed keys: each key compared with them,
/// nothing written unless every key is one of them.
fn cheap_few(a: &mut [u16], s: &[u16; 18]) -> bool {
    let mut reps = [s[0]; 4];
    let mut k = 1;
    for &v in &s[1..] {
        if !reps[..k].contains(&v) {
            if k == 4 {
                return false;
            }
            reps[k] = v;
            k += 1;
        }
    }
    reps[..k].sort_unstable_by_key(|&v| ord(v));
    let done = match k {
        1 => cheap_few_k::<1>(a, &reps),
        2 => cheap_few_k::<2>(a, &reps),
        3 => cheap_few_k::<3>(a, &reps),
        _ => cheap_few_k::<4>(a, &reps),
    };
    if done {
        trace!("cheap_few");
    }
    done
}

fn cheap_few_k<const KN: usize>(a: &mut [u16], reps: &[u16; 4]) -> bool {
    let mut c = [0usize; KN];
    for &x in a.iter() {
        for j in 0..KN {
            c[j] += (x == reps[j]) as usize;
        }
    }
    if c.iter().sum::<usize>() != a.len() {
        return false;
    }
    let mut at = 0;
    for j in 0..KN {
        a[at..at + c[j]].fill(reps[j]);
        at += c[j];
    }
    true
}

/// From this size the counts of the probes on four tables (keys in turn: a
/// repeated value does not wait on its own counter); below, one.
const LANES_MIN: usize = 1024;

/// Up to 4 counting tables, one per lane (keys in turn). Built for recent
/// Intel CPUs (`target-cpu=native` on a Core Ultra: `meteorlake`), the
/// compiler loaded the 4 counters of a block by one vector gather, slow
/// there: Bimodal, Range 16, Aligned, Small up to 2x slower than the generic
/// build. There each counter is read by a volatile load, which it cannot
/// gather; the lanes stay separate arrays, so their updates still overlap.
struct Lanes<const L: usize> {
    t: [[u32; 256]; L],
}

impl<const L: usize> Lanes<L> {
    #[inline(always)]
    fn new() -> Self { Lanes { t: [[0; 256]; L] } }

    /// One more key of digit `d` (below 256) in lane `j` (below `L`).
    #[inline(always)]
    fn add(&mut self, j: usize, d: usize) {
        let c = &mut self.t[j][d & 255];
        // Built for recent Intel CPUs (AVX-VNNI: Alder Lake, Meteor Lake;
        // AVX-512: their server parts), whose tuning makes the compiler
        // gather the counters: the counter read by a volatile load, which
        // it cannot gather. Elsewhere the plain increment (the volatile
        // load costs Aligned 30-40 % on the Ryzen, generic build).
        #[cfg(any(target_feature = "avxvnni", target_feature = "avx512f"))]
        {
            // SAFETY: `c` is a valid, aligned, exclusive reference.
            let v = unsafe { std::ptr::read_volatile(c) };
            *c = v + 1;
        }
        #[cfg(not(any(target_feature = "avxvnni", target_feature = "avx512f")))]
        {
            *c += 1;
        }
    }

    /// The lanes summed.
    #[inline(always)]
    fn sum(&self) -> [u32; 256] {
        // One pass over the counters, the lanes added at each (as before).
        let mut c = [0u32; 256];
        for d in 0..256 {
            let mut v = 0;
            for j in 0..L {
                v += self.t[j][d];
            }
            c[d] = v;
        }
        c
    }
}

/// The counts of `digit` over `a`, on `L` tables (keys in turn).
#[inline(always)]
fn count_lanes<const L: usize>(a: &[u16], digit: impl Fn(u16) -> usize) -> [u32; 256] {
    let mut cs = Lanes::<L>::new();
    let (blocks, rest) = a.as_chunks::<L>();
    for q in blocks {
        for j in 0..L {
            cs.add(j, digit(q[j]) & 255);
        }
    }
    for &x in rest {
        cs.add(0, digit(x) & 255);
    }
    cs.sum()
}

/// One byte varies among the probed keys: counted while every other bit is
/// checked in the same read; each count is one value.
fn cheap_one_byte(a: &mut [u16], s: &[u16; 18]) -> bool {
    let first = a[0];
    let vary = s.iter().fold(0, |m, &x| m | (x ^ first));
    if vary == 0 || (vary & 0xFF != 0 && vary >> 8 != 0) {
        return false;
    }
    let done = match (vary >> 8 == 0, a.len() < LANES_MIN) {
        (true, true) => one_byte_in::<0, 1>(a),
        (true, false) => one_byte_in::<0, 4>(a),
        (false, true) => one_byte_in::<8, 1>(a),
        (false, false) => one_byte_in::<8, 4>(a),
    };
    if done {
        trace!("cheap_one_byte");
    }
    done
}

fn one_byte_in<const S: u32, const L: usize>(a: &mut [u16]) -> bool {
    let first = a[0];
    let other = !(255u16 << S);
    let mut cs = Lanes::<L>::new();
    let mut bad = [0u16; L];
    let (blocks, rest) = a.as_chunks::<L>();
    for q in blocks {
        for j in 0..L {
            bad[j] |= (q[j] ^ first) & other;
            cs.add(j, ((ord(q[j]) >> S) & 255) as usize);
        }
    }
    for &x in rest {
        bad[0] |= (x ^ first) & other;
        cs.add(0, ((ord(x) >> S) & 255) as usize);
    }
    if bad.iter().fold(0, |m, &b| m | b) != 0 {
        return false;
    }
    let c = cs.sum();
    crate::v2::engines::lsd::short16::put_byte::<F, S>(a, &c);
    true
}

/// The keys within 256 values of order (`R`: the order rotated by half the
/// domain, keys near both ends): checked on all, counted, written.
fn cheap_range<const R: u16>(a: &mut [u16], s: &[u16; 18]) -> bool {
    let key = |x: u16| ord(x) ^ R;
    let (lo, hi) = s.iter().fold((u16::MAX, 0), |(l, h), &v| (l.min(key(v)), h.max(key(v))));
    if hi - lo > 255 {
        return false;
    }
    let (lo, hi) = a.iter().fold((u16::MAX, 0u16), |(l, h), &v| (l.min(key(v)), h.max(key(v))));
    if hi - lo > 255 {
        return false;
    }
    let digit = |x: u16| (key(x).wrapping_sub(lo) & 255) as usize;
    let c = if a.len() < LANES_MIN { count_lanes::<1>(a, digit) } else { count_lanes::<4>(a, digit) };
    // Rotated keys from R up are the smallest in order: written first.
    let span = (hi - lo) as usize + 1;
    let split = if R == 0 { 0 } else { (R.max(lo) - lo).min(span as u16) as usize };
    let mut at = 0;
    for d in (split..span).chain(0..split) {
        let m = c[d] as usize;
        crate::v2::engines::lsd::short16::put(a, at, m, ord(lo.wrapping_add(d as u16) ^ R));
        at += m;
    }
    trace!("cheap_range");
    true
}

/// Up to `LSD_STACK` keys the chain keeps the near-sorted repair,
/// `few_compare` and `range` (and `few` from `SHORT_FEW_MIN` keys), then
/// the LSD: no other detector, no Verge (on 30 distributions, 200 keys:
/// 11-13 % faster on average than the whole chain, 1000: 3 %, 2000: 1-2 %;
/// plain random keys 20-30 % at 200).
/// From this size the short chain tries `few` (FakeZipf, Late outl. twice
/// as fast at 1500-2000 keys, Few 16 20-37 %); below, its failures and its
/// table cost more (Heavy 4 1.7 times slower, Sparse 1.3-1.6).
const SHORT_FEW_MIN: usize = 1024;

/// Below this size, the short preparation (`tiny`) in front of the
/// whole path. Idea: blazing-radix `prepare` (cheap probes on a few keys,
/// each giving up at once).
const TINY_MAX: usize = 1100;

/// The probes of `tiny` (18-key sample): sorts and returns `true` when one
/// fires, else nothing written.
fn tiny_probe(arr: &mut [u16]) -> bool {
    let n = arr.len();
    let mut reps = [arr[0]; 4];
    let (mut k, mut many) = (1, false);
    let (mut lo, mut hi) = (ord(arr[0]), ord(arr[0]));
    let mut sample_rot = [0u16; 18];
    sample_rot[17] = ord(arr[0]) ^ (1 << 15);
    let mut varies = 0u16;
    for (i, &v) in arr[1..16].iter().chain([&arr[n / 2], &arr[n - 1]]).enumerate() {
        sample_rot[i] = ord(v) ^ (1 << 15);
        varies |= v ^ arr[0];
        lo = lo.min(ord(v));
        hi = hi.max(ord(v));
        if !many && !reps[..k].contains(&v) {
            if k == 4 {
                many = true;
            } else {
                reps[k] = v;
                k += 1;
            }
        }
    }
    if !many {
        reps[..k].sort_unstable_by_key(|&v| ord(v));
        let done = match k {
            1 => few_compare_k::<1>(arr, &reps),
            2 => few_compare_k::<2>(arr, &reps),
            3 => few_compare_k::<3>(arr, &reps),
            _ => few_compare_k::<4>(arr, &reps),
        };
        if done {
            trace!("tiny_few");
            return true;
        }
    }
    if hi - lo <= 255 && tiny_range::<0, u16>(arr) {
        trace!("tiny_range");
        return true;
    }
    // Same keys rotated by half the domain: values near 0 and near 65535.
    const R: u16 = 1 << 15;
    let (rlo, rhi) = sample_rot.iter().fold((u16::MAX, 0), |(l, h), &v| (l.min(v), h.max(v)));
    if rhi - rlo <= 255 && tiny_range::<R, u16>(arr) {
        trace!("tiny_range_rot");
        return true;
    }
    // Only the high byte varies on the sample (Aligned).
    if varies & 0xFF == 0 && tiny_high(arr) {
        trace!("tiny_high");
        return true;
    }
    if tiny_dominant(arr) {
        trace!("tiny_dominant");
        return true;
    }
    false
}

/// Per architecture, where the CPUs disagree (the code each one runs best,
/// chosen at compile time). The order scan by blocks of 16: ARM (Surface:
/// 3-4 % faster below 1000 keys); x86 keeps the shared scan (Zen 2: the
/// blocks 3-5 % slower).
#[inline(always)]
fn blocks16_scan() -> bool { cfg!(target_arch = "aarch64") }

/// 16-bit counters in the chunk LSD below 65 536 keys: ARM (Surface 2-7 %
/// faster on random keys at 1100-10k); x86 32-bit (Zen 2: 16-bit 25-30 %
/// slower).
#[inline(always)]
fn narrow_counters() -> bool { cfg!(target_arch = "aarch64") }

/// The high pass of the LSD one key at a time (blazing-style): x86 (Zen 2
/// 1-2 %, Haswell 2-6 % faster); ARM keeps the branch-free positions
/// (Surface: 1-3 % faster).
#[inline(always)]
fn seq_high_pass() -> bool { cfg!(not(target_arch = "aarch64")) }

/// `presorted` for u16 (on ARM, see `blocks16_scan`; on x86: 3-5 % slower on
/// Zen 2 and Haswell, Sorted desc / Desc Saw 1.2-1.3; faster only on the
/// Surface) (idea: blazing-radix's
/// `ordered_scan`): blocks of 16 keys compared without branches, the last
/// key of a block carried to the next (no overlapping windows, no slice to
/// array conversion); a descending array is scanned descending, then
/// reversed (no spread pairs, no second scan).
fn presorted16(arr: &mut [u16]) -> Result<(), Unsorted> {
    let n = arr.len();
    let (first, last) = (arr[0], arr[n - 1]);
    if first == last && first == arr[n / 2] && arr.iter().fold(0, |d, &v| d | (v ^ first)) == 0 {
        return Ok(());
    }
    if ord(first) <= ord(last) {
        let p = scan16::<false>(arr);
        return if p == n { Ok(()) } else { Err(Unsorted { ascending_prefix: p, probes: None }) };
    }
    if scan16::<true>(arr) == n {
        arr.reverse();
        return Ok(());
    }
    Err(Unsorted::default())
}

/// Length of the run in order (ascending, or descending with `DESC`) from
/// the start: blocks of 16, the exact position searched in the failing one.
#[inline(always)]
fn scan16<const DESC: bool>(arr: &[u16]) -> usize {
    let ok = |a: u16, b: u16| if DESC { ord(a) >= ord(b) } else { ord(a) <= ord(b) };
    let n = arr.len();
    let mut prev = arr[0];
    let (blocks, _) = arr.as_chunks::<16>();
    let mut i = 0;
    for b in blocks {
        let mut good = ok(prev, b[0]);
        for j in 0..15 {
            good &= ok(b[j], b[j + 1]);
        }
        if !good {
            break;
        }
        prev = b[15];
        i += 16;
    }
    let mut j = i.max(1);
    while j < n && ok(arr[j - 1], arr[j]) {
        j += 1;
    }
    j
}

/// One chain for every size, the questions in order of
/// cost: one inspection (`presorted`); the detectors that read a sample
/// (18 keys below `TINY_MAX`, else the spread sample of 64); from
/// `domain_min()` the domain count; the detectors that read the whole array
/// (`few` from `SHORT_FEW_MIN`, heavy keys only when the sample shows one);
/// then what writes (sorted head, repair, Verge) and the engine.
fn chain2(arr: &mut [u16]) {
    let n = arr.len();
    let mut known = match if blocks16_scan() { presorted16(arr) } else { presorted(keys_of(arr)) } {
        Ok(()) => {
            trace!("presorted");
            return;
        },
        Err(known) => known,
    };
    // A first key above the last made the scan go descending and fail at
    // once, the ascending head left unknown (Pipe organ: half the array
    // sorted, then the tail): measured (it stops at the first inversion,
    // nothing on random keys). Pipe organ 20-26 % faster (Zen 2, Surface).
    if known.ascending_prefix == 0 {
        known.ascending_prefix = scan16::<false>(arr);
    }
    if n < TINY_MAX {
        if tiny_probe(arr) {
            return;
        }
    } else {
        let sample = spread(arr);
        if few_compare(arr, &sample) {
            trace!("few_cmp");
            return;
        }
        // The window before the domain count (a 256-value window costs less
        // than 65 536 counters: Range 16, Bimodal from 131 072 keys) while
        // the array fits in the L2 (2 n bytes): its second read is free
        // (Zen 2, 512 KiB: the 262 144 keys measured, beyond them the domain
        // count's single read is faster; the Surface's 12 MiB L2: the window
        // won at every size measured, Range 16 0.49 to 0.31 ns per key at
        // 200k, 0.41 to 0.33 at 1M-5M).
        let window_first = 2 * n <= crate::v2::cache::l2_bytes();
        if window_first && window256(arr, &sample) {
            trace!("window256");
            return;
        }
        if n >= domain_min() {
            // Few values, then one value holding half of the sample, before
            // the domain count, whose counter of a frequent value waits on
            // its own previous increment: on ARM (Surface from 131 072
            // keys: FakeZipf 1.33-1.48 to 0.50-0.62 ns per key, std ~1.0;
            // Sparse 1.2-1.6 to 0.44-0.54; Clusters 2 times faster; Few 16
            // 0.40 to 0.51-0.66); x86 keeps the domain count first (Zen 2:
            // Few 16 0.37 to 0.79, Clusters 0.34 to 0.84 at 1M, only
            // FakeZipf faster).
            if cfg!(target_arch = "aarch64") {
                // Only when one value holds a quarter of the sample
                // (FakeZipf, Sparse); values spread evenly (Clusters, Few 16)
                // count faster in the domain (i16 Clusters 0.31 to 0.46-0.70
                // ns per key with `few` first, Few 16 0.41 to 0.51-0.66).
                let mut sorted = sample;
                merge_sort(&mut sorted[..], $enc, $enc);
                let top = sorted.chunk_by(|a, b| a == b).map(|r| r.len()).max().unwrap_or(0);
                if top >= SAMPLE / 4 {
                    if few16(arr, &sample) {
                        trace!("few");
                        return;
                    }
                    if top >= SAMPLE / 2
                        && heavy_sorted_by(
                            arr,
                            &sorted,
                            HEAVY_MIN,
                            &|r: &mut [u16]| sort(r),
                            |x, y| x == y,
                            |x, y| ord(x) < ord(y),
                        )
                    {
                        trace!("heavy_first");
                        return;
                    }
                }
            }
            if flat_byte16::<u32, true>(arr) {
                trace!("flat_byte");
            } else {
                trace!("domain16");
                domain_count16(arr);
            }
            return;
        }
        if !window_first && window256(arr, &sample) {
            trace!("window256");
            return;
        }
        // One low byte on the sample (Aligned): only the high byte counted,
        // checked on every key (`flat_byte16`).
        if sample.iter().all(|&v| v & 255 == sample[0] & 255)
            && flat_byte16::<u32, true>(arr)
        {
            trace!("flat_byte");
            return;
        }
        // A sample that looks random (`looks_random`) goes straight to the
        // engine: `few`, heavy keys and `range` would all fail on it, and the
        // repair and Verge checks find nothing (Zen 2: Unif, Gaussian,
        // Normal 13, Sensor 3-6 % faster at 1100-10k, Runs 8 4-11 %;
        // FakeZipf, Few 16 ~4 % slower below 2000, still far ahead of
        // blazing).
        if looks_random(arr, &sample) {
            trace!("random_sample");
            // On ARM the LSD below 131 072 keys: random keys hold about one
            // key per counter of the domain count (Surface, Unif 65k-100k:
            // 1.27-1.71 to 1.15-1.18 ns per key, = blazing; Spread 4k, few
            // values, is not caught here).
            if cfg!(target_arch = "aarch64") && (1 << 16..1 << 17).contains(&n) {
                lsd16_mode::<3, true>(arr);
            } else {
                sort_chunk(bits_of(arr), &mut Vec::new());
            }
            return;
        }
        if detectors::range(keys_of(arr), &sample.map(|v| v as K)) {
            trace!("range");
            return;
        }
        // One value holding half of the sample (Sparse) extracted before
        // `few`, which counts until `FEW_MAX` and fails (Zen 2 at 5k: Sparse
        // 2.13 to 1.18 ns per key, FakeZipf 0.97 to 1.50, blazing 3.3-3.6:
        // both kept well ahead, more evenly).
        // Below `HEAVY_MIN` only one dominant value (`dominant`: Sparse at
        // 1100-2047 keys went to `few`, whose counter of that value waits on
        // its own increment: Surface 3.1-3.3 to 0.9-1.2 ns per key; FakeZipf,
        // several heavy values, keeps `few`).
        if may_be_heavy(&sample, |x, y| x == y)
            && (n >= HEAVY_MIN || dominant(&sample, |x: u16, y: u16| x == y))
        {
            let mut sorted = sample;
            merge_sort(&mut sorted[..], $enc, $enc);
            let top = sorted.chunk_by(|a, b| a == b).map(|r| r.len()).max().unwrap_or(0);
            if top >= SAMPLE / 2
                && heavy_sorted_by(
                    arr,
                    &sorted,
                    TINY_MAX,
                    &|r: &mut [u16]| sort(r),
                    |x, y| x == y,
                    |x, y| ord(x) < ord(y),
                )
            {
                trace!("heavy_first");
                return;
            }
        }
        if n >= SHORT_FEW_MIN && few16(arr, &sample) {
            trace!("few");
            return;
        }
        if n >= HEAVY_MIN && may_be_heavy(&sample, |x, y| x == y) {
            let mut sorted = sample;
            merge_sort(&mut sorted[..], $enc, $enc);
            // The extraction only when one value holds half of the sample
            // (Sparse); several heavy values go to the LSD (Heavy 4 5k-65k:
            // 1.62-1.20 of blazing down to 1.30-0.94 on Zen 2).
            let top = sorted
                .chunk_by(|a, b| a == b)
                .map(|r| r.len())
                .max()
                .unwrap_or(0);
            if top < SAMPLE / 2 {
                trace!("heavy_skipped");
            } else if heavy_sorted_by(
                arr,
                &sorted,
                HEAVY_MIN,
                &|r: &mut [u16]| sort(r),
                |x, y| x == y,
                |x, y| ord(x) < ord(y),
            ) {
                trace!("heavy");
                return;
            }
        }
    }
    let buffer = std::cell::RefCell::new(Vec::new());
    let sort_chunk = |chunk: &mut [u16]| sort_chunk(bits_of(chunk), &mut buffer.borrow_mut());
    path(arr, known, &|_: &mut [u16], _: usize| false, &sort_chunk);
}

/// A random array, cheapest test first (see its use in `chain2`): the
/// spread sample spans beyond what `range` counts; 16 evenly spaced keys
/// change direction at least 5 times (random: ~9; Pipe organ, Near sorted,
/// a sorted head: 0-2); the sample holds at most 2 repeats in 4096 hashed
/// bits (random keys: ~0.5 by chance; Clusters, 256 values: ~7 true ones).
fn looks_random(arr: &[u16], sample: &[u16; SAMPLE]) -> bool {
    let n = arr.len();
    let (lo, hi) =
        sample.iter().fold((u16::MAX, 0u16), |(l, h), &v| (l.min(ord(v)), h.max(ord(v))));
    if ((hi - lo) as usize) < n / crate::v2::special::RANGE_KEYS_PER_COUNTER {
        return false;
    }
    let step = n / 16;
    let mut turns = 0usize;
    for k in 1..15 {
        let (a, b, c) = (ord(arr[(k - 1) * step]), ord(arr[k * step]), ord(arr[(k + 1) * step]));
        turns += ((a < b) != (b < c)) as usize;
    }
    if turns < 5 {
        return false;
    }
    let mut bits = [0u64; 64];
    let mut repeats = 0u32;
    for &v in sample.iter() {
        let h = ((v as u32).wrapping_mul(0x9E37_79B1) >> 20) as usize;
        repeats += ((bits[h >> 6] >> (h & 63)) & 1) as u32;
        bits[h >> 6] |= 1 << (h & 63);
    }
    repeats <= 2
}

/// Few distinct values (at most `FEW_MAX`): the sample of the shared `few`
/// (at least 64 keys) with an estimate of the distinct values (Chao1: seen +
/// f1 (f1 - 1) / (2 (f2 + 1)), f1 and f2 the values seen once and twice):
/// beyond `FEW_MAX` nothing is counted, else the shared counting. Heavy 4
/// (4 values for 60 % of the keys, the rest random) passed the
/// half-distinct rule and counted until `FEW_MAX`, 1.6 times the LSD at
/// 2000 keys.
#[inline(always)]
fn few16(values: &mut [u16], sample: &[u16; SAMPLE]) -> bool {
    few16_under(values, sample, crate::v2::special::FEW_MAX)
}

/// `few16` with the estimate held under `limit`.
#[inline(never)]
fn few16_under(values: &mut [u16], sample: &[u16; SAMPLE], limit: usize) -> bool {
    use crate::v2::special::{FEW_BIG_MIN, FEW_MAX, FEW_TABLE, FEW_TABLE_BITS};
    let n = values.len();
    let hash = |v: u16| {
        ((v as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - FEW_TABLE_BITS)) as usize
    };
    // At least the 64 keys of the spread sample (Heavy 4 at 1100-1500 keys
    // 18-24 % faster than 16 as the shared `few`).
    let s = (n / 32).clamp(64, 256);
    let mut keys = [0u16; FEW_TABLE];
    let mut seen = [0u16; FEW_TABLE];
    let (mut distinct, mut f1, mut f2) = (0, 0usize, 0usize);
    for i in 0..s as u64 {
        let v = if (i as usize) < SAMPLE {
            sample[i as usize]
        } else {
            values[(((i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) * n as u64) >> 32) as usize]
        };
        let mut h = hash(v);
        while seen[h] != 0 && keys[h] != v {
            h = (h + 1) & (FEW_TABLE - 1);
        }
        if seen[h] == 0 {
            keys[h] = v;
            distinct += 1;
            if distinct > s / 2 {
                return false;
            }
        }
        // Values seen once (f1) and twice (f2), kept as the sample goes
        // (without a branch: a match mispredicted on repeated values).
        let c = seen[h];
        f1 = f1 + (c == 0) as usize - (c == 1) as usize;
        f2 = f2 + (c == 1) as usize - (c == 2) as usize;
        seen[h] = c + 1;
    }
    let _ = FEW_MAX;
    if distinct + f1 * f1.saturating_sub(1) / (2 * (f2 + 1)) > limit {
        trace!("few_estimate");
        return false;
    }
    if distinct > FEW_TABLE / 8 && n >= FEW_BIG_MIN {
        detectors::few_count::<{ 4 * FEW_TABLE }>(keys_of(values))
    } else {
        detectors::few_count::<FEW_TABLE>(keys_of(values))
    }
}

/// One value holding at least 7 of 18 sampled keys
/// (Sparse, FakeZipf): its keys counted, the others gathered in a stack
/// buffer and sorted by the small engines or the LSD, then written: the
/// smaller ones, the value, the larger ones. Equal keys chain the counters
/// of a radix pass; here they are only counted.
fn tiny_dominant(arr: &mut [u16]) -> bool {
    let n = arr.len();
    debug_assert!(n < TINY_MAX);
    // Majority vote (Boyer-Moore) on the 18 keys, then its count.
    let sample = arr[..16].iter().chain([&arr[n / 2], &arr[n - 1]]);
    let (mut best, mut votes) = (arr[0], 0i32);
    for &v in sample.clone() {
        if votes == 0 {
            best = v;
        }
        votes += if v == best { 1 } else { -1 };
    }
    if sample.filter(|&&v| v == best).count() < 7 {
        return false;
    }
    let mut buf = [0u16; TINY_MAX];
    let mut m = 0;
    for &k in arr.iter() {
        buf[m] = k;
        m += (k != best) as usize;
    }
    let c = n - m;
    let others = &mut buf[..m];
    if m <= NETWORK_MAX {
        network_sort(others, $enc, $enc);
    } else if m <= MERGE_MAX {
        merge_sort(others, $enc, $enc);
    } else if tiny_probe(others) {
        // The other keys asked again (FakeZipf: the next value holds half of
        // them, and so on).
    } else {
        lsd16_mode::<3, true>(others);
    }
    let p = others.partition_point(|&k| ord(k) < ord(best));
    arr[..p].copy_from_slice(&others[..p]);
    arr[p..p + c].fill(best);
    arr[p + c..].copy_from_slice(&others[p..]);
    true
}

/// Keys within a window of 256 values around the spread sample's range when
/// the sample spans at most 64 values (Range 16: 0.95-1.00 of blazing down
/// to 0.81-0.91; a range nearly full, Small, stays faster in `range`):
/// straight, or rotated by half the domain (values near both ends, Bimodal;
/// The window checked by its own pass (an OR of the
/// offsets' high bits), then 4 lanes of 32-bit counters; nothing written
/// when a key falls outside. Then the used buckets written from the counts
/// by 32-key stores, the empty ones skipped (Bimodal 1100-5000 keys 10-28 %
/// faster on the Surface; rotated on Zen 2: 0.60 to 0.48 at 10k, = blazing).
/// Idea: blazing-radix `compact_range`.
fn window256(arr: &mut [u16], sample: &[u16; SAMPLE]) -> bool {
    const R: u16 = 1 << 15;
    let (mut lo, mut hi, mut rlo, mut rhi) = (u16::MAX, 0u16, u16::MAX, 0u16);
    for &v in sample.iter() {
        lo = lo.min(ord(v));
        hi = hi.max(ord(v));
        rlo = rlo.min(ord(v) ^ R);
        rhi = rhi.max(ord(v) ^ R);
    }
    if hi - lo <= 64 {
        let base = lo.saturating_sub((255 - (hi - lo)) / 2).min(u16::MAX - 255);
        window_count256::<0>(arr, base)
    } else if hi >> 8 == lo >> 8 {
        // One high byte on the sample (Small): the window is that block of
        // 256 values (Surface: 1100-5000 keys 18-32 % faster than `range`,
        // 10-14k 15-30 % slower, still under blazing).
        window_count256::<0>(arr, lo & 0xFF00)
    } else if rhi - rlo <= 64 {
        let base = rlo.saturating_sub((255 - (rhi - rlo)) / 2).min(u16::MAX - 255);
        window_count256::<R>(arr, base)
    } else if hi - lo <= 255 {
        // A sample within 256 values across two high bytes (i16 Small:
        // -128..126, keys around 0x8000; the sample misses an end value of
        // 255 three times in four, so a window centred on it often falls
        // short): the exact minimum and maximum first (one vectorized
        // pass), then the window from the minimum. Idea: blazing-radix
        // `compact_range` (its separate narrow reduction).
        let (lo, hi) = arr.iter().fold((u16::MAX, 0u16), |(l, h), &v| (l.min(ord(v)), h.max(ord(v))));
        hi - lo <= 255 && window_count256_in::<0, false>(arr, lo.min(u16::MAX - 255))
    } else {
        false
    }
}

/// The counting of `window256` in [`base`, `base + 255`] (keys xored with
/// `ROT`).
fn window_count256<const ROT: u16>(arr: &mut [u16], base: u16) -> bool {
    window_count256_in::<ROT, true>(arr, base)
}

/// `window_count256`; `CHECK` false: every key known inside the window
/// (exact minimum and maximum taken), no checking pass.
#[inline(always)]
fn window_count256_in<const ROT: u16, const CHECK: bool>(arr: &mut [u16], base: u16) -> bool {
    let n = arr.len();
    let mut c = Lanes::<4>::new();
    let (quads, rest) = arr.as_chunks::<4>();
    // The window checked first by its own pass (vectorized, nothing
    // counted when a key falls outside), then a counting loop with
    // nothing else in it. Idea: blazing-radix `compact_range`.
    if CHECK {
        let out = arr.iter().fold(0u16, |o, &k| o | (k ^ ROT ^ F).wrapping_sub(base));
        if out > 255 {
            return false;
        }
    }
    for q in quads {
        for l in 0..4 {
            c.add(l, ((q[l] ^ ROT ^ F).wrapping_sub(base) & 255) as usize);
        }
    }
    for &k in rest {
        c.add(0, ((k ^ ROT ^ F).wrapping_sub(base) & 255) as usize);
    }
    let counts = c.sum();
    // Rotated keys from `ROT` on are the smallest original values: written
    // first when the window crosses that point.
    let hi = base as usize + 255;
    let split = if ROT != 0 && (base as usize) < ROT as usize && hi >= ROT as usize {
        (ROT - base) as usize
    } else {
        0
    };
    // Only from the first to the last bucket used, in writing order
    // (Bimodal fills 32 of the 256).
    let at = |p: usize| (p + split) & 255;
    let first = (0..256).find(|&p| counts[at(p)] != 0).unwrap_or(0);
    let last = (0..256).rev().find(|&p| counts[at(p)] != 0).unwrap_or(0);
    let mut start = 0;
    for d in (first..=last).map(at) {
        let cnt = counts[d] as usize;
        // Empty buckets skipped (predictable: a rotated window centred on
        // the split leaves its used buckets at both ends of the writing
        // order, Bimodal: 224 empty of 256).
        if cnt == 0 {
            continue;
        }
        let v = (base.wrapping_add(d as u16)) ^ ROT ^ F;
        if cnt <= 32 && n - start >= 32 {
            arr[start..start + 32].copy_from_slice(&[v; 32]);
        } else if cnt > 32 && cnt <= 128 {
            // 33-128 keys: 32-key stores (the excess overwritten by the
            // next buckets), the last one ending at the bucket's end; a
            // `fill` call per bucket cost more (Small 10-14k keys).
            let end = start + cnt;
            let mut o = start;
            while end - o > 32 {
                arr[o..o + 32].copy_from_slice(&[v; 32]);
                o += 32;
            }
            arr[end - 32..end].copy_from_slice(&[v; 32]);
        } else {
            arr[start..start + cnt].fill(v);
        }
        start += cnt;
    }
    debug_assert_eq!(start, n);
    true
}

/// Keys within 256 values once xored with `ROT` (0, or half the domain for
/// values near both ends), exact min / max first: counted on 2 lanes of
/// 16-bit counters, then written in the order of the original values, each
/// bucket by one 32-key store when it fits (overwritten by the next buckets:
/// no branch on the count, empty buckets included), else `fill`. Nothing
/// written when the range is wider.
fn tiny_range<const ROT: u16, C: Count16>(arr: &mut [u16]) -> bool {
    let n = arr.len();
    debug_assert!(n <= u32::MAX as usize);
    let (lo, hi) =
        arr.iter().fold((u16::MAX, 0u16), |(l, h), &v| (l.min(v ^ ROT ^ F), h.max(v ^ ROT ^ F)));
    let span = (hi - lo) as usize;
    if span > 255 {
        return false;
    }
    let mut c = [[C::ZERO; 256]; 2];
    // Digits masked by 255 (the span is checked): no bounds check per key,
    // 10-20 % faster at 500-1000 keys (Surface, Bimodal, Range 16, Small).
    // From 768 keys 4 lanes of 32-bit counters. Idea: blazing-radix
    // `compact_range`.
    if n >= 768 {
        let mut q = Lanes::<4>::new();
        let (quads, rest) = arr.as_chunks::<4>();
        for k in quads {
            for l in 0..4 {
                q.add(l, ((k[l] ^ ROT ^ F).wrapping_sub(lo) & 255) as usize);
            }
        }
        for &v in rest {
            q.add(0, ((v ^ ROT ^ F).wrapping_sub(lo) & 255) as usize);
        }
        let q = q.sum();
        for d in 0..=span {
            c[0][d].put(q[d] as usize);
        }
    } else {
        let (pairs, rest) = arr.as_chunks::<2>();
        for p in pairs {
            c[0][((p[0] ^ ROT ^ F).wrapping_sub(lo) & 255) as usize].inc();
            c[1][((p[1] ^ ROT ^ F).wrapping_sub(lo) & 255) as usize].inc();
        }
        for &v in rest {
            c[0][((v ^ ROT ^ F).wrapping_sub(lo) & 255) as usize].inc();
        }
        for d in 0..=span {
            c[0][d].put(c[0][d].get() + c[1][d].get());
        }
    }
    // Rotated keys from `ROT` on are the smallest original values: written
    // first when the range crosses that point.
    let split = if ROT != 0 && lo < ROT && hi >= ROT { (ROT - lo) as usize } else { 0 };
    tiny_write(
        arr,
        &c[0],
        (split..=span).chain(0..split),
        |d| (lo + d as u16) ^ ROT ^ F,
        |v| ((v ^ ROT ^ F).wrapping_sub(lo) & 255) as usize,
    );
    true
}

/// Keys equal outside their high byte (checked on every key while counting
/// it, on 2 lanes): written from the counts of the high byte. Nothing
/// written when a low byte differs.
fn tiny_high(arr: &mut [u16]) -> bool {
    let low = arr[0] & 0xFF;
    let mut c = [[0u16; 256]; 2];
    let mut diff = [0u16; 2];
    let (pairs, rest) = arr.as_chunks::<2>();
    for p in pairs {
        diff[0] |= p[0] ^ low;
        diff[1] |= p[1] ^ low;
        c[0][(ord(p[0]) >> 8) as usize] += 1;
        c[1][(ord(p[1]) >> 8) as usize] += 1;
    }
    for &v in rest {
        diff[0] |= v ^ low;
        c[0][(ord(v) >> 8) as usize] += 1;
    }
    if (diff[0] | diff[1]) & 0xFF != 0 {
        return false;
    }
    for d in 0..256 {
        c[0][d] += c[1][d];
    }
    tiny_write(arr, &c[0], 0..256, |d| (((d as u16) << 8) | low) ^ F, |v| (ord(v) >> 8) as usize);
    true
}

/// Writes counted keys in the order of `order` (bucket `d` holds the value
/// `value(d)`; `digit` is the bucket of a key). Fewer keys than buckets:
/// every key placed at its position in a stack buffer, then copied back
/// (idea: blazing-radix `compact_scatter`; a walk over the buckets would
/// cost more than the keys); else each bucket by one 32-key store when it
/// fits (overwritten by the next buckets: no branch on the count, empty
/// buckets included), else `fill`.
#[inline(always)]
fn tiny_write<C: Count16>(
    arr: &mut [u16],
    counts: &[C; 256],
    order: impl Iterator<Item = usize> + Clone,
    value: impl Fn(usize) -> u16,
    digit: impl Fn(u16) -> usize,
) {
    let n = arr.len();
    let buckets = order.clone().count();
    if n < buckets {
        let mut pos = [0u16; 256];
        let mut s = 0u16;
        for d in order {
            pos[d] = s;
            s += counts[d].get() as u16;
        }
        let mut buf = [0u16; 256];
        for &k in arr.iter() {
            let d = digit(k);
            buf[pos[d] as usize] = k;
            pos[d] += 1;
        }
        arr.copy_from_slice(&buf[..n]);
        return;
    }
    let mut start = 0;
    for d in order {
        let cnt = counts[d].get();
        let v = value(d);
        if cnt <= 32 && n - start >= 32 {
            arr[start..start + 32].copy_from_slice(&[v; 32]);
        } else {
            arr[start..start + cnt].fill(v);
        }
        start += cnt;
    }
    debug_assert_eq!(start, n);
}

#[inline(always)]
fn small_chain(n: usize) -> bool { n <= LSD_STACK }

/// At most 4 distinct values in the spread sample: every key compared with
/// them (no table; independent counts the compiler vectorizes), nothing
/// written when a key is not among them. Idea: blazing-radix `few_values`.
fn few_compare(arr: &mut [u16], sample: &[u16; SAMPLE]) -> bool {
    let mut reps = [sample[0]; 4];
    let mut k = 1;
    for &v in sample.iter() {
        if !reps[..k].contains(&v) {
            if k == 4 {
                return false;
            }
            reps[k] = v;
            k += 1;
        }
    }
    reps[..k].sort_unstable_by_key(|&v| ord(v));
    match k {
        1 => few_compare_k::<1>(arr, &reps),
        2 => few_compare_k::<2>(arr, &reps),
        3 => few_compare_k::<3>(arr, &reps),
        _ => few_compare_k::<4>(arr, &reps),
    }
}

/// The counting pass of `few_compare` against `K` sorted values.
#[inline(never)]
fn few_compare_k<const KN: usize>(arr: &mut [u16], reps: &[u16; 4]) -> bool {
    let keys: [u16; KN] = std::array::from_fn(|i| reps[i]);
    let mut counts = [0usize; KN];
    // Blocks of 4096 keys: u16 counters, one check per block.
    for block in arr.chunks(4096) {
        let mut c = [0u16; KN];
        let mut miss = 0u16;
        for &v in block {
            let mut known = 0u16;
            for i in 0..KN {
                let m = (v == keys[i]) as u16;
                c[i] += m;
                known |= m;
            }
            miss |= known ^ 1;
        }
        if miss != 0 {
            return false;
        }
        for i in 0..KN {
            counts[i] += c[i] as usize;
        }
    }
    let mut start = 0;
    for i in 0..KN {
        arr[start..start + counts[i]].fill(keys[i]);
        start += counts[i];
    }
    true
}

/// From this size the whole domain is counted (`range_domain`): two keys per
/// counter of the 65536.
const DOMAIN_MIN: usize = 1 << 17;

/// From this size the domain count.
#[inline(always)]
fn domain_min() -> usize { DOMAIN_MIN }

/// Counting sort over the 16-bit domain (copy of the shared `domain_count`,
/// one lane of counters).
///
/// - Counting: 16-bit counters (`count_narrow16`, a 128 KiB table) when the
///   32-bit table (256 KiB) does not fit in half the L2 (Haswell: 15-38 %
///   faster); else 32-bit counters (Zen 2: 16-bit ones 10-22 % slower on
///   values that stay in cache).
/// - Beyond twice the largest cache (`n` keys of 2 bytes against
///   `largest_bytes`): `stream_write`.
/// - Else `write_buckets`.
#[inline(never)]
fn domain_count16(keys: &mut [u16]) {
    let n = keys.len();
    debug_assert!(n <= u32::MAX as usize);
    let big = n > crate::v2::cache::largest_bytes();
    let run4 = runs_pay(keys);
    // 16-bit counters on older Intel cores (`old_intel`: the 32-bit table
    // fills their 256 KiB L2), unless the keys span few counters (64 spread
    // keys: Gaussian, Sensor, Clusters):
    // their part of the table stays in the cache anyway, and the wrap test
    // costs 3 instructions per key (small intel, Gaussian 1M: 7.4 against
    // blazing's 4.4).
    let l2 = crate::v2::cache::l2_bytes();
    let narrow_span = || {
        let step = (n / 64).max(1);
        let (lo, hi) = keys.iter().step_by(step).take(64).fold((u16::MAX, 0u16), |(l, h), &k| {
            (l.min(ord(k)), h.max(ord(k)))
        });
        (hi - lo) as usize * 4 * 4 <= l2
    };
    if crate::v2::cache::old_intel() && !narrow_span() {
        let (low, wraps) = count_narrow16(keys, run4);
        if wraps.is_empty() && !big {
            write_buckets(keys, &low);
            return;
        }
        let mut wide: Vec<u32> = low.iter().map(|&c| c as u32).collect();
        for &k in &wraps {
            wide[k as usize] += 1 << 16;
        }
        if big && stream_write(keys, &wide) {
            return;
        }
        write_buckets(keys, &wide);
        return;
    }
    // The table on the stack (256 KiB): on the heap it is new pages at each
    // call (64 page faults of 4 KiB: Core Ultra 185H 200k-1M up to 1.5x
    // slower).
    let mut counts = [0u32; 1 << 16];
    {
        let (quads, tail) = keys.as_chunks::<4>();
        if !run4 {
            // 4 keys read by one 64-bit load, the indexes taken by shifts:
            // the loop is bound by its loads (Zen 2: 2 per cycle; 4 key loads
            // + 4 counter loads per 4 keys, 5 this way). Zen 2 200k-5M 10-20 %
            // faster, Surface 3-13 % (i16 Unif 1M: 0.76 to 0.66 ns per key,
            // = blazing, whose i16 code does it by a vector load: 3.1 cycles
            // per key against 3.8 for its u16 code).
            for q in quads {
                let w = u64::from_le_bytes(unsafe { std::mem::transmute::<[u16; 4], [u8; 8]>(*q) });
                let k0 = ord(w as u16) as usize;
                let k1 = ord((w >> 16) as u16) as usize;
                let k2 = ord((w >> 32) as u16) as usize;
                let k3 = ord((w >> 48) as u16) as usize;
                // SAFETY: a u16 key is below 65536.
                unsafe {
                    *counts.get_unchecked_mut(k0) += 1;
                    *counts.get_unchecked_mut(k1) += 1;
                    *counts.get_unchecked_mut(k2) += 1;
                    *counts.get_unchecked_mut(k3) += 1;
                }
            }
            for &k in tail {
                counts[ord(k) as usize] += 1;
            }
        } else {
        for q in quads {
            // 4 equal keys (runs of a value: sorted parts): one addition
            // instead of 4 increments waiting on each other.
            if run4 && q[0] == q[1] && q[1] == q[2] && q[2] == q[3] {
                counts[ord(q[0]) as usize] += 4;
                continue;
            }
            // SAFETY: a u16 key is below 65536.
            unsafe {
                *counts.get_unchecked_mut(ord(q[0]) as usize) += 1;
                *counts.get_unchecked_mut(ord(q[1]) as usize) += 1;
                *counts.get_unchecked_mut(ord(q[2]) as usize) += 1;
                *counts.get_unchecked_mut(ord(q[3]) as usize) += 1;
            }
        }
        for &k in tail {
            counts[ord(k) as usize] += 1;
        }
        }
    }
    // Arrays beyond twice the largest cache: lines already out of the cache,
    // written without being read first.
    if big && stream_write(keys, &counts) {
        return;
    }
    write_buckets(keys, &counts);
}

/// Whether the 4-equal-keys shortcut of the counting pays: on 64 aligned
/// quadruples spread over the array, at least half of them made of one value
/// (else its branch mispredicts more than it saves: Runs 8 1M +136-196 %,
/// FakeZipf +40-80 %, Near sorted 300k +70-85 %; with long runs of a value,
/// Near sorted / Sort rnd tl / Pipe organ 3M-10M, Sparse 90 %: -20 to -61 %).
fn runs_pay(keys: &[u16]) -> bool {
    let (quads, _) = keys.as_chunks::<4>();
    if quads.len() < 64 {
        return false;
    }
    let step = quads.len() / 64;
    let same = (0..64)
        .filter(|&i| {
            let q = quads[i * step];
            q[0] == q[1] && q[1] == q[2] && q[2] == q[3]
        })
        .count();
    same >= 32
}

/// 16-bit counters (half the table of 32-bit ones): a counter wrapping to 0
/// records its key (a branch almost never taken). Returns the low 16 bits
/// of each count and the keys of the wraps (one per 65536 keys of a value).
fn count_narrow16(keys: &[u16], run4: bool) -> (Vec<u16>, Vec<u16>) {
    let mut low = vec![0u16; 1 << 16];
    let mut wraps = Vec::new();
    let (quads, tail) = keys.as_chunks::<4>();
    let mut add = |k: u16, d: u16| {
        // SAFETY: a u16 key is below 65536.
        let c = unsafe { low.get_unchecked_mut(k as usize) };
        let (v, wrapped) = c.overflowing_add(d);
        *c = v;
        if wrapped {
            wraps.push(k);
        }
    };
    if !run4 {
        // 4 keys per 64-bit load, as the 32-bit table (see `domain_count16`).
        for q in quads {
            let w = u64::from_le_bytes(unsafe { std::mem::transmute::<[u16; 4], [u8; 8]>(*q) });
            add(ord(w as u16), 1);
            add(ord((w >> 16) as u16), 1);
            add(ord((w >> 32) as u16), 1);
            add(ord((w >> 48) as u16), 1);
        }
    } else {
        for q in quads {
            if run4 && q[0] == q[1] && q[1] == q[2] && q[2] == q[3] {
                add(ord(q[0]), 4);
                continue;
            }
            add(ord(q[0]), 1);
            add(ord(q[1]), 1);
            add(ord(q[2]), 1);
            add(ord(q[3]), 1);
        }
    }
    for &k in tail {
        add(ord(k), 1);
    }
    (low, wraps)
}

/// Writes the buckets of a full count table in order, one loop shape per
/// call:
///
/// - empty and full buckets alternating often (`flat_pays`): every bucket,
///   empty or not, writes one 32-key block at its position (no test on
///   empty buckets, which mispredicts when they alternate at random: Zipf,
///   Heavy 4 14-46 % faster);
/// - else the touched buckets only, by 32-key stores when they hold 12 keys
///   or more on average (from 256 spread counters), else by 16-key stores;
///   no branch per bucket on its count (a test of each count against 16
///   mispredicts around 15 keys per value). Idea: blazing-radix
///   `counting16` (fixed 64-byte stores). Unif 1M 15 % faster, Sensor 300k
///   22-25 %.
///
/// The copies written past a bucket are overwritten by the next buckets.
#[inline(never)]
fn write_buckets<C: Copy + Into<u32>>(keys: &mut [u16], counts: &[C]) {
    let n = keys.len();
    let at = |d: usize| counts[d].into() as usize;
    let mut start = 0;
    let (long, sparse) = {
        let (mut sum, mut used) = (0usize, 0usize);
        for i in 0..256 {
            let c = at(i * 256 + 128);
            sum += c;
            used += (c != 0) as usize;
        }
        // Most blocks of 16 counters empty (a probed counter used with
        // probability used / 256): the walk tests a block at once (Normal 8,
        // Clusters, Few 16 at 200k: Zen 2 15-20 %, Haswell 5-8 % faster in
        // 6 code placements; 1M: 3-5 %). Denser tables keep the plain walk
        // (Spread 4k: the block test 7-14 % slower there).
        (sum >= 12 * used.max(1), (1.0 - used as f64 / 256.0).powi(16) > 0.5)
    };
    let flat = flat_pays(counts, long);
    if flat {
        for d in 0..counts.len() {
            let c = at(d);
            let v = (d as u16) ^ F;
            let end = start + c;
            if n - start >= 32 {
                keys[start..start + 32].copy_from_slice(&[v; 32]);
                let mut p = start + 32;
                while p < end {
                    if n - p >= 32 {
                        keys[p..p + 32].copy_from_slice(&[v; 32]);
                    } else {
                        keys[p..end].fill(v);
                    }
                    p += 32;
                }
            } else {
                keys[start..end].fill(v);
            }
            start = end;
        }
        debug_assert_eq!(start, n);
        return;
    }
    if long {
        if sparse {
            for b in (0..counts.len()).step_by(16) {
                if counts[b..b + 16].iter().fold(0u32, |a, &c| a | c.into()) == 0 {
                    continue;
                }
                for d in b..b + 16 {
                    let c = at(d);
                    if c != 0 {
                        let v = (d as u16) ^ F;
                        let end = start + c;
                        let mut p = start;
                        while p < end && n - p >= 32 {
                            keys[p..p + 32].copy_from_slice(&[v; 32]);
                            p += 32;
                        }
                        if p < end {
                            keys[p..end].fill(v);
                        }
                        start = end;
                    }
                }
            }
        } else {
            for d in 0..counts.len() {
                let c = at(d);
                if c != 0 {
                    let v = (d as u16) ^ F;
                    let end = start + c;
                    let mut p = start;
                    while p < end && n - p >= 32 {
                        keys[p..p + 32].copy_from_slice(&[v; 32]);
                        p += 32;
                    }
                    if p < end {
                        keys[p..end].fill(v);
                    }
                    start = end;
                }
            }
        }
    } else {
        if sparse {
            for b in (0..counts.len()).step_by(16) {
                if counts[b..b + 16].iter().fold(0u32, |a, &c| a | c.into()) == 0 {
                    continue;
                }
                for d in b..b + 16 {
                    let c = at(d);
                    if c != 0 {
                        let v = (d as u16) ^ F;
                        let end = start + c;
                        if c <= 64 && end + 16 <= n {
                            let block = [v; 16];
                            let mut p = start;
                            while p < end {
                                keys[p..p + 16].copy_from_slice(&block);
                                p += 16;
                            }
                        } else {
                            keys[start..end].fill(v);
                        }
                        start = end;
                    }
                }
            }
        } else {
            for d in 0..counts.len() {
                let c = at(d);
                if c != 0 {
                    let v = (d as u16) ^ F;
                    let end = start + c;
                    if c <= 64 && end + 16 <= n {
                        let block = [v; 16];
                        let mut p = start;
                        while p < end {
                            keys[p..p + 16].copy_from_slice(&block);
                            p += 16;
                        }
                    } else {
                        keys[start..end].fill(v);
                    }
                    start = end;
                }
            }
        }
    }
    debug_assert_eq!(start, n);
}

/// Whether writing every bucket pays against testing for empty ones, from
/// 256 pairs of neighbour counters. A mispredicted test costs about 18
/// cycles; the flat write wastes about 4 per empty bucket and, against the
/// 16-key stores (`long` false), about 2 per full bucket (32-key blocks).
/// Few empty buckets: little to save (Unif 300k 10 % slower flat).
fn flat_pays<C: Copy + Into<u32>>(counts: &[C], long: bool) -> bool {
    let (mut turns, mut empty) = (0usize, 0usize);
    for i in 0..256 {
        let a = counts[i * 256 + 37].into() == 0;
        let b = counts[i * 256 + 38].into() == 0;
        turns += (a != b) as usize;
        empty += a as usize;
    }
    let full = if long { 0 } else { 256 - empty };
    18 * turns > 4 * empty + 2 * full
}

/// Writes the buckets of `counts` by non-temporal 32-byte stores (lines sent
/// to memory whole, not read into the cache first): a 16-lane accumulator
/// filled by masked blends, stored once full; long buckets store their value
/// directly. The keys before the first 32-byte boundary and the last
/// partial block by plain stores. False where unavailable (no AVX2, other
/// targets, Miri): nothing written. Twice the largest cache and beyond:
/// 6-15 % faster (Haswell from 10M keys, Zen 2 and Snapdragon X from 30M);
/// a loss below (up to 25 %).
#[allow(unreachable_code)]
fn stream_write(keys: &mut [u16], counts: &[u32]) -> bool {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return false;
        }
        // SAFETY: AVX2 checked above.
        unsafe { stream_write_avx2(keys, counts) };
        return true;
    }
    #[cfg(all(target_arch = "aarch64", not(miri)))]
    {
        // SAFETY: NEON is part of aarch64.
        unsafe { stream_write_neon(keys, counts) };
        return true;
    }
    let _ = (keys, counts);
    false
}

/// Fills `keys` from the start until `head` keys are written (plain stores);
/// returns the next bucket and what is left of the current one.
#[inline(always)]
fn stream_head(keys: &mut [u16], counts: &[u32], head: usize) -> (usize, usize) {
    let (mut d, mut rem, mut p) = (0usize, 0usize, 0usize);
    while p < head {
        while rem == 0 {
            rem = counts[d] as usize;
            d += 1;
        }
        let k = rem.min(head - p);
        keys[p..p + k].fill(((d - 1) as u16) ^ F);
        p += k;
        rem -= k;
    }
    (d, rem)
}

#[cfg(all(target_arch = "x86_64", not(miri)))]
#[target_feature(enable = "avx2")]
unsafe fn stream_write_avx2(keys: &mut [u16], counts: &[u32]) {
    use std::arch::x86_64::*;
    let n = keys.len();
    let base = keys.as_mut_ptr();
    let head = ((32 - (base as usize & 31)) & 31) / 2;
    let head = head.min(n);
    let (d, rem) = stream_head(keys, counts, head);
    let lanes = _mm256_setr_epi16(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    // SAFETY: `head <= n`; every 32-byte store below holds 16 keys of
    // `keys` (the counts sum to n), aligned by the head.
    let mut q = unsafe { base.add(head) } as *mut __m256i;
    let mut acc = _mm256_setzero_si256();
    let mut f = 0usize;
    let mut put = |v: u16, c: usize| {
        let vv = _mm256_set1_epi16(v as i16);
        let e = f + c;
        let from = _mm256_cmpgt_epi16(lanes, _mm256_set1_epi16(f as i16 - 1));
        if e < 16 {
            let below = _mm256_cmpgt_epi16(_mm256_set1_epi16(e as i16), lanes);
            acc = _mm256_blendv_epi8(acc, vv, _mm256_and_si256(from, below));
            f = e;
            return;
        }
        acc = _mm256_blendv_epi8(acc, vv, from);
        unsafe {
            _mm256_stream_si256(q, acc);
            q = q.add(1);
        }
        let mut r = e - 16;
        while r >= 16 {
            unsafe {
                _mm256_stream_si256(q, vv);
                q = q.add(1);
            }
            r -= 16;
        }
        acc = vv;
        f = r;
    };
    if rem != 0 {
        put(((d - 1) as u16) ^ F, rem);
    }
    for (i, &c) in counts.iter().enumerate().skip(d) {
        if c != 0 {
            put((i as u16) ^ F, c as usize);
        }
    }
    _mm_sfence();
    let mut last = [0u16; 16];
    unsafe { _mm256_storeu_si256(last.as_mut_ptr() as *mut __m256i, acc) };
    keys[n - f..].copy_from_slice(&last[..f]);
}

#[cfg(all(target_arch = "aarch64", not(miri)))]
unsafe fn stream_write_neon(keys: &mut [u16], counts: &[u32]) {
    use std::arch::aarch64::*;
    let n = keys.len();
    let base = keys.as_mut_ptr();
    let head = ((32 - (base as usize & 31)) & 31) / 2;
    let head = head.min(n);
    let (d, rem) = stream_head(keys, counts, head);
    let (l0, l1): ([u16; 8], [u16; 8]) = ([0, 1, 2, 3, 4, 5, 6, 7], [8, 9, 10, 11, 12, 13, 14, 15]);
    // SAFETY: as in `stream_write_avx2`: 16 keys of `keys` per store.
    let (lane0, lane1) = unsafe { (vld1q_u16(l0.as_ptr()), vld1q_u16(l1.as_ptr())) };
    let mut q = unsafe { base.add(head) };
    let (mut a0, mut a1) = (vdupq_n_u16(0), vdupq_n_u16(0));
    let mut f = 0usize;
    let stream = |q: &mut *mut u16, x: uint16x8_t, y: uint16x8_t| unsafe {
        std::arch::asm!("stnp {a:q}, {b:q}, [{p}]", a = in(vreg) x, b = in(vreg) y,
            p = in(reg) *q, options(nostack, preserves_flags));
        *q = q.add(16);
    };
    let mut put = |v: u16, c: usize| {
        let vv = vdupq_n_u16(v);
        let e = f + c;
        let fv = vdupq_n_u16(f as u16);
        let (m0, m1) = (vcgeq_u16(lane0, fv), vcgeq_u16(lane1, fv));
        if e < 16 {
            let ev = vdupq_n_u16(e as u16);
            a0 = vbslq_u16(vandq_u16(m0, vcltq_u16(lane0, ev)), vv, a0);
            a1 = vbslq_u16(vandq_u16(m1, vcltq_u16(lane1, ev)), vv, a1);
            f = e;
            return;
        }
        a0 = vbslq_u16(m0, vv, a0);
        a1 = vbslq_u16(m1, vv, a1);
        stream(&mut q, a0, a1);
        let mut r = e - 16;
        while r >= 16 {
            stream(&mut q, vv, vv);
            r -= 16;
        }
        a0 = vv;
        a1 = vv;
        f = r;
    };
    if rem != 0 {
        put(((d - 1) as u16) ^ F, rem);
    }
    for (i, &c) in counts.iter().enumerate().skip(d) {
        if c != 0 {
            put((i as u16) ^ F, c as usize);
        }
    }
    let mut last = [0u16; 16];
    unsafe {
        vst1q_u16(last.as_mut_ptr(), a0);
        vst1q_u16(last.as_mut_ptr().add(8), a1);
    }
    keys[n - f..].copy_from_slice(&last[..f]);
}

/// Up to this size the buffer of `lsd16` lives on the stack (4 KiB).
const LSD_STACK: usize = 2048;

/// Counter of `lsd16`: 16 bits below 65536 keys (tables of 512 bytes, prefix
/// sums on half the bytes), 32 bits beyond.
trait Count16: Copy {
    const ZERO: Self;
    fn inc(&mut self);
    fn get(self) -> usize;
    fn put(&mut self, v: usize);
}

impl Count16 for u16 {
    const ZERO: Self = 0;
    #[inline(always)]
    fn inc(&mut self) { *self = self.wrapping_add(1) }
    #[inline(always)]
    fn get(self) -> usize { self as usize }
    #[inline(always)]
    fn put(&mut self, v: usize) { *self = v as u16 }
}

impl Count16 for u32 {
    const ZERO: Self = 0;
    #[inline(always)]
    fn inc(&mut self) { *self += 1 }
    #[inline(always)]
    fn get(self) -> usize { self as usize }
    #[inline(always)]
    fn put(&mut self, v: usize) { *self = v as u32 }
}

/// `lsd16` with the scatters chosen (`M`: 0 branch-free positions on both
/// bytes; 1 one counter at a time on both, one histogram lane under 4096
/// keys; 2 one counter at a time on the low byte, branch-free on the high
/// byte, whose input holds equal keys side by side; 3 as 2, a byte
/// constant on a sample counted alone by `flat_byte16`) and the counters
/// (`WIDE`: 32 bits at every size).
fn lsd16_mode<const M: u8, const WIDE: bool>(keys: &mut [u16]) {
    let n = keys.len();
    if n < 2 {
        return;
    }
    if n <= LSD_STACK {
        let mut buf = [MaybeUninit::<u16>::uninit(); LSD_STACK];
        if WIDE {
            lsd16_with::<u32, M>(keys, &mut buf[..n]);
        } else {
            lsd16_with::<u16, M>(keys, &mut buf[..n]);
        }
    } else {
        let mut buf: Vec<MaybeUninit<u16>> = Vec::with_capacity(n);
        // SAFETY: `MaybeUninit` needs no initialization.
        unsafe { buf.set_len(n) };
        if !WIDE && n < 1 << 16 {
            lsd16_with::<u16, M>(keys, &mut buf);
        } else {
            lsd16_with::<u32, M>(keys, &mut buf);
        }
    }
}

/// `lsd16` with counters `C` (able to hold `keys.len()`), `buf` of the same
/// length as `keys`.
fn lsd16_with<C: Count16, const M: u8>(keys: &mut [u16], buf: &mut [MaybeUninit<u16>]) {
    let n = keys.len();
    debug_assert_eq!(buf.len(), n);
    if M == 3 && flat_byte16::<C, false>(keys) {
        return;
    }
    let one_lane = (M == 1 && n < 4096) || (M == 3 && n < 300);
    let (mut lo, mut hi) = if one_lane {
        // Short arrays: one counter per byte value (less to zero and add).
        let mut lo = [C::ZERO; 256];
        let mut hi = [C::ZERO; 256];
        for &k in keys.iter() {
            lo[(k & 255) as usize].inc();
            hi[(k >> 8) as usize].inc();
        }
        (lo, hi)
    } else {
        // Two lanes per byte (consecutive keys on different counters).
        let mut lo = [[C::ZERO; 256]; 2];
        let mut hi = [[C::ZERO; 256]; 2];
        if n >= 4096 && smooth_high(keys) {
            // Smooth keys (`smooth_high`): 8 keys sharing their high byte add
            // 8 to its counter at once (the same counter otherwise waits on
            // its own previous store; Depth 5k-10k: Surface 25-28 %, Zen 2
            // 9 % faster). Idea: blazing-radix.
            let (oct, rest) = keys.as_chunks::<8>();
            for o in oct {
                for i in 0..4 {
                    lo[0][(o[2 * i] & 255) as usize].inc();
                    lo[1][(o[2 * i + 1] & 255) as usize].inc();
                }
                let h = o[0] >> 8;
                if o.iter().all(|&k| k >> 8 == h) {
                    let c = &mut hi[0][h as usize];
                    c.put(c.get() + 8);
                } else {
                    for i in 0..4 {
                        hi[0][(o[2 * i] >> 8) as usize].inc();
                        hi[1][(o[2 * i + 1] >> 8) as usize].inc();
                    }
                }
            }
            for &k in rest {
                lo[0][(k & 255) as usize].inc();
                hi[0][(k >> 8) as usize].inc();
            }
        } else {
            let (pairs, rest) = keys.as_chunks::<2>();
            for p in pairs {
                lo[0][(p[0] & 255) as usize].inc();
                hi[0][(p[0] >> 8) as usize].inc();
                lo[1][(p[1] & 255) as usize].inc();
                hi[1][(p[1] >> 8) as usize].inc();
            }
            for &k in rest {
                lo[0][(k & 255) as usize].inc();
                hi[0][(k >> 8) as usize].inc();
            }
        }
        let (mut a, mut b) = (lo[0], hi[0]);
        for d in 0..256 {
            a[d].put(a[d].get() + lo[1][d].get());
            b[d].put(b[d].get() + hi[1][d].get());
        }
        (a, b)
    };
    let first = keys[0];
    let lo_varies = lo[(first & 255) as usize].get() != n;
    let hi_varies = hi[(first >> 8) as usize].get() != n;
    match (lo_varies, hi_varies) {
        (false, false) => {},
        (true, false) => write_counts16(keys, &lo, first & 0xFF00, 0),
        (false, true) => write_counts16(keys, &hi, first & 0x00FF, 8),
        (true, true) => {
            // The high byte's buckets in key order (`F`: the sign bit flips
            // the top bit of that byte); the low byte's in their own.
            let prefix = |table: &mut [C; 256], rot: usize| {
                let mut sum = 0usize;
                for i in 0..256 {
                    let c = &mut table[i ^ rot];
                    let next = sum + c.get();
                    c.put(sum);
                    sum = next;
                }
            };
            prefix(&mut lo, 0);
            prefix(&mut hi, (F >> 8) as usize);
            let dst = buf.as_mut_ptr() as *mut u16;
            // SAFETY: `buf` holds `n` slots, all written by the first pass
            // (the heads are the prefix sums of the low byte over the keys),
            // then read as `[u16]` by the second; no overlap with `keys`.
            unsafe {
                if M != 0 {
                    scatter16::<C, 0, true>(keys, dst, &mut lo);
                } else {
                    scatter16::<C, 0, false>(keys, dst, &mut lo);
                }
                let mid = std::slice::from_raw_parts(dst as *const u16, n);
                // Mode 3: the high pass one key at a time too, a block of 4
                // sharing its byte in one update (idea: blazing-radix's
                // scatter): fewer instructions than the branch-free positions,
                // the branch predictable on varied keys (Unif 1k: 1.11 to 1.04
                // of blazing on Zen 2, 1.11 to 0.96 on Haswell; 31
                // distributions 1-2 % faster on Zen 2, 2-6 % on Haswell).
                if M == 1 || (M == 3 && seq_high_pass()) {
                    scatter16::<C, 8, true>(mid, keys.as_mut_ptr(), &mut hi);
                } else {
                    scatter16::<C, 8, false>(mid, keys.as_mut_ptr(), &mut hi);
                }
            }
        },
    }
}

/// Whether at least 60 of the first 64 neighbours share their high byte
/// (Depth: ~62; Runs 8: ~37 at 5k, ~57 at 20k, where the shared-byte test of 8 keys
/// mispredicts: 15-23 % slower on Zen 2).
fn smooth_high(keys: &[u16]) -> bool {
    let mut same = 0usize;
    for i in 0..63 {
        same += (keys[i] >> 8 == keys[i + 1] >> 8) as usize;
    }
    same >= 60
}

/// A byte constant on 16 spread keys (idea of the shared LSD): only the
/// other byte counted (2 lanes), every key ORed to check the first one; when
/// it is constant indeed, the keys are written from the counts and `true`
/// returned (else `false`, nothing changed: the full histograms follow).
/// Saves the increments chained on one counter (Small: high byte 0).
/// `LOW_ONLY`: only a constant low byte (keys 256 apart) is looked for.
fn flat_byte16<C: Count16, const LOW_ONLY: bool>(keys: &mut [u16]) -> bool {
    let n = keys.len();
    let first = keys[0];
    // Odd step: alternating keys (Bimodal) are not all sampled on one side.
    let diff = keys
        .iter()
        .step_by((n / 16).max(1) | 1)
        .take(16)
        .fold(0u16, |acc, &k| acc | (k ^ first));
    let shift = if !LOW_ONLY && diff >> 8 == 0 {
        0
    } else if diff & 255 == 0 {
        8
    } else {
        return false;
    };
    let other = 0xFFu16 << (8 - shift);
    // 4 lanes of 32-bit counters, written by 32-key stores (as the
    // window).
    let mut q = Lanes::<4>::new();
    let mut x = [0u16; 4];
    let (quads, rest) = keys.as_chunks::<4>();
    for k in quads {
        for l in 0..4 {
            q.add(l, ((k[l] >> shift) & 255) as usize);
            x[l] |= k[l] ^ first;
        }
    }
    for &k in rest {
        q.add(0, ((k >> shift) & 255) as usize);
        x[0] |= k ^ first;
    }
    if (x[0] | x[1] | x[2] | x[3]) & other != 0 {
        return false;
    }
    let counts = q.sum();
    if counts[((first >> shift) & 255) as usize] as usize != n {
        let rest = first & other;
        let rot = if shift == 8 { (F >> 8) as usize } else { 0 };
        let mut start = 0;
        for d0 in 0..256 {
            let d = d0 ^ rot;
            let cnt = counts[d] as usize;
            if cnt == 0 {
                continue;
            }
            let v = rest | ((d as u16) << shift);
            if cnt <= 32 && n - start >= 32 {
                keys[start..start + 32].copy_from_slice(&[v; 32]);
            } else if cnt > 32 && cnt <= 128 {
                let end = start + cnt;
                let mut o = start;
                while end - o > 32 {
                    keys[o..o + 32].copy_from_slice(&[v; 32]);
                    o += 32;
                }
                keys[end - 32..end].copy_from_slice(&[v; 32]);
            } else {
                keys[start..start + cnt].fill(v);
            }
            start += cnt;
        }
    }
    true
}

/// Keys with one varying byte (at `shift`, the other byte `rest` in every
/// key): written in order from the counts of that byte.
fn write_counts16<C: Count16>(keys: &mut [u16], counts: &[C; 256], rest: u16, shift: u32) {
    let n = keys.len();
    let rot = if shift == 8 { (F >> 8) as usize } else { 0 };
    let mut start = 0;
    for b0 in 0..256 {
        let b = b0 ^ rot;
        let c = counts[b].get();
        if c != 0 {
            let v = rest | ((b as u16) << shift);
            let end = start + c;
            if c <= 64 && end + 16 <= n {
                let block = [v; 16];
                let mut p = start;
                while p < end {
                    keys[p..p + 16].copy_from_slice(&block);
                    p += 16;
                }
            } else {
                keys[start..end].fill(v);
            }
            start = end;
        }
    }
}

/// One stable scatter pass on the byte at `SHIFT`: 4 keys read before being
/// written; positions resolved inside the block (each key adds the number
/// of previous keys of the block in its bucket: independent counter loads).
///
/// # Safety
/// `dst` valid for `src.len()` writes, not overlapping `src`; `heads` the
/// exclusive prefix sums of that byte over exactly the keys of `src`.
#[inline(always)]
unsafe fn scatter16<C: Count16, const SHIFT: u32, const SEQ: bool>(
    src: &[u16],
    dst: *mut u16,
    heads: &mut [C; 256],
) {
    let dig = |k: u16| ((k >> SHIFT) & 255) as usize;
    let (blocks, rest) = src.as_chunks::<4>();
    if SEQ {
        // One counter at a time (a block sharing its digit: one update).
        for b in blocks {
            let (a, bb, c, d) = (b[0], b[1], b[2], b[3]);
            let (da, db, dc, dd) = (dig(a), dig(bb), dig(c), dig(d));
            let (pa, pb, pc, pd);
            if da == dd && da == db && dc == dd {
                pa = heads[da].get();
                (pb, pc, pd) = (pa + 1, pa + 2, pa + 3);
                heads[da].put(pa + 4);
            } else {
                pa = heads[da].get();
                heads[da].put(pa + 1);
                pb = heads[db].get();
                heads[db].put(pb + 1);
                pc = heads[dc].get();
                heads[dc].put(pc + 1);
                pd = heads[dd].get();
                heads[dd].put(pd + 1);
            }
            // SAFETY: each position is a free slot of its bucket, below len.
            unsafe {
                dst.add(pa).write(a);
                dst.add(pb).write(bb);
                dst.add(pc).write(c);
                dst.add(pd).write(d);
            }
        }
        for &k in rest {
            let dk = dig(k);
            let p = heads[dk].get();
            heads[dk].put(p + 1);
            // SAFETY: as above.
            unsafe { dst.add(p).write(k) };
        }
        return;
    }
    for b in blocks {
        let (a, bb, c, d) = (b[0], b[1], b[2], b[3]);
        let (da, db, dc, dd) = (dig(a), dig(bb), dig(c), dig(d));
        let pa = heads[da].get();
        let pb = heads[db].get() + usize::from(db == da);
        let pc = heads[dc].get() + usize::from(dc == da) + usize::from(dc == db);
        let pd = heads[dd].get()
            + usize::from(dd == da)
            + usize::from(dd == db)
            + usize::from(dd == dc);
        heads[da].put(pa + 1);
        heads[db].put(pb + 1);
        heads[dc].put(pc + 1);
        heads[dd].put(pd + 1);
        // SAFETY: each position is a free slot of its bucket, below len.
        unsafe {
            dst.add(pa).write(a);
            dst.add(pb).write(bb);
            dst.add(pc).write(c);
            dst.add(pd).write(d);
        }
    }
    for &k in rest {
        let dk = dig(k);
        let p = heads[dk].get();
        heads[dk].put(p + 1);
        // SAFETY: as above.
        unsafe { dst.add(p).write(k) };
    }
}

/// The engine of one chunk (the whole array, or a gap between Verge's runs).
#[inline(always)]
fn sort_chunk(bits: &mut [u16], _: &mut Vec<MaybeUninit<u16>>) {
    // Two byte passes at most, always.
    trace!("lsd");
    // The domain count from 65 536 keys (one key per counter): 7-9 %
    // faster at 100k (Zen 2, Haswell, Surface).
    if bits.len() >= 1 << 16 {
        domain_count16(bits);
    } else if bits.len() <= NETWORK_MAX {
        // Short chunks: our small engines.
        network_sort(bits, $enc, $enc);
    } else if bits.len() <= MERGE_MAX {
        merge_sort(bits, $enc, $enc);
    } else if bits.len() <= short_lsd_max() {
        crate::v2::engines::lsd::short16::lsd_short::<F>(bits);
    } else if narrow_counters() {
        lsd16_mode::<3, false>(bits);
    } else {
        lsd16_mode::<3, true>(bits);
    }
}

/// Up to this size the short LSD (`short16`: one counting table unless
/// neighbours share their high byte, less fixed cost): Zen 2 from 300 to
/// 1000 keys 5-13 % faster on the random-like distributions, won in all 6
/// code placements, nothing lost; Surface below 700 keys (beyond, this
/// LSD's two tables are faster on repeated keys); Haswell neutral.
#[inline(always)]
fn short_lsd_max() -> usize { if cfg!(target_arch = "aarch64") { 600 } else { 1024 } }

/// Directions of 32 spread adjacent pairs (copy of `probe_pairs`, without a
/// branch: random keys mispredicted half of them, ~50 ns at 1100 keys on
/// Zen 2).
fn probe_pairs16(arr: &[u16]) -> crate::v2::inspect::presorted::Probes {
    let step = (arr.len() - 1) / 32;
    let (mut ascending, mut descending) = (0, 0);
    for k in 0..32 {
        let (a, b) = (ord(arr[k * step]), ord(arr[k * step + 1]));
        ascending += (a < b) as usize;
        descending += (a > b) as usize;
    }
    crate::v2::inspect::presorted::Probes { ascending, descending }
}

/// No monotone run of n / log2 n keys (copy of `can_skip_verge`, each grid
/// window checked without a branch per pair: random keys mispredicted
/// them, ~150 ns at 5000 keys on Zen 2).
fn can_skip_verge16(arr: &[u16]) -> bool {
    const PROBE: usize = 8;
    let n = arr.len();
    let step = crate::v2::runs::verge::compute_big_enough_run(n) / 2;
    if step <= PROBE {
        return false;
    }
    let mut p = 0;
    let mut monotone = false;
    while p + PROBE < n {
        let w = &arr[p..=p + PROBE];
        let (mut up, mut down) = (true, true);
        for i in 0..PROBE {
            up &= ord(w[i]) <= ord(w[i + 1]);
            down &= ord(w[i]) >= ord(w[i + 1]);
        }
        monotone |= up | down;
        p += step;
    }
    !monotone
}

/// Two sorted runs `[..mid]` and `[mid..]` merged without the keys already
/// in place: the head of the first run not above the second's first key,
/// the tail of the second not below the first's last key (Desc Saw at 70k:
/// a run of 4464 keys and one of 65 536, of which 61 072 stay put; blazing
/// 2 times faster).
fn merge_two_trimmed(arr: &mut [u16], mid: usize) {
    let (first, last) = (ord(arr[mid]), ord(arr[mid - 1]));
    if last <= first {
        return;
    }
    let lo = arr[..mid].partition_point(|&x| ord(x) <= first);
    let hi = mid + arr[mid..].partition_point(|&x| ord(x) < last);
    merge_runs(keys_of(&mut arr[lo..hi]), &mut vec![0, mid - lo, hi - lo]);
}

/// The radix path (copy of `decide::path`); the first matching
/// rule wins.
fn path(
    arr: &mut [u16],
    known: Unsorted,
    detect: &dyn Fn(&mut [u16], usize) -> bool,
    sort: &dyn Fn(&mut [u16]),
) {
    let n = arr.len();
    let p = known.ascending_prefix;
    chain! {
        // A sorted prefix of at least half the array: the tail alone goes
        // through the path, then one merge. A short tail: sorted (or
        // reversed) as it is, else the engine.
        "pipe" if pipe_direct() && tail_first(n, p) && pipe_merge(arr, p) => do {};
        "tail_first" if tail_first(n, p) => do {
            let tail = &mut arr[p..];
            if tail.len() >= TAIL_PATH_MIN {
                path(tail, Unsorted::default(), detect, sort);
            } else if presorted(keys_of(tail)).is_err() {
                sort(tail);
            }
            merge_runs(keys_of(arr), &mut vec![0, p, n]);
        };
    }
    let simd_rule = merge::simd_rule_applies::<K>(n);
    let random_runs = !simd_rule && starts_with_random_runs(&*keys_of(arr), p);
    let mut known = known;
    if known.probes.is_none() {
        known.probes = Some(probe_pairs16(arr));
    }
    // Mostly descending probes: no repair (it reverses the whole array,
    // inserts, gives up, and Verge redoes the work: Desc Saw at 70-100k
    // 0.24-0.45 to 0.14-0.39 ns per key on the Surface, = blazing; neutral
    // on Zen 2).
    let mostly_descending = known.probes.is_some_and(|p| {
        p.ascending <= 8 && (p.descending > p.ascending || (p.descending == p.ascending && ord(arr[0]) > ord(arr[n - 1])))
    });
    let (sorted, equal_pairs) = if random_runs || mostly_descending {
        (false, known.probes.map_or(0, |p| p.equal()))
    } else {
        repair_nearly_sorted(keys_of(arr), known)
    };
    if random_runs {
        trace!("random_runs");
    }
    chain! {
        "repaired" if sorted => do {};
        _ => try detect(arr, equal_pairs);
        "no_verge" if small_chain(n)
            || random_runs
            || can_skip_verge16(arr) => do {
            sort(arr)
        };
    }
    // Verge: the unsorted chunks are recorded, sorted after the merge rule.
    let base = arr.as_ptr() as usize;
    let chunks: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    let mut separators =
        verge_sort_preprocessing_fast::<_, false>(keys_of(arr), 8, &|chunk: &mut [K], _| {
            let start = (chunk.as_ptr() as usize - base) / std::mem::size_of::<K>();
            chunks.borrow_mut().push((start, chunk.len()));
        });
    let chunks = chunks.into_inner();
    // SIMD merges: merging beats the radix sort up to a few merge levels.
    let segments = separators.len().saturating_sub(1);
    let levels = (usize::BITS - segments.saturating_sub(1).leading_zeros()) as usize;
    let simd_merge = simd_rule && levels <= merge::SIMD_LEVELS;
    let unpredictable = merges_unpredictable(&*keys_of(arr), &separators, &chunks, levels);
    let sort_chunks = |arr: &mut [u16]| {
        for &(start, len) in chunks.iter() {
            sort(&mut arr[start..start + len]);
        }
    };
    chain! {
        "verge_whole" if unpredictable && !simd_merge => do { sort(arr) };
        "verge_simd_merge" if unpredictable => do {
            sort_chunks(arr);
            merge::merge_runs_simd(keys_of(arr), &mut separators);
        };
        "verge_merge" => do {
            sort_chunks(arr);
            if separators.len() == 3 {
                merge_two_trimmed(arr, separators[1]);
            } else {
                merge_runs(keys_of(arr), &mut separators);
            }
        };
    }
}

/// The direct merge of `pipe_merge` on x86 (Zen 2: Pipe organ 10-25 %
/// faster below 1000 keys, 2 % beyond); ARM keeps the vector merges of
/// `merge_runs` (3 times faster there).
#[inline(always)]
fn pipe_direct() -> bool { cfg!(not(target_arch = "aarch64")) }

/// An ascending run of `p` keys followed by a
/// descending one to the end (Pipe organ): merged straight into a buffer,
/// the descending run read from its end (no reversal), from both ends at
/// once (smallest keys from the front, largest from the back: two
/// independent chains), then copied back. `false`, nothing changed, when
/// the tail is not descending.
fn pipe_merge(arr: &mut [u16], p: usize) -> bool {
    let n = arr.len();
    if p == 0 || p >= n || scan16::<true>(&arr[p..]) != n - p {
        return false;
    }
    let mut buf: Vec<MaybeUninit<u16>> = Vec::with_capacity(n);
    // SAFETY: MaybeUninit needs no initialization; every slot is written
    // below before the copy back.
    unsafe { buf.set_len(n) };
    let (a, b) = arr.split_at(p);
    // Front: a[i] ascending, b[j] descending read from its end.
    let (mut i, mut j) = (0usize, b.len());
    // Back: a[ie - 1] its largest left, b[jb] the largest left of b.
    let (mut ie, mut jb) = (a.len(), 0usize);
    let half = n / 2;
    let (ap, bp, out) = (a.as_ptr(), b.as_ptr(), buf.as_mut_ptr() as *mut u16);
    let (al, bl) = (a.len(), b.len());
    // Compared as order keys (`ord`), written back as bits (`^ F`).
    // SAFETY: each read is guarded by its index test (i < al, j > 0,
    // ie > 0, jb < bl); k < half and n - 1 - k >= half, both below n.
    unsafe {
        for k in 0..half {
            let x = if i < al { ord(*ap.add(i)) } else { u16::MAX };
            let y = if j > 0 { ord(*bp.add(j - 1)) } else { u16::MAX };
            let take_a = x <= y;
            *out.add(k) = (if take_a { x } else { y }) ^ F;
            i += take_a as usize;
            j -= (!take_a) as usize;
            let xa = if ie > 0 { ord(*ap.add(ie - 1)) } else { 0 };
            let yb = if jb < bl { ord(*bp.add(jb)) } else { 0 };
            let take_ab = xa > yb;
            *out.add(n - 1 - k) = (if take_ab { xa } else { yb }) ^ F;
            ie -= take_ab as usize;
            jb += (!take_ab) as usize;
        }
    }
    if n % 2 == 1 {
        let x = if i < al { ord(a[i]) } else { u16::MAX };
        let y = if j > 0 { ord(b[j - 1]) } else { u16::MAX };
        // SAFETY: half < n.
        unsafe { *out.add(half) = x.min(y) ^ F };
    }
    // SAFETY: the n slots of `buf` are written; `buf` and `arr` are distinct.
    unsafe { std::ptr::copy_nonoverlapping(out, arr.as_mut_ptr(), n) };
    true
}

/// Tails from this length go through the whole path (repair, detectors);
/// shorter ones go to the engine.
const TAIL_PATH_MIN: usize = 4096;

/// A sorted prefix of `p` keys out of `n`: the tail-first path.
fn tail_first(n: usize, p: usize) -> bool { p >= n / 2 && p < n }

}
    };
}

narrow16!(nu16, u16, 0, id16);
narrow16!(ni16, i16, 0x8000, flip16);
