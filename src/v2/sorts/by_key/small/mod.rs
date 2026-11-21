//! Records of at most 8 bytes: the pieces of their own path (the path
//! itself is copied in each `types::records` chain of at most 8 bytes).
//!
//! The first records sort, still the
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

pub(crate) mod lsd;
mod merge;
pub(crate) mod verge;

/// What one read of the keys learned.
pub(crate) struct Scan {
    pub(crate) lo: u128,
    pub(crate) hi: u128,
    pub(crate) ascending: bool,
    pub(crate) strictly_descending: bool,
    /// Changes of direction between neighbouring pairs (equal keys keep the
    /// direction): about 2n/3 on random keys, one per tooth on sawtooth
    /// data, one on a pipe organ. Counted only if `TURNS`.
    pub(crate) turns: usize,
}

/// Direction of a pair of keys: -1 descending, 0 equal, 1 ascending.
#[inline(always)]
fn direction(a: u128, b: u128) -> i8 { (b > a) as i8 - (b < a) as i8 }

/// Reads every key once. `TURNS`: the turns counted; `HIST`: the byte
/// histograms of the low 64 bits taken in `hist`.
#[inline(always)]
pub(crate) fn scan<T, E: Fn(&T) -> u128, const TURNS: bool, const HIST: bool>(
    arr: &[T],
    enc: &E,
    hist: &mut [[usize; 256]],
) -> Scan {
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
