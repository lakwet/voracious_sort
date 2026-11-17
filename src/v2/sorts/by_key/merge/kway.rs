//! Merges of many segments: k-way rounds between the array and a buffer
//! (4-way: one move per two levels), or pairwise rounds through half a
//! buffer.

use std::mem::MaybeUninit;

use super::*;

/// Stable merge of the sorted segments `bounds[i]..bounds[i + 1]`, pairs of
/// neighbours per round, each merged in the array through one buffer of
/// half the array (`merge_two_with`): half the memory of `merge_all`.
pub(crate) fn merge_all_half<T, E: Fn(&T) -> u128>(
    arr: &mut [T],
    bounds: &[usize],
    enc: &E,
    branchy: bool,
) {
    let mut bounds = bounds.to_vec();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(arr.len() / 2 + 1);
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
pub(crate) static FOUR_WAY_MIN: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(32);

pub(crate) fn merge_all_kway<T, E: Fn(&T) -> u128>(
    arr: &mut [T],
    bounds: &[usize],
    enc: &E,
    branchy: bool,
) {
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
    let two = std::mem::size_of::<T>() < FOUR_WAY_MIN.load(std::sync::atomic::Ordering::Relaxed);
    let mut rounds = levels.div_ceil(2);
    rounds += rounds % 2;
    let fours = levels.saturating_sub(rounds);
    let (rounds, fours) = if two { (levels, 0) } else { (rounds, fours) };
    let mut bounds = bounds.to_vec();
    let mut own: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    prefault(&mut own);
    let b = own.as_mut_ptr() as *mut T;
    let mut in_buf = false;
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
                    4 => merge4m(src, dst, [r(0), r(1), r(2), r(3)], bounds[i], enc),
                    3 => merge3(src, dst, [r(0), r(1), r(2)], bounds[i], enc),
                    2 if two && branchy => {
                        merge_into_b(src, dst, bounds[i], bounds[i + 1], bounds[i + 2], enc)
                    },
                    2 => merge_into_p4(src, dst, bounds[i], bounds[i + 1], bounds[i + 2], enc),
                    _ => std::ptr::copy_nonoverlapping(
                        src.add(bounds[i]),
                        dst.add(bounds[i]),
                        bounds[i + 1] - bounds[i],
                    ),
                }
                i += g;
                next.push(bounds[i]);
            }
            bounds = next;
            in_buf = !in_buf;
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
}

/// Index of the smallest of four keys, the first on ties (stable).
#[inline(always)]
pub(crate) fn argmin4(k: [u128; 4]) -> usize {
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
/// The ranges are valid in both, `src` and `dst` do not overlap; the
/// elements are moved (bitwise).
#[inline(never)]
pub(crate) unsafe fn merge4f<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    r: [(usize, usize); 4],
    d0: usize,
    enc: &E,
) {
    let mut i = [r[0].0, r[1].0, r[2].0, r[3].0];
    let e = [r[0].1, r[1].1, r[2].1, r[3].1];
    let mut d = d0;
    loop {
        let safe = (e[0] - i[0]).min(e[1] - i[1]).min(e[2] - i[2]).min(e[3] - i[3]);
        if safe == 0 {
            break;
        }
        for _ in 0..safe {
            let k = [
                enc(&*src.add(i[0])),
                enc(&*src.add(i[1])),
                enc(&*src.add(i[2])),
                enc(&*src.add(i[3])),
            ];
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
/// The ranges are valid in `src`, `src` and `dst` do not overlap,
/// `dst[d0..]` has room for all the runs; the elements are moved (bitwise).
#[inline(never)]
pub(crate) unsafe fn merge3<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    r: [(usize, usize); 3],
    d0: usize,
    enc: &E,
) {
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
pub(crate) unsafe fn merge_rest<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    rest: &[(usize, usize)],
    d: usize,
    enc: &E,
) {
    match rest.len() {
        3 => merge3(src, dst, [rest[0], rest[1], rest[2]], d, enc),
        2 => merge2r(src, dst, rest[0], rest[1], d, enc),
        1 => std::ptr::copy_nonoverlapping(src.add(rest[0].0), dst.add(d), rest[0].1 - rest[0].0),
        _ => {},
    }
}

/// Stable merge of two runs of `src` (`x` before `y`, not necessarily
/// adjacent) into `dst` from `d`, from both ends at once.
///
/// # Safety
/// As `merge3`.
#[inline(never)]
pub(crate) unsafe fn merge2r<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    x: (usize, usize),
    y: (usize, usize),
    d: usize,
    enc: &E,
) {
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

/// `x` when `c`, else `y`, as a mask (kept a select by LLVM).
#[inline(always)]
pub(crate) fn pick_k(c: bool, x: u128, y: u128) -> u128 {
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
pub(crate) unsafe fn merge4m<T, E: Fn(&T) -> u128>(
    src: *const T,
    dst: *mut T,
    r: [(usize, usize); 4],
    d0: usize,
    enc: &E,
) {
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
            let (k0, k1, k2, k3) =
                (enc(&*b0.sub(1)), enc(&*b1.sub(1)), enc(&*b2.sub(1)), enc(&*b3.sub(1)));
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
