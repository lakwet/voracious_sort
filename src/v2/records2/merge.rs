//! Stable merges of sorted runs of records.

use std::mem::MaybeUninit;

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer the size of the
/// shorter one: each element moved once (the shorter run copied out, the
/// merge writing from the end or the start into the array).
/// Merges of two runs: the elements already beyond the other run moved as
/// one block (binary search) before merging the rest.
/// The buffers of `merge_two` and `merge_asc_desc` prefaulted too.
pub(crate) const PREFAULT2: bool = true;
/// Pipe organs: the descending run reversed in place, then `merge_out`.
pub(crate) const ORGAN2: bool = false;

/// Merge of `arr[..m]` and `arr[m..]` into a buffer with the four-chain
/// kernel, then copied back (two moves, no branch).
pub(super) fn merge_out<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E) {
    let n = arr.len();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    prefault(&mut buf);
    // SAFETY: every element is moved once into the buffer by the merge,
    // then the buffer copied back over the array.
    unsafe {
        let b = buf.as_mut_ptr() as *mut T;
        merge_into_p4(arr.as_ptr(), b, 0, m, n, enc);
        std::ptr::copy_nonoverlapping(b, arr.as_mut_ptr(), n);
    }
}

/// Experiment: the k-way buffer kept on the thread between sorts.
pub(crate) const REUSE: bool = false;
/// Experiment: unpredictable merges through half a buffer (as glidesort).
pub(crate) const HALF_KWAY: bool = false;

/// A scratch block of bytes, aligned for any record (64), prefaulted.
pub(super) struct Scratch {
    pub(super) ptr: *mut u8,
    bytes: usize,
}

impl Scratch {
    const ALIGN: usize = 64;
    fn take(&mut self) -> Scratch {
        std::mem::replace(self, Scratch { ptr: std::ptr::null_mut(), bytes: 0 })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if self.bytes > 0 {
            // SAFETY: allocated by `take_scratch` with this layout.
            unsafe { std::alloc::dealloc(self.ptr, std::alloc::Layout::from_size_align_unchecked(self.bytes, Self::ALIGN)) };
        }
    }
}

thread_local! {
    static SCRATCH: std::cell::Cell<Option<Scratch>> = const { std::cell::Cell::new(None) };
}

/// The thread's scratch block if large enough (else a new prefaulted one).
pub(super) fn take_scratch(bytes: usize) -> Scratch {
    let bytes = bytes.max(Scratch::ALIGN);
    if let Some(s) = SCRATCH.with(|c| c.take()) {
        if s.bytes >= bytes {
            return s;
        }
    }
    // SAFETY: non-zero size, valid alignment; the bytes written once.
    unsafe {
        let layout = std::alloc::Layout::from_size_align_unchecked(bytes, Scratch::ALIGN);
        let ptr = std::alloc::alloc(layout);
        assert!(!ptr.is_null());
        std::ptr::write_bytes(ptr, 0, bytes);
        Scratch { ptr, bytes }
    }
}

/// Gives the block back to the thread (the larger one kept).
pub(super) fn give_scratch(s: Scratch) {
    SCRATCH.with(|c| {
        let old = c.take();
        let keep = match old {
            Some(o) if o.bytes >= s.bytes => o,
            _ => s,
        };
        c.set(Some(keep));
    });
}

pub(crate) const TRIM: bool = true;

pub(super) fn merge_two<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E, branchy: bool) {
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(m.min(arr.len() - m));
    if PREFAULT2 {
        prefault(&mut buf);
    }
    merge_two_with(arr, m, enc, &mut buf, branchy);
}

/// `merge_two` with a buffer of at least the shorter run (capacity).
/// `branchy`: branches on the comparison (faster when the merge is
/// predictable), else selects without branches.
pub(super) fn merge_two_with<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E, buf: &mut Vec<MaybeUninit<T>>, branchy: bool) {
    // Balanced runs only: a short run merged into a long one is mostly
    // block moves, which the one-chain merge with its trim does as well.
    if X2 && !branchy && 16 * m.min(arr.len() - m) >= arr.len() {
        return merge_two_x(arr, m, enc, buf);
    }
    if branchy {
        merge_two_body::<T, E, true>(arr, m, enc, buf)
    } else {
        merge_two_body::<T, E, false>(arr, m, enc, buf)
    }
}

#[inline(always)]
fn merge_two_body<T, E: Fn(&T) -> u128, const BRANCHY: bool>(arr: &mut [T], m: usize, enc: &E, buf: &mut Vec<MaybeUninit<T>>) {
    let n = arr.len();
    if m == 0 || m == n || enc(&arr[m - 1]) <= enc(&arr[m]) {
        return;
    }
    let a = arr.as_mut_ptr();
    // SAFETY: the shorter run is moved out to the buffer; the merge then
    // fills the array from the side where the moved-out run was, never
    // overtaking the unread elements of the other run (the write position
    // is always at least as far from that side as the read position), and
    // every element is moved once; nothing is dropped or duplicated. `enc`
    // already ran on every element.
    unsafe {
        if n - m <= m {
            // Right run out; merge from the end.
            let len = n - m;
            debug_assert!(buf.capacity() >= len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a.add(m), b, len);
            // The left keys above every right key move as one block.
            let top = enc(&*b.add(len - 1));
            let p = if TRIM {
                std::slice::from_raw_parts(a, m).partition_point(|x| enc(x) <= top)
            } else {
                m
            };
            std::ptr::copy(a.add(p), a.add(p + len), m - p);
            let (mut i, mut j, mut k) = (p, len, p + len);
            while i > 0 && j > 0 {
                // Stable: on equal keys the right element goes last.
                let take_left = enc(&*a.add(i - 1)) > enc(&*b.add(j - 1));
                k -= 1;
                if BRANCHY {
                    if take_left {
                        i -= 1;
                        a.add(k).write(std::ptr::read(a.add(i)));
                    } else {
                        j -= 1;
                        a.add(k).write(std::ptr::read(b.add(j)));
                    }
                    continue;
                }
                let from = pick(take_left, a.add(i - 1) as *const T, b.add(j - 1) as *const T);
                a.add(k).write(std::ptr::read(from));
                i -= take_left as usize;
                j -= (!take_left) as usize;
            }
            std::ptr::copy_nonoverlapping(b, a.add(k - j), j);
        } else {
            // Left run out; merge from the start.
            let len = m;
            debug_assert!(buf.capacity() >= len);
            let b = buf.as_mut_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(a, b, len);
            // The right keys below every left key move as one block.
            let bottom = enc(&*b);
            let q = if TRIM {
                m + std::slice::from_raw_parts(a.add(m), n - m).partition_point(|x| enc(x) < bottom)
            } else {
                m
            };
            std::ptr::copy(a.add(m), a, q - m);
            let (mut i, mut j, mut k) = (0, q, q - m);
            while i < len && j < n {
                // Stable: on equal keys the left element goes first.
                let take_right = enc(&*a.add(j)) < enc(&*b.add(i));
                if BRANCHY {
                    if take_right {
                        a.add(k).write(std::ptr::read(a.add(j)));
                        j += 1;
                    } else {
                        a.add(k).write(std::ptr::read(b.add(i)));
                        i += 1;
                    }
                    k += 1;
                    continue;
                }
                let from = pick(take_right, a.add(j) as *const T, b.add(i) as *const T);
                a.add(k).write(std::ptr::read(from));
                j += take_right as usize;
                i += (!take_right) as usize;
                k += 1;
            }
            std::ptr::copy_nonoverlapping(b.add(i), a.add(k), len - i);
        }
    }
}

