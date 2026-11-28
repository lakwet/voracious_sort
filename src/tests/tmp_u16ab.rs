//! Local only: u16 / i16 with the rule under test on, against std.
use crate::v2::dev::ab::NEW_RULE;
use std::sync::atomic::Ordering::Relaxed;

fn rng(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[test]
fn tmp_u16_ab() {
    let mut seed = 0x1234_5678_9abc_def1u64;
    let sizes = [151usize, 200, 300, 511, 512, 500, 1000, 4095, 4096, 4097, 5000, 10000, 80000, 131072, 600000];
    for &n in &sizes {
        for shape in 0..17 {
            let mut v: Vec<u16> = (0..n)
                .map(|i| {
                    let r = rng(&mut seed);
                    match shape {
                        0 => r as u16,
                        1 => [7u16, 900, 40000, 65535][(r % 4) as usize],
                        2 => [3u16, 9][(r % 2) as usize],
                        // a 5th value late
                        3 => if i == n - 1 { 12345 } else { [1u16, 2, 3, 4][(r % 4) as usize] },
                        4 => if i == n / 2 + 7 { 5 } else { [1u16, 2, 3, 4][(r % 4) as usize] },
                        5 => (n - i) as u16,
                        6 => (r % 16) as u16,
                        7 => if r % 10 == 0 { r as u16 } else { 42 },
                        8 => (i as u16).wrapping_mul(3) ^ (r % 3) as u16,
                        9 => ((r % 1000) as u16).wrapping_add(30000),
                        10 => (r as u16) & 0xFF00 | 0x37,
                        11 => ((r % 200) as u16) + 65300,
                        13 => if i % 2 == 0 { (r % 16) as u16 } else { 65535 - (r % 16) as u16 },
                        14 => if i == n * 95 / 100 { 65535 } else { (r % 16) as u16 + 100 },
                        15 => (r % 250) as u16 + 65400,
                        12 => if r % 10 == 0 { r as u16 } else { 0 },
                        _ => if r % 10 < 6 { [11u16, 22, 33, 44][(r % 4) as usize] } else { r as u16 },
                    }
                })
                .collect();
            let mut w: Vec<i16> = v.iter().map(|&x| x as i16).collect();
            let mut e = v.clone();
            e.sort();
            let mut ei = w.clone();
            ei.sort();
            NEW_RULE.store(true, Relaxed);
            crate::v2::devour::devour_sort(&mut v);
            crate::v2::devour::devour_sort(&mut w);
            NEW_RULE.store(false, Relaxed);
            assert_eq!(v, e, "u16 n {} shape {}", n, shape);
            assert_eq!(w, ei, "i16 n {} shape {}", n, shape);
        }
    }
}

#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_micro() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    use std::time::Instant;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&g::helper_random_array_small_u16, "Small"),
        (&x::heavy4, "Heavy 4"),
        (&x::aligned, "Aligned"),
        (&x::gaussian, "Gaussian"),
        (&g::helper_random_array_allequals_u16, "Equal"),
    ];
    for &n in &[500usize, 1000, 5000] {
        for (gen, name) in &gens {
            let arrays: Vec<Vec<u16>> = (0..64).map(|_| gen(n)).collect();
            let mut work = vec![0u16; n];
            let mut time = |f: &dyn Fn(&mut [u16])| {
                let mut best = f64::MAX;
                for _ in 0..7 {
                    let t = Instant::now();
                    for _ in 0..(400_000 / n).max(4) {
                        for a in &arrays {
                            work.copy_from_slice(a);
                            f(&mut work);
                        }
                    }
                    let keys = ((400_000 / n).max(4) * arrays.len() * n) as f64;
                    best = best.min(t.elapsed().as_nanos() as f64 / keys);
                }
                best
            };
            let copy = time(&|_a| {});
            let v2 = time(&|a| crate::v2::devour::devour_sort(a));
            let lsd = time(&|a| crate::v2::engines::lsd::wu16::lsd(a));
            let blz = time(&|a| blazing_radix::radix_sort::<u16>(a));
            let st = time(&|a| a.sort_unstable());
            println!(
                "{:>5} {:9} copy {:.2} | v2 {:.2} lsd {:.2} blz {:.2} std {:.2}",
                n, name, copy, v2 - copy, lsd - copy, blz - copy, st - copy
            );
        }
    }
}

#[cfg(feature = "v2_trace")]
#[test]
#[ignore]
fn tmp_u16_paths() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&g::helper_random_array_small_u16, "Small"),
        (&x::heavy4, "Heavy 4"),
        (&x::aligned, "Aligned"),
        (&x::gaussian, "Gaussian"),
    ];
    for &n in &[500usize, 1000, 5000] {
        for (gen, name) in &gens {
            for _ in 0..3 {
                let mut a = gen(n);
                crate::v2::dev::trace::take();
                crate::v2::devour::devour_sort(&mut a);
                println!("{:>5} {:9} {:?}", n, name, crate::v2::dev::trace::take());
            }
        }
    }
}


