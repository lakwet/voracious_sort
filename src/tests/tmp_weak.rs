//! Weak points of the short bench (2026-11-15): where the time goes.

use super::super::generators as g;
use super::super::v2::ravenous::ravenous_sort2;
use super::super::v2::devour::devour_sort2;
use super::super::v2::inspect::presorted::Unsorted;
use super::super::v2::decide::path::radix_path;
use std::cell::RefCell;

fn best(reps: usize, a: &[f32], f: &dyn Fn(&mut [f32])) -> f64 {
    let mut t = f64::MAX;
    for _ in 0..reps {
        let mut c = a.to_vec();
        let s = std::time::Instant::now();
        f(&mut c);
        t = t.min(s.elapsed().as_nanos() as f64 / a.len() as f64);
    }
    t
}

#[test]
#[ignore]
fn tmp_weak_saw() {
    for &n in &[10_000usize, 100_000, 1_000_000] {
        let a = g::float_32::helper_random_array_asc_sawtooth_f32(n);
        let chunks = RefCell::new(Vec::new());
        let mut c = a.clone();
        radix_path(&mut c, Unsorted::default(), false, &|_, _| false, &|ch: &mut [f32]| {
            chunks.borrow_mut().push(ch.len());
            ravenous_sort2(ch)
        });
        let ch = chunks.into_inner();
        let mut shuffled = a.clone();
        shuffled.reverse();
        shuffled.chunks_mut(7).for_each(|c| c.rotate_left(c.len() / 2));
        let v2 = best(5, &a, &|x| ravenous_sort2(x));
        let path = best(5, &a, &|x| radix_path(x, Unsorted::default(), false, &|_, _| false, &|ch: &mut [f32]| ravenous_sort2(ch)));
        let std = best(5, &a, &|x| x.sort_unstable_by(|p, q| p.total_cmp(q)));
        let u: Vec<u32> = a.iter().map(|x| x.to_bits()).collect();
        let mut tu = f64::MAX;
        for _ in 0..5 {
            let mut c = u.clone();
            let s = std::time::Instant::now();
            devour_sort2(&mut c);
            tu = tu.min(s.elapsed().as_nanos() as f64 / n as f64);
        }
        println!("n {:>8}: v2 {:.2} radix_path {:.2} std {:.2} u32 bits {:.2}; sort calls {} sizes {:?}", n, v2, path, std, tu, ch.len(), &ch[..ch.len().min(6)]);
        // the teeth
        let teeth = a.windows(2).filter(|w| w[0] > w[1]).count() + 1;
        println!("   teeth {} first values {:?}", teeth, &a[..4]);
    }
    let _ = devour_sort2::<u32>;
}

fn teeth_seps<T: PartialOrd>(a: &[T]) -> Vec<usize> {
    let mut s = vec![0];
    for i in 1..a.len() {
        if a[i - 1] > a[i] {
            s.push(i);
        }
    }
    s.push(a.len());
    s
}

/// Saw of every float / int type: scalar merges, SIMD merges, whole radix.
#[test]
#[ignore]
fn tmp_weak_saw_paths() {
    use super::super::v2::runs::merge::{merge_runs, merge_runs_simd};
    macro_rules! go {
        ($t:ty, $gen:expr, $sort:expr) => {
            for &n in &[10_000usize, 100_000, 1_000_000, 10_000_000] {
                let a: Vec<$t> = $gen(n);
                let mut shuf = a.clone();
                let mut x = 0x9E3779B97F4A7C15u64;
                for i in (1..n).rev() {
                    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                    shuf.swap(i, (x % (i as u64 + 1)) as usize);
                }
                let seps = teeth_seps(&a);
                let reps = (20_000_000 / n).clamp(3, 20);
                let mut t = [f64::MAX; 4];
                for _ in 0..reps {
                    for k in 0..4 {
                        let mut c = a.clone();
                        let mut sp = seps.clone();
                        let s = std::time::Instant::now();
                        match k {
                            0 => merge_runs(&mut c, &mut sp),
                            1 => merge_runs_simd(&mut c, &mut sp),
                            2 => $sort(&mut c),
                            _ => {
                                c.copy_from_slice(&shuf);
                                let s2 = std::time::Instant::now();
                                $sort(&mut c);
                                t[k] = t[k].min(s2.elapsed().as_nanos() as f64 / n as f64);
                                continue;
                            }
                        }
                        t[k] = t[k].min(s.elapsed().as_nanos() as f64 / n as f64);
                        if k < 2 {
                            assert!(c.windows(2).all(|w| w[0] <= w[1]));
                        }
                    }
                }
                println!("{:4} n {:>8} teeth {:3}: scalar merges {:.2} simd merges {:.2} v2 {:.2} v2 shuffled {:.2}", stringify!($t), n, seps.len() - 1, t[0], t[1], t[2], t[3]);
            }
        };
    }
    go!(f32, g::float_32::helper_random_array_asc_sawtooth_f32, |c: &mut [f32]| ravenous_sort2(c));
    go!(f64, g::float_64::helper_random_array_asc_sawtooth_f64, |c: &mut [f64]| ravenous_sort2(c));
    go!(u32, g::unsigned_u32::helper_random_array_asc_sawtooth_u32, |c: &mut [u32]| devour_sort2(c));
    go!(u64, g::unsigned_u64::helper_random_array_asc_sawtooth_u64, |c: &mut [u64]| devour_sort2(c));
}

/// Shuffled float sawtooth values: which step of the radix path costs.
#[test]
#[ignore]
fn tmp_weak_saw_engines() {
    use super::super::v2::engines::lsd::w32;
    use super::super::v2::engines::roller::r32;
    for &n in &[100_000usize, 1_000_000] {
        for kind in 0..2 {
            let mut a = if kind == 0 { g::float_32::helper_random_array_asc_sawtooth_f32(n) } else { g::float_32::helper_random_array_uniform_f32(n) };
            let mut x = 0x9E3779B97F4A7C15u64;
            for i in (1..n).rev() {
                x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                a.swap(i, (x % (i as u64 + 1)) as usize);
            }
            let bits = |c: &mut [f32]| unsafe { std::slice::from_raw_parts_mut(c.as_mut_ptr() as *mut u32, c.len()) }.to_vec();
            let t = |f: &dyn Fn(&mut Vec<u32>)| {
                let mut best = f64::MAX;
                for _ in 0..7 {
                    let mut c = bits(&mut a.clone());
                    let s = std::time::Instant::now();
                    f(&mut c);
                    best = best.min(s.elapsed().as_nanos() as f64 / n as f64);
                }
                best
            };
            let v2 = best(7, &a, &|x| ravenous_sort2(x));
            let lsd = t(&|c| w32::lsd(c));
            let roller = t(&|c| r32::roller(c));
            let det = best(7, &a, &|x| {
                std::hint::black_box(super::super::v2::special::floats::detect32(x));
            });
            println!("{} n {:>8}: v2 {:.2} lsd {:.2} roller {:.2} detect32 {:.2}", if kind == 0 { "saw " } else { "unif" }, n, v2, lsd, roller, det);
        }
    }
}

