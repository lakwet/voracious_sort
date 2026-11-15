//! Merging sorted segments (after Verge): rounds of 2-way merges.

mod bidir;
mod classic;
mod gap;
mod gallop;
mod predictable;
pub(crate) mod simd;
mod two_way;

pub(crate) use predictable::predictable_merge;

/// Balanced two-run merges in place by gap merges (`gap`: buffer of the
/// shorter run, four chains, no copy back).
pub(crate) static GAP2: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Merges of unpredictable 32-bit unsigned runs by blocks in vector
/// registers (`simd`), where the branch-free merge waits on each load.
pub(crate) static SIMD_MERGE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
/// With `SIMD_MERGE`, unpredictable merges are merged instead of sorting
/// the whole array by radix (`merges_unpredictable`).
pub(crate) static SIMD_RULE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// With `SIMD_RULE`: unpredictable merges merged (instead of the radix
/// sort) up to this many merge levels.
pub(crate) static SIMD_LEVELS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(3);

/// Whether the SIMD block merge applies to keys of type `T`.
pub(crate) fn simd_applies<T>() -> bool {
    SIMD_MERGE.load(std::sync::atomic::Ordering::Relaxed)
        && simd_flip::<T>().is_some_and(|(w, _)| w == 64 || simd::available())
}

/// Keys the vector merge takes: (bits, mode of `simd::Lane::key`): u32,
/// i32, f32 (32 bits), u64, i64, f64 (64 bits; modes 0 unsigned, 1 signed,
/// 2 float).
fn simd_flip<T>() -> Option<(u32, u8)> {
    use std::sync::atomic::Ordering::Relaxed;
    let w64 = SIMD_64.load(Relaxed) && simd::available64();
    match std::any::type_name::<T>() {
        "u32" => Some((32, 0)),
        "i32" if SIMD_I32.load(Relaxed) => Some((32, 1)),
        "f32" if SIMD_F32.load(Relaxed) => Some((32, 2)),
        "u64" if w64 => Some((64, 0)),
        "i64" if w64 => Some((64, 1)),
        "f64" if w64 && SIMD_F64.load(Relaxed) => Some((64, 2)),
        _ => None,
    }
}

/// The vector merge on 64-bit keys too.
pub(crate) static SIMD_64: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
/// The vector merge on f64 keys. OFF: the f64 path (mode 2 of the 64-bit
/// network in `simd.rs`, tested correct) is kept but never runs, because it
/// lost everywhere on the Snapdragon X: Runs 8 100k 2.90 to 4.05, 1M 4.25 to
/// 6.14 ns per key (the f64 radix sort is faster than these merges, unlike
/// u64 / i64). Kept to measure on x86 (Ryzen, Intel) before removing it:
/// turn this on and compare in the same run.
pub(crate) static SIMD_F64: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// With `SIMD_RULE`: the smallest array of 64-bit keys whose unpredictable
/// merges are merged by vector blocks (below, the radix sort in the cache
/// wins: i64 Runs 8 10k 2.02 against 2.65 ns per key on the Snapdragon X).
/// Measured on one machine only: to check on x86 (Ryzen, Intel).
pub(crate) static SIMD_MIN_ARRAY_SIZE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1 << 16);

/// Whether unpredictable merges of `n` keys of type `T` are merged by
/// vector blocks instead of sorted by radix (`SIMD_RULE`): 32-bit keys at
/// every size, 64-bit keys from `SIMD_MIN_ARRAY_SIZE`.
pub(crate) fn simd_rule_applies<T>(n: usize) -> bool {
    let min64 = SIMD_MIN_ARRAY_SIZE.load(std::sync::atomic::Ordering::Relaxed);
    simd_applies::<T>() && simd_flip::<T>().is_some_and(|(w, _)| w == 32 || n >= min64)
}

/// The vector merge on i32 keys too.
pub(crate) static SIMD_I32: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
/// The vector merge on f32 keys too.
pub(crate) static SIMD_F32: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Balanced merge of `a` and `b` into `dst`: the SIMD block merge for
/// unpredictable u32 runs, else the two-ended branch-free merge.
fn merge_balanced<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
    if simd_applies::<T>() && a.len() >= 4 && b.len() >= 4 && !predictable::predictable_pair(a, b) {
        // SAFETY: `T` is one of the key types of `simd_flip` (checked by
        // name), of the width given there: the same layout as u32 / u64.
        unsafe {
            macro_rules! go {
                ($u:ty, $f:ident, $m:literal) => {
                    simd::$f::<$m>(
                        std::slice::from_raw_parts(a.as_ptr() as *const $u, a.len()),
                        std::slice::from_raw_parts(b.as_ptr() as *const $u, b.len()),
                        std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut $u, dst.len()),
                    )
                };
            }
            match simd_flip::<T>() {
                Some((32, 0)) => go!(u32, merge_u32, 0),
                Some((32, 1)) => go!(u32, merge_u32, 1),
                Some((32, _)) => go!(u32, merge_u32, 2),
                Some((_, 0)) => go!(u64, merge_u64, 0),
                Some((_, 1)) => go!(u64, merge_u64, 1),
                _ => go!(u64, merge_u64, 2),
            }
        }
        return;
    }
    merge_bidir(a, b, dst);
}