/// Detector matrix: the LSD alone, then the LSD plus one detector at a time
/// (`u16::VARIANTS`), v2 and blazing, interleaved on the same arrays.
/// `MATRIX_OUT`: csv file; `MATRIX_SIZES`: sizes.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_matrix() {
    use crate::generators as g;
    use crate::v2::devour::u16::{variant_sort, VARIANTS};
    use std::io::Write;
    use std::time::Instant;
    let out = std::env::var("MATRIX_OUT").unwrap_or_else(|_| "matrix-u16.csv".into());
    let sizes: Vec<usize> = std::env::var("MATRIX_SIZES")
        .unwrap_or_else(|_| "500,1000,2000,5000,10000,20000,50000,80000,100000,200000,500000,1000000,2000000,5000000,10000000".into())
        .split(',')
        .map(|x| x.trim().parse().unwrap())
        .collect();
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    let mut names: Vec<String> = VARIANTS.iter().map(|s| s.to_string()).collect();
    names.push("v2".into());
    names.push("blazing".into());
    let mut f = std::fs::File::create(&out).unwrap();
    writeln!(f, "size,gen,{}", names.join(",")).unwrap();
    const BATCH: usize = 2_000_000;
    const REPS: usize = 5;
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name = title.trim_start_matches("--").trim_end_matches(':').trim().to_string();
            let count = (BATCH / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work: Vec<Vec<u16>> = arrays.clone();
            let run = |v: usize, a: &mut [u16]| {
                if v < VARIANTS.len() {
                    variant_sort(a, v)
                } else if v == VARIANTS.len() {
                    crate::v2::devour::devour_sort(a)
                } else {
                    blazing_radix::radix_sort::<u16>(a)
                }
            };
            let mut times = vec![Vec::new(); names.len() + 1];
            for rep in 0..REPS {
                // Copy alone (subtracted), then each variant, random order.
                for k in 0..=names.len() {
                    let v = order(rep, k, names.len() + 1);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < names.len() {
                            run(v, w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < names.len() {
                        assert!(work[0].windows(2).all(|x| x[0] <= x[1]), "{} {} {}", names[v], n, name);
                    }
                }
            }
            let med = |mut x: Vec<f64>| {
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(times[names.len()].clone());
            let cells: Vec<String> =
                (0..names.len()).map(|v| format!("{:.4}", (med(times[v].clone()) - copy).max(0.0001))).collect();
            writeln!(f, "{},{},{}", n, name, cells.join(",")).unwrap();
            f.flush().unwrap();
        }
        println!("size {} done", n);
    }
}


/// Stage 1 of the u16 chain rebuild: the engines alone (any input), v2 and
/// blazing as references, interleaved on the same arrays (copy time
/// subtracted). `MATRIX_OUT`: csv file; `MATRIX_SIZES`: sizes.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_engines() {
    use crate::generators as g;
    use std::io::Write;
    use std::time::Instant;
    let out = std::env::var("MATRIX_OUT").unwrap_or_else(|_| "engines-u16.csv".into());
    let sizes: Vec<usize> = std::env::var("MATRIX_SIZES")
        .unwrap_or_else(|_| {
            "200,300,500,1000,2000,5000,10000,20000,50000,80000,100000,150000,200000,300000,             500000,1000000,2000000,5000000,10000000"
                .into()
        })
        .split(',')
        .map(|x| x.trim().parse().unwrap())
        .collect();
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    type Sort = Box<dyn Fn(&mut [u16])>;
    let sorts: Vec<(&str, Sort)> = vec![
        ("lsd", Box::new(|a: &mut [u16]| crate::v2::engines::lsd::wu16::lsd(a))),
        ("domain", Box::new(|a: &mut [u16]| crate::v2::engines::counting::domain_count(a, |k| k, |k| k))),
        ("std", Box::new(|a: &mut [u16]| a.sort_unstable())),
        ("v2", Box::new(|a: &mut [u16]| crate::v2::devour::devour_sort(a))),
        ("blazing", Box::new(|a: &mut [u16]| blazing_radix::radix_sort::<u16>(a))),
    ];
    let names: Vec<&str> = sorts.iter().map(|s| s.0).collect();
    let mut f = std::fs::File::create(&out).unwrap();
    writeln!(f, "size,gen,{}", names.join(",")).unwrap();
    const BATCH: usize = 2_000_000;
    const REPS: usize = 5;
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name = title.trim_start_matches("--").trim_end_matches(':').trim().to_string();
            let count = (BATCH / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..REPS {
                // The copy alone (index k - 1), then each sort; start rotated.
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < sorts.len() {
                            (sorts[v].1)(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < sorts.len() {
                        assert!(work[0].windows(2).all(|x| x[0] <= x[1]), "{} {} {}", names[v], n, name);
                    }
                }
            }
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            let cells: Vec<String> =
                (0..sorts.len()).map(|v| format!("{:.4}", (med(&times[v]) - copy).max(0.0001))).collect();
            writeln!(f, "{},{},{}", n, name, cells.join(",")).unwrap();
            f.flush().unwrap();
        }
        println!("size {} done", n);
    }
}


/// domain_count against its u16 copies and blazing (large sizes).
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_domain() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    use std::time::Instant;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&x::gaussian, "Gaussian"),
        (&g::helper_random_array_zipf_u16, "Zipf"),
        (&x::sensor, "Sensor"),
        (&x::heavy4, "Heavy 4"),
        (&x::range16, "Range 16"),
    ];
    type Sort = Box<dyn Fn(&mut [u16])>;
    let sorts: Vec<(&str, Sort)> = vec![
        ("domain", Box::new(|a: &mut [u16]| crate::v2::engines::counting::domain_count(a, |k| k, |k| k))),
        ("pick", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_domain_count16(a))),
        ("dcopy", Box::new(|a: &mut [u16]| tmp_dcopy(a, |k| k, |k| k))),
        ("pm128", Box::new(tmp_pickm16::<128>)),
        ("nt", Box::new(tmp_nt16)),
        ("blazing", Box::new(|a: &mut [u16]| blazing_radix::radix_sort::<u16>(a))),
    ];
    println!("{:>9} {:9} {}", "size", "gen", sorts.iter().map(|s| format!("{:>9}", s.0)).collect::<String>());
    let sizes: Vec<usize> = std::env::var("DOM_SIZES").ok().map(|v| v.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![150_000, 300_000, 1_000_000, 3_000_000, 10_000_000]);
    for &n in &sizes {
        for (gen, name) in &gens {
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..7 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < sorts.len() {
                            (sorts[v].1)(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < sorts.len() {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {}", sorts[v].0, n);
                    }
                }
            }
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            println!(
                "{:>9} {:9} {}",
                n,
                name,
                (0..sorts.len()).map(|v| format!("{:9.3}", med(&times[v]) - copy)).collect::<String>()
            );
        }
    }
}

/// Experiment: domain counting whose write pass uses non-temporal 32-byte
/// stores (AVX2 `_mm256_stream_si256`): a 16-lane accumulator is filled by
/// masked blends, streamed when full, and long buckets stream splats.
#[cfg(all(feature = "bench", target_arch = "x86_64"))]
fn tmp_nt16(keys: &mut [u16]) { tmp_acc16::<true>(keys) }
#[cfg(all(feature = "bench", target_arch = "x86_64"))]
fn tmp_acc16<const NT: bool>(keys: &mut [u16]) {
    let n = keys.len();
    let mut counts = vec![0u32; 1 << 16];
    let (quads, tail) = keys.as_chunks::<4>();
    for q in quads {
        unsafe {
            *counts.get_unchecked_mut(q[0] as usize) += 1;
            *counts.get_unchecked_mut(q[1] as usize) += 1;
            *counts.get_unchecked_mut(q[2] as usize) += 1;
            *counts.get_unchecked_mut(q[3] as usize) += 1;
        }
    }
    for &k in tail {
        counts[k as usize] += 1;
    }
    assert!(is_x86_feature_detected!("avx2"));
    unsafe { nt_write::<NT>(keys, &counts) }
}

