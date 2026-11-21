//! `hybrid`, buffer n/2: a powersort tree over the runs (each split at the
//! boundary where the midpoints of its two runs differ at the highest bit,
//! the leftmost on ties), each node merged one of the ways below, chosen
//! bottom up by a cost in moves (elements written):
//!
//! ```text
//! home   the node ends in its own range: one child sorted into the buffer
//!        (that child's children home, then merge_out), the other home, then
//!        a hole merge (trimmed, galloping when skewed); two runs: merge_two
//! buf    the node ends in the buffer: both children home, then merge_out
//! rot    home by the rotation of three zones over the node's range, when
//!        its middle is a run boundary (no run cut in two), each half first
//!        brought to a power-of-two count of runs when cheap (`shrink`)
//! done   children already in order (highest left <= lowest right): nothing
//! inplace both children home, then merge_two
//! ```
//! The plan reads only the runs' ends (lowest and highest value of each
//! node): the elements the trims leave in place are counted from whole runs
//! (exact when the runs' ranges do not overlap), so that the cost of each
//! way follows the values, in O(runs) per tree level.
//! Only one buffer zone is in use at a time: a node's children are done
//! (using the buffer as their scratch) before the node writes into it.

use super::inplace::{gallop_back_from, gallop_forward_from, merge_two};
use super::kernel::{count, hole_bwd, hole_fwd, merge_out, skewed};
use super::rotation::{check, levels, room, rotation_in, ShrinkWork};
use core::ptr;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum How {
    #[default]
    Leaf,
    /// Two runs: `merge_two`.
    Two,
    /// The left child into the buffer.
    Left,
    /// The right child into the buffer.
    Right,
    Rot,
    /// Values already in order between the children: both home, nothing.
    Done,
    /// Both children home, then `merge_two` (trims, galloping): cheap when
    /// the values leave much in place.
    InPlace,
}

/// A boundary between two runs, and the node it splits (the internal nodes
/// of the tree are the boundaries; the leaves, the runs, need nothing).
#[derive(Clone, Copy, Default)]
struct Bd {
    pw: u32,
    /// Children in the Cartesian tree by power (0: a run).
    cl: u32,
    cr: u32,
    /// The boundary below on the building stack.
    up: u32,
    how: How,
    /// Cost (moves) to end home.
    home: u64,
}

/// Powersort power of the boundary `s1` between runs `s0..s1` and `s1..s2`.
fn power(s0: usize, s1: usize, s2: usize, n: usize) -> u32 {
    if n < 1 << 31 {
        // 64-bit division (the 128-bit one is a library call): midpoints on
        // 32 bits, enough while a run holds at least one element.
        let (a, b) = ((((s0 + s1) as u64) << 31) / n as u64, (((s1 + s2) as u64) << 31) / n as u64);
        return ((a ^ b) as u32).leading_zeros();
    }
    let n = n as u128;
    let a = (((s0 + s1) as u128) << 63) / n;
    let b = (((s1 + s2) as u128) << 63) / n;
    (a as u64 ^ b as u64).leading_zeros()
}

/// A half of `c` runs that `shrink` leaves as it is.
fn no_shrink(c: usize) -> bool { c <= 2 || c.is_power_of_two() }

struct Plan<'a, T> {
    bounds: &'a [usize],
    /// The values (null: not looked at), read only while planning, before
    /// any merge writes (`Done`, trims).
    vals: *const T,
    /// Indexed by boundary, `1..k` (one allocation for the whole plan).
    t: Vec<Bd>,
    /// Room of the buffer.
    cap: usize,
    rot: bool,
    /// Allocates only when used.
    work: ShrinkWork,
}