/// Stable merge of the sorted segments `bounds[i]..bounds[i + 1]`, pairs of
/// neighbours per round, alternating between the array and a buffer.
pub(super) fn merge_all<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E, branchy: bool) {
    let n = arr.len();
    if bounds.len() <= 2 {
        return;
    }
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    prefault(&mut buf);
    let mut in_buf = false;
    let mut t = std::time::Instant::now();
    // SAFETY: each round moves every element exactly once from one side to
    // the other (each segment pair merged or copied into the same range);
    // at the end the elements are copied back if they are in the buffer.
    // Nothing is dropped or duplicated; `enc` already ran on every element.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        while bounds.len() > 2 {
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            let mut next = vec![0];
            let mut i = 0;
            while i + 2 < bounds.len() {
                let (s, m, e) = (bounds[i], bounds[i + 1], bounds[i + 2]);
                if branchy {
                    merge_into_b(src, dst, s, m, e, enc);
                } else {
                    merge_into(src, dst, s, m, e, enc);
                }
                next.push(e);
                i += 2;
            }
            if i + 1 < bounds.len() {
                let (s, e) = (bounds[i], bounds[i + 1]);
                std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
                next.push(e);
            }
            bounds = next;
            in_buf = !in_buf;
            if super::verge::TRACE {
                eprintln!("  full round {:.2} ns", t.elapsed().as_nanos() as f64 / n as f64);
                t = std::time::Instant::now();
            }
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
        if super::verge::TRACE {
            eprintln!("  copy back {} {:.2} ns", in_buf, t.elapsed().as_nanos() as f64 / n as f64);
        }
    }
}

/// Stable merge of `src[s..m]` and `src[m..e]` into `dst[s..e]`, from both
/// ends at once without branches (two independent chains of loads and
/// selects, as `pipeline::merge::bidir`): as many two-ended steps as the
/// shorter run has elements, then the middle merged forward.
///
/// # Safety
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
#[inline(never)]
unsafe fn merge_into<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let steps = (m - s).min(e - m);
    // Left run [lf, le), right run [rf, re); output [df, de).
    let (mut lf, mut le, mut rf, mut re, mut df, mut de) = (s, m, m, e, s, e);
    for _ in 0..steps {
        let right = enc(&*src.add(rf)) < enc(&*src.add(lf));
        let from = if right { rf } else { lf };
        dst.add(df).write(std::ptr::read(src.add(from)));
        rf += right as usize;
        lf += (!right) as usize;
        df += 1;
        let left = enc(&*src.add(re - 1)) < enc(&*src.add(le - 1));
        let from = if left { le - 1 } else { re - 1 };
        de -= 1;
        dst.add(de).write(std::ptr::read(src.add(from)));
        le -= left as usize;
        re -= (!left) as usize;
    }
    while lf < le && rf < re {
        let right = enc(&*src.add(rf)) < enc(&*src.add(lf));
        let from = if right { rf } else { lf };
        dst.add(df).write(std::ptr::read(src.add(from)));
        rf += right as usize;
        lf += (!right) as usize;
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
    debug_assert_eq!(df + (re - rf), de);
}

/// Stable merge of the sorted segments `bounds[i]..bounds[i + 1]`, pairs of
/// neighbours per round, each merged in the array through one buffer of
/// half the array (`merge_two_with`): half the memory of `merge_all`.
pub(super) fn merge_all_half<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E, branchy: bool) {
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(arr.len() / 2 + 1);
    if HALF_KWAY {
        prefault(&mut buf);
    }
    if super::verge::TRACE {
        let t = std::time::Instant::now();
        // SAFETY: bytes of the spare capacity, written only.
        unsafe { std::ptr::write_bytes(buf.as_mut_ptr() as *mut u8, 0, buf.capacity() * std::mem::size_of::<T>()) };
        eprintln!("  fault-in half buffer {:.2} ns per element", t.elapsed().as_nanos() as f64 / arr.len() as f64);
    }
    while bounds.len() > 2 {
        let mut next = vec![0];
        let mut i = 0;
        while i + 2 < bounds.len() {
            let (s, m, e) = (bounds[i], bounds[i + 1], bounds[i + 2]);
            merge_two_with(&mut arr[s..e], m - s, enc, &mut buf, branchy);
            next.push(e);
            i += 2;
        }
        if i + 1 < bounds.len() {
            next.push(bounds[i + 1]);
        }
        bounds = next;
    }
}

/// Stable merge of the sorted segments, four at a time (an idea of
/// quadsort, also in glidesort): segments 1+2 and 3+4 merged into the
/// buffer (the two merges interleaved: four independent chains), then the
/// two results merged back into the array. Two moves per element for two
/// levels, no copy back; a last pair merged through the buffer (`merge_two`).
/// Buffer of n elements.
pub(super) fn merge_all_quad<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E) {
    let n = arr.len();
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    while bounds.len() > 3 {
        let mut next = vec![0];
        let mut i = 0;
        // SAFETY: each group of segments is moved from the array into the
        // same range of the buffer and back (each element once per move);
        // a segment left alone stays in the array. `enc` already ran.
        unsafe {
            let a = arr.as_mut_ptr();
            let b = buf.as_mut_ptr() as *mut T;
            while i + 1 < bounds.len() {
                let left = bounds.len() - 1 - i;
                if left >= 4 {
                    let (s0, s1, s2, s3, s4) = (bounds[i], bounds[i + 1], bounds[i + 2], bounds[i + 3], bounds[i + 4]);
                    merge_into_pair(a, b, (s0, s1, s2), (s2, s3, s4), enc);
                    merge_into(b, a, s0, s2, s4, enc);
                    next.push(s4);
                    i += 4;
                } else if left == 3 {
                    let (s0, s1, s2, s3) = (bounds[i], bounds[i + 1], bounds[i + 2], bounds[i + 3]);
                    merge_into(a, b, s0, s1, s2, enc);
                    std::ptr::copy_nonoverlapping(a.add(s2), b.add(s2), s3 - s2);
                    merge_into(b, a, s0, s2, s3, enc);
                    next.push(s3);
                    i += 3;
                } else {
                    // One or two segments: kept for the next round.
                    for k in 1..=left {
                        next.push(bounds[i + k]);
                    }
                    i += left;
                }
            }
        }
        bounds = next;
    }
    if bounds.len() == 3 {
        merge_two_with(arr, bounds[1], enc, &mut buf, false);
    }
}