/// As the bench: a batch of 4M keys cloned, every array sorted in a row.
fn batch_ns<T: Clone>(a: &[T], f: &dyn Fn(&mut [T])) -> f64 {
    let n = a.len();
    let count = (4_000_000 / n).max(1);
    let batch: Vec<Vec<T>> = (0..count).map(|_| a.to_vec()).collect();
    let mut best = f64::MAX;
    for _ in 0..3 {
        let mut work = batch.clone();
        let s = std::time::Instant::now();
        for w in work.iter_mut() {
            f(w);
        }
        best = best.min(s.elapsed().as_nanos() as f64 / (count * n) as f64);
    }
    best
}

#[test]
#[ignore]
fn tmp_weak_saw_batch() {
    use super::super::v2::runs::merge::{merge_runs, merge_runs_simd};
    for &n in &[10_000usize, 100_000, 1_000_000, 10_000_000] {
        for kind in 0..3 {
            let mut a = match kind {
                0 => g::float_32::helper_random_array_asc_sawtooth_f32(n),
                1 => g::float_32::helper_random_array_asc_sawtooth_f32(n),
                _ => g::float_32::helper_random_array_uniform_f32(n),
            };
            if kind == 1 {
                let mut x = 0x9E3779B97F4A7C15u64;
                for i in (1..n).rev() {
                    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                    a.swap(i, (x % (i as u64 + 1)) as usize);
                }
            }
            let seps = teeth_seps(&a);
            let v2 = batch_ns(&a, &|x| ravenous_sort2(x));
            let bl = batch_ns(&a, &|x| blazing_radix::radix_sort::<f32>(x));
            let sc = if kind == 0 { batch_ns(&a, &|x| merge_runs(x, &mut seps.clone())) } else { 0.0 };
            let si = if kind == 0 { batch_ns(&a, &|x| merge_runs_simd(x, &mut seps.clone())) } else { 0.0 };
            println!("{} n {:>8}: v2 {:.2} blazing {:.2} scalar merges {:.2} simd merges {:.2}", ["saw     ", "saw shuf", "unif    "][kind], n, v2, bl, sc, si);
        }
    }
}

/// f64 sawtooth values shuffled: v2, DLSD, LSD, rollercoaster in batch.
#[test]
#[ignore]
fn tmp_weak_saw_f64_engines() {
    use super::super::v2::engines::dlsd::d64;
    use super::super::v2::engines::lsd::w64;
    use super::super::v2::engines::roller::r64;
    for &n in &[10_000usize, 100_000, 200_000, 600_000, 1_000_000] {
        for kind in 0..2 {
            let mut a = if kind == 0 { g::float_64::helper_random_array_asc_sawtooth_f64(n) } else { g::float_64::helper_random_array_uniform_f64(n) };
            let mut x = 0x9E3779B97F4A7C15u64;
            for i in (1..n).rev() {
                x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                a.swap(i, (x % (i as u64 + 1)) as usize);
            }
            let b: Vec<u64> = a.iter().map(|v| v.to_bits()).collect();
            let v2 = batch_ns(&a, &|x| ravenous_sort2(x));
            let dl = batch_ns(&b, &|x| d64::dlsd(x));
            let ls = batch_ns(&b, &|x| w64::lsd(x));
            let ro = batch_ns(&b, &|x| r64::roller(x));
            println!("{} n {:>8}: v2 {:.2} dlsd {:.2} lsd {:.2} roller {:.2}", if kind == 0 { "saw " } else { "unif" }, n, v2, dl, ls, ro);
        }
    }
}

/// Pipe organ u32 / u64 in batch: v2, blazing, glidesort, and v2's steps.
#[test]
#[ignore]
fn tmp_weak_organ() {
    use super::super::v2::runs::merge::merge_runs;
    for &n in &[10_000usize, 100_000, 1_000_000, 10_000_000] {
        let a = g::unsigned_u32::helper_random_array_pipe_organ_u32(n);
        let m = n / 2;
        let v2 = batch_ns(&a, &|x| devour_sort2(x));
        let bl = batch_ns(&a, &|x| blazing_radix::radix_sort::<u32>(x));
        let gl = batch_ns(&a, &|x| glidesort::sort(x));
        let rev = batch_ns(&a, &|x| x[m..].reverse());
        let mut b = a.clone();
        b[m..].reverse();
        let mg = batch_ns(&b, &|x| merge_runs(x, &mut vec![0, m, x.len()]));
        let path = batch_ns(&a, &|x| radix_path(x, Unsorted::default(), true, &|_, _| false, &|c: &mut [u32]| c.sort_unstable()));
        println!("u32 organ n {:>8}: v2 {:.2} blazing {:.2} glidesort {:.2} | reverse {:.2} merge {:.2} radix_path(no detect) {:.2}", n, v2, bl, gl, rev, mg, path);
    }
}

/// Pipe organ u32 at 1M: each integer detector alone (batch timing).
#[test]
#[ignore]
fn tmp_weak_organ_detectors() {
    use super::super::v2::special::{plateaus::plateaus, spread, DENSE_MIN};
    use super::super::v2::special::heavy::heavy_sorted_by;
    use super::super::v2::devour::detect_u32 as d;
    for &n in &[100_000usize, 1_000_000] {
        let a = g::unsigned_u32::helper_random_array_pipe_organ_u32(n);
        let r = |f: &dyn Fn(&mut [u32]) -> bool| {
            let mut c = a.clone();
            let hit = f(&mut c);
            (batch_ns(&a, &|x| { std::hint::black_box(f(x)); }), hit)
        };
        let sp = r(&|x| { std::hint::black_box(spread(x)); false });
        let range = r(&|x| { let s = spread(x); d::range(x, &s) });
        let pl = r(&|x| plateaus(x, 0));
        let few = r(&|x| { let s = spread(x); d::few(x, &s) });
        let heavy = r(&|x| { let mut s = spread(x); s.sort_unstable(); heavy_sorted_by(x, &s, DENSE_MIN, &|q: &mut [u32]| devour_sort2(q), |p, q| p == q, |p, q| p < q) });
        let dense = r(&|x| { let mut s = spread(x); s.sort_unstable(); d::dense(x, &s) });
        println!("n {:>8}: spread {:?} range {:?} plateaus {:?} few {:?} heavy {:?} dense {:?}", n, sp, range, pl, few, heavy, dense);
    }
}