use bidir::merge_bidir;
use gallop::GALLOP_MIN;
use two_way::merge_in_place;

/// Merges the sorted segments delimited by `separators` (stable), pairs of
/// neighbours per round. From 3 segments, in the cache, the rounds alternate
/// between the array and a buffer (ping-pong: no copy back after each
/// merge); else each merge is done in place.
pub(crate) fn merge_runs<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    if separators.len() <= 2 {
        return;
    }
    // Ping-pong while the array and its buffer fit in a quarter of the L2,
    // from 4096 keys (journal §52).
    let bytes = 2 * arr.len() * std::mem::size_of::<T>();
    if separators.len() >= 4
        && arr.len() >= 4096
        && bytes <= super::super::cache::budget(super::super::cache::Budget::PingPong) / 4
    {
        pingpong(arr, separators);
        return;
    }
    // Several merges: allocate the scratch once (as the original merge).
    let mut copy: Vec<T> = if separators.len() > 3 {
        vec![arr[0]; arr.len() / 2 + 2]
    } else {
        Vec::new()
    };
    while separators.len() > 2 {
        let half = (separators.len() - 1) / 2;
        for i in 0..half {
            let i2 = i * 2;
            merge_in_place(arr, &mut copy, separators[i2], separators[i2 + 1], separators[i2 + 2], separators.len() == 3);
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
}

/// Unpredictable merges of u32 runs by SIMD blocks (instead of the radix
/// sort): ping-pong rounds at every size (the block merges stream through
/// memory).
pub(crate) fn merge_runs_simd<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    if separators.len() > 2 {
        pingpong(arr, separators);
    }
}

/// Ping-pong rounds (see `merge_runs`).
fn pingpong<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    let n = arr.len();
    let mut buf: Vec<T> = vec![arr[0]; n];
    let mut in_buf = false;
    while separators.len() > 2 {
        {
            let (src, dst): (&[T], &mut [T]) = if in_buf { (&buf[..], &mut arr[..]) } else { (&arr[..], &mut buf[..]) };
            let segs = separators.len() - 1;
            let mut i = 0;
            while i + 1 < segs {
                let (s, m, e) = (separators[i], separators[i + 1], separators[i + 2]);
                merge_out(&src[s..m], &src[m..e], &mut dst[s..e]);
                i += 2;
            }
            if i < segs {
                let (s, e) = (separators[i], separators[i + 1]);
                dst[s..e].copy_from_slice(&src[s..e]);
            }
        }
        in_buf = !in_buf;
        let half = (separators.len() - 1) / 2;
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
    if in_buf {
        arr.copy_from_slice(&buf[..n]);
    }
}

/// Stable merge of the sorted runs `a` (first) and `b` into `dst`, out of
/// place: blocks already in order copied whole; balanced runs by the
/// two-ended branch-free merge; skewed runs by stretches, element by
/// element up to `GALLOP_MIN`, then galloping and one block copy.
fn merge_out<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
    if a.is_empty() || b.is_empty() || a[a.len() - 1] <= b[0] {
        dst[..a.len()].copy_from_slice(a);
        dst[a.len()..].copy_from_slice(b);
        return;
    }
    // Head and tail blocks already ordered against the other run.
    let head = b.partition_point(|x| *x < a[0]);
    let tail = a.len() - a.partition_point(|x| *x <= b[b.len() - 1]);
    let len = dst.len();
    dst[..head].copy_from_slice(&b[..head]);
    dst[len - tail..].copy_from_slice(&a[a.len() - tail..]);
    let (a, b) = (&a[..a.len() - tail], &b[head..]);
    let dst = &mut dst[head..len - tail];
    if a.is_empty() || b.is_empty() {
        dst[..a.len()].copy_from_slice(a);
        dst[a.len()..].copy_from_slice(b);
        return;
    }
    // Balanced runs: two-ended branch-free merge (as the in-place version;
    // a branchy merge does not beat it even on predictable interleavings).
    if 4 * a.len().min(b.len()) >= a.len() + b.len() {
        merge_balanced(a, b, dst);
        return;
    }
    // Skewed runs: stretches, galloping from `GALLOP_MIN` on.
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        // Stretch of `a` not above b[j] (stable: a first on ties).
        let v = b[j];
        let mut c = 0;
        while c < GALLOP_MIN && i + c < a.len() && a[i + c] <= v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += a[i + c..].partition_point(|x| *x <= v);
        }
        dst[k..k + c].copy_from_slice(&a[i..i + c]);
        i += c;
        k += c;
        if i == a.len() {
            break;
        }
        // Stretch of `b` below a[i].
        let v = a[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < b.len() && b[j + c] < v {
            c += 1;
        }
        if c == GALLOP_MIN {
            c += b[j + c..].partition_point(|x| *x < v);
        }
        dst[k..k + c].copy_from_slice(&b[j..j + c]);
        j += c;
        k += c;
    }
    dst[k..k + (a.len() - i)].copy_from_slice(&a[i..]);
    k += a.len() - i;
    dst[k..].copy_from_slice(&b[j..]);
}

/// Tests: the two-ended branch-free merge.
#[cfg(test)]
pub(crate) fn bidir_pub<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
    merge_bidir(a, b, dst)
}