/// Two independent merges `src[x.0..x.1] + src[x.1..x.2]` and the same for
/// `y`, into the same ranges of `dst`, interleaved step by step (each from
/// both ends: four independent chains), the rest finished one by one.
///
/// # Safety
/// As `merge_into`, for both merges.
#[inline(never)]
unsafe fn merge_into_pair<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, x: (usize, usize, usize), y: (usize, usize, usize), enc: &E) {
    // A merge from both ends: (lf, le, rf, re, df, de).
    struct M {
        lf: usize,
        le: usize,
        rf: usize,
        re: usize,
        df: usize,
        de: usize,
    }
    let init = |(s, m, e): (usize, usize, usize)| M { lf: s, le: m, rf: m, re: e, df: s, de: e };
    let (mut p, mut q) = (init(x), init(y));
    #[inline(always)]
    unsafe fn step<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, g: &mut M, enc: &E) {
        let right = enc(&*src.add(g.rf)) < enc(&*src.add(g.lf));
        let from = if right { g.rf } else { g.lf };
        dst.add(g.df).write(std::ptr::read(src.add(from)));
        g.rf += right as usize;
        g.lf += (!right) as usize;
        g.df += 1;
        let left = enc(&*src.add(g.re - 1)) < enc(&*src.add(g.le - 1));
        let from = if left { g.le - 1 } else { g.re - 1 };
        g.de -= 1;
        dst.add(g.de).write(std::ptr::read(src.add(from)));
        g.le -= left as usize;
        g.re -= (!left) as usize;
    }
    // Two-ended steps stay inside the runs for min(left, right) steps.
    let sp = (x.1 - x.0).min(x.2 - x.1);
    let sq = (y.1 - y.0).min(y.2 - y.1);
    for _ in 0..sp.min(sq) {
        step(src, dst, &mut p, enc);
        step(src, dst, &mut q, enc);
    }
    for _ in sp.min(sq)..sp {
        step(src, dst, &mut p, enc);
    }
    for _ in sp.min(sq)..sq {
        step(src, dst, &mut q, enc);
    }
    for g in [&mut p, &mut q] {
        while g.lf < g.le && g.rf < g.re {
            let right = enc(&*src.add(g.rf)) < enc(&*src.add(g.lf));
            let from = if right { g.rf } else { g.lf };
            dst.add(g.df).write(std::ptr::read(src.add(from)));
            g.rf += right as usize;
            g.lf += (!right) as usize;
            g.df += 1;
        }
        std::ptr::copy_nonoverlapping(src.add(g.lf), dst.add(g.df), g.le - g.lf);
        g.df += g.le - g.lf;
        std::ptr::copy_nonoverlapping(src.add(g.rf), dst.add(g.df), g.re - g.rf);
    }
}

/// Stable merge of the sorted segments with as few moves as possible:
/// rounds of 4-way merges (two levels in one move) and 2-way merges, an
/// even number of rounds so that the result lands in the array (no copy
/// back). Runs 8: one 4-way and one 2-way round (two moves; pairwise
/// rounds through half a buffer: 4.5). Buffer of n elements.
/// k-way merges as rounds of 2-way merges with the four-chain kernel.
/// 4-way rounds with the masked kernel (records of 32 bytes and more).
/// CPU parameter: records of at least this size merged by 4-way rounds
/// (one move per two levels), smaller ones by 2-way rounds (more moves,
/// cheaper steps). Where the line falls depends on memory bandwidth against
/// the latency of a merge step: 32 on the Snapdragon X (2-way four-chain
/// 0.7-1.3 ns per level, 4-way 1.9-2.3 per two levels, 128 B 5.0 against
/// 10.4); to be measured on x86 (bench-cpu).
pub(crate) static FOUR_WAY_MIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(32);
pub(crate) const K4M: bool = true;
pub(crate) const KWAY2: bool = true;

pub(super) fn merge_all_kway<T, E: Fn(&T) -> u128>(arr: &mut [T], bounds: &[usize], enc: &E, branchy: bool) {
    let n = arr.len();
    let segs = bounds.len() - 1;
    if segs <= 1 {
        return;
    }
    if segs == 2 {
        merge_two(arr, bounds[1], enc, false);
        return;
    }
    let levels = (usize::BITS - (segs - 1).leading_zeros()) as usize;
    let k4m = K4M;
    let two = KWAY2
        && std::mem::size_of::<T>() < if k4m { FOUR_WAY_MIN.load(std::sync::atomic::Ordering::Relaxed) } else { 128 };
    let mut rounds = levels.div_ceil(2);
    rounds += rounds % 2;
    let fours = levels.saturating_sub(rounds);
    let (rounds, fours) = if two { (levels, 0) } else { (rounds, fours) };
    let mut bounds = bounds.to_vec();
    let mut own: Vec<MaybeUninit<T>> = Vec::new();
    let mut held = None;
    let b: *mut T = if REUSE {
        let mut s = take_scratch(n * std::mem::size_of::<T>());
        let p = s.ptr as *mut T;
        held = Some(s.take());
        p
    } else {
        own.reserve_exact(n);
        prefault(&mut own);
        own.as_mut_ptr() as *mut T
    };
    let mut in_buf = false;
    let mut t = std::time::Instant::now();
    // SAFETY: each round moves every element once from one side to the
    // same range of the other (groups merged or copied); at the end the
    // elements are in the array (even number of rounds, or copied back).
    unsafe {
        let a = arr.as_mut_ptr();
        for round in 0..rounds {
            if bounds.len() <= 2 && !in_buf {
                break;
            }
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            let k = if round < fours { 4 } else { 2 };
            let mut next = vec![0];
            let mut i = 0;
            while i + 1 < bounds.len() {
                let g = (bounds.len() - 1 - i).min(k);
                let r = |j: usize| (bounds[i + j], bounds[i + j + 1]);
                match g {
                    4 if k4m => merge4m(src, dst, [r(0), r(1), r(2), r(3)], bounds[i], enc),
                    4 => merge4b(src, dst, [r(0), r(1), r(2), r(3)], bounds[i], enc),
                    3 => merge3(src, dst, [r(0), r(1), r(2)], bounds[i], enc),
                    2 if two && branchy => merge_into_b(src, dst, bounds[i], bounds[i + 1], bounds[i + 2], enc),
                    2 if two || k4m => merge_into_p4(src, dst, bounds[i], bounds[i + 1], bounds[i + 2], enc),
                    2 => merge_into(src, dst, bounds[i], bounds[i + 1], bounds[i + 2], enc),
                    _ => std::ptr::copy_nonoverlapping(src.add(bounds[i]), dst.add(bounds[i]), bounds[i + 1] - bounds[i]),
                }
                i += g;
                next.push(bounds[i]);
            }
            bounds = next;
            in_buf = !in_buf;
            if super::verge::TRACE {
                eprintln!("  round {} k {} {:.2} ns", round, k, t.elapsed().as_nanos() as f64 / n as f64);
                t = std::time::Instant::now();
            }
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
    if let Some(s) = held {
        give_scratch(s);
    }
}

/// Index of the smallest of four keys, the first on ties (stable).
#[inline(always)]
fn argmin4(k: [u128; 4]) -> usize {
    let ab = (k[1] < k[0]) as usize;
    let kab = if ab == 1 { k[1] } else { k[0] };
    let cd = (k[3] < k[2]) as usize;
    let kcd = if cd == 1 { k[3] } else { k[2] };
    if kcd < kab {
        2 + cd
    } else {
        ab
    }
}

/// 4-way stable merge of the consecutive runs `r` of `src` into the same
/// range of `dst`: chunks of steps during which no run can run out (the
/// shortest remaining length), then the three left merged by `merge3`.
///
/// # Safety
/// As `merge_into`.
#[inline(never)]
unsafe fn merge4f<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, r: [(usize, usize); 4], d0: usize, enc: &E) {
    let mut i = [r[0].0, r[1].0, r[2].0, r[3].0];
    let e = [r[0].1, r[1].1, r[2].1, r[3].1];
    let mut d = d0;
    loop {
        let safe = (e[0] - i[0]).min(e[1] - i[1]).min(e[2] - i[2]).min(e[3] - i[3]);
        if safe == 0 {
            break;
        }
        for _ in 0..safe {
            let k = [enc(&*src.add(i[0])), enc(&*src.add(i[1])), enc(&*src.add(i[2])), enc(&*src.add(i[3]))];
            let m = argmin4(k);
            dst.add(d).write(std::ptr::read(src.add(i[m])));
            d += 1;
            i[m] += 1;
        }
    }
    let mut rest = [(0, 0); 3];
    let mut c = 0;
    for j in 0..4 {
        if i[j] < e[j] {
            rest[c] = (i[j], e[j]);
            c += 1;
        }
    }
    merge_rest(src, dst, &rest[..c], d, enc);
}

/// 3-way stable merge (runs in order) into `dst` from `d0`.
///
/// # Safety
/// As `merge_into`; `dst[d0..]` has room for all the runs.
#[inline(never)]
unsafe fn merge3<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, r: [(usize, usize); 3], d0: usize, enc: &E) {
    let mut i = [r[0].0, r[1].0, r[2].0];
    let e = [r[0].1, r[1].1, r[2].1];
    let mut d = d0;
    loop {
        let safe = (e[0] - i[0]).min(e[1] - i[1]).min(e[2] - i[2]);
        if safe == 0 {
            break;
        }
        for _ in 0..safe {
            let (k0, k1, k2) = (enc(&*src.add(i[0])), enc(&*src.add(i[1])), enc(&*src.add(i[2])));
            let a = (k1 < k0) as usize;
            let ka = if a == 1 { k1 } else { k0 };
            let m = if k2 < ka { 2 } else { a };
            dst.add(d).write(std::ptr::read(src.add(i[m])));
            d += 1;
            i[m] += 1;
        }
    }
    let mut rest = [(0, 0); 2];
    let mut c = 0;
    for j in 0..3 {
        if i[j] < e[j] {
            rest[c] = (i[j], e[j]);
            c += 1;
        }
    }
    merge_rest(src, dst, &rest[..c], d, enc);
}