/// Pipe organ u32: presorted, then radix_path given what it learned.
#[test]
#[ignore]
fn tmp_weak_organ_presorted() {
    use super::super::v2::inspect::presorted::presorted;
    for &n in &[100_000usize, 1_000_000, 10_000_000] {
        let a = g::unsigned_u32::helper_random_array_pipe_organ_u32(n);
        let pre = batch_ns(&a, &|x| { std::hint::black_box(presorted(x).is_ok()); });
        let with_known = batch_ns(&a, &|x| {
            if let Err(k) = presorted(x) {
                radix_path(x, k, true, &|_, _| false, &|c: &mut [u32]| c.sort_unstable());
            }
        });
        let without = batch_ns(&a, &|x| radix_path(x, Unsorted::default(), true, &|_, _| false, &|c: &mut [u32]| c.sort_unstable()));
        let v2 = batch_ns(&a, &|x| devour_sort2(x));
        println!("n {:>8}: presorted {:.3} presorted+path {:.2} path alone {:.2} v2 {:.2}", n, pre, with_known, without, v2);
    }
}

/// f32 timestamps: distinct values, breaks, and the cost of each step.
#[test]
#[ignore]
fn tmp_weak_ts() {
    use super::super::v2::special::floats::{detect32, few32};
    use super::super::v2::inspect::presorted::presorted;
    for &n in &[10_000usize, 100_000, 1_000_000, 10_000_000] {
        let a: Vec<f32> = g::float_extra::extra_f32::timestamps(n);
        let mut d = a.clone();
        d.sort_by(|x, y| x.total_cmp(y));
        d.dedup();
        let breaks = a.windows(2).filter(|w| w[0] > w[1]).count();
        let mut c = a.clone();
        let det = detect32(&mut c);
        let mut c = a.clone();
        let few = few32(&mut c, None);
        let v2 = batch_ns(&a, &|x| ravenous_sort2(x));
        let bl = batch_ns(&a, &|x| blazing_radix::radix_sort::<f32>(x));
        let t_det = batch_ns(&a, &|x| { std::hint::black_box(detect32(x)); });
        let t_few = batch_ns(&a, &|x| { std::hint::black_box(few32(x, None)); });
        let t_pre = batch_ns(&a, &|x| { std::hint::black_box(presorted(x).is_ok()); });
        println!("n {:>8}: distinct {} breaks {} detect32 {} few32 {} | v2 {:.2} blazing {:.2} detect32 {:.2} few32 {:.2} presorted {:.2}", n, d.len(), breaks, det, few, v2, bl, t_det, t_few, t_pre);
    }
}

/// Regressions against v1, in batch (as the bench).
#[test]
#[ignore]
fn tmp_weak_v1_f64() {
    use super::super::RadixSort;
    for &n in &[10_000usize, 100_000] {
        for (name, a) in [
            ("asc saw", g::float_64::helper_random_array_asc_sawtooth_f64(n)),
            ("pipe organ", g::float_64::helper_random_array_pipe_organ_f64(n)),
        ] {
            let v2 = batch_ns(&a, &|x| ravenous_sort2(x));
            let v1 = batch_ns(&a, &|x| x.voracious_sort());
            let path = batch_ns(&a, &|x| radix_path(x, Unsorted::default(), false, &|_, _| false, &|c: &mut [f64]| c.sort_unstable_by(|p, q| p.total_cmp(q))));
            println!("f64 {:10} n {:>7}: v2 {:.2} v1 {:.2} radix_path(merge only) {:.2}", name, n, v2, v1, path);
        }
    }
}

/// Two-ended merge of two f64 teeth: f64 comparisons against the same keys
/// as u64 (float order encoded), to find the kernel `target-cpu=native` slows.
#[test]
#[ignore]
fn tmp_weak_bidir_native() {
    use super::super::v2::runs::merge::bidir_pub;
    let n = 100_000;
    let a = g::float_64::helper_random_array_asc_sawtooth_f64(n);
    let s = teeth_seps(&a);
    let (x, y) = (&a[s[0]..s[1]], &a[s[1]..s[2]]);
    let mut d = vec![0f64; x.len() + y.len()];
    let enc = |v: &[f64]| v.iter().map(|f| { let b = f.to_bits(); b ^ (((b as i64) >> 63) as u64 | 1 << 63) }).collect::<Vec<u64>>();
    let (xu, yu) = (enc(x), enc(y));
    let mut du = vec![0u64; d.len()];
    let mut t = [f64::MAX; 2];
    for _ in 0..200 {
        let st = std::time::Instant::now();
        bidir_pub(x, y, &mut d);
        t[0] = t[0].min(st.elapsed().as_nanos() as f64 / d.len() as f64);
        let st = std::time::Instant::now();
        bidir_pub(&xu, &yu, &mut du);
        t[1] = t[1].min(st.elapsed().as_nanos() as f64 / d.len() as f64);
    }
    println!("bidir teeth {}+{}: f64 {:.2} u64 {:.2} ns per key", x.len(), y.len(), t[0], t[1]);
}

/// Same cases in both builds (plain / target-cpu=native): v2 against v1.
#[test]
#[ignore]
fn tmp_weak_native_scan() {
    use super::super::RadixSort;
    let n = 100_000;
    macro_rules! go {
        ($t:ty, $name:expr, $a:expr, $v2:expr) => {{
            let a: Vec<$t> = $a;
            println!("{:5} {:12} v2 {:.2} v1 {:.2}", stringify!($t), $name, batch_ns(&a, &$v2), batch_ns(&a, &|x: &mut [$t]| x.voracious_sort()));
        }};
    }
    go!(u32, "unif", g::unsigned_u32::helper_random_array_uniform_u32(n), |x: &mut [u32]| devour_sort2(x));
    go!(u32, "pipe organ", g::unsigned_u32::helper_random_array_pipe_organ_u32(n), |x: &mut [u32]| devour_sort2(x));
    go!(u64, "unif", g::unsigned_u64::helper_random_array_uniform_u64(n), |x: &mut [u64]| devour_sort2(x));
    go!(u64, "pipe organ", g::unsigned_u64::helper_random_array_pipe_organ_u64(n), |x: &mut [u64]| devour_sort2(x));
    go!(f32, "unif", g::float_32::helper_random_array_uniform_f32(n), |x: &mut [f32]| ravenous_sort2(x));
    go!(f32, "asc saw", g::float_32::helper_random_array_asc_sawtooth_f32(n), |x: &mut [f32]| ravenous_sort2(x));
    go!(f64, "unif", g::float_64::helper_random_array_uniform_f64(n), |x: &mut [f64]| ravenous_sort2(x));
    go!(f64, "asc saw", g::float_64::helper_random_array_asc_sawtooth_f64(n), |x: &mut [f64]| ravenous_sort2(x));
    go!(u128, "asc rnd tail", g::int128_extra::extra_u128::generators().into_iter().find(|(_, s)| s.contains("rnd tail")).unwrap().0(n), |x: &mut [u128]| devour_sort2(x));
}

