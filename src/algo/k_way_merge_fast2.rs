//! Work copy of `k_way_merge_fast` (algorithmic phase): ping-pong rounds.
//!
//! Faster variant of `k_way_merge` (same pairwise merge order, same stable
//! result):
//!
//! - each 2-way merge first trims, by binary search, the elements already at
//!   their final place (head of the left run, tail of the right run);
//! - when one of the two remaining runs is tiny, its elements are inserted by
//!   binary search and a block shift (`rotate`, a memmove), instead of moving
//!   the other run one element at a time (e.g. a sorted array with one element
//!   moved: a single shift instead of a full merge).
//!
//! The original module is kept unchanged for the sorts that use it.

use super::k_way_merge::merge2;

/// Up to this many elements, a run is inserted instead of merged.
const INSERT_MAX: usize = 16;

/// Stable merge of the sorted runs `[start, middle)` and `[middle, end)`.
fn merge2_fast<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut Vec<T>,
    start: usize,
    middle: usize,
    end: usize,
) {
    if start >= middle || middle >= end || arr[middle - 1] <= arr[middle] {
        return;
    }
    // Left elements <= the first right element are already in place (stable:
    // equal elements of the left run stay first).
    let first_right = arr[middle];
    let start = start + arr[start..middle].partition_point(|x| *x <= first_right);
    // Right elements >= the last left element are already in place.
    let last_left = arr[middle - 1];
    let end = middle + arr[middle..end].partition_point(|x| *x < last_left);

    let left = middle - start;
    let right = end - middle;
    if arr[end - 1] < arr[start] {
        // The right run lies entirely below the left one (e.g. a rotated
        // sorted array): a block rotation, no merge.
        arr[start..end].rotate_left(left);
        return;
    }
    if right <= INSERT_MAX {
        // Insert the right elements, in order, after the equal left ones.
        let mut lo = start;
        for idx in middle..end {
            let value = arr[idx];
            let pos = lo + arr[lo..idx].partition_point(|x| *x <= value);
            arr[pos..=idx].rotate_right(1);
            lo = pos + 1;
        }
    } else if left <= INSERT_MAX {
        // Insert the left elements, last first, before the equal right ones.
        let mut hi = end;
        for idx in (start..middle).rev() {
            let value = arr[idx];
            let pos = idx + 1 + arr[idx + 1..hi].partition_point(|x| *x < value);
            arr[idx..pos].rotate_left(1);
            hi = pos - 1;
        }
    } else {
        // Balanced runs (the shorter at least a quarter) within half the L2:
        // two-ended merge into the scratch, copied back. Skewed runs keep the
        // merge below (shorter run copied, long predictable stretches), and
        // so do merges beyond the cache, where the extra copy costs more.
        let len = end - start;
        if bidir_enabled()
            && 4 * left.min(right) >= len
            && len * std::mem::size_of::<T>() <= super::super::dedicated::cache_info::l2_bytes() / 2
        {
            if copy.len() < len {
                copy.resize(len, arr[0]);
            }
            // Blocks already ordered against the other run go straight to
            // their place: the left elements above the last right one end the
            // output, the right elements below the first left one start it
            // (u16 sawtooth with two teeth at 100k keys: 0.70 to 0.47 ns
            // per key).
            let src = &arr[start..end];
            let mid = middle - start;
            let (mut tail, mut head) = (0, 0);
            if blocks_enabled() {
                tail = mid - src[..mid].partition_point(|x| *x <= src[len - 1]);
                head = src[mid..].partition_point(|x| *x < src[0]);
            }
            let dst = &mut copy[..len];
            dst[..head].copy_from_slice(&src[mid..mid + head]);
            dst[len - tail..].copy_from_slice(&src[mid - tail..mid]);
            merge_bidir(&src[..mid - tail], &src[mid + head..], &mut dst[head..len - tail]);
            arr[start..end].copy_from_slice(&copy[..len]);
            return;
        }
        let needed = left.min(right);
        if copy.len() < needed {
            copy.resize(needed, arr[0]);
        }
        if gallop_enabled() && 4 * needed < len {
            if right < left {
                gallop_back(arr, &mut copy[..right], start, middle, end);
            } else {
                gallop_forward(arr, &mut copy[..left], start, middle, end);
            }
            return;
        }
        merge2(arr, copy, start, middle, end);
    }
}

/// Stretch length from which the skewed merge gallops (TimSort's
/// `MIN_GALLOP` is 7): shorter stretches are moved one element at a time.
const GALLOP_MIN: usize = 8;