/// What is left of a k-way merge: at most three runs (in order), merged
/// into `dst` from `d`.
///
/// # Safety
/// As `merge3`.
unsafe fn merge_rest<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, rest: &[(usize, usize)], d: usize, enc: &E) {
    match rest.len() {
        3 => merge3(src, dst, [rest[0], rest[1], rest[2]], d, enc),
        2 => merge2r(src, dst, rest[0], rest[1], d, enc),
        1 => std::ptr::copy_nonoverlapping(src.add(rest[0].0), dst.add(d), rest[0].1 - rest[0].0),
        _ => {}
    }
}

/// Stable merge of two runs of `src` (`x` before `y`, not necessarily
/// adjacent) into `dst` from `d`, from both ends at once.
///
/// # Safety
/// As `merge3`.
#[inline(never)]
unsafe fn merge2r<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, x: (usize, usize), y: (usize, usize), d: usize, enc: &E) {
    let (mut lf, mut le, mut rf, mut re) = (x.0, x.1, y.0, y.1);
    let (mut df, mut de) = (d, d + (x.1 - x.0) + (y.1 - y.0));
    let steps = (x.1 - x.0).min(y.1 - y.0);
    for _ in 0..steps {
        let right = enc(&*src.add(rf)) < enc(&*src.add(lf));
        let from = if right { rf } else { lf };
        dst.add(df).write(std::ptr::read(src.add(from)));
        rf += right as usize;
        lf += (!right) as usize;
        df += 1;
        let left = enc(&*src.add(re - 1)) < enc(&*src.add(le - 1));
        let from = if left { le - 1 } else { re - 1 };
        de -= 1;
        dst.add(de).write(std::ptr::read(src.add(from)));
        le -= left as usize;
        re -= (!left) as usize;
    }
    while lf < le && rf < re {
        let right = enc(&*src.add(rf)) < enc(&*src.add(lf));
        let from = if right { rf } else { lf };
        dst.add(df).write(std::ptr::read(src.add(from)));
        rf += right as usize;
        lf += (!right) as usize;
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
}

/// 4-way stable merge of the runs `r` of `src` into `dst` from `d0`, from
/// both ends at once (two independent chains), indices in registers. Each
/// chunk runs as many two-ended steps as half the shortest run (no run can
/// run out from either end), then the middles are merged forward.
///
/// # Safety
/// As `merge_into`; `dst[d0..]` has room for all the runs.
#[inline(never)]
unsafe fn merge4b<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, r: [(usize, usize); 4], d0: usize, enc: &E) {
    let (mut f0, mut f1, mut f2, mut f3) = (r[0].0, r[1].0, r[2].0, r[3].0);
    let (mut b0, mut b1, mut b2, mut b3) = (r[0].1, r[1].1, r[2].1, r[3].1);
    let mut df = d0;
    let mut db = d0 + (b0 - f0) + (b1 - f1) + (b2 - f2) + (b3 - f3);
    let k = |i: usize| enc(&*src.add(i));
    loop {
        let safe = (b0 - f0).min(b1 - f1).min(b2 - f2).min(b3 - f3) / 2;
        if safe == 0 {
            break;
        }
        for _ in 0..safe {
            // Front: the smallest head, the first run on ties.
            let (k0, k1, k2, k3) = (k(f0), k(f1), k(f2), k(f3));
            let ab = k1 < k0;
            let (kab, iab) = if ab { (k1, f1) } else { (k0, f0) };
            let cd = k3 < k2;
            let (kcd, icd) = if cd { (k3, f3) } else { (k2, f2) };
            let right = kcd < kab;
            let from = if right { icd } else { iab };
            dst.add(df).write(std::ptr::read(src.add(from)));
            df += 1;
            f0 += (!right && !ab) as usize;
            f1 += (!right && ab) as usize;
            f2 += (right && !cd) as usize;
            f3 += (right && cd) as usize;
            // Back: the largest tail, the last run on ties.
            let (k0, k1, k2, k3) = (k(b0 - 1), k(b1 - 1), k(b2 - 1), k(b3 - 1));
            let ab = k1 >= k0;
            let (kab, iab) = if ab { (k1, b1 - 1) } else { (k0, b0 - 1) };
            let cd = k3 >= k2;
            let (kcd, icd) = if cd { (k3, b3 - 1) } else { (k2, b2 - 1) };
            let right = kcd >= kab;
            let from = if right { icd } else { iab };
            db -= 1;
            dst.add(db).write(std::ptr::read(src.add(from)));
            b0 -= (!right && !ab) as usize;
            b1 -= (!right && ab) as usize;
            b2 -= (right && !cd) as usize;
            b3 -= (right && cd) as usize;
        }
    }
    let mut rest = [(0, 0); 4];
    let mut c = 0;
    for (f, b) in [(f0, b0), (f1, b1), (f2, b2), (f3, b3)] {
        if f < b {
            rest[c] = (f, b);
            c += 1;
        }
    }
    match c {
        4 => merge4f(src, dst, [rest[0], rest[1], rest[2], rest[3]], df, enc),
        _ => merge_rest(src, dst, &rest[..c], df, enc),
    }
}