/// Float sawtooth with the L2 the thresholds see forced (x86: 512 KiB L2).
#[test]
#[ignore]
fn tmp_weak_saw_l2() {
    use super::super::v2::cache::L2_OVERRIDE;
    use super::super::RadixSort;
    for &n in &[10_000usize, 100_000, 1_000_000] {
        let a = g::float_32::helper_random_array_asc_sawtooth_f32(n);
        let b = g::float_64::helper_random_array_asc_sawtooth_f64(n);
        let mut line = format!("n {:>8}:", n);
        for l2 in [0usize, 4 << 20, 16 << 20] {
            L2_OVERRIDE.store(l2, std::sync::atomic::Ordering::Relaxed);
            line += &format!(" L2 {:>2} MiB f32 {:.2} f64 {:.2} |", l2 >> 20, batch_ns(&a, &|x| ravenous_sort2(x)), batch_ns(&b, &|x| ravenous_sort2(x)));
        }
        L2_OVERRIDE.store(0, std::sync::atomic::Ordering::Relaxed);
        line += &format!(" v1 f32 {:.2} f64 {:.2}", batch_ns(&a, &|x| x.voracious_sort()), batch_ns(&b, &|x| x.voracious_sort()));
        println!("{}", line);
    }
}

/// Float sawtooth: one merge budget at a time forced to 16 MiB.
#[test]
#[ignore]
fn tmp_weak_saw_budget() {
    use super::super::v2::cache::{Budget, BUDGET_OVERRIDE};
    for &n in &[100_000usize, 1_000_000] {
        let a = g::float_32::helper_random_array_asc_sawtooth_f32(n);
        let b = g::float_64::helper_random_array_asc_sawtooth_f64(n);
        let mut line = format!("n {:>8}:", n);
        for (name, which) in [("none", None), ("twoway", Some(Budget::MergeTwoWay)), ("pingpong", Some(Budget::PingPong)), ("both", Some(Budget::Records))] {
            for i in 0..13 {
                BUDGET_OVERRIDE[i].store(0, std::sync::atomic::Ordering::Relaxed);
            }
            match (name, which) {
                ("both", _) => {
                    BUDGET_OVERRIDE[Budget::MergeTwoWay as usize].store(16 << 20, std::sync::atomic::Ordering::Relaxed);
                    BUDGET_OVERRIDE[Budget::PingPong as usize].store(16 << 20, std::sync::atomic::Ordering::Relaxed);
                }
                (_, Some(w)) => BUDGET_OVERRIDE[w as usize].store(16 << 20, std::sync::atomic::Ordering::Relaxed),
                _ => {}
            }
            line += &format!(" {} f32 {:.2} f64 {:.2} |", name, batch_ns(&a, &|x| ravenous_sort2(x)), batch_ns(&b, &|x| ravenous_sort2(x)));
        }
        for i in 0..13 {
            BUDGET_OVERRIDE[i].store(0, std::sync::atomic::Ordering::Relaxed);
        }
        println!("{}", line);
    }
}

/// u128 Asc rnd tail (90 % sorted, random tail) and Equal: v2's steps.
#[test]
#[ignore]
fn tmp_weak_u128_tail() {
    use super::super::v2::inspect::presorted::presorted;
    use super::super::RadixSort;
    let gens = g::int128_extra::extra_u128::generators();
    let tail = gens.iter().find(|(_, s)| s.contains("rnd tail")).unwrap().0;
    let equal = gens.iter().find(|(_, s)| s.contains("Equal")).unwrap().0;
    for &n in &[10_000usize, 100_000, 1_000_000] {
        let a: Vec<u128> = tail(n);
        let cut = n * 9 / 10;
        let v2 = batch_ns(&a, &|x| devour_sort2(x));
        let v1 = batch_ns(&a, &|x| x.voracious_sort());
        let pre = batch_ns(&a, &|x| { std::hint::black_box(presorted(x).is_ok()); });
        let tail_sort = batch_ns(&a, &|x| devour_sort2(&mut x[cut..]));
        let mut b = a.clone();
        b[cut..].sort_unstable();
        let merge = batch_ns(&b, &|x| super::super::v2::runs::merge::merge_runs(x, &mut vec![0, cut, x.len()]));
        let path = batch_ns(&a, &|x| radix_path(x, Unsorted::default(), true, &|_, _| false, &|c: &mut [u128]| devour_sort2(c)));
        let e: Vec<u128> = equal(n);
        println!("n {:>8}: tail v2 {:.2} v1 {:.2} | presorted {:.2} tail sort {:.2} merge {:.2} radix_path {:.2} || equal v2 {:.2} v1 {:.2} presorted {:.2}", n, v2, v1, pre, tail_sort, merge, path,
            batch_ns(&e, &|x| devour_sort2(x)), batch_ns(&e, &|x| x.voracious_sort()), batch_ns(&e, &|x| { std::hint::black_box(presorted(x).is_ok()); }));
    }
}


/// Which f32 / f64 input overflows the stack with the tail-first path.
#[test]
#[ignore]
fn tmp_weak_overflow() {
    use std::io::Write;
    for size in [5_000usize, 8_191, 8_192, 10_000, 100_000] {
        for (gen, name) in g::float_32::generators_f32().iter() {
            print!("f32 {} {} | ", name, size);
            std::io::stdout().flush().unwrap();
            let mut a = gen(size);
            ravenous_sort2(&mut a);
        }
        for (gen, name) in g::float_64::generators_f64().iter() {
            print!("f64 {} {} | ", name, size);
            std::io::stdout().flush().unwrap();
            let mut a = gen(size);
            ravenous_sort2(&mut a);
        }
    }
}

