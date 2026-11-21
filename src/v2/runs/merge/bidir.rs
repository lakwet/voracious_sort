//! Two-ended merges: branchy for predictable interleavings, branch-free for
//! irregular ones.

/// Stable merge of the sorted runs `a` (first) and `b` into `dst`, from both
/// ends at once: each step writes the smallest remaining element at the
/// front and the largest at the back, two independent chains. The two-ended
/// phase lasts as many steps as the shorter run has elements, so no read
/// leaves a run; the middle that remains is merged forward.
///
/// The form of the loop is fixed instead of left to LLVM, which made it
/// branchy in a generic build and branch-free under `target-cpu=native`
/// (three times slower on one of the two). `BRANCHY`: one real branch per
/// step and end (an empty `asm!` in one arm keeps LLVM from turning it
/// into a select), for
/// predictable interleavings, where the branch predictor runs ahead; else
/// selects for the values and the indices (`select_unpredictable`), for
/// irregular ones, where a branch mispredicts one step in two.
#[inline]
pub(super) fn merge_bidir<T: Copy + PartialOrd, const BRANCHY: bool>(
    a: &[T],
    b: &[T],
    dst: &mut [T],
) {
    let len = a.len() + b.len();
    debug_assert!(dst.len() == len && !a.is_empty() && !b.is_empty());
    let steps = a.len().min(b.len());
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
            if BRANCHY {
                if y < x {
                    *d.add(df) = y;
                    rf += 1;
                    keep_branch();
                } else {
                    *d.add(df) = x;
                    lf += 1;
                }
            } else {
                let take_right = y < x;
                *d.add(df) = std::hint::select_unpredictable(take_right, y, x);
                rf = std::hint::select_unpredictable(take_right, rf + 1, rf);
                lf = std::hint::select_unpredictable(take_right, lf, lf + 1);
            }
            df += 1;
            let (x, y) = (*pa.add(le - 1), *pb.add(re - 1));
            de -= 1;
            if BRANCHY {
                if y < x {
                    *d.add(de) = x;
                    le -= 1;
                    keep_branch();
                } else {
                    *d.add(de) = y;
                    re -= 1;
                }
            } else {
                let take_left = y < x;
                *d.add(de) = std::hint::select_unpredictable(take_left, x, y);
                le = std::hint::select_unpredictable(take_left, le - 1, le);
                re = std::hint::select_unpredictable(take_left, re, re - 1);
            }
        }
    }
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

/// An empty `asm!` in a branch: the compiler keeps the branch instead of
/// turning it into a conditional move. Where inline assembly is not stable
/// (wasm32...) and under Miri, nothing.
#[inline(always)]
fn keep_branch() {
    #[cfg(all(
        not(miri),
        any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "arm",
            target_arch = "aarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch64"
        )
    ))]
    // SAFETY: an empty block, no memory, no stack, flags kept.
    unsafe {
        std::arch::asm!("", options(nomem, nostack, preserves_flags));
    }
}