/// Number of trailing elements of the sorted `a` greater than `v`, by
/// exponential search from the end, then binary search: O(log count).
#[inline]
fn count_back_greater<T: Copy + PartialOrd>(a: &[T], v: T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[n - hi] > v {
        lo = hi;
        hi *= 2;
    }
    let hi = hi.min(n + 1);
    let (s, e) = (n + 1 - hi, n - lo);
    lo + (e - s) - a[s..e].partition_point(|x| *x <= v)
}

/// Number of leading elements of the sorted `a` lower than `v` (exponential
/// then binary search).
#[inline]
fn count_front_lower<T: Copy + PartialOrd>(a: &[T], v: T) -> usize {
    let n = a.len();
    let (mut lo, mut hi) = (0, 1);
    while hi <= n && a[hi - 1] < v {
        lo = hi;
        hi *= 2;
    }
    let e = hi.min(n + 1) - 1;
    lo + a[lo..e].partition_point(|x| *x < v)
}

/// Stable merge of a long left run `[start, middle)` and a short right run
/// `[middle, end)` (copied to `buf`), from the back: before each right
/// element, the left elements greater than it are moved as one block
/// (galloping, as in TimSort), instead of one at a time. A short run
/// merged into a long one (a sawtooth with a short last tooth, a few late
/// elements) moves the long stretches with memmove.
fn gallop_back<T: Copy + PartialOrd>(arr: &mut [T], buf: &mut [T], start: usize, middle: usize, end: usize) {
    buf.copy_from_slice(&arr[middle..end]);
    let (mut j, mut i, mut pos) = (middle, buf.len(), end);
    while i > 0 && j > start {
        let v = buf[i - 1];
        // Short stretches element by element; from GALLOP_MIN on, galloping.
        let mut c = 0;
        while c < GALLOP_MIN && c < j - start && arr[j - 1 - c] > v {
            arr[pos - 1 - c] = arr[j - 1 - c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_back_greater(&arr[start..j - c], v);
            arr.copy_within(j - c - more..j - c, pos - c - more);
            c += more;
        }
        pos -= c;
        j -= c;
        pos -= 1;
        arr[pos] = v;
        i -= 1;
    }
    arr[start..start + i].copy_from_slice(&buf[..i]);
}

/// Mirror of `gallop_back`: a short left run (copied to `buf`) merged into
/// a long right run from the front, the right elements lower than each
/// left element moved as one block.
fn gallop_forward<T: Copy + PartialOrd>(arr: &mut [T], buf: &mut [T], start: usize, middle: usize, end: usize) {
    buf.copy_from_slice(&arr[start..middle]);
    let m = buf.len();
    let (mut j, mut i, mut pos) = (middle, 0, start);
    while i < m && j < end {
        let v = buf[i];
        let mut c = 0;
        while c < GALLOP_MIN && j + c < end && arr[j + c] < v {
            arr[pos + c] = arr[j + c];
            c += 1;
        }
        if c == GALLOP_MIN {
            let more = count_front_lower(&arr[j + c..end], v);
            arr.copy_within(j + c..j + c + more, pos + c);
            c += more;
        }
        pos += c;
        j += c;
        arr[pos] = v;
        pos += 1;
        i += 1;
    }
    arr[pos..pos + (m - i)].copy_from_slice(&buf[i..]);
}

/// Tests / benchmarks: no head / tail blocks before the two-ended merge.
#[cfg(test)]
pub(crate) static BLOCKS_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn blocks_enabled() -> bool {
    #[cfg(test)]
    {
        !BLOCKS_OFF.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Tests / benchmarks: skewed runs merged by `merge2` (as before).
#[cfg(test)]
pub(crate) static GALLOP_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn gallop_enabled() -> bool {
    #[cfg(test)]
    {
        !GALLOP_OFF.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Tests / benchmarks: previous merge for balanced runs.
#[cfg(test)]
pub(crate) static BIDIR_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn bidir_enabled() -> bool {
    #[cfg(test)]
    {
        !BIDIR_OFF.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Stable merge of the sorted runs `a` (first) and `b` into `dst`, from both ends
/// at once and without branches: each step writes the smallest remaining
/// element at the front and the largest at the back, two independent
/// chains of loads and selects (a single branch-free chain waits on each
/// load; a branchy merge mispredicts on irregular interleavings). The
/// two-ended phase lasts as many steps as the shorter run has elements, so
/// no read leaves a run; the middle that remains is merged forward.
#[inline]
fn merge_bidir<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
    let len = a.len() + b.len();
    debug_assert!(dst.len() == len && !a.is_empty() && !b.is_empty());
    let steps = a.len().min(b.len());
    // Left run a[lf..le), right run b[rf..re); output [df, de).
    let (mut lf, mut le, mut rf, mut re) = (0usize, a.len(), 0usize, b.len());
    let (mut df, mut de) = (0usize, len);
    // SAFETY: before each of the `steps` two-ended steps each end has taken
    // fewer than `steps` elements from each run, so the reads stay inside
    // the runs; each output slot is written once (df < de).
    unsafe {
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let d = dst.as_mut_ptr();
        for _ in 0..steps {
            let (x, y) = (*pa.add(lf), *pb.add(rf));
            let take_right = y < x;
            *d.add(df) = if take_right { y } else { x };
            rf += take_right as usize;
            lf += (!take_right) as usize;
            df += 1;
            let (x, y) = (*pa.add(le - 1), *pb.add(re - 1));
            let take_left = y < x;
            de -= 1;
            *d.add(de) = if take_left { x } else { y };
            le -= take_left as usize;
            re -= (!take_left) as usize;
        }
    }
    // Middle: what is left of both runs, between df and de.
    while lf < le && rf < re {
        let (x, y) = (a[lf], b[rf]);
        if y < x {
            dst[df] = y;
            rf += 1;
        } else {
            dst[df] = x;
            lf += 1;
        }
        df += 1;
    }
    dst[df..df + (le - lf)].copy_from_slice(&a[lf..le]);
    df += le - lf;
    dst[df..df + (re - rf)].copy_from_slice(&b[rf..re]);
    debug_assert_eq!(df + (re - rf), de);
}

/// Whether a branchy merge of the sorted runs `run[..mid]` and `run[mid..]`
/// is predictable, from its first 64 steps: few switches between the runs
/// (long stretches), or a sequence of choices (left or right run) periodic
/// with a period of at most 16 (runs holding the same values alternate). A
/// branch predictor learns such patterns (about 1 ns per element and per
/// merge round measured); on irregular interleavings (random runs) it
/// mispredicts about one step in two (about 3.5-4 ns per element and per
/// round, as much as a whole radix sort).
pub(crate) fn predictable_merge<T: Copy + PartialOrd>(run: &[T], mid: usize) -> bool {
    let (mut i, mut j) = (0, mid);
    let mut bits = 0u64;
    let mut steps = 0;
    while steps < 64 && i < mid && j < run.len() {
        let take_left = run[i] <= run[j];
        bits |= u64::from(take_left) << steps;
        if take_left {
            i += 1;
        } else {
            j += 1;
        }
        steps += 1;
    }
    if steps < 64 {
        return true;
    }
    if (bits ^ (bits << 1)).count_ones() - (bits & 1) as u32 <= 16 {
        return true;
    }
    (1..=16).any(|p| ((bits ^ (bits << p)) >> p).count_ones() <= 4)
}

/// Same as `k_way_merge::k_way_merge` (same result), with the fast 2-way
/// merge. From 3 runs on (2 rounds or more), the rounds alternate between
/// the array and a buffer of the same size (ping-pong): each round writes
/// every element once into the other side, no copy back after each merge,
/// one final copy when the rounds end in the buffer.
pub fn k_way_merge_fast2<T: Copy + PartialOrd>(arr: &mut [T], separators: &mut Vec<usize>) {
    if separators.len() <= 2 {
        return;
    }
    // Ping-pong while the array and its buffer stay in the cache (a quarter
    // of the L2: beyond, each round streams both through memory and the
    // in-place rounds, copying only the shorter run, win), from 4096 keys
    // (below, the full buffer costs more than the copies it saves). Float
    // sawtooth at 10k-200k: 10-20 % faster; at 600k-1M without the limit,
    // 1.4-1.7x slower.
    let bytes = 2 * arr.len() * std::mem::size_of::<T>();
    if separators.len() >= 4
        && arr.len() >= 4096
        && bytes <= super::super::dedicated::cache_info::l2_bytes() / 4
        && pingpong_enabled()
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
            merge2_fast(arr, &mut copy, separators[i2], separators[i2 + 1], separators[i2 + 2]);
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
}

/// Ping-pong rounds (see `k_way_merge_fast2`).
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
        merge_bidir(a, b, dst);
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

/// Tests / benchmarks: rounds merged in place as before (no ping-pong).
#[cfg(test)]
pub(crate) static PINGPONG_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
fn pingpong_enabled() -> bool {
    #[cfg(test)]
    {
        !PINGPONG_OFF.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}
