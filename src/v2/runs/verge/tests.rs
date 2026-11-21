//! Unit tests of `v2/runs/verge/mod.rs` (a child module: sees its private items).

use super::*;
use crate::v2::runs::merge::merge_runs;
use crate::v2::testing::*;

/// Verge preprocessing then the merge of its runs sorts every shape,
/// stable or not; the separators cut the array into sorted runs. Runs
/// are built in: a long ascending part, a long descending part, then a
/// random tail (and the reverse layout).
#[test]
fn runs_then_merge() {
    let fallback = |a: &mut [u64], _: usize| a.sort_unstable();
    for_each_case::<u64>(|shape, raw| {
        let n = raw.len();
        let mut layouts = vec![raw.clone()];
        let mut runs = raw.clone();
        runs[..n / 3].sort_unstable();
        runs[n / 3..2 * n / 3].sort_unstable_by(|a, b| b.cmp(a));
        layouts.push(runs.clone());
        runs.reverse();
        layouts.push(runs);
        for (k, a) in layouts.into_iter().enumerate() {
            for &stable in [false, true].iter() {
                let mut got = a.clone();
                let mut separators = if stable {
                    verge_sort_preprocessing_fast::<u64, true>(&mut got, 8, &fallback)
                } else {
                    verge_sort_preprocessing_fast::<u64, false>(&mut got, 8, &fallback)
                };
                let what = format!("{} layout {} n={} stable {}", shape, k, n, stable);
                assert_eq!(separators.first(), Some(&0), "{}", what);
                assert_eq!(separators.last(), Some(&n), "{}", what);
                for w in separators.windows(2) {
                    assert!(w[0] <= w[1] && sorted_by(&got[w[0]..w[1]], |x| x), "{}: run", what);
                }
                merge_runs(&mut got, &mut separators);
                assert!(sorted_by(&got, |x| x), "{}: not sorted", what);
                assert!(same_values(&got, &a), "{}: values changed", what);
            }
        }
    });
}
