//! Branch-free merge from both ends at once.

/// Stable merge of the sorted runs `a` (first) and `b` into `dst`, from both ends
/// at once and without branches: each step writes the smallest remaining
/// element at the front and the largest at the back, two independent
/// chains of loads and selects (a single branch-free chain waits on each
/// load; a branchy merge mispredicts on irregular interleavings). The
/// two-ended phase lasts as many steps as the shorter run has elements, so
/// no read leaves a run; the middle that remains is merged forward.
#[inline]
pub(super) fn merge_bidir<T: Copy + PartialOrd>(a: &[T], b: &[T], dst: &mut [T]) {
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
            let (x, y) = (pa.add(lf), pb.add(rf));
            let take_right = *y < *x;
            *d.add(df) = *pick(take_right, y, x);
            rf += take_right as usize;
            lf += (!take_right) as usize;
            df += 1;
            let (x, y) = (pa.add(le - 1), pb.add(re - 1));
            let take_left = *y < *x;
            de -= 1;
            *d.add(de) = *pick(take_left, x, y);
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

/// `a` when `c`, else `b`, kept a select by LLVM (an `if` on the values
/// became a branch on aarch64: mispredicted half the time on random merges,
/// u32 halves at 100k-10M 2.7-2.9 ns per key). Masked on aarch64,
/// `select_unpredictable` (cmov) on x86 (the same rule as the records'
/// merges).
#[inline(always)]
fn pick<T>(c: bool, a: *const T, b: *const T) -> *const T {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    return std::hint::select_unpredictable(c, a, b);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    return (b as usize ^ ((a as usize ^ b as usize) & (c as usize).wrapping_neg())) as *const T;
}