/// 1k regressions against v1: zeros +- and sparse 90 % (floats, i16).
#[test]
#[ignore]
fn tmp_weak_1k() {
    use super::super::RadixSort;
    for &n in &[1_000usize, 3_000, 5_000] {
        for (gen, name) in g::float_32::generators_f32().iter().chain(g::float_extra::extra_f32::generators().iter()) {
            if !(name.contains("Zeros") || name.contains("Sparse")) { continue; }
            let a = gen(n);
            println!("f32 {:14} {:>5}: v2 {:.2} v1 {:.2}", name, n, batch_ns(&a, &|x| ravenous_sort2(x)), batch_ns(&a, &|x| x.voracious_sort()));
        }
        for (gen, name) in g::float_64::generators_f64().iter().chain(g::float_extra::extra_f64::generators().iter()) {
            if !(name.contains("Zeros") || name.contains("Sparse")) { continue; }
            let a = gen(n);
            println!("f64 {:14} {:>5}: v2 {:.2} v1 {:.2}", name, n, batch_ns(&a, &|x| ravenous_sort2(x)), batch_ns(&a, &|x| x.voracious_sort()));
        }
        for (gen, name) in g::signed_i16::generators_i16().iter().chain(g::narrow_extra::extra_i16::generators().iter()) {
            if !name.contains("Sparse") { continue; }
            let a = gen(n);
            println!("i16 {:14} {:>5}: v2 {:.2} v1 {:.2}", name, n, batch_ns(&a, &|x| devour_sort2(x)), batch_ns(&a, &|x| x.voracious_sort()));
        }
    }
}

/// Regressions to recheck without native: u16 Clusters, u32 Normale 13,
/// i64 Unif / Aligned 2^16 at large sizes.
#[test]
#[ignore]
fn tmp_weak_recheck() {
    use super::super::RadixSort;
    macro_rules! go {
        ($t:ty, $gens:expr, $want:expr, $sizes:expr) => {
            for &n in $sizes {
                for (gen, name) in $gens.iter() {
                    if !$want.iter().any(|w: &&str| name.contains(w)) { continue; }
                    let a: Vec<$t> = gen(n);
                    println!("{:4} {:16} {:>9}: v2 {:.2} v1 {:.2}", stringify!($t), name, n, batch_ns(&a, &|x| devour_sort2(x)), batch_ns(&a, &|x| x.voracious_sort()));
                }
            }
        };
    }
    go!(u16, g::unsigned_u16::generators_u16().into_iter().chain(g::narrow_extra::extra_u16::generators()).collect::<Vec<_>>(), ["Clusters"], &[10_000usize, 100_000, 1_000_000]);
    go!(u32, g::unsigned_u32::generators_u32(), ["Normale 13", "Normale  8"], &[1_000_000usize, 5_000_000, 10_000_000]);
    go!(i64, g::signed_i64::generators_i64().into_iter().chain(g::int_extra::extra_i64::generators()).collect::<Vec<_>>(), ["Unif", "Aligned"], &[1_000_000usize, 5_000_000, 10_000_000]);
}

/// u32 Normale 13 at 1M-10M: which step costs (v2 0.75 vs v1 0.59).
#[test]
#[ignore]
fn tmp_weak_normale13() {
    use super::super::v2::special::{spread, RANGE_MAX};
    use super::super::v2::engines::counting::window_count;
    use super::super::RadixSort;
    for &n in &[1_000_000usize, 10_000_000] {
        let a = g::unsigned_u32::helper_random_array_normale_13_u32(n);
        let s = spread(&a);
        let (lo, hi) = (*s.iter().min().unwrap(), *s.iter().max().unwrap());
        let (mn, mx) = (*a.iter().min().unwrap(), *a.iter().max().unwrap());
        let limit = RANGE_MAX.min(n / 2);
        let v2 = batch_ns(&a, &|x| devour_sort2(x));
        let v1 = batch_ns(&a, &|x| x.voracious_sort());
        let win = batch_ns(&a, &|x| { std::hint::black_box(window_count(x, lo, hi, limit, |k| k, |k| k)); });
        let minmax = batch_ns(&a, &|x| { std::hint::black_box(x.iter().fold((u32::MAX, 0u32), |(l, h), &v| (l.min(v), h.max(v)))); });
        println!("n {:>8}: sample {}..{} true {}..{} limit {} | v2 {:.2} v1 {:.2} window_count {:.2} min/max pass {:.2}", n, lo, hi, mn, mx, limit, v2, v1, win, minmax);
    }
}