#[cfg(all(feature = "bench", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn nt_write<const NT: bool>(keys: &mut [u16], counts: &[u32]) {
    use std::arch::x86_64::*;
    let n = keys.len();
    let base = keys.as_mut_ptr();
    let head = ((((32 - (base as usize & 31)) & 31) / 2) as usize).min(n);
    // Scalar head until the output is 32-byte aligned.
    let mut d = 0usize;
    let mut rem = 0usize;
    let mut p = 0usize;
    while p < head {
        while rem == 0 {
            rem = counts[d] as usize;
            d += 1;
        }
        let k = rem.min(head - p);
        keys[p..p + k].fill((d - 1) as u16);
        p += k;
        rem -= k;
    }
    let lanes = _mm256_setr_epi16(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let mut q = unsafe { base.add(head) } as *mut __m256i;
    let mut acc = _mm256_setzero_si256();
    let mut f = 0usize;
    let mut put = |v: u16, c: usize, acc: &mut __m256i, f: &mut usize, q: &mut *mut __m256i| {
        let vv = _mm256_set1_epi16(v as i16);
        let e = *f + c;
        let lo = _mm256_cmpgt_epi16(lanes, _mm256_set1_epi16(*f as i16 - 1));
        if e < 16 {
            let hi = _mm256_cmpgt_epi16(_mm256_set1_epi16(e as i16), lanes);
            *acc = _mm256_blendv_epi8(*acc, vv, _mm256_and_si256(lo, hi));
            *f = e;
            return;
        }
        *acc = _mm256_blendv_epi8(*acc, vv, lo);
        unsafe {
            if NT { _mm256_stream_si256(*q, *acc) } else { _mm256_store_si256(*q, *acc) }
            *q = q.add(1);
        }
        let mut r = e - 16;
        while r >= 16 {
            unsafe {
                if NT { _mm256_stream_si256(*q, vv) } else { _mm256_store_si256(*q, vv) }
                *q = q.add(1);
            }
            r -= 16;
        }
        *acc = vv;
        *f = r;
    };
    if rem != 0 {
        put((d - 1) as u16, rem, &mut acc, &mut f, &mut q);
    }
    for (i, &c) in counts.iter().enumerate().skip(d) {
        if c != 0 {
            put(i as u16, c as usize, &mut acc, &mut f, &mut q);
        }
    }
    _mm_sfence();
    let mut last = [0u16; 16];
    unsafe { _mm256_storeu_si256(last.as_mut_ptr() as *mut __m256i, acc) };
    let start = n - f;
    keys[start..].copy_from_slice(&last[..f]);
}