/// Touches every page of the buffer's capacity before the merges: the page
/// faults of a fresh allocation cost about twice as much when they happen
/// in the middle of a merge (32-byte records at 1M: 7.0 ns for the first
/// round against 3.0 for zeroing + 1.4 for the round).
pub(super) fn prefault<T>(buf: &mut Vec<MaybeUninit<T>>) {
    if !PREFAULT {
        return;
    }
    // SAFETY: bytes of the spare capacity, written only.
    // WATCH: these bytes are never read, so LLVM may drop the fill when the
    // surrounding code changes (seen once after a refactor of
    // `merge_all_kway`: 0.01 ns per element instead of 0.74, and the faults
    // back in the first merge round; journal §71). If the "buffer" trace of
    // `TRACE` falls near zero, write one byte per page with `write_volatile`.
    unsafe { std::ptr::write_bytes(buf.as_mut_ptr() as *mut u8, 0, buf.capacity() * std::mem::size_of::<T>()) };
}

pub(crate) const PREFAULT: bool = true;

/// `merge_into` with branches (both ends): faster when the merge is
/// predictable (long stretches from one run, or a short period).
///
/// # Safety
/// As `merge_into`.
#[inline(never)]
unsafe fn merge_into_b<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let steps = (m - s).min(e - m);
    let (mut lf, mut le, mut rf, mut re, mut df, mut de) = (s, m, m, e, s, e);
    for _ in 0..steps {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
        de -= 1;
        if enc(&*src.add(re - 1)) < enc(&*src.add(le - 1)) {
            le -= 1;
            dst.add(de).write(std::ptr::read(src.add(le)));
        } else {
            re -= 1;
            dst.add(de).write(std::ptr::read(src.add(re)));
        }
    }
    while lf < le && rf < re {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
}

/// Stable merge of the ascending run `arr[..m]` and the strictly descending
/// run `arr[m..]`: the descending run copied reversed into a buffer, then
/// the two merged from the back into the array (the write position never
/// passes the unread part of the first run). Branches: pipe organs
/// interleave regularly.
pub(super) fn merge_asc_desc<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E) {
    let n = arr.len();
    if ORGAN2 {
        arr[m..].reverse();
        merge_out(arr, m, enc);
        return;
    }
    let len = n - m;
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(len);
    if PREFAULT2 {
        prefault(&mut buf);
    }
    // SAFETY: the descending run is moved out reversed; each later write
    // lands on a slot already read (the back of the array first holds the
    // moved-out run, then slots of the first run already taken); every
    // element is moved once more. `enc` already ran on every element.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        for k in 0..len {
            b.add(k).write(std::ptr::read(a.add(n - 1 - k)));
        }
        if X2 {
            // The descending run, reversed in the buffer, is the right run
            // of a right gap merge.
            gap_right(a, m, b, len, enc);
            return;
        }
        let (mut i, mut j, mut d) = (m, len, n);
        while i > 0 && j > 0 {
            d -= 1;
            // Stable: on equal keys the element of the second run goes last.
            if enc(&*a.add(i - 1)) > enc(&*b.add(j - 1)) {
                i -= 1;
                a.add(d).write(std::ptr::read(a.add(i)));
            } else {
                j -= 1;
                a.add(d).write(std::ptr::read(b.add(j)));
            }
        }
        std::ptr::copy_nonoverlapping(b, a, j);
    }
}

/// Split of the merge of `src[s..m]` and `src[m..e]` after `d` outputs:
/// the number of left elements among them (stable: left first on ties).
///
/// # Safety
/// The ranges are valid in `src`; `d <= e - s`.
unsafe fn corank<T, E: Fn(&T) -> u128>(src: *const T, s: usize, m: usize, e: usize, d: usize, enc: &E) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(e - m), d.min(m - s));
    while lo < hi {
        let i = (lo + hi) / 2;
        if enc(&*src.add(s + i)) <= enc(&*src.add(m + d - i - 1)) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Forward branchy merge of `src[lf..le]` and `src[rf..re]` into `dst[df..]`.
///
/// # Safety
/// As `merge_into`.
#[inline(always)]
unsafe fn merge_fwd_b<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, mut lf: usize, le: usize, mut rf: usize, re: usize, mut df: usize, enc: &E) {
    while lf < le && rf < re {
        if enc(&*src.add(rf)) < enc(&*src.add(lf)) {
            dst.add(df).write(std::ptr::read(src.add(rf)));
            rf += 1;
        } else {
            dst.add(df).write(std::ptr::read(src.add(lf)));
            lf += 1;
        }
        df += 1;
    }
    std::ptr::copy_nonoverlapping(src.add(lf), dst.add(df), le - lf);
    df += le - lf;
    std::ptr::copy_nonoverlapping(src.add(rf), dst.add(df), re - rf);
}