impl<T: Copy + PartialOrd> Plan<'_, T> {
    /// Costs (home, buffer) of the node of runs `i..j`, split at boundary
    /// `x` (unused for one run), and the positions of its lowest and highest
    /// values (with `vals`); its way stored at `x`.
    fn build(&mut self, i: usize, j: usize, x: usize) -> (u64, u64, usize, usize) {
        let b = self.bounds;
        let m = b[j] - b[i];
        if j == i + 1 {
            return (0, m as u64, b[i], b[j] - 1);
        }
        let k = x;
        let (ah, ab, alo, ahi) = self.build(i, k, self.t[k].cl as usize);
        let (ch, cb, clo, chi) = self.build(k, j, self.t[k].cr as usize);
        let (la, lb) = (b[k] - b[i], b[j] - b[k]);
        let mm = m as u64;
        let buf = mm + ah + ch;
        let (mut lo, mut hi) = (alo, ahi);
        let mut best = (u64::MAX, How::Leaf);
        if !self.vals.is_null() {
            let v = |i: usize| unsafe { &*self.vals.add(i) };
            if v(clo) < v(lo) {
                lo = clo;
            }
            if v(chi) >= v(hi) {
                hi = chi;
            }
            if v(ahi) <= v(clo) {
                best = (ah + ch, How::Done);
            }
        }
        // Elements left in place at both ends by the trims: the left ones
        // not above the lowest right one, the right ones not below the
        // highest left one.
        let (pre, suf) = if self.vals.is_null() || best.1 != How::Leaf {
            (0, 0)
        } else {
            (self.count(i, k, clo, true), self.count(k, j, ahi, false))
        };
        let (ta, tb) = ((la - pre) as u64, (lb - suf) as u64);
        if best.1 != How::Leaf {
        } else if j == i + 2 {
            best = (ta + tb + ta.min(tb), How::Two);
        } else {
            if la <= self.cap {
                best = (ab + ch + mm - suf as u64, How::Left);
            }
            if lb <= self.cap && ah + cb + mm - (pre as u64) < best.0 {
                best = (ah + cb + mm - pre as u64, How::Right);
            }
            if !self.vals.is_null() && ah + ch + ta + tb + ta.min(tb) < best.0 {
                best = (ah + ch + ta + tb + ta.min(tb), How::InPlace);
            }
        }
        let mid =
            if m.is_multiple_of(2) { b[i..j].binary_search(&(b[i] + m / 2)).ok() } else { None };
        let free = best.1 == How::Done;
        if let (true, true, Some(c0), false) = (self.rot, j - i >= 3, mid, free) {
            // No run cut: the halves are slices of the bounds.
            let (c1, h) = (j - i - c0, m / 2);
            // shrink saves at most one level per half.
            let low = |c: usize| levels(c) - !no_shrink(c) as usize;
            if (((low(c0) + low(c1)) * h + m) as u64) <= best.0 {
                let mut lv = [0; 2];
                let mut extra = 0;
                for (s, (o, c)) in [(i, c0), (i + c0, c1)].iter().copied().enumerate() {
                    lv[s] = match self.work.run(c, |y| b[o + y + 2] - b[o + y], h) {
                        Some(cost) => {
                            extra += cost;
                            levels(c - self.work.picks.len())
                        },
                        None => levels(c),
                    };
                }
                let bad = (lv[0] == 0 && lv[1] % 2 == 0) || (lv[1] == 0 && lv[0] % 2 == 0);
                let rc = ((lv[0] + lv[1] + bad as usize) * h + m + extra) as u64;
                // On a tie the tree (measured faster).
                if rc < best.0 {
                    best = (rc, How::Rot);
                }
            }
        }
        let t = &mut self.t[k];
        t.how = best.1;
        t.home = best.0;
        (best.0, buf, lo, hi)
    }

    /// In runs `i..j`, the count of elements not above (`low`) or not below
    /// the value at position `at`, from the runs' ends only: whole runs on
    /// the right side of it (a run that straddles it counts 0; exact when
    /// the runs' ranges do not overlap).
    fn count(&self, i: usize, j: usize, at: usize, low: bool) -> usize {
        let b = self.bounds;
        let v = |p: usize| unsafe { &*self.vals.add(p) };
        let x = v(at);
        let mut c = 0;
        for r in i..j {
            let (s, e) = (b[r], b[r + 1]);
            if (low && v(e - 1) <= x) || (!low && v(s) >= x) {
                c += e - s;
            }
        }
        c
    }

    /// Buffer room the node of runs `i..j` (split at `x`) needs to end home:
    /// what its way writes into the buffer, or its children's need.
    fn need_home(&self, i: usize, j: usize, x: usize) -> usize {
        if j == i + 1 {
            return 0;
        }
        let b = self.bounds;
        let (la, lb) = (b[x] - b[i], b[j] - b[x]);
        let t = &self.t[x];
        let (cl, cr) = (t.cl as usize, t.cr as usize);
        match t.how {
            How::Leaf => unreachable!(),
            How::Two => la.min(lb),
            How::Rot => (la + lb) / 2 + 1,
            How::Done => self.need_home(i, x, cl).max(self.need_home(x, j, cr)),
            How::InPlace => la.min(lb).max(self.need_home(i, x, cl)).max(self.need_home(x, j, cr)),
            How::Left => self.need_home(x, j, cr).max(self.need_buf(i, x, cl)),
            How::Right => self.need_home(i, x, cl).max(self.need_buf(x, j, cr)),
        }
    }

    /// Buffer room to sort the node into the buffer: the node itself, after
    /// its children home.
    fn need_buf(&self, i: usize, j: usize, x: usize) -> usize {
        let b = self.bounds;
        let m = b[j] - b[i];
        if j == i + 1 {
            return m;
        }
        let t = &self.t[x];
        m.max(self.need_home(i, x, t.cl as usize)).max(self.need_home(x, j, t.cr as usize))
    }

    /// Node of runs `i..j` (split at `x`) sorted in its own range.
    fn home(&self, v: &mut [T], i: usize, j: usize, x: usize, bp: *mut T) {
        if j == i + 1 {
            return;
        }
        let b = self.bounds;
        let (s, mid, e) = (b[i], b[x], b[j]);
        let t = &self.t[x];
        let (cl, cr) = (t.cl as usize, t.cr as usize);
        match t.how {
            How::Leaf => unreachable!(),
            How::Two => merge_two(v, bp, s, mid, e),
            How::Rot => rotation_in(&mut v[s..e], &b[i..=j], s, bp, true),
            How::Done => {
                self.home(v, i, x, cl, bp);
                self.home(v, x, j, cr, bp);
            },
            How::InPlace => {
                self.home(v, i, x, cl, bp);
                self.home(v, x, j, cr, bp);
                merge_two(v, bp, s, mid, e);
            },
            How::Left => {
                self.home(v, x, j, cr, bp);
                self.to_buf(v, i, x, cl, bp);
                unsafe { hole_left(v, bp, s, mid, e) };
            },
            How::Right => {
                self.home(v, i, x, cl, bp);
                self.to_buf(v, x, j, cr, bp);
                unsafe { hole_right(v, bp, s, mid, e) };
            },
        }
    }

    /// Node of runs `i..j` (split at `x`) sorted into the buffer.
    fn to_buf(&self, v: &mut [T], i: usize, j: usize, x: usize, bp: *mut T) {
        let b = self.bounds;
        let (s, e) = (b[i], b[j]);
        if j == i + 1 {
            unsafe { ptr::copy_nonoverlapping(v.as_ptr().add(s), bp, e - s) };
            count(e - s);
            return;
        }
        let mid = b[x];
        self.home(v, i, x, self.t[x].cl as usize, bp);
        self.home(v, x, j, self.t[x].cr as usize, bp);
        let p = v.as_mut_ptr();
        unsafe { merge_out(p.add(s), mid - s, p.add(mid), e - mid, bp) };
    }
}