/// Same experiment on ARM: two NEON accumulators (16 lanes), streamed by
/// `stnp q, q` (32 bytes, non-temporal hint).
#[cfg(all(feature = "bench", target_arch = "aarch64"))]
fn tmp_nt16(keys: &mut [u16]) { tmp_acc16::<true>(keys) }
#[cfg(all(feature = "bench", target_arch = "aarch64"))]
fn tmp_acc16<const NT: bool>(keys: &mut [u16]) {
    use std::arch::aarch64::*;
    let n = keys.len();
    let mut counts = vec![0u32; 1 << 16];
    let (quads, tail) = keys.as_chunks::<4>();
    for q in quads {
        unsafe {
            *counts.get_unchecked_mut(q[0] as usize) += 1;
            *counts.get_unchecked_mut(q[1] as usize) += 1;
            *counts.get_unchecked_mut(q[2] as usize) += 1;
            *counts.get_unchecked_mut(q[3] as usize) += 1;
        }
    }
    for &k in tail {
        counts[k as usize] += 1;
    }
    let base = keys.as_mut_ptr();
    let head = (((32 - (base as usize & 31)) & 31) / 2).min(n);
    let (mut d, mut rem, mut p) = (0usize, 0usize, 0usize);
    while p < head {
        while rem == 0 {
            rem = counts[d] as usize;
            d += 1;
        }
        let k = rem.min(head - p);
        keys[p..p + k].fill((d - 1) as u16);
        p += k;
        rem -= k;
    }
    unsafe {
        let l0: [u16; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let l1: [u16; 8] = [8, 9, 10, 11, 12, 13, 14, 15];
        let (lane0, lane1) = (vld1q_u16(l0.as_ptr()), vld1q_u16(l1.as_ptr()));
        let mut q = base.add(head);
        let (mut a0, mut a1) = (vdupq_n_u16(0), vdupq_n_u16(0));
        let mut f = 0usize;
        let stream = |q: &mut *mut u16, x: uint16x8_t, y: uint16x8_t| {
            if NT {
                core::arch::asm!("stnp {a:q}, {b:q}, [{p}]", a = in(vreg) x, b = in(vreg) y,
                    p = in(reg) *q, options(nostack, preserves_flags));
            } else {
                core::arch::asm!("stp {a:q}, {b:q}, [{p}]", a = in(vreg) x, b = in(vreg) y,
                    p = in(reg) *q, options(nostack, preserves_flags));
            }
            *q = q.add(16);
        };
        let mut put = |v: u16, c: usize, a0: &mut uint16x8_t, a1: &mut uint16x8_t, f: &mut usize,
                       q: &mut *mut u16| {
            let vv = vdupq_n_u16(v);
            let e = *f + c;
            let fv = vdupq_n_u16(*f as u16);
            let (m0, m1) = (vcgeq_u16(lane0, fv), vcgeq_u16(lane1, fv));
            if e < 16 {
                let ev = vdupq_n_u16(e as u16);
                *a0 = vbslq_u16(vandq_u16(m0, vcltq_u16(lane0, ev)), vv, *a0);
                *a1 = vbslq_u16(vandq_u16(m1, vcltq_u16(lane1, ev)), vv, *a1);
                *f = e;
                return;
            }
            *a0 = vbslq_u16(m0, vv, *a0);
            *a1 = vbslq_u16(m1, vv, *a1);
            stream(q, *a0, *a1);
            let mut r = e - 16;
            while r >= 16 {
                stream(q, vv, vv);
                r -= 16;
            }
            *a0 = vv;
            *a1 = vv;
            *f = r;
        };
        if rem != 0 {
            put((d - 1) as u16, rem, &mut a0, &mut a1, &mut f, &mut q);
        }
        for (i, &c) in counts.iter().enumerate().skip(d) {
            if c != 0 {
                put(i as u16, c as usize, &mut a0, &mut a1, &mut f, &mut q);
            }
        }
        let mut last = [0u16; 16];
        vst1q_u16(last.as_mut_ptr(), a0);
        vst1q_u16(last.as_mut_ptr().add(8), a1);
        keys[n - f..].copy_from_slice(&last[..f]);
    }
}

/// Experiment: `pick` whose buckets of `M` keys or more are written by
/// `fill` (memset) whatever the mode.
#[cfg(feature = "bench")]
fn tmp_pickm16<const M: usize>(keys: &mut [u16]) {
    let n = keys.len();
    let mut counts = vec![0u32; 1 << 16];
    let (quads, tail) = keys.as_chunks::<4>();
    for q in quads {
        unsafe {
            *counts.get_unchecked_mut(q[0] as usize) += 1;
            *counts.get_unchecked_mut(q[1] as usize) += 1;
            *counts.get_unchecked_mut(q[2] as usize) += 1;
            *counts.get_unchecked_mut(q[3] as usize) += 1;
        }
    }
    for &k in tail {
        counts[k as usize] += 1;
    }
    let long = {
        let (mut sum, mut used) = (0usize, 0usize);
        for i in 0..256 {
            let c = counts[i * 256 + 128] as usize;
            sum += c;
            used += (c != 0) as usize;
        }
        sum >= 12 * used.max(1)
    };
    let mut start = 0;
    for (d, &c) in counts.iter().enumerate() {
        let c = c as usize;
        if c != 0 {
            let v = d as u16;
            let end = start + c;
            if c >= M {
                keys[start..end].fill(v);
            } else if long {
                let mut p = start;
                while p < end && n - p >= 32 {
                    keys[p..p + 32].copy_from_slice(&[v; 32]);
                    p += 32;
                }
                if p < end {
                    keys[p..end].fill(v);
                }
            } else if c <= 64 && end + 16 <= n {
                let mut p = start;
                while p < end {
                    keys[p..p + 16].copy_from_slice(&[v; 16]);
                    p += 16;
                }
            } else {
                keys[start..end].fill(v);
            }
            start = end;
        }
    }
}

/// Diagnosis: verbatim copy of the shared `domain_count` path taken by u16
/// (one lane, no mixed index), generic as the original.
#[cfg(feature = "bench")]
#[inline]
fn tmp_dcopy<U: crate::v2::devour::keys::Key, E: Fn(U) -> U + Copy, D: Fn(U) -> U + Copy>(
    keys: &mut [U],
    encode: E,
    decode: D,
) {
    let n = keys.len();
    let width = (U::MAX_U64 + 1) as usize;
    let mut counts = vec![0u32; width];
    let idx = |k: U| encode(k).to_u64() as usize;
    let (quads, tail) = keys.as_chunks::<4>();
    for q in quads {
        unsafe {
            *counts.get_unchecked_mut(idx(q[0])) += 1;
            *counts.get_unchecked_mut(idx(q[1])) += 1;
            *counts.get_unchecked_mut(idx(q[2])) += 1;
            *counts.get_unchecked_mut(idx(q[3])) += 1;
        }
    }
    for &k in tail {
        counts[idx(k)] += 1;
    }
    let mut start = 0;
    for d in 0..width {
        let c = counts[d];
        if c != 0 {
            let v = decode(U::from_u64(d as u64));
            let c = c as usize;
            let end = start + c;
            if c <= 64 && end + 16 <= keys.len() {
                let block = [v; 16];
                let mut p = start;
                while p < end {
                    keys[p..p + 16].copy_from_slice(&block);
                    p += 16;
                }
            } else {
                keys[start..end].fill(v);
            }
            start = end;
        }
    }
    debug_assert_eq!(start, n);
}

/// `domain_count16` with the largest cache forced small: the streamed write
/// on every size and alignment.
#[test]
#[ignore]
fn tmp_u16_stream() {
    use crate::v2::cache::L2_OVERRIDE;
    let mut seed = 0x1234_5678_9abc_def0u64;
    for &cache in &[4096usize, 1 << 18, 0] {
    L2_OVERRIDE.store(cache, Relaxed);
    for &n in &[4097usize, 5000, 70_001, 131_072, 200_003, 300_017] {
        for shape in 0..7 {
            let base: Vec<u16> = (0..n + 16)
                .map(|i| {
                    let r = rng(&mut seed);
                    match shape {
                        0 => r as u16,
                        1 => 7,
                        2 => (r % 16) as u16 + 1000,
                        3 => (r % 3) as u16 * 20_000,
                        4 => ((r % 64) * (r >> 20) % 64) as u16,
                        5 => (n + 16 - i) as u16,
                        _ => (r % 30_000) as u16,
                    }
                })
                .collect();
            for off in 0..16 {
                let mut a = base.clone();
                let s = &mut a[off..off + n];
                let mut r = s.to_vec();
                r.sort_unstable();
                crate::v2::devour::u16::tmp_domain_count16(s);
                assert!(s == &r[..], "cache {} n {} shape {} off {}", cache, n, shape, off);
                assert_eq!(&a[..off], &base[..off]);
                assert_eq!(&a[off + n..], &base[off + n..]);
            }
        }
    }
    }
    L2_OVERRIDE.store(0, Relaxed);
}

/// Time split of the u16 sort: whole sort, engine alone, counting alone
/// (table allocated), allocation alone; blazing whole. ns per key, the copy
/// of the input subtracted.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_split() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    use std::hint::black_box;
    use std::time::Instant;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&x::gaussian, "Gaussian"),
        (&g::helper_random_array_zipf_u16, "Zipf"),
        (&x::sensor, "Sensor"),
    ];
    fn count_only(keys: &mut [u16]) {
        let mut counts = vec![0u32; 1 << 16];
        let (quads, tail) = keys.as_chunks::<4>();
        for q in quads {
            unsafe {
                *counts.get_unchecked_mut(q[0] as usize) += 1;
                *counts.get_unchecked_mut(q[1] as usize) += 1;
                *counts.get_unchecked_mut(q[2] as usize) += 1;
                *counts.get_unchecked_mut(q[3] as usize) += 1;
            }
        }
        for &k in tail {
            counts[k as usize] += 1;
        }
        black_box(&counts);
    }
    type Sort = Box<dyn Fn(&mut [u16])>;
    let sorts: Vec<(&str, Sort)> = vec![
        ("v2", Box::new(|a: &mut [u16]| crate::v2::devour::devour_sort(a))),
        ("engine", Box::new(|a: &mut [u16]| {
            if a.len() >= 1 << 17 {
                crate::v2::devour::u16::tmp_domain_count16(a)
            } else {
                crate::v2::engines::lsd::wu16::lsd(a)
            }
        })),
        ("count", Box::new(count_only)),
        ("alloc", Box::new(|_a: &mut [u16]| {
            black_box(vec![0u32; 1 << 16]);
        })),
        ("blazing", Box::new(|a: &mut [u16]| blazing_radix::radix_sort::<u16>(a))),
    ];
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![1000, 10_000, 300_000, 1_000_000, 10_000_000]);
    println!("{:>9} {:9} {}", "size", "gen", sorts.iter().map(|s| format!("{:>9}", s.0)).collect::<String>());
    for &n in &sizes {
        for (gen, name) in &gens {
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < sorts.len() {
                            (sorts[v].1)(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                }
            }
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            println!(
                "{:>9} {:9} {}",
                n,
                name,
                (0..sorts.len()).map(|v| format!("{:9.3}", med(&times[v]) - copy)).collect::<String>()
            );
        }
    }
}