/// `merge_into` as two independent merges (the output split in half by a
/// binary search), each from both ends: four chains of loads and compares
/// in flight instead of two. Branchy steps when `BR`.
///
/// # Safety
/// As `merge_into`.
#[inline(never)]
pub(super) unsafe fn merge_into_split<T, E: Fn(&T) -> u128, const BR: bool>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let d = (e - s) / 2;
    let i = corank(src, s, m, e, d, enc);
    let k = |x: usize| enc(&*src.add(x));
    // Merge A: [s, s + i) and [m, m + d - i) into [s, s + d); merge B: the rest.
    let (mut alf, mut ale, mut arf, mut are, mut adf, mut ade) = (s, s + i, m, m + d - i, s, s + d);
    let (mut blf, mut ble, mut brf, mut bre, mut bdf, mut bde) = (s + i, m, m + d - i, e, s + d, e);
    let steps = (ale - alf).min(are - arf).min(ble - blf).min(bre - brf);
    macro_rules! front {
        ($lf:ident, $rf:ident, $df:ident) => {{
            let r = k($rf) < k($lf);
            if BR {
                if r {
                    dst.add($df).write(std::ptr::read(src.add($rf)));
                    $rf += 1;
                } else {
                    dst.add($df).write(std::ptr::read(src.add($lf)));
                    $lf += 1;
                }
            } else {
                let from = if r { $rf } else { $lf };
                dst.add($df).write(std::ptr::read(src.add(from)));
                $rf += r as usize;
                $lf += (!r) as usize;
            }
            $df += 1;
        }};
    }
    macro_rules! back {
        ($le:ident, $re:ident, $de:ident) => {{
            let l = k($re - 1) < k($le - 1);
            $de -= 1;
            if BR {
                if l {
                    $le -= 1;
                    dst.add($de).write(std::ptr::read(src.add($le)));
                } else {
                    $re -= 1;
                    dst.add($de).write(std::ptr::read(src.add($re)));
                }
            } else {
                let from = if l { $le - 1 } else { $re - 1 };
                dst.add($de).write(std::ptr::read(src.add(from)));
                $le -= l as usize;
                $re -= (!l) as usize;
            }
        }};
    }
    for _ in 0..steps {
        front!(alf, arf, adf);
        front!(blf, brf, bdf);
        back!(ale, are, ade);
        back!(ble, bre, bde);
    }
    let _ = (ade, bde);
    merge_fwd_b(src, dst, alf, ale, arf, are, adf, enc);
    merge_fwd_b(src, dst, blf, ble, brf, bre, bdf, enc);
}

/// `a` when `c`, else `b`, written as a mask so that LLVM keeps a
/// conditional select (an `if` there becomes a branch, mispredicted half
/// the time on random merges).
/// On x86 LLVM turns the mask back into a branch, but honours
/// `select_unpredictable` (cmov); on aarch64 it is the reverse (checked on
/// rustc 1.99).
#[inline(always)]
fn pick<T>(c: bool, a: *const T, b: *const T) -> *const T {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    return std::hint::select_unpredictable(c, a, b);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    return (b as usize ^ ((a as usize ^ b as usize) & (c as usize).wrapping_neg())) as *const T;
}

/// `merge_into` on pointers with masked selects (no branch in the loop).
///
/// # Safety
/// As `merge_into`.
#[inline(never)]
pub(super) unsafe fn merge_into_p<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let steps = (m - s).min(e - m);
    let (mut lf, mut le, mut rf, mut re) = (src.add(s), src.add(m), src.add(m), src.add(e));
    let (mut df, mut de) = (dst.add(s), dst.add(e));
    for _ in 0..steps {
        let r = enc(&*rf) < enc(&*lf);
        df.write(std::ptr::read(pick(r, rf, lf)));
        df = df.add(1);
        rf = rf.add(r as usize);
        lf = lf.add(!r as usize);
        let l = enc(&*re.sub(1)) < enc(&*le.sub(1));
        de = de.sub(1);
        de.write(std::ptr::read(pick(l, le, re).sub(1)));
        le = le.sub(l as usize);
        re = re.sub(!l as usize);
    }
    let (lf, le, rf, re) = (lf.offset_from(src) as usize, le.offset_from(src) as usize, rf.offset_from(src) as usize, re.offset_from(src) as usize);
    merge_fwd_b(src, dst, lf, le, rf, re, df.offset_from(dst) as usize, enc);
}

/// `merge_into_p` as two independent merges (the output split in half by
/// a binary search): four chains in flight.
///
/// # Safety
/// As `merge_into`.
#[inline(never)]
pub(super) unsafe fn merge_into_p4<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, s: usize, m: usize, e: usize, enc: &E) {
    if m == s || m == e || enc(&*src.add(m - 1)) <= enc(&*src.add(m)) {
        std::ptr::copy_nonoverlapping(src.add(s), dst.add(s), e - s);
        return;
    }
    let d = (e - s) / 2;
    let i = corank(src, s, m, e, d, enc);
    let (mut alf, mut ale, mut arf, mut are) = (src.add(s), src.add(s + i), src.add(m), src.add(m + d - i));
    let (mut blf, mut ble, mut brf, mut bre) = (src.add(s + i), src.add(m), src.add(m + d - i), src.add(e));
    let (mut adf, mut ade, mut bdf, mut bde) = (dst.add(s), dst.add(s + d), dst.add(s + d), dst.add(e));
    let steps = i.min(d - i).min(m - s - i).min(e - m - d + i);
    for _ in 0..steps {
        let r = enc(&*arf) < enc(&*alf);
        adf.write(std::ptr::read(pick(r, arf, alf)));
        adf = adf.add(1);
        arf = arf.add(r as usize);
        alf = alf.add(!r as usize);
        let r = enc(&*brf) < enc(&*blf);
        bdf.write(std::ptr::read(pick(r, brf, blf)));
        bdf = bdf.add(1);
        brf = brf.add(r as usize);
        blf = blf.add(!r as usize);
        let l = enc(&*are.sub(1)) < enc(&*ale.sub(1));
        ade = ade.sub(1);
        ade.write(std::ptr::read(pick(l, ale, are).sub(1)));
        ale = ale.sub(l as usize);
        are = are.sub(!l as usize);
        let l = enc(&*bre.sub(1)) < enc(&*ble.sub(1));
        bde = bde.sub(1);
        bde.write(std::ptr::read(pick(l, ble, bre).sub(1)));
        ble = ble.sub(l as usize);
        bre = bre.sub(!l as usize);
    }
    let _ = (ade, bde);
    let o = |p: *const T| p.offset_from(src) as usize;
    merge_fwd_b(src, dst, o(alf), o(ale), o(arf), o(are), adf.offset_from(dst) as usize, enc);
    merge_fwd_b(src, dst, o(blf), o(ble), o(brf), o(bre), bdf.offset_from(dst) as usize, enc);
}

/// Two-run merges through the shorter run in the buffer with several chains
/// in flight (`merge_two`, pipe organs): the merge is cut at crossover
/// points into merges whose two inputs both lie outside their output, each
/// done from both ends on both halves (four chains), instead of one chain
/// from the side the buffered run came from. Idea of glidesort (Orson
/// Peters, 2023: gap merges split at the crossover point); own code.
pub(crate) const X2: bool = true;
/// Below this many elements on the shorter side a gap merge is finished
/// with one chain.
const X2_SMALL: usize = 64;

/// Number of elements of `l[..ln]` among the first `d` of the stable merge
/// of `l[..ln]` and `r[..rn]` (left first on ties).
///
/// # Safety
/// `l`, `r` valid for `ln`, `rn` elements; `d <= ln + rn`.
unsafe fn corank2<T, E: Fn(&T) -> u128>(l: *const T, ln: usize, r: *const T, rn: usize, d: usize, enc: &E) -> usize {
    let (mut lo, mut hi) = (d.saturating_sub(rn), d.min(ln));
    while lo < hi {
        let i = (lo + hi) / 2;
        if enc(&*l.add(i)) <= enc(&*r.add(d - i - 1)) {
            lo = i + 1;
        } else {
            hi = i;
        }
    }
    lo
}

