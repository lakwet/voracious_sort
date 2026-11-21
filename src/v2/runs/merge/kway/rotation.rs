//! The rotation of three zones (copy of kwaymerge `merges::rotation_in`):
//! the range's two halves merge level by level, each move into the zone
//! left empty (start of the range, its end, the buffer), then one hole
//! merge. Used by the hybrid on nodes whose middle is a run boundary.

use super::inplace::merge_two;
use super::kernel::{count, hole_bwd, hole_fwd, merge_out};
use core::ptr;

/// Levels of pairing to merge `c` runs into one: ceil(log2 c).
pub fn levels(c: usize) -> usize {
    if c <= 1 {
        0
    } else {
        (usize::BITS - (c - 1).leading_zeros()) as usize
    }
}

/// The caller's scratch buffer with room for `need` elements (grown if
/// needed, else reused as it is: no allocation, no page fault).
pub(crate) fn room<T>(scratch: &mut Vec<T>, need: usize) -> *mut T {
    scratch.clear();
    scratch.reserve(need);
    scratch.as_mut_ptr()
}

pub(crate) fn check(v_len: usize, bounds: &[usize]) {
    assert!(bounds.len() >= 2 && bounds[0] == 0 && *bounds.last().unwrap() == v_len);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    Start,
    End,
    Buf,
}

const ZONES: [Zone; 3] = [Zone::Start, Zone::End, Zone::Buf];

fn free_zone(x: Zone, y: Zone) -> Zone {
    *ZONES.iter().find(|&&z| z != x && z != y).unwrap()
}

/// A half of `c = 2^k + r` runs pays a whole level (`h` moves) for its `r`
/// extra runs: the `r` disjoint neighbour pairs of smallest total merged in
/// place first (`merge_two`, about 1.5 moves per element), when that costs
/// less than `h`. Returns the first index of each pair (ascending) and the
/// cost, or `None`.
pub fn shrink(l: &[usize], h: usize) -> Option<(Vec<usize>, usize)> {
    let mut w = ShrinkWork::default();
    let cost = w.run(l.len(), |i| l[i] + l[i + 1], h)?;
    Some((w.picks, cost))
}

/// The work vectors of `shrink`, reusable (the plan of `tree::hybrid`
/// computes it at many nodes).
#[derive(Default)]
pub struct ShrinkWork {
    pairs: Vec<(usize, usize)>,
    used: Vec<bool>,
    pub picks: Vec<usize>,
}

impl ShrinkWork {
    /// `shrink` over `c` runs, `pair(i)` the length of runs `i` and `i + 1`;
    /// the pairs left in `self.picks`.
    pub fn run(&mut self, c: usize, pair: impl Fn(usize) -> usize, h: usize) -> Option<usize> {
        if c <= 2 || c.is_power_of_two() {
            return None;
        }
        let r = c - (1 << (levels(c) - 1));
        // Lower bound before sorting: r pairs, each at least the shortest
        // (exact for equal runs).
        let low = (0..c - 1).map(&pair).min().unwrap_or(0) * r;
        if low + low / 2 >= h {
            return None;
        }
        self.pairs.clear();
        self.pairs.extend((0..c - 1).map(|i| (pair(i), i)));
        self.pairs.sort_unstable();
        self.used.clear();
        self.used.resize(c, false);
        self.picks.clear();
        let mut sum = 0;
        for &(len, i) in self.pairs.iter() {
            if self.picks.len() == r {
                break;
            }
            if !self.used[i] && !self.used[i + 1] {
                self.used[i] = true;
                self.used[i + 1] = true;
                self.picks.push(i);
                sum += len;
            }
        }
        let cost = sum + sum / 2;
        if self.picks.len() < r || cost >= h {
            return None;
        }
        self.picks.sort_unstable();
        Some(cost)
    }
}

/// The lengths after merging the pairs `picks`.
pub fn merged(l: &[usize], picks: &[usize]) -> Vec<usize> {
    let mut out = Vec::with_capacity(l.len());
    let mut i = 0;
    while i < l.len() {
        if picks.binary_search(&i).is_ok() {
            out.push(l[i] + l[i + 1]);
            i += 2;
        } else {
            out.push(l[i]);
            i += 1;
        }
    }
    out
}

/// Order of the moves of the rotation, X having `a` levels and Y `b`: one
/// of them odd, all of X then all of Y; both even, X, Y, the rest of X, the
/// rest of Y (pairs of moves of one half cancel out; checked by the test
/// `order_rule`). `None`: (0, even) or (even, 0). Phases: (half, moves).
pub fn order(a: usize, b: usize) -> Option<[(usize, usize); 4]> {
    if (a == 0 && b % 2 == 0) || (b == 0 && a % 2 == 0) {
        return None;
    }
    if a % 2 == 1 || b % 2 == 1 {
        Some([(0, a), (1, b), (0, 0), (1, 0)])
    } else {
        Some([(0, 1), (1, 1), (0, a - 1), (1, b - 1)])
    }
}