/// One sort in a loop, for `perf record` (PROF = v2 | blazing, PROF_N, PROF_GEN).
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_prof() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    let which = std::env::var("PROF").unwrap_or("v2".into());
    let n: usize = std::env::var("PROF_N").ok().map(|v| v.parse().unwrap()).unwrap_or(1_000_000);
    let gen = std::env::var("PROF_GEN").unwrap_or("Unif".into());
    let a = match gen.as_str() {
        "Gaussian" => x::gaussian(n),
        "Sensor" => x::sensor(n),
        _ => g::helper_random_array_uniform_u16(n),
    };
    let mut w = a.clone();
    let reps = (400_000_000 / n).max(4);
    for _ in 0..reps {
        w.copy_from_slice(&a);
        if which == "blazing" {
            blazing_radix::radix_sort::<u16>(&mut w);
        } else if which == "lsd" {
            crate::v2::engines::lsd::wu16::lsd(&mut w);
        } else {
            crate::v2::devour::devour_sort(&mut w);
        }
        std::hint::black_box(&w);
    }
}

// ---- Points 1 and 2 (22/11): 16-bit counters, branch-free bucket writes ----

#[cfg(feature = "bench")]
fn cnt32(keys: &[u16]) -> Vec<u32> {
    let mut counts = vec![0u32; 1 << 16];
    let (quads, tail) = keys.as_chunks::<4>();
    for q in quads {
        unsafe {
            *counts.get_unchecked_mut(q[0] as usize) += 1;
            *counts.get_unchecked_mut(q[1] as usize) += 1;
            *counts.get_unchecked_mut(q[2] as usize) += 1;
            *counts.get_unchecked_mut(q[3] as usize) += 1;
        }
    }
    for &k in tail {
        counts[k as usize] += 1;
    }
    counts
}

/// 16-bit counters (128 KiB table): a counter wrapping to 0 records its key
/// (a branch almost never taken). Returns the low counts and the wraps.
#[cfg(feature = "bench")]
fn cnt16(keys: &[u16]) -> (Vec<u16>, Vec<u16>) {
    let mut low = vec![0u16; 1 << 16];
    let mut wraps = Vec::new();
    let (quads, tail) = keys.as_chunks::<4>();
    macro_rules! inc {
        ($k:expr) => {{
            let k = $k as usize;
            let c = unsafe { low.get_unchecked_mut(k) };
            *c = c.wrapping_add(1);
            if *c == 0 {
                wraps.push(k as u16);
            }
        }};
    }
    for q in quads {
        inc!(q[0]);
        inc!(q[1]);
        inc!(q[2]);
        inc!(q[3]);
    }
    for &k in tail {
        inc!(k);
    }
    (low, wraps)
}

#[cfg(feature = "bench")]
fn widen(low: &[u16], wraps: &[u16]) -> Vec<u32> {
    let mut c: Vec<u32> = low.iter().map(|&x| x as u32).collect();
    for &k in wraps {
        c[k as usize] += 1 << 16;
    }
    c
}

/// The production write (pick), copied.
#[cfg(feature = "bench")]
fn write_pick<C: Copy>(keys: &mut [u16], counts: &[C], get: impl Fn(C) -> usize) {
    let n = keys.len();
    let long = {
        let (mut sum, mut used) = (0usize, 0usize);
        for i in 0..256 {
            let c = get(counts[i * 256 + 128]);
            sum += c;
            used += (c != 0) as usize;
        }
        sum >= 12 * used.max(1)
    };
    let mut start = 0;
    for (d, &c) in counts.iter().enumerate() {
        let c = get(c);
        if c != 0 {
            let v = d as u16;
            let end = start + c;
            if long {
                let mut p = start;
                while p < end && n - p >= 32 {
                    keys[p..p + 32].copy_from_slice(&[v; 32]);
                    p += 32;
                }
                if p < end {
                    keys[p..end].fill(v);
                }
            } else if c <= 64 && end + 16 <= n {
                let mut p = start;
                while p < end {
                    keys[p..p + 16].copy_from_slice(&[v; 16]);
                    p += 16;
                }
            } else {
                keys[start..end].fill(v);
            }
            start = end;
        }
    }
}

/// Branch-free on the bucket size: every bucket (SKIP: every non-empty one)
/// writes one 32-key block at its start, overwritten by the following
/// buckets; only buckets over 32 keys loop.
#[cfg(feature = "bench")]
fn write_flat<C: Copy, const SKIP: bool>(
    keys: &mut [u16],
    counts: &[C],
    get: impl Fn(C) -> usize,
) {
    let n = keys.len();
    let mut start = 0;
    for (d, &c) in counts.iter().enumerate() {
        let c = get(c);
        if SKIP && c == 0 {
            continue;
        }
        let v = d as u16;
        let end = start + c;
        if n - start >= 32 {
            keys[start..start + 32].copy_from_slice(&[v; 32]);
            if c > 32 {
                let mut p = start + 32;
                while p < end {
                    if n - p >= 32 {
                        keys[p..p + 32].copy_from_slice(&[v; 32]);
                    } else {
                        keys[p..end].fill(v);
                    }
                    p += 32;
                }
            }
        } else {
            keys[start..end].fill(v);
        }
        start = end;
    }
}

/// Engine variants: counting (32 or 16 bits) x write (0 pick, 1 flat,
/// 2 flat skipping empty buckets); beyond `largest_bytes` the streamed
/// write, as in production.
#[cfg(feature = "bench")]
fn eng<const C16: bool, const W: u8>(keys: &mut [u16]) {
    let big = keys.len() > crate::v2::cache::largest_bytes();
    let wr = |keys: &mut [u16], c: &[u32]| {
        if big && crate::v2::devour::u16::tmp_stream_write(keys, c) {
            return;
        }
        match W {
            0 => write_pick(keys, c, |x| x as usize),
            1 => write_flat::<u32, false>(keys, c, |x| x as usize),
            _ => write_flat::<u32, true>(keys, c, |x| x as usize),
        }
    };
    if C16 {
        let (low, wraps) = cnt16(keys);
        if !wraps.is_empty() || big {
            let c = widen(&low, &wraps);
            wr(keys, &c);
        } else {
            match W {
                0 => write_pick(keys, &low, |x| x as usize),
                1 => write_flat::<u16, false>(keys, &low, |x| x as usize),
                _ => write_flat::<u16, true>(keys, &low, |x| x as usize),
            }
        }
    } else {
        let c = cnt32(keys);
        wr(keys, &c);
    }
}