#[test]
#[ignore]
fn tmp_weak_span_ab() {
    use super::super::RadixSort;
    use std::sync::atomic::Ordering::Relaxed;
    type G = (&'static str, fn(usize) -> Vec<u32>);
    fn modn(n: usize, m: u32) -> Vec<u32> { g::unsigned_u32::helper_random_array_uniform_u32(n).into_iter().map(|x| x % m).collect() }
    let gens: Vec<G> = vec![
        ("Normale8", g::unsigned_u32::helper_random_array_normale_8_u32),
        ("Normale10", g::unsigned_u32::helper_random_array_normale_10_u32),
        ("Normale13", g::unsigned_u32::helper_random_array_normale_13_u32),
        ("Normale16", g::unsigned_u32::helper_random_array_normale_16_u32),
        ("Alt16", g::unsigned_u32::helper_random_array_alternating16_u32),
        ("Sqrt", g::unsigned_u32::helper_random_array_sqrt_u32),
        ("Zipf", g::unsigned_u32::helper_random_array_zipf_u32),
        ("mod10k", |n| modn(n, 10_000)),
        ("mod20k", |n| modn(n, 20_000)),
        ("mod40k", |n| modn(n, 40_000)),
        ("mod60k", |n| modn(n, 60_000)),
        ("Middle16", g::unsigned_u32::helper_random_array_middle16_u32),
    ];
    let caps = [0usize];
    let sizes: Vec<usize> = std::env::var("GP_SIZES").ok().map(|s| s.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![30_000, 100_000, 300_000, 1_000_000, 3_000_000, 10_000_000]);
    for (name, gen) in &gens {
        for &n in &sizes {
            let a = gen(n);
            let mut line = format!("{:>10} {:>9}:", name, n);
            let v1 = batch_ns(&a, &|x| x.voracious_sort());
            for &c in &caps {
                let t = (0..3).map(|_| batch_ns(&a, &|x| devour_sort2(x))).fold(f64::MAX, f64::min);
                line += &format!(" {} {:.2}", ["old", "lanes", "win4", "both"][c], t);
            }
            line += &format!(" | v1 {:.2}", v1);
            println!("{}", line);
        }
    }
}

#[test]
#[ignore]
fn tmp_weak_count_parts() {
    // Normale 13: the count loop alone (1 lane / 4 lanes, exact range) and the fill alone.
    for &n in &[1_000_000usize, 10_000_000] {
        let a = g::unsigned_u32::helper_random_array_normale_13_u32(n);
        let (mn, mx) = (*a.iter().min().unwrap(), *a.iter().max().unwrap());
        let w = (mx - mn + 1) as usize;
        let c1 = batch_ns(&a, &|x| { let mut c = vec![0u32; w]; for &k in x.iter() { unsafe { *c.get_unchecked_mut((k - mn) as usize) += 1 } } std::hint::black_box(&c); });
        let c4 = batch_ns(&a, &|x| { let mut c = vec![0u32; 4 * w]; for q in x.chunks_exact(4) { unsafe { *c.get_unchecked_mut((q[0] - mn) as usize) += 1; *c.get_unchecked_mut(w + (q[1] - mn) as usize) += 1; *c.get_unchecked_mut(2 * w + (q[2] - mn) as usize) += 1; *c.get_unchecked_mut(3 * w + (q[3] - mn) as usize) += 1; } } std::hint::black_box(&c); });
        let mut cnt = vec![0u32; w];
        for &k in a.iter() { cnt[(k - mn) as usize] += 1; }
        let fill = batch_ns(&a, &|x| { let mut s = 0; for (d, &c) in cnt.iter().enumerate() { let c = c as usize; x[s..s + c].fill(mn + d as u32); s += c; } });
        let copy = batch_ns(&a, &|x| { let y = x.to_vec(); std::hint::black_box(&y); });
        println!("n {:>8} w {}: count 1 lane {:.2}, 4 lanes {:.2}, fill {:.2}, plain copy {:.2}", n, w, c1, c4, fill, copy);
    }
}

#[test]
#[ignore]
fn tmp_weak_v1_count() {
    use super::super::sorts::counting_sort::counting_sort;
    use super::super::v2::engines::counting::{exact_count, window_count};
    use super::super::RadixSort;
    for &n in &[100_000usize, 1_000_000, 3_000_000, 10_000_000] {
        let a = g::unsigned_u32::helper_random_array_normale_13_u32(n);
        let mx = *a.iter().max().unwrap();
        let rep = |f: &dyn Fn(&mut [u32])| (0..3).map(|_| batch_ns(&a, f)).fold(f64::MAX, f64::min);
        let v1c = rep(&|x| counting_sort(x, 16));
        let v1 = rep(&|x| x.voracious_sort());
        let ex = rep(&|x| exact_count(x, 0, mx as usize + 1, |k| k, |k| k));
        let ex64 = rep(&|x| exact_count(x, 0, 65536, |k| k, |k| k));
        let win = rep(&|x| { window_count(x, 383, 22643, 65536, |k| k, |k| k); });
        let v2 = rep(&|x| devour_sort2(x));
        println!("n {:>8}: v1 counting_sort(16) {:.2} v1 {:.2} | exact_count {:.2} exact 64k {:.2} window_count {:.2} v2 {:.2}", n, v1c, v1, ex, ex64, win, v2);
    }
}

#[test]
#[ignore]
fn tmp_weak_middle16() {
    
    use super::super::RadixSort;
    for (name, gen) in [("Middle16", g::unsigned_u32::helper_random_array_middle16_u32 as fn(usize) -> Vec<u32>), ("Normale13", g::unsigned_u32::helper_random_array_normale_13_u32)] {
        for &n in &[10_000usize, 100_000, 1_000_000, 3_000_000, 10_000_000] {
            let a = gen(n);
            let rep = |f: &dyn Fn(&mut [u32])| (0..3).map(|_| batch_ns(&a, f)).fold(f64::MAX, f64::min);
            let lsd = rep(&|x| { if let Some(s) = two_varying_bytes(x, |k| k) { bytes_lsd2(x, s, |k| k) } else { x.sort_unstable() } });
            let v2 = rep(&|x| devour_sort2(x));
            let v1 = rep(&|x| x.voracious_sort());
            let mut c = a.clone(); let sh = two_varying_bytes(&c, |k| k).unwrap(); bytes_lsd2(&mut c, sh, |k| k);
            assert!(c.windows(2).all(|p| p[0] <= p[1]));
            println!("{:>9} n {:>8}: bytes_lsd2 {:.2} | v2 {:.2} v1 {:.2}", name, n, lsd, v2, v1);
        }
    }
}

// Kept for later (user, 16 Nov.): LSD on the two varying bytes, wherever they are.
/// LSD radix sort on two bytes of the encoded keys (at bit offsets
/// `shifts`, low one first), the other bytes being equal in all keys: one
/// read for both histograms, then keys to a buffer by the first byte and
/// back by the second.
pub(crate) fn bytes_lsd2<U: super::super::v2::devour::keys::Key, E: Fn(U) -> U + Copy>(keys: &mut [U], shifts: [u32; 2], encode: E) {
    let n = keys.len();
    let [s0, s1] = shifts;
    let d0 = move |k: U| (encode(k).to_u64() >> s0) as usize & 0xFF;
    let d1 = move |k: U| (encode(k).to_u64() >> s1) as usize & 0xFF;
    let mut lo = [0usize; 256];
    let mut hi = [0usize; 256];
    for &k in keys.iter() {
        lo[d0(k)] += 1;
        hi[d1(k)] += 1;
    }
    for h in [&mut lo, &mut hi] {
        let mut sum = 0;
        for c in h.iter_mut() {
            let x = *c;
            *c = sum;
            sum += x;
        }
    }
    let mut buf: Vec<U> = vec![keys[0]; n];
    scatter(keys, &mut buf, &mut lo, d0);
    scatter(&buf, keys, &mut hi, d1);
}

/// Bit offsets of the bytes that differ among the encoded keys (OR of
/// `key ^ first key`, one read), when there are exactly two.
pub(crate) fn two_varying_bytes<U: super::super::v2::devour::keys::Key, E: Fn(U) -> U + Copy>(keys: &[U], encode: E) -> Option<[u32; 2]> {
    let e0 = encode(keys[0]).to_u64();
    let diff = keys.iter().fold(0u64, |d, &k| d | (encode(k).to_u64() ^ e0));
    let mut shifts = (0..64).step_by(8).filter(|&s| (diff >> s) & 0xFF != 0);
    match (shifts.next(), shifts.next(), shifts.next()) {
        (Some(a), Some(b), None) => Some([a, b]),
        _ => None,
    }
}

/// Keys of `src` to `dst` at the heads of their digit (prefix sums).
#[inline(always)]
fn scatter<U: Copy, F: Fn(U) -> usize>(src: &[U], dst: &mut [U], heads: &mut [usize; 256], digit: F) {
    for &k in src.iter() {
        let b = digit(k) & 0xFF;
        let h = heads[b];
        dst[h] = k;
        heads[b] = h + 1;
    }
}

#[test]
#[ignore]
fn tmp_weak_shifted() {
    use super::super::RadixSort;
    let a16: fn(usize) -> Vec<u32> = |n| g::unsigned_u32::helper_random_array_uniform_u32(n).into_iter().map(|x| x & !0xFFFF).collect();
    let a8: fn(usize) -> Vec<u32> = |n| g::unsigned_u32::helper_random_array_uniform_u32(n).into_iter().map(|x| (x % 30_000) << 8 | 7).collect();
    let i: fn(usize) -> Vec<i32> = |n| g::unsigned_u32::helper_random_array_middle16_u32(n).into_iter().map(|x| (x as i32).wrapping_sub(0x6000_0000)).collect();
    for (name, gen) in [("Middle16", g::unsigned_u32::helper_random_array_middle16_u32 as fn(usize) -> Vec<u32>), ("Aligned16", a16), ("Shift8x30k", a8)] {
        for &n in &[1_000usize, 10_000, 100_000, 1_000_000, 10_000_000] {
            let a = gen(n);
            let v2 = batch_ns(&a, &|x| devour_sort2(x));
            let v1 = batch_ns(&a, &|x| x.voracious_sort());
            let mut c = a.clone(); devour_sort2(&mut c); let mut d = a.clone(); d.sort_unstable(); assert_eq!(c, d);
            println!("{:>10} n {:>8}: v2 {:.2} v1 {:.2}", name, n, v2, v1);
        }
    }
    for &n in &[1_000usize, 100_000, 10_000_000] {
        let a = i(n);
        let mut c = a.clone(); devour_sort2(&mut c); let mut d = a.clone(); d.sort_unstable(); assert_eq!(c, d);
        println!("i32 middle16 n {:>8}: v2 {:.2} v1 {:.2}", n, batch_ns(&a, &|x| devour_sort2(x)), batch_ns(&a, &|x| x.voracious_sort()));
    }
}

#[test]
#[ignore]
fn tmp_weak_u16_clusters() {
    use super::super::RadixSort;
    use super::super::v2::engines::counting::domain_count;
    for &n in &[10_000usize, 30_000, 50_000, 100_000, 200_000, 300_000, 600_000, 1_000_000] {
        let a = g::narrow_extra::extra_u16::clusters(n);
        let rep = |f: &dyn Fn(&mut [u16])| (0..5).map(|_| batch_ns(&a, f)).fold(f64::MAX, f64::min);
        let v2 = rep(&|x| devour_sort2(x));
        let v1 = rep(&|x| x.voracious_sort());
        let dc = rep(&|x| domain_count(x, |k| k, |k| k));
        let mut d: Vec<u16> = a.clone(); d.sort_unstable(); d.dedup();
        println!("u16 clusters n {:>8} ({} values): v2 {:.2} v1 {:.2} domain_count {:.2}", n, d.len(), v2, v1, dc);
    }
}

#[test]
#[ignore]
fn tmp_weak_u16_few() {
    use super::super::v2::special::spread; use super::super::v2::devour::detect_u16;
    for &n in &[30_000usize, 50_000, 100_000] {
        let a = g::narrow_extra::extra_u16::clusters(n);
        let mut b = a.clone();
        let s = spread(&b);
        let r = detect_u16::range(&mut b, &s);
        let mut c = a.clone();
        let f = detect_u16::few(&mut c, &s);
        let few_t = batch_ns(&a, &|x| { let s = spread(x); detect_u16::few(x, &s); });
        println!("n {}: range {} few {} few time {:.2}", n, r, f, few_t);
    }
}

#[test]
#[ignore]
fn tmp_weak_u16_parts() {
    use super::super::v2::special::spread; use super::super::v2::devour::detect_u16;
    use super::super::v2::inspect::presorted::presorted;
    for &n in &[30_000usize, 50_000, 100_000, 200_000] {
        let a = g::narrow_extra::extra_u16::clusters(n);
        let rep = |f: &dyn Fn(&mut [u16])| (0..5).map(|_| batch_ns(&a, f)).fold(f64::MAX, f64::min);
        let pre = rep(&|x| { let _ = std::hint::black_box(presorted(x)); });
        let few = rep(&|x| { let s = spread(x); detect_u16::few(x, &s); });
        let rng = rep(&|x| { let s = spread(x); std::hint::black_box(detect_u16::range(x, &s)); });
        let v2 = rep(&|x| devour_sort2(x));
        println!("n {:>7}: presorted {:.2} range {:.2} few {:.2} | v2 {:.2}", n, pre, rng, few, v2);
    }
}

#[test]
#[ignore]
fn tmp_weak_u16_few_var() {
    use super::super::v2::special::spread; use super::super::v2::devour::detect_u16;
    for _ in 0..8 {
        let a = g::narrow_extra::extra_u16::clusters(100_000);
        let few = (0..5).map(|_| batch_ns(&a, &|x| { let s = spread(x); detect_u16::few(x, &s); })).fold(f64::MAX, f64::min);
        let few2 = (0..5).map(|_| batch_ns(&a, &|x| { let s = spread(x); detect_u16::few(x, &s); })).fold(f64::MAX, f64::min);
        print!("fmix {:.2} | ", few2);
        let mut d = a.clone(); d.sort_unstable(); d.dedup();
        // probe lengths with the same hash
        let hash = |v: u16| (((v as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) >> (64 - 9)) as usize;
        let mut used = [false; 512]; let mut keys = [0u16; 512]; let mut total = 0usize;
        for &v in d.iter() { let mut h = hash(v); while used[h] { h = (h + 1) & 511; total += 1; } used[h] = true; keys[h] = v; }
        // distribution of adjacent equal keys (store-forward chains)
        let eq = a.windows(2).filter(|w| w[0] == w[1]).count();
        println!("few {:.2} distinct {} extra probes at insert {} adjacent equal {}", few, d.len(), total, eq);
    }
}

#[test]
#[ignore]
fn tmp_weak_1k_sparse() {
    use super::super::RadixSort;
    for &n in &[500usize, 1000, 1500, 2000, 3000] {
        let rep32 = |a: &Vec<u32>, f: &dyn Fn(&mut [u32])| (0..5).map(|_| batch_ns(a, f)).fold(f64::MAX, f64::min);
        let rep64 = |a: &Vec<f64>, f: &dyn Fn(&mut [f64])| (0..5).map(|_| batch_ns(a, f)).fold(f64::MAX, f64::min);
        let a = g::int_extra::extra_u32::sparse90(n);
        println!("u32 sparse n {:>5}: v2 {:.2} v1 {:.2} std {:.2}", n, rep32(&a, &|x| devour_sort2(x)), rep32(&a, &|x| x.voracious_sort()), rep32(&a, &|x| x.sort_unstable()));
        let a = g::int_extra::extra_u64::sparse90(n);
        let rep = |f: &dyn Fn(&mut [u64])| (0..5).map(|_| batch_ns(&a, f)).fold(f64::MAX, f64::min);
        println!("u64 sparse n {:>5}: v2 {:.2} v1 {:.2} std {:.2}", n, rep(&|x| devour_sort2(x)), rep(&|x| x.voracious_sort()), rep(&|x| x.sort_unstable()));
        for (name, gen) in [("f64 sparse", g::float_64::helper_random_array_sparse90_f64 as fn(usize) -> Vec<f64>), ("f64 zeros", g::float_extra::extra_f64::zeros)] {
            let a = gen(n);
            println!("{} n {:>5}: v2 {:.2} v1 {:.2} std {:.2}", name, n, rep64(&a, &|x| ravenous_sort2(x)), rep64(&a, &|x| x.voracious_sort()), rep64(&a, &|x| x.sort_unstable_by(|p, q| p.total_cmp(q))));
        }
    }
}

/// Detectors' cost: v2 against the same path without detectors (engine
/// alone after the presorted check), every generator. PO_SIZES, PO_TYPES.
#[test]
#[ignore]
fn tmp_weak_detect_cost() {
    use super::super::v2::engines::dlsd::du64;
    use super::super::v2::engines::lsd::wu32;
    use super::super::v2::inspect::presorted::presorted;
    let sizes: Vec<usize> = std::env::var("PO_SIZES").ok().map(|s| s.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![1_000, 10_000, 100_000, 1_000_000, 10_000_000]);
    let types = std::env::var("PO_TYPES").unwrap_or("u32,u64".into());
    fn go<T: Copy + PartialOrd + Ord>(label: &str, gens: Vec<(&'static dyn Fn(usize) -> Vec<T>, &'static str)>, sizes: &[usize], v2: &dyn Fn(&mut [T]), eng: &dyn Fn(&mut [T])) {
        for &n in sizes {
            for (gen, name) in gens.iter() {
                let a = gen(n);
                let nod = |x: &mut [T]| {
                    if let Err(k) = presorted(x) {
                        radix_path(x, k, true, &|_, _| false, eng);
                    }
                };
                let (mut t2, mut tn) = (f64::MAX, f64::MAX);
                for _ in 0..2 {
                    t2 = t2.min(batch_ns(&a, v2));
                    tn = tn.min(batch_ns(&a, &nod));
                }
                let r = t2 / tn;
                println!("{:4} {:9} {:16} v2 {:6.2} nodetect {:6.2} {}", label, n, name.trim_start_matches("-- ").trim_end_matches(':').trim(), t2, tn, if r > 1.05 { "COST" } else if r < 0.95 { "gain" } else { "" });
            }
        }
    }
    for t in types.split(',') {
        match t {
            "u32" => go("u32", g::unsigned_u32::generators_u32(), &sizes, &|x| devour_sort2(x), &|x| wu32::lsd(x)),
            "u64" => go("u64", g::unsigned_u64::generators_u64(), &sizes, &|x| devour_sort2(x), &|x| du64::dlsd_with(x, &mut Vec::new())),
            _ => {}
        }
    }
}

/// Detectors one by one (cumulative) on non-detected distributions.
#[test]
#[ignore]
fn tmp_weak_detect_parts() {
    use super::super::v2::special::heavy::{heavy_sorted_by, may_be_heavy, HEAVY_MIN};
    use super::super::v2::special::plateaus::plateaus;
    use super::super::v2::special::{spread, DENSE_MIN};
    use super::super::v2::engines::dlsd::du64;
    use super::super::v2::engines::lsd::wu32;
    use super::super::v2::devour::{detect_u32, detect_u64};
    use super::super::v2::inspect::presorted::presorted;
    let sizes: Vec<usize> = std::env::var("PO_SIZES").ok().map(|s| s.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![1_000, 3_000, 10_000, 30_000, 100_000]);
    macro_rules! parts {
        ($t:ty, $m:ident, $gens:expr, $eng:expr) => {{
            let eng = $eng;
            for &n in &sizes {
                for (gen, name) in $gens.iter() {
                    if !["Unif ", "Normale 16", "Normale 24", "Zipf", "Small 5", "Small2"].iter().any(|k| name.contains(k)) { continue; }
                    let a = gen(n);
                    let mut ts = vec![];
                    for stage in 0..6 {
                        let f = |x: &mut [$t]| {
                            if let Err(k) = presorted(x) {
                                let det = |arr: &mut [$t], eq: usize| -> bool {
                                    if stage == 5 { return false; }
                                    let mut sample = spread(arr);
                                    if stage >= 1 && $m::range(arr, &sample) { return true; }
                                    if stage >= 2 && plateaus(arr, eq) { return true; }
                                    if stage >= 3 && $m::few(arr, &sample) { return true; }
                                    if stage >= 4 && arr.len() >= HEAVY_MIN && (arr.len() >= DENSE_MIN || may_be_heavy(&sample, |x, y| x == y)) {
                                        sample.sort_unstable();
                                        if heavy_sorted_by(arr, &sample, HEAVY_MIN, &|r: &mut [$t]| devour_sort2(r), |x, y| x == y, |x, y| x < y) { return true; }
                                        if arr.len() >= DENSE_MIN && $m::dense(arr, &sample) { return true; }
                                    }
                                    std::hint::black_box(&sample);
                                    false
                                };
                                radix_path(x, k, true, &det, &eng);
                            }
                        };
                        ts.push((0..2).map(|_| batch_ns(&a, &f)).fold(f64::MAX, f64::min));
                    }
                    let v2 = (0..2).map(|_| batch_ns(&a, &|x: &mut [$t]| devour_sort2(x))).fold(f64::MAX, f64::min);
                    println!("{} {:7} {:12} none {:.2} spread {:.2} +range {:.2} +plat {:.2} +few {:.2} +heavy/dense {:.2} | v2 {:.2}", stringify!($t), n, name.trim_start_matches("-- ").trim_end_matches(':').trim(), ts[5], ts[0], ts[1], ts[2], ts[3], ts[4], v2);
                }
            }
        }};
    }
    parts!(u32, detect_u32, g::unsigned_u32::generators_u32(), |x: &mut [u32]| wu32::lsd(x));
    parts!(u64, detect_u64, g::unsigned_u64::generators_u64(), |x: &mut [u64]| du64::dlsd_with(x, &mut Vec::new()));
}