/// Forward one-chain merge of `l[..ln]` (outside the output) and `r[..rn]`,
/// which is the tail of the output `d[..ln + rn]` (`r == d + ln`), left
/// first on ties. Each write lands at or before the next unread element
/// of `r`.
///
/// # Safety
/// As stated.
#[inline(always)]
unsafe fn merge_fwd_gap<T, E: Fn(&T) -> u128>(mut l: *const T, ln: usize, mut r: *const T, rn: usize, mut d: *mut T, enc: &E) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        let t = enc(&*r) < enc(&*l);
        d.write(std::ptr::read(pick(t, r, l)));
        d = d.add(1);
        r = r.add(t as usize);
        l = l.add(!t as usize);
    }
    // What is left of `l` fills the slots before the unread rest of `r`,
    // which is already in place.
    std::ptr::copy_nonoverlapping(l, d, le.offset_from(l) as usize);
}

/// Backward one-chain merge of `l[..ln]`, the head of the output
/// (`l == d`), and `r[..rn]` (outside the output) into `d[..ln + rn]`,
/// right last on ties.
///
/// # Safety
/// As stated.
#[inline(always)]
unsafe fn merge_bwd_gap<T, E: Fn(&T) -> u128>(l: *const T, ln: usize, r: *const T, rn: usize, d: *mut T, enc: &E) {
    let (mut le, mut re, mut de) = (l.add(ln), r.add(rn), d.add(ln + rn));
    while le > l && re > r {
        let t = enc(&*re.sub(1)) < enc(&*le.sub(1));
        de = de.sub(1);
        de.write(std::ptr::read(pick(t, le, re).sub(1)));
        le = le.sub(t as usize);
        re = re.sub(!t as usize);
    }
    // What is left of `r` fills the slots after the unread rest of `l`.
    let rest = re.offset_from(r) as usize;
    std::ptr::copy_nonoverlapping(r, de.sub(rest), rest);
}

/// Forward branchy merge of two inputs into `d`, none overlapping.
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(always)]
unsafe fn merge_fwd_b_ptr<T, E: Fn(&T) -> u128>(mut l: *const T, ln: usize, mut r: *const T, rn: usize, mut d: *mut T, enc: &E) {
    let (le, re) = (l.add(ln), r.add(rn));
    while l < le && r < re {
        if enc(&*r) < enc(&*l) {
            d.write(std::ptr::read(r));
            r = r.add(1);
        } else {
            d.write(std::ptr::read(l));
            l = l.add(1);
        }
        d = d.add(1);
    }
    let k = le.offset_from(l) as usize;
    std::ptr::copy_nonoverlapping(l, d, k);
    std::ptr::copy_nonoverlapping(r, d.add(k), re.offset_from(r) as usize);
}

/// Merge of `l[..ln]` and `r[..rn]` into `d`, neither input overlapping the
/// output: split in half by a binary search, both halves from both ends
/// (four chains).
///
/// # Safety
/// Valid, non-overlapping ranges.
#[inline(never)]
unsafe fn merge_ext4<T, E: Fn(&T) -> u128>(l: *const T, ln: usize, r: *const T, rn: usize, d: *mut T, enc: &E) {
    if ln == 0 || rn == 0 || enc(&*l.add(ln - 1)) <= enc(&*r) {
        std::ptr::copy_nonoverlapping(l, d, ln);
        std::ptr::copy_nonoverlapping(r, d.add(ln), rn);
        return;
    }
    let h = (ln + rn) / 2;
    let i = corank2(l, ln, r, rn, h, enc);
    let (mut alf, mut ale, mut arf, mut are) = (l, l.add(i), r, r.add(h - i));
    let (mut blf, mut ble, mut brf, mut bre) = (l.add(i), l.add(ln), r.add(h - i), r.add(rn));
    let (mut adf, mut ade, mut bdf, mut bde) = (d, d.add(h), d.add(h), d.add(ln + rn));
    let steps = i.min(h - i).min(ln - i).min(rn - (h - i));
    for _ in 0..steps {
        let t = enc(&*arf) < enc(&*alf);
        adf.write(std::ptr::read(pick(t, arf, alf)));
        adf = adf.add(1);
        arf = arf.add(t as usize);
        alf = alf.add(!t as usize);
        let t = enc(&*brf) < enc(&*blf);
        bdf.write(std::ptr::read(pick(t, brf, blf)));
        bdf = bdf.add(1);
        brf = brf.add(t as usize);
        blf = blf.add(!t as usize);
        let t = enc(&*are.sub(1)) < enc(&*ale.sub(1));
        ade = ade.sub(1);
        ade.write(std::ptr::read(pick(t, ale, are).sub(1)));
        ale = ale.sub(t as usize);
        are = are.sub(!t as usize);
        let t = enc(&*bre.sub(1)) < enc(&*ble.sub(1));
        bde = bde.sub(1);
        bde.write(std::ptr::read(pick(t, ble, bre).sub(1)));
        ble = ble.sub(t as usize);
        bre = bre.sub(!t as usize);
    }
    let _ = (ade, bde);
    let len = |a: *const T, b: *const T| b.offset_from(a) as usize;
    merge_fwd_b_ptr(alf, len(alf, ale), arf, len(arf, are), adf, enc);
    merge_fwd_b_ptr(blf, len(blf, ble), brf, len(brf, bre), bdf, enc);
}

/// Left gap merge: `l[..ln]` (outside the array) and `r[..rn]`, which lies
/// at `d + ln`, merged into `d[..ln + rn]`. Cut at the crossover point of
/// the first `ln` outputs: `l[..i]` and `r[..ln - i]` merged into `d[..ln]`
/// (both inputs outside it), then the same with the rest, which has the
/// same shape (`l[i..]` outside, `r[ln - i..]` after a gap of `ln - i`).
///
/// # Safety
/// `l` does not overlap `d[..ln + rn]`; `r == d + ln`.
unsafe fn gap_left<T, E: Fn(&T) -> u128>(mut l: *const T, mut ln: usize, mut r: *mut T, mut rn: usize, mut d: *mut T, enc: &E) {
    while ln > 0 {
        if rn == 0 {
            std::ptr::copy_nonoverlapping(l, d, ln);
            return;
        }
        if ln.min(rn) < X2_SMALL {
            merge_fwd_gap(l, ln, r, rn, d, enc);
            return;
        }
        let i = corank2(l, ln, r, rn, ln, enc);
        let j = ln - i;
        merge_ext4(l, i, r, j, d, enc);
        l = l.add(i);
        d = d.add(ln);
        r = r.add(j);
        rn -= j;
        ln = j;
    }
}