/// Write mode from 256 pairs of neighbour counters: the flat write (no test
/// on empty buckets) when empty / full buckets alternate often against the
/// number of empty ones (a mispredicted test ~18 cycles, a wasted block ~4):
/// transitions > empties / 4.5.
#[cfg(feature = "bench")]
fn flat_pays<C: Copy>(counts: &[C], get: impl Fn(C) -> usize) -> bool {
    let (mut t, mut z) = (0usize, 0usize);
    for i in 0..256 {
        let a = get(counts[i * 256 + 37]) == 0;
        let b = get(counts[i * 256 + 38]) == 0;
        t += (a != b) as usize;
        z += a as usize;
    }
    9 * t > 2 * z
}

/// 16-bit counters when the 32-bit table (256 KiB) does not fit in half the
/// L2.
#[cfg(feature = "bench")]
fn c16_rule() -> bool { crate::v2::cache::l2_bytes() < 2 * (1 << 18) }

#[cfg(feature = "bench")]
fn write_auto<C: Copy>(keys: &mut [u16], counts: &[C], get: impl Fn(C) -> usize + Copy) {
    if flat_pays(counts, get) {
        write_flat::<C, false>(keys, counts, get)
    } else {
        write_pick(keys, counts, get)
    }
}

/// Round 2: counting 16 bits (C16 0 no, 1 yes, 2 by the L2 rule) x auto write.
#[cfg(feature = "bench")]
fn eng2<const C16: u8>(keys: &mut [u16]) {
    let big = keys.len() > crate::v2::cache::largest_bytes();
    let c16 = C16 == 1 || (C16 == 2 && c16_rule());
    if c16 {
        let (low, wraps) = cnt16(keys);
        if !wraps.is_empty() || big {
            let c = widen(&low, &wraps);
            if !(big && crate::v2::devour::u16::tmp_stream_write(keys, &c)) {
                write_auto(keys, &c, |x| x as usize);
            }
        } else {
            write_auto(keys, &low, |x| x as usize);
        }
    } else {
        let c = cnt32(keys);
        if !(big && crate::v2::devour::u16::tmp_stream_write(keys, &c)) {
            write_auto(keys, &c, |x| x as usize);
        }
    }
}

/// Points 1 and 2 against production and blazing (ns per key).
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_engine2() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    use std::time::Instant;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&x::gaussian, "Gaussian"),
        (&g::helper_random_array_zipf_u16, "Zipf"),
        (&x::sensor, "Sensor"),
        (&x::heavy4, "Heavy 4"),
        (&x::range16, "Range 16"),
        (&g::helper_random_array_spread4k_u16, "Spread 4k"),
        (&g::helper_random_array_spread256_u16, "Spread256"),
        (&g::helper_random_array_normal13_u16, "Normal 13"),
        (&g::helper_random_array_depth_u16, "Depth"),
    ];
    type Sort = Box<dyn Fn(&mut [u16])>;
    let sorts: Vec<(&str, Sort)> = vec![
        ("old", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_domain_count16_old(a))),
        ("new", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_domain_count16(a))),
        ("v2", Box::new(|a: &mut [u16]| crate::v2::devour::devour_sort(a))),
        ("flat", Box::new(crate::v2::devour::u16::tmp_domain_forced::<true>)),
        ("touched", Box::new(crate::v2::devour::u16::tmp_domain_forced::<false>)),
        ("blazing", Box::new(|a: &mut [u16]| blazing_radix::radix_sort::<u16>(a))),
    ];
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![150_000, 300_000, 1_000_000, 3_000_000, 10_000_000]);
    let head: String = sorts.iter().map(|s| format!("{:>8}", s.0)).collect();
    println!("{:>9} {:9} {}", "size", "gen", head);
    for &n in &sizes {
        for (gen, name) in &gens {
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < sorts.len() {
                            (sorts[v].1)(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < sorts.len() {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", sorts[v].0, n, name);
                    }
                }
            }
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            let cells: String =
                (0..sorts.len()).map(|v| format!("{:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:9} {}", n, name, cells);
        }
    }
}

/// Measurement order: a different random permutation of the `k` columns on
/// each repetition (the first column of a fixed rotation ran 7 % faster at
/// 10M on the Ryzen, whatever its code).
fn order(rep: usize, j: usize, k: usize) -> usize {
    let mut p: Vec<usize> = (0..k).collect();
    let mut seed = 0x9E37_79B9_7F4A_7C15u64 ^ (rep as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
    for i in (1..k).rev() {
        let r = rng(&mut seed) as usize % (i + 1);
        p.swap(i, r);
    }
    p[j]
}

/// Small arrays: whole sort, LSD alone, presorted scan alone, detectors
/// alone (sorting when one fires), blazing. ns per key.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_small_split() {
    use crate::generators::narrow_extra::extra_u16 as x;
    use crate::generators::unsigned_u16 as g;
    use std::hint::black_box;
    use std::time::Instant;
    let gens: Vec<(&dyn Fn(usize) -> Vec<u16>, &str)> = vec![
        (&g::helper_random_array_uniform_u16, "Unif"),
        (&x::gaussian, "Gaussian"),
        (&g::helper_random_array_zipf_u16, "Zipf"),
        (&x::sensor, "Sensor"),
        (&g::helper_random_array_small_u16, "Small"),
        (&g::helper_random_array_spread256_u16, "Spread256"),
    ];
    type Sort = Box<dyn Fn(&mut [u16])>;
    let sorts: Vec<(&str, Sort)> = vec![
        ("v2", Box::new(|a: &mut [u16]| crate::v2::devour::devour_sort(a))),
        ("lsd", Box::new(|a: &mut [u16]| crate::v2::engines::lsd::wu16::lsd(a))),
        ("lsd16", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_lsd16(a))),
        ("lsd16s", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_lsd16s(a))),
        ("lsd32s", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_lsd32s(a))),
        ("lsd32m", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_lsd32m(a))),
        ("lsd32f", Box::new(|a: &mut [u16]| crate::v2::devour::u16::tmp_lsd32f(a))),
        ("presort", Box::new(|a: &mut [u16]| {
            let _ = black_box(crate::v2::inspect::presorted::presorted(a).is_ok());
        })),
        ("detect", Box::new(|a: &mut [u16]| {
            black_box(crate::v2::devour::u16::tmp_detect(a));
        })),
        ("std", Box::new(|a: &mut [u16]| a.sort_unstable())),
        ("blazing", Box::new(|a: &mut [u16]| blazing_radix::radix_sort::<u16>(a))),
    ];
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![200, 500, 1000, 2000, 5000, 10_000, 50_000]);
    let head: String = sorts.iter().map(|s| format!("{:>8}", s.0)).collect();
    println!("{:>9} {:9} {}", "size", "gen", head);
    for &n in &sizes {
        for (gen, name) in &gens {
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = sorts.len() + 1;
            let mut times = vec![Vec::new(); k];
            for rep in 0..11 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < sorts.len() {
                            (sorts[v].1)(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                }
            }
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            let cells: String =
                (0..sorts.len()).map(|v| format!("{:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:9} {}", n, name, cells);
        }
    }
}

