//! Correctness at the thresholds derived from the cache (internal: reads
//! the budgets of `v2::cache`). The public API: `tests/correctness`.

use super::super::generators::unsigned_u32::generators_u32;
use rayon::slice::ParallelSliceMut;

/// `V2_QUICK` set: sizes up to 100k only.
fn quick_max() -> usize {
    if std::env::var("V2_QUICK").is_ok() {
        100_000
    } else {
        usize::MAX
    }
}

/// Every key type around the thresholds derived from the cache of this
/// machine (`testing::cache_sizes`: rollercoaster, LSD digits, counting
/// lanes, merges...), on the shapes that reach those engines: random keys,
/// sorted runs, sawtooth, few distinct values.
#[test]
fn test_v2_cache_thresholds() {
    use super::super::generators::float_32::generators_f32;
    use super::super::generators::float_64::generators_f64;
    use super::super::generators::int128_extra::extra_u128;
    use super::super::generators::int_extra::{extra_i32, extra_i64, extra_u32, extra_u64};
    use super::super::generators::narrow_extra::extra_u16;
    use super::super::generators::signed_i32::generators_i32;
    use super::super::generators::signed_i64::generators_i64;
    use super::super::generators::unsigned_u128::generators_u128;
    use super::super::generators::unsigned_u16::generators_u16;
    use super::super::generators::unsigned_u64::generators_u64;
    use super::super::v2::sorts::devour::{devour_sort, DevourSort};
    use super::super::v2::sorts::ravenous::ravenous_sort;
    const SHAPES: [&str; 5] = ["Unif", "Runs 8", "Asc Saw", "Few 16", "Sqrt"];
    let sizes = super::super::v2::testing::cache_sizes(quick_max().min(16_000_000));
    let wanted = |name: &str| SHAPES.iter().any(|s| name.contains(s));
    fn ints<T: DevourSort + std::fmt::Debug + Send>(
        mut gens: Vec<(&dyn Fn(usize) -> Vec<T>, &str)>,
        extra: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>,
        sizes: &[usize],
        wanted: &dyn Fn(&str) -> bool,
        ty: &str,
    ) {
        gens.extend(extra);
        for &size in sizes {
            for (gen, name) in gens.iter().filter(|g| wanted(g.1)) {
                let mut arr = gen(size);
                let mut expected = arr.clone();
                expected.par_sort_unstable();
                devour_sort(&mut arr);
                assert!(arr == expected, "{} {} size {}", ty, name, size);
            }
        }
    }
    ints(generators_u32(), extra_u32::generators(), &sizes, &wanted, "u32");
    ints(generators_i32(), extra_i32::generators(), &sizes, &wanted, "i32");
    ints(generators_u64(), extra_u64::generators(), &sizes, &wanted, "u64");
    ints(generators_i64(), extra_i64::generators(), &sizes, &wanted, "i64");
    ints(generators_u16(), extra_u16::generators(), &sizes, &wanted, "u16");
    ints(generators_u128(), extra_u128::generators(), &sizes, &wanted, "u128");
    for &size in sizes.iter() {
        for (gen, name) in generators_f32().iter().filter(|g| wanted(g.1)) {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.par_sort_unstable_by(|a, b| a.total_cmp(b));
            ravenous_sort(&mut arr);
            assert!(
                arr.iter().map(|x| x.to_bits()).eq(expected.iter().map(|x| x.to_bits())),
                "f32 {} size {}",
                name,
                size
            );
        }
        for (gen, name) in generators_f64().iter().filter(|g| wanted(g.1)) {
            let mut arr = gen(size);
            let mut expected = arr.clone();
            expected.par_sort_unstable_by(|a, b| a.total_cmp(b));
            ravenous_sort(&mut arr);
            assert!(
                arr.iter().map(|x| x.to_bits()).eq(expected.iter().map(|x| x.to_bits())),
                "f64 {} size {}",
                name,
                size
            );
        }
    }
}