/// Right gap merge: `l[..ln]`, which lies at the head of the output `l`,
/// and `r[..rn]` (outside the array) merged into `l[..ln + rn]`, cut from
/// the end (mirror of `gap_left`).
///
/// # Safety
/// `r` does not overlap `l[..ln + rn]`.
unsafe fn gap_right<T, E: Fn(&T) -> u128>(l: *mut T, mut ln: usize, r: *const T, mut rn: usize, enc: &E) {
    while rn > 0 {
        if ln == 0 {
            std::ptr::copy_nonoverlapping(r, l, rn);
            return;
        }
        if ln.min(rn) < X2_SMALL {
            merge_bwd_gap(l, ln, r, rn, l, enc);
            return;
        }
        // Left elements among the last `rn` outputs, right ones the rest.
        let a = ln - corank2(l, ln, r, rn, ln, enc);
        let b = rn - a;
        merge_ext4(l.add(ln - a), a, r.add(rn - b), b, l.add(ln), enc);
        ln -= a;
        rn = a;
    }
}

/// Stable merge of `arr[..m]` and `arr[m..]` with a buffer of at least the
/// shorter run (see `X2`).
pub(super) fn merge_two_x<T, E: Fn(&T) -> u128>(arr: &mut [T], m: usize, enc: &E, buf: &mut Vec<MaybeUninit<T>>) {
    let n = arr.len();
    if m == 0 || m == n || enc(&arr[m - 1]) <= enc(&arr[m]) {
        return;
    }
    // Keys already in place at both ends stay.
    let (top, bottom) = (enc(&arr[m - 1]), enc(&arr[m]));
    let s = arr[..m].partition_point(|x| enc(x) <= bottom);
    let e = m + arr[m..].partition_point(|x| enc(x) < top);
    let a = &mut arr[s..e];
    let (m, n) = (m - s, e - s);
    let p = a.as_mut_ptr();
    let b = buf.as_mut_ptr() as *mut T;
    // SAFETY: the shorter run is moved to the buffer; the gap merges then
    // fill the array, each write on a slot already read (see `gap_left`,
    // `gap_right`); every element moved once more.
    unsafe {
        if m <= n - m {
            debug_assert!(buf.capacity() >= m);
            std::ptr::copy_nonoverlapping(p, b, m);
            gap_left(b, m, p.add(m), n - m, p, enc);
        } else {
            debug_assert!(buf.capacity() >= n - m);
            std::ptr::copy_nonoverlapping(p.add(m), b, n - m);
            gap_right(p, m, b, n - m, enc);
        }
    }
}

/// `x` when `c`, else `y`, as a mask (kept a select by LLVM).
#[inline(always)]
fn pick_k(c: bool, x: u128, y: u128) -> u128 {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    return std::hint::select_unpredictable(c, x, y);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    return y ^ ((x ^ y) & (c as u128).wrapping_neg());
}

/// `merge4b` on pointers with masked selects (no branch in the loop).
///
/// # Safety
/// As `merge4b`.
#[inline(never)]
pub(super) unsafe fn merge4m<T, E: Fn(&T) -> u128>(src: *const T, dst: *mut T, r: [(usize, usize); 4], d0: usize, enc: &E) {
    let p = |i: usize| src.add(i);
    let (mut f0, mut f1, mut f2, mut f3) = (p(r[0].0), p(r[1].0), p(r[2].0), p(r[3].0));
    let (mut b0, mut b1, mut b2, mut b3) = (p(r[0].1), p(r[1].1), p(r[2].1), p(r[3].1));
    let total = (r[0].1 - r[0].0) + (r[1].1 - r[1].0) + (r[2].1 - r[2].0) + (r[3].1 - r[3].0);
    let mut df = dst.add(d0);
    let mut db = dst.add(d0 + total);
    let len = |f: *const T, b: *const T| b.offset_from(f) as usize;
    loop {
        let safe = len(f0, b0).min(len(f1, b1)).min(len(f2, b2)).min(len(f3, b3)) / 2;
        if safe == 0 {
            break;
        }
        for _ in 0..safe {
            let (k0, k1, k2, k3) = (enc(&*f0), enc(&*f1), enc(&*f2), enc(&*f3));
            let ab = k1 < k0;
            let (kab, iab) = (pick_k(ab, k1, k0), pick(ab, f1, f0));
            let cd = k3 < k2;
            let (kcd, icd) = (pick_k(cd, k3, k2), pick(cd, f3, f2));
            let right = kcd < kab;
            df.write(std::ptr::read(pick(right, icd, iab)));
            df = df.add(1);
            f0 = f0.add((!right & !ab) as usize);
            f1 = f1.add((!right & ab) as usize);
            f2 = f2.add((right & !cd) as usize);
            f3 = f3.add((right & cd) as usize);
            let (k0, k1, k2, k3) = (enc(&*b0.sub(1)), enc(&*b1.sub(1)), enc(&*b2.sub(1)), enc(&*b3.sub(1)));
            let ab = k1 >= k0;
            let (kab, iab) = (pick_k(ab, k1, k0), pick(ab, b1, b0));
            let cd = k3 >= k2;
            let (kcd, icd) = (pick_k(cd, k3, k2), pick(cd, b3, b2));
            let right = kcd >= kab;
            db = db.sub(1);
            db.write(std::ptr::read(pick(right, icd, iab).sub(1)));
            b0 = b0.sub((!right & !ab) as usize);
            b1 = b1.sub((!right & ab) as usize);
            b2 = b2.sub((right & !cd) as usize);
            b3 = b3.sub((right & cd) as usize);
        }
    }
    let o = |x: *const T| x.offset_from(src) as usize;
    let mut rest = [(0, 0); 4];
    let mut c = 0;
    for (f, b) in [(f0, b0), (f1, b1), (f2, b2), (f3, b3)] {
        if f < b {
            rest[c] = (o(f), o(b));
            c += 1;
        }
    }
    let df = df.offset_from(dst) as usize;
    match c {
        4 => merge4f(src, dst, [rest[0], rest[1], rest[2], rest[3]], df, enc),
        _ => merge_rest(src, dst, &rest[..c], df, enc),
    }
}

/// Tests: the kernels alone (src to dst).
#[cfg(test)]
pub(crate) unsafe fn bench_kernel<T, E: Fn(&T) -> u128>(which: usize, src: *const T, dst: *mut T, b: &[usize], enc: &E) {
    match which {
        0 => merge_into(src, dst, b[0], b[1], b[2], enc),
        1 => merge_into_b(src, dst, b[0], b[1], b[2], enc),
        2 => merge4b(src, dst, [(b[0], b[1]), (b[1], b[2]), (b[2], b[3]), (b[3], b[4])], b[0], enc),
        4 => merge_into_split::<T, E, false>(src, dst, b[0], b[1], b[2], enc),
        5 => merge_into_split::<T, E, true>(src, dst, b[0], b[1], b[2], enc),
        8 => merge4m(src, dst, [(b[0], b[1]), (b[1], b[2]), (b[2], b[3]), (b[3], b[4])], b[0], enc),
        6 => merge_into_p(src, dst, b[0], b[1], b[2], enc),
        7 => merge_into_p4(src, dst, b[0], b[1], b[2], enc),
        _ => merge4f(src, dst, [(b[0], b[1]), (b[1], b[2]), (b[2], b[3]), (b[3], b[4])], b[0], enc),
    }
}