/// `lsd16` against `sort_unstable`: sizes around the stack / counter limits.
#[test]
#[ignore]
fn tmp_u16_lsd16() {
    let mut seed = 0x0123_4567_89ab_cdefu64;
    let sizes = [2usize, 3, 5, 151, 1000, 2047, 2048, 2049, 4097, 65535, 65536, 70001];
    for &n in &sizes {
        for shape in 0..11 {
            let a: Vec<u16> = (0..n)
                .map(|i| {
                    let r = rng(&mut seed);
                    match shape {
                        0 => r as u16,
                        1 => 7,
                        2 => (r % 256) as u16,
                        3 => ((r % 256) as u16) << 8 | 0x34,
                        4 => (n - i) as u16,
                        // A byte constant except on keys the sample skips.
                        6 if i % 97 == 5 => r as u16,
                        6 => (r % 256) as u16,
                        7 if i % 97 == 5 => r as u16,
                        7 => ((r % 256) as u16) << 8 | 0x34,
                        // Few values on both bytes (product counting).
                        8 => (r % 16) as u16 * 4099,
                        9 if i % 2 == 0 => (r % 16) as u16,
                        9 => u16::MAX - (r % 16) as u16,
                        10 => [0x1234u16, 0xFF00, 0x00FF, 7][(i / 50) % 4],
                        _ => (r % 3) as u16 * 20_000,
                    }
                })
                .collect();
            let mut b = a.clone();
            crate::v2::devour::u16::tmp_lsd16(&mut b);
            let mut r = a.clone();
            r.sort_unstable();
            assert!(b == r, "n {} shape {}", n, shape);
            let mut c = a.clone();
            crate::v2::devour::u16::tmp_lsd16s(&mut c);
            assert!(c == r, "seq n {} shape {}", n, shape);
            let mut e = a.clone();
            crate::v2::devour::u16::tmp_lsd32s(&mut e);
            assert!(e == r, "seq32 n {} shape {}", n, shape);
            let mut f = a.clone();
            crate::v2::devour::u16::tmp_lsd32m(&mut f);
            assert!(f == r, "mixed32 n {} shape {}", n, shape);
            let mut g = a.clone();
            crate::v2::devour::u16::tmp_lsd32f(&mut g);
            assert!(g == r, "flat32 n {} shape {}", n, shape);
        }
    }
}

/// The chunk engine: shared LSD (old) against `lsd16_mode::<3, true>` (new),
/// both through the whole v2 sort in the library; all u16 distributions.
/// ns per key, random order per repetition.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_chunk_ab() {
    use crate::generators as g;
    use crate::v2::devour::u16::OLD_CHUNK;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![200, 500, 1000, 2000, 5000, 10_000, 50_000, 100_000, 300_000, 1_000_000]);
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    println!("{:>9} {:12} {:>8} {:>8} {:>8}", "size", "gen", "old", "new", "blazing");
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String = title
                .trim_start_matches("--")
                .trim_end_matches(':')
                .trim()
                .replace(' ', "_");
            let count = (2_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = 4;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    OLD_CHUNK.store(v == 0, Relaxed);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        match v {
                            0 | 1 => crate::v2::devour::devour_sort(w),
                            2 => blazing_radix::radix_sort::<u16>(w),
                            _ => {},
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < 3 {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            OLD_CHUNK.store(false, Relaxed);
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[3]);
            println!(
                "{:>9} {:12} {:8.3} {:8.3} {:8.3}",
                n,
                name,
                med(&times[0]) - copy,
                med(&times[1]) - copy,
                med(&times[2]) - copy
            );
        }
    }
}

/// From `DOMAIN_MIN`: the former chain (old) against `sort_big` (new),
/// both through the whole v2 sort in the library; all u16 distributions.
/// ns per key, random order per repetition.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_big_ab() {
    use crate::generators as g;
    use crate::v2::devour::u16::OLD_BIG;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![150_000, 200_000, 300_000, 600_000, 1_000_000, 3_000_000, 10_000_000]);
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    println!("{:>9} {:12} {:>8} {:>8} {:>8}", "size", "gen", "old", "new", "blazing");
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String = title
                .trim_start_matches("--")
                .trim_end_matches(':')
                .trim()
                .replace(' ', "_");
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = 4;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    OLD_BIG.store(v == 0, Relaxed);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        match v {
                            0 | 1 => crate::v2::devour::devour_sort(w),
                            2 => blazing_radix::radix_sort::<u16>(w),
                            _ => {},
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < 3 {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            OLD_BIG.store(false, Relaxed);
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[3]);
            println!(
                "{:>9} {:12} {:8.3} {:8.3} {:8.3}",
                n,
                name,
                med(&times[0]) - copy,
                med(&times[1]) - copy,
                med(&times[2]) - copy
            );
        }
    }
}

/// The chain with one rule switched off at a time (`SKIP` bits), through the
/// whole v2 sort in the library; all u16 distributions. ns per key, random
/// order per repetition. Columns: full chain, then each bit off.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_skip() {
    use crate::generators as g;
    use crate::v2::devour::u16::SKIP;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    // SKIP_MASKS="mask:name,..." (decimal masks) replaces the default set.
    let masks: Vec<(u64, String)> = match std::env::var("SKIP_MASKS") {
        Ok(v) => v
            .split(',')
            .map(|m| {
                let (a, b) = m.split_once(':').unwrap();
                (a.parse().unwrap(), b.to_string())
            })
            .collect(),
        Err(_) => [
            (0, "full"),
            (1, "-range"),
            (2, "-plat"),
            (4, "-few"),
            (8, "-heavy"),
            (16, "-fewcmp"),
            (32, "-detect"),
            (64, "-verge"),
            (128, "-repair"),
            (32 | 64 | 128, "-all"),
        ]
        .iter()
        .map(|&(a, b)| (a, b.to_string()))
        .collect(),
    };
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![200, 500, 1000, 2000, 5000, 10_000, 30_000, 50_000, 100_000]);
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    let head: String = masks.iter().map(|m| format!(" {:>8}", m.1)).collect();
    println!("{:>9} {:12}{} {:>8}", "size", "gen", head, "blazing");
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String = title
                .trim_start_matches("--")
                .trim_end_matches(':')
                .trim()
                .replace(' ', "_");
            let count = (2_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = masks.len() + 2;
            let mut times = vec![Vec::new(); k];
            for rep in 0..7 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    if v < masks.len() {
                        SKIP.store(masks[v].0, Relaxed);
                    }
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        if v < masks.len() {
                            crate::v2::devour::devour_sort(w);
                        } else if v == masks.len() {
                            blazing_radix::radix_sort::<u16>(w);
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v <= masks.len() {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            SKIP.store(0, Relaxed);
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let copy = med(&times[k - 1]);
            let cells: String =
                (0..=masks.len()).map(|v| format!(" {:8.3}", med(&times[v]) - copy)).collect();
            println!("{:>9} {:12}{}", n, name, cells);
        }
    }
}

