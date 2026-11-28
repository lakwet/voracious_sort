//! Temporary: NaN in float sorts.
use super::super::RadixSort;

fn data(n: usize, every: usize) -> Vec<f64> {
    let mut s = 0x1234_5678u64;
    (0..n).map(|i| {
        s ^= s << 13; s ^= s >> 7; s ^= s << 17;
        if i % every == 0 { if (i / every) % 2 == 0 { f64::NAN } else { -f64::NAN } } else { (s >> 11) as f64 / 1e6 - 4e9 }
    }).collect()
}

#[test]
fn tmp_nan() {
    for &n in &[100usize, 1_000, 10_000, 100_000, 1_000_000] {
        for &every in &[2usize, 7, 1000] {
            for which in ["v1 f64", "v1 f32", "v2 f64", "v2 f32", "v1 f64 stable", "v1 f64 mt"] {
                let r = std::panic::catch_unwind(|| {
                    match which {
                        "v1 f64" => { let mut a = data(n, every); a.voracious_sort(); check(&a.iter().map(|&x| x).collect::<Vec<f64>>()) }
                        "v1 f32" => { let mut a: Vec<f32> = data(n, every).iter().map(|&x| x as f32).collect(); a.voracious_sort(); check(&a.iter().map(|&x| x as f64).collect::<Vec<f64>>()) }
                        "v1 f64 stable" => { let mut a = data(n, every); a.voracious_stable_sort(); check(&a) }
                        "v1 f64 mt" => { let mut a = data(n, every); a.voracious_mt_sort(4); check(&a) }
                        "v2 f64" => { let mut a = data(n, every); super::super::v2::ravenous::ravenous_sort2(&mut a); check(&a) }
                        _ => { let mut a: Vec<f32> = data(n, every).iter().map(|&x| x as f32).collect(); super::super::v2::ravenous::ravenous_sort2(&mut a); check(&a.iter().map(|&x| x as f64).collect::<Vec<f64>>()) }
                    }
                });
                match r {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => println!("{} n {} every {}: WRONG {}", which, n, every, e),
                    Err(_) => println!("{} n {} every {}: PANIC", which, n, every),
                }
            }
        }
    }
}

/// Sorted by total order on the non-NaN values, every NaN at one end (both
/// signs allowed at their total-order ends), none lost.
fn check(a: &[f64]) -> Result<(), String> {
    let ok = a.windows(2).all(|w| w[0].total_cmp(&w[1]) != std::cmp::Ordering::Greater);
    if ok { Ok(()) } else {
        let nn: Vec<f64> = a.iter().cloned().filter(|x| !x.is_nan()).collect();
        let sorted_nn = nn.windows(2).all(|w| w[0] <= w[1]);
        Err(format!("total order broken (non-NaN sorted: {}, first NaN at {:?})", sorted_nn, a.iter().position(|x| x.is_nan())))
    }
}

/// The float part of `test_sort_trait_voracious(_stable)`, repeated (the
/// generators are random): the NaN set-aside must not change the results.
#[test]
fn tmp_nan_floats_trait() {
    use super::super::generators::custom::*;
    use super::super::generators::float_32::*;
    use super::super::generators::float_64::*;
    use super::sorts::helper_sort;
    for _ in 0..3 {
        for size in [0usize, 1, 200, 400, 500, 1_500, 3_000, 20_000, 40_000, 90_000, 200_000, 600_000, 3_100_000].iter() {
            helper_sort(false, &|a| a.voracious_stable_sort(), generators_f32(), *size);
            helper_sort(false, &|a| a.voracious_stable_sort(), generators_f64(), *size);
            helper_sort(false, &|a| a.voracious_stable_sort(), generators_structf32(), *size);
            helper_sort(false, &|a| a.voracious_stable_sort(), generators_structf64(), *size);
            helper_sort(false, &|a| a.voracious_sort(), generators_f32(), *size);
            helper_sort(false, &|a| a.voracious_sort(), generators_f64(), *size);
            helper_sort(false, &|a| a.voracious_sort(), generators_structf32(), *size);
            helper_sort(false, &|a| a.voracious_sort(), generators_structf64(), *size);
            helper_sort(false, &|a| a.voracious_mt_sort(4), generators_f64(), *size);
            helper_sort(false, &|a| a.voracious_mt_sort(4), generators_structf32(), *size);
        }
    }
}

#[test]
fn tmp_nan_find() {
    use super::super::generators::float_64::*;
    for size in [200usize, 400, 500, 1_500, 3_000, 20_000, 40_000, 90_000, 200_000, 600_000].iter() {
        for (g, name) in generators_f64().iter() {
            for which in ["stable", "unstable", "mt"] {
                let a = g(*size);
                let mut v = a.clone();
                match which { "stable" => v.voracious_stable_sort(), "unstable" => v.voracious_sort(), _ => v.voracious_mt_sort(4) }
                let mut w = a.clone();
                w.sort_by(|x, y| x.partial_cmp(y).unwrap());
                if v != w {
                    let i = (0..v.len()).find(|&i| v[i] != w[i]).unwrap();
                    println!("MISMATCH {} {} {} at {}: got {} want {} (sorted input: {})", which, name, size, i, v[i], w[i], a.windows(2).all(|p| p[0] <= p[1]));
                }
            }
        }
    }
}

#[test]
fn tmp_nan_ts() {
    use super::super::generators::float_64::*;
    let gens = generators_f64();
    let (g, _) = gens.iter().find(|(_, n)| n.contains("Timestamps")).unwrap();
    let (mut bad, mut panics, mut runs) = (0, 0, 0);
    for size in [20_000usize, 40_000, 90_000, 200_000].iter() {
        for _ in 0..10 {
            let a = g(*size);
            let r = std::panic::catch_unwind(|| { let mut v = a.clone(); v.voracious_stable_sort(); v });
            runs += 1;
            match r {
                Ok(v) => { let mut w = a.clone(); w.sort_by(|x, y| x.partial_cmp(y).unwrap()); if v != w { bad += 1; } }
                Err(_) => panics += 1,
            }
        }
    }
    println!("TS stable f64: runs {} wrong {} panics {}", runs, bad, panics);
}