/// The rotation over `v` whose runs are `bounds` minus `off` (absolute
/// bounds of a range starting at `off`), with the buffer `bp` (room for
/// `v.len() / 2 + 1`); `sh`: the halves shrunk first (`shrink`). One
/// allocation (the run lengths), none for the order of the moves.
pub fn rotation_in<T: Copy + PartialOrd>(v: &mut [T], bounds: &[usize], off: usize, bp: *mut T, sh: bool) {
    let n = v.len();
    if bounds.len() <= 3 {
        // At most 2 runs: one in-place merge.
        if bounds.len() == 3 {
            merge_two(v, bp, 0, bounds[1] - off, n);
        }
        return;
    }
    if n % 2 == 1 {
        // TODO: odd n; for now the last element inserted afterwards.
        let mut b: Vec<usize> = bounds.iter().map(|&x| (x - off).min(n - 1)).collect();
        b.dedup();
        rotation_in(&mut v[..n - 1], &b, 0, bp, sh);
        insert_last(v);
        return;
    }
    let h = n / 2;
    // Run lengths of both halves in one vector (the run across the middle
    // cut in two): half s = lens[seg[s].0..][..seg[s].1].
    let mid = off + h;
    let mut lens = Vec::with_capacity(bounds.len());
    let mut c0 = 0;
    for w in bounds.windows(2) {
        if w[0] < mid && w[1] > mid {
            lens.push(mid - w[0]);
            c0 = lens.len();
            lens.push(w[1] - mid);
        } else {
            lens.push(w[1] - w[0]);
            if w[1] == mid {
                c0 = lens.len();
            }
        }
    }
    let mut seg = [(0, c0), (c0, lens.len() - c0)];
    if sh {
        for (s, sg) in seg.iter_mut().enumerate() {
            let l = &lens[sg.0..sg.0 + sg.1];
            if let Some((picks, _)) = shrink(l, h) {
                let mut at = s * h;
                let mut i = 0;
                for &pk in picks.iter() {
                    while i < pk {
                        at += l[i];
                        i += 1;
                    }
                    let (a, b) = (l[pk], l[pk + 1]);
                    merge_two(v, bp, at, at + a, at + a + b);
                    at += a + b;
                    i += 2;
                }
                let m = merged(l, &picks);
                lens[sg.0..sg.0 + m.len()].copy_from_slice(&m);
                sg.1 = m.len();
            }
        }
    }
    let mut zone = [Zone::Start, Zone::End];
    let lv = [levels(seg[0].1), levels(seg[1].1)];

    let p = v.as_mut_ptr();
    let base = |z: Zone| -> *mut T {
        match z {
            Zone::Start => p,
            Zone::End => unsafe { p.add(h) },
            Zone::Buf => bp,
        }
    };

    // One move: half `s` merges one level into the empty zone; its lengths
    // compacted in place.
    let mut play = |s: usize, zone: &mut [Zone; 2]| {
        let f = free_zone(zone[0], zone[1]);
        let (src, dst) = (base(zone[s]), base(f));
        let (st, c) = seg[s];
        let (mut i, mut o, mut at) = (0, st, 0);
        unsafe {
            while i + 1 < c {
                let (x, y) = (lens[st + i], lens[st + i + 1]);
                merge_out(src.add(at), x, src.add(at + x), y, dst.add(at));
                lens[o] = x + y;
                at += x + y;
                i += 2;
                o += 1;
            }
            if i < c {
                let x = lens[st + i];
                ptr::copy_nonoverlapping(src.add(at), dst.add(at), x);
                count(x);
                lens[o] = x;
                o += 1;
            }
        }
        seg[s].1 = o - st;
        zone[s] = f;
    };

    match order(lv[0], lv[1]) {
        Some(phases) => {
            for (s, k) in phases {
                for _ in 0..k {
                    play(s, &mut zone);
                }
            }
        },
        None => {
            // One half without level, the other with an even number: the
            // latter plays alone (back home), the former copied to the buffer.
            let s = if lv[0] > 0 { 0 } else { 1 };
            for _ in 0..lv[s] {
                play(s, &mut zone);
            }
            let o = 1 - s;
            unsafe { ptr::copy_nonoverlapping(base(zone[o]), bp, h) };
            count(h);
            zone[o] = Zone::Buf;
        },
    }

    // The final hole merge: one half in the buffer, the other at one end.
    let (a, b) = (base(zone[0]), base(zone[1]));
    unsafe {
        match (zone[0], zone[1]) {
            (Zone::Buf, Zone::End) => hole_fwd(a, h, b, h, p, false),
            (Zone::Buf, Zone::Start) => hole_bwd(a, h, b, h, p, false),
            (Zone::End, Zone::Buf) => hole_fwd(a, h, b, h, p, true),
            (Zone::Start, Zone::Buf) => hole_bwd(a, h, b, h, p, true),
            z => unreachable!("{:?}", z),
        }
    }
}

/// The last element moved to its place in the sorted rest (stable).
fn insert_last<T: Copy + PartialOrd>(v: &mut [T]) {
    let n = v.len();
    let x = v[n - 1];
    let pos = v[..n - 1].partition_point(|y| *y <= x);
    v.copy_within(pos..n - 1, pos + 1);
    v[pos] = x;
    count(n - pos);
}