/// From `DOMAIN_MIN`: former chain (old), `sort_big` (new), `sort_big`
/// without the 4-equal-keys counting shortcut (norun), blazing. ns per key.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_big3() {
    use crate::generators as g;
    use crate::v2::devour::u16::{NO_RUN4, OLD_BIG};
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![150_000, 300_000, 1_000_000, 3_000_000, 10_000_000]);
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    println!("{:>9} {:12} {:>8} {:>8} {:>8} {:>8}", "size", "gen", "old", "new", "norun", "blazing");
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String = title
                .trim_start_matches("--")
                .trim_end_matches(':')
                .trim()
                .replace(' ', "_");
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = 5;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    OLD_BIG.store(v == 0, Relaxed);
                    NO_RUN4.store(v == 2, Relaxed);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        match v {
                            0..=2 => crate::v2::devour::devour_sort(w),
                            3 => blazing_radix::radix_sort::<u16>(w),
                            _ => {},
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < 4 {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            OLD_BIG.store(false, Relaxed);
            NO_RUN4.store(false, Relaxed);
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let c = med(&times[4]);
            println!(
                "{:>9} {:12} {:8.3} {:8.3} {:8.3} {:8.3}",
                n,
                name,
                med(&times[0]) - c,
                med(&times[1]) - c,
                med(&times[2]) - c,
                med(&times[3]) - c
            );
        }
    }
}

/// Quadruples of one value among 64 sampled (`runs_pay`), per distribution.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_runs_same() {
    use crate::generators as g;
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    for &n in &[150_000usize, 300_000, 1_000_000, 3_000_000, 10_000_000] {
        let cells: Vec<String> = gens
            .iter()
            .map(|(gen, title)| {
                let name = title.trim_start_matches("--").trim_end_matches(':').trim().replace(' ', "_");
                format!("{}={}", name, crate::v2::devour::u16::tmp_runs_same(&gen(n)))
            })
            .collect();
        println!("{} {}", n, cells.join(" "));
    }
}

/// Tonight's changes together: v2 with every switch on the former code
/// (shared LSD for chunks, former chain from `DOMAIN_MIN`, whole chain on
/// small arrays, no product counting, no 4-equal shortcut) against v2 now,
/// and blazing. ns per key, random order.
#[cfg(feature = "bench")]
#[test]
#[ignore]
fn tmp_u16_total() {
    use crate::generators as g;
    use crate::v2::devour::u16::{NO_RUN4, OLD_BIG, OLD_CHUNK, SKIP};
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    let sizes: Vec<usize> = std::env::var("DOM_SIZES")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![
            200, 500, 1000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000,
            3_000_000, 10_000_000,
        ]);
    let mut gens = g::unsigned_u16::generators_u16();
    gens.extend(g::narrow_extra::extra_u16::generators());
    let set_old = |old: bool| {
        OLD_CHUNK.store(old, Relaxed);
        OLD_BIG.store(old, Relaxed);
        NO_RUN4.store(old, Relaxed);
        SKIP.store(if old { 512 } else { 0 }, Relaxed);
    };
    println!("{:>9} {:12} {:>8} {:>8} {:>8}", "size", "gen", "old", "new", "blazing");
    for &n in &sizes {
        for (gen, title) in gens.iter() {
            let name: String = title
                .trim_start_matches("--")
                .trim_end_matches(':')
                .trim()
                .replace(' ', "_");
            let count = (4_000_000 / n).max(1);
            let arrays: Vec<Vec<u16>> = (0..count).map(|_| gen(n)).collect();
            let mut work = arrays.clone();
            let k = 4;
            let mut times = vec![Vec::new(); k];
            for rep in 0..9 {
                for j in 0..k {
                    let v = order(rep, j, k);
                    set_old(v == 0);
                    let t = Instant::now();
                    for (w, a) in work.iter_mut().zip(arrays.iter()) {
                        w.copy_from_slice(a);
                        match v {
                            0 | 1 => crate::v2::devour::devour_sort(w),
                            2 => blazing_radix::radix_sort::<u16>(w),
                            _ => {},
                        }
                    }
                    times[v].push(t.elapsed().as_nanos() as f64 / (count * n) as f64);
                    if rep == 0 && v < 3 {
                        let mut r = arrays[0].clone();
                        r.sort_unstable();
                        assert!(work[0] == r, "{} {} {}", v, n, name);
                    }
                }
            }
            set_old(false);
            let med = |x: &Vec<f64>| {
                let mut x = x.clone();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            let c = med(&times[3]);
            println!(
                "{:>9} {:12} {:8.3} {:8.3} {:8.3}",
                n,
                name,
                med(&times[0]) - c,
                med(&times[1]) - c,
                med(&times[2]) - c
            );
        }
    }
}

/// The short preparation against `sort_unstable`: random shapes (few values,
/// straight and wrapped ranges, crossing the middle, random) at 65..1100.
#[test]
#[ignore]
fn tmp_u16_tiny_ok() {
    let mut seed = 0x1234_5678_9abc_def1u64;
    for n in 65..1100 {
        for shape in 0..8u64 {
            let base = rng(&mut seed) as u16;
            let a: Vec<u16> = (0..n)
                .map(|_| {
                    let r = rng(&mut seed);
                    match shape {
                        0 => base.wrapping_add((r % 3) as u16),
                        1 => base.wrapping_add((r % 256) as u16),
                        2 => (r % 256) as u16 ^ 0x8000 ^ 0x80,
                        3 => {
                            if r & 1 == 0 {
                                (r >> 8) as u16 % 100
                            } else {
                                u16::MAX - (r >> 8) as u16 % 100
                            }
                        },
                        4 => 0x7F80 + (r % 256) as u16,
                        5 => base.wrapping_add((r % 300) as u16),
                        6 => (r % 5) as u16 * 1000,
                        _ => r as u16,
                    }
                })
                .collect();
            let mut w = a.clone();
            crate::v2::devour::devour_sort(&mut w);
            let mut r = a.clone();
            r.sort_unstable();
            assert!(w == r, "n {} shape {}", n, shape);
        }
    }
}