/// The left run in `bp[..mid - s]`, the right one home at `v[mid..e]`:
/// merged into `v[s..e]`.
unsafe fn hole_left<T: Copy + PartialOrd>(v: &mut [T], bp: *mut T, s: usize, mid: usize, e: usize) {
    let la = mid - s;
    let a = std::slice::from_raw_parts(bp as *const T, la);
    let last = a[la - 1];
    // The right elements not below the last left one are in place.
    let lb = v[mid..e].partition_point(|x| *x < last);
    if lb == 0 {
        ptr::copy_nonoverlapping(bp, v.as_mut_ptr().add(s), la);
        count(la);
        return;
    }
    if skewed(la, la + lb) {
        gallop_forward_from(v, a, s, mid, mid + lb);
    } else {
        let p = v.as_mut_ptr();
        hole_fwd(bp, la, p.add(mid), lb, p.add(s), false);
    }
}

/// The right run in `bp[..e - mid]`, the left one home at `v[s..mid]`.
unsafe fn hole_right<T: Copy + PartialOrd>(
    v: &mut [T],
    bp: *mut T,
    s: usize,
    mid: usize,
    e: usize,
) {
    let lb = e - mid;
    let b = std::slice::from_raw_parts(bp as *const T, lb);
    let first = b[0];
    // The left elements not above the first right one are in place.
    let start = s + v[s..mid].partition_point(|x| *x <= first);
    let la = mid - start;
    if la == 0 {
        ptr::copy_nonoverlapping(bp, v.as_mut_ptr().add(mid), lb);
        count(lb);
        return;
    }
    if skewed(lb, la + lb) {
        gallop_back_from(v, b, start, mid, e);
    } else {
        let p = v.as_mut_ptr();
        hole_bwd(p.add(start), la, bp, lb, p.add(start), true);
    }
}

