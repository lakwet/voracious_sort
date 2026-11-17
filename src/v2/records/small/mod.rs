//! Records of at most 8 bytes: the first records sort (2026-11), still the
//! fastest for them (its first read with the byte histograms, the LSD on
//! the records themselves).
//!
//! One read of the keys gives their range, whether they are already in
//! order and, beyond the cache, how often they change direction (and the
//! byte histograms). Then:
//! - ascending: nothing to do; strictly descending: reversed (still stable);
//! - few distinct keys: a stable counting sort;
//! - beyond the cache, few turns: Verge on the records (long runs kept);
//! - small records beyond the cache: the LSD on the records themselves;
//! - else the packed (key, index) sort, the records moved once (in place
//!   beyond the cache, through a buffer in it).
//!
//! Keys too wide for a u128 with their index fall back to `sort_by`.

mod few;
mod lsd;
mod merge;
mod verge;

use crate::v2::dev::trace::hit;
use std::mem::MaybeUninit;

use super::packed::{sort_in_place, sort_into};
use few::few_keys;
use lsd::lsd;
use verge::{verge, VERGE_MIN};

/// What one read of the keys learned.
struct Scan {
    lo: u128,
    hi: u128,
    ascending: bool,
    strictly_descending: bool,
    /// Changes of direction between neighbouring pairs (equal keys keep the
    /// direction): about 2n/3 on random keys, one per tooth on sawtooth
    /// data, one on a pipe organ. Counted only if `TURNS`.
    turns: usize,
}

/// Direction of a pair of keys: -1 descending, 0 equal, 1 ascending.
#[inline(always)]
fn direction(a: u128, b: u128) -> i8 {
    (b > a) as i8 - (b < a) as i8
}

/// Reads every key once. `TURNS`: the turns counted; `HIST`: the byte
/// histograms of the low 64 bits taken in `hist`.
#[inline(always)]
fn scan<T, E: Fn(&T) -> u128, const TURNS: bool, const HIST: bool>(arr: &[T], enc: &E, hist: &mut [[usize; 256]]) -> Scan {
    let count = |hist: &mut [[usize; 256]], k: u128| {
        for (d, h) in hist.iter_mut().enumerate() {
            h[((k as u64 >> (8 * d)) & 0xFF) as usize] += 1;
        }
    };
    let first = enc(&arr[0]);
    if HIST {
        count(hist, first);
    }
    let mut s = Scan { lo: first, hi: first, ascending: true, strictly_descending: true, turns: 0 };
    let (mut prev, mut last) = (first, 0i8);
    for x in arr[1..].iter() {
        let k = enc(x);
        s.lo = s.lo.min(k);
        s.hi = s.hi.max(k);
        s.ascending &= prev <= k;
        s.strictly_descending &= prev > k;
        if TURNS {
            let d = direction(prev, k);
            s.turns += (d != 0 && d != last && last != 0) as usize;
            last = if d != 0 { d } else { last };
        }
        if HIST {
            count(hist, k);
        }
        prev = k;
    }
    s
}

pub(crate) fn sort_encoded<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    // Beyond half the L2 the moves of the records dominate: the LSD on small
    // records, Verge on the records (both fed by the first read).
    let size = std::mem::size_of::<T>();
    let beyond_cache = n.saturating_mul(size) > super::super::cache::budget(super::super::cache::Budget::Records) / 2;
    let small_records = size <= 8 && beyond_cache;
    let mut hist = if small_records { vec![[0usize; 256]; 8] } else { Vec::new() };
    let s = if small_records {
        scan::<T, E, true, true>(arr, enc, &mut hist)
    } else if beyond_cache {
        scan::<T, E, true, false>(arr, enc, &mut hist)
    } else {
        scan::<T, E, false, false>(arr, enc, &mut hist)
    };
    if s.ascending {
        trace!("presorted");
        return;
    }
    if s.strictly_descending {
        trace!("reversed");
        arr.reverse();
        return;
    }
    if hit(few_keys(arr, enc), "few") {
        return;
    }
    // Few turns: long runs likely.
    if beyond_cache && n >= VERGE_MIN && 8 * s.turns <= n && hit(verge(arr, enc), "verge") {
        return;
    }
    if small_records && s.hi <= u64::MAX as u128 {
        trace!("lsd");
        lsd(arr, enc, &hist);
        return;
    }
    let key_bits = u128::BITS - (s.hi - s.lo).leading_zeros();
    // In the cache the gather through a buffer is cheaper than scanning the
    // permutation first; beyond it, the permutation applied in place.
    if beyond_cache {
        trace!("packed_in_place");
        if !sort_in_place(arr, enc, s.lo, key_bits) {
            trace!("std_fallback");
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
        }
        return;
    }
    trace!("packed");
    let mut buffer: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    // SAFETY: the records are moved into the buffer in key order, then the
    // buffer copied back over the array (see `sort_into`).
    unsafe {
        let dst = buffer.as_mut_ptr() as *mut T;
        if !sort_into(arr.as_ptr(), dst, n, enc, s.lo, key_bits) {
            trace!("std_fallback");
            arr.sort_by(|a, b| enc(a).cmp(&enc(b)));
            return;
        }
        std::ptr::copy_nonoverlapping(dst, arr.as_mut_ptr(), n);
    }
}