/// The plan, its root boundary and its cost.
fn plan<'a, T: Copy + PartialOrd>(
    bounds: &'a [usize],
    vals: *const T,
    cap: usize,
    rot: bool,
) -> (Plan<'a, T>, usize, u64) {
    let n = *bounds.last().unwrap();
    let k = bounds.len() - 1;
    let mut t = vec![Bd::default(); k];
    // Stack (linked through `up`, 0 at the bottom) of boundaries of
    // increasing power: the leftmost of the lowest power ends at the root
    // (powersort merges equal powers left first).
    let mut top = 0;
    for x in 1..k {
        let pw = power(bounds[x - 1], bounds[x], bounds[x + 1], n);
        t[x].pw = pw;
        let mut last = 0;
        while top != 0 && t[top].pw > pw {
            last = top;
            top = t[top].up as usize;
        }
        t[x].cl = last as u32;
        if top != 0 {
            t[top].cr = x as u32;
        }
        t[x].up = top as u32;
        top = x;
    }
    // The root: the bottom of the stack.
    let mut root = top;
    while root != 0 && t[root].up != 0 {
        root = t[root].up as usize;
    }
    let mut p = Plan { bounds, vals, t, cap, rot, work: ShrinkWork::default() };
    let (home, ..) = p.build(0, k, root);
    (p, root, home)
}

/// Powersort tree, home / buffer / rotation / in place per node by cost,
/// looking at the runs' ends.
pub fn hybrid<T: Copy + PartialOrd>(v: &mut [T], bounds: &[usize], scratch: &mut Vec<T>) {
    run(v, bounds, scratch, true, true)
}

fn run<T: Copy + PartialOrd>(
    v: &mut [T],
    bounds: &[usize],
    scratch: &mut Vec<T>,
    rot: bool,
    val: bool,
) {
    check(v.len(), bounds);
    if bounds.len() <= 2 {
        return;
    }
    let n = v.len();
    if bounds.len() == 3 {
        let bp = room(scratch, bounds[1].min(n - bounds[1]) + 1);
        return merge_two(v, bp, 0, bounds[1], n);
    }
    let vals = if val { v.as_ptr() } else { ptr::null() };
    let (p, root, _) = plan(bounds, vals, n / 2, rot);
    // Only the room the plan uses: each page of a fresh buffer costs a page
    // fault, so the buffer is not made larger than the plan needs.
    let k = bounds.len() - 1;
    let need = p.need_home(0, k, root) + 1;
    #[cfg(any())]
    {
        // Tests of kwaymerge only: a guard after the room, checked untouched.
        const GUARD: usize = 4096;
        let bytes = GUARD * size_of::<T>().max(1);
        let bp = room(scratch, need + GUARD);
        let g = unsafe { (bp as *mut u8).add(need * size_of::<T>()) };
        unsafe { ptr::write_bytes(g, 0xA5, bytes) };
        p.home(v, 0, k, root, bp);
        let ok = unsafe { std::slice::from_raw_parts(g, bytes) }.iter().all(|&x| x == 0xA5);
        assert!(ok, "buffer written beyond the planned room {need}");
        return;
    }
    #[allow(unreachable_code)]
    {
        let bp = room(scratch, need);
        p.home(v, 0, k, root, bp);
    }
}
