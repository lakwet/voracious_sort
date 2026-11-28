use crate::RadixSort;
use rand::RngExt;

#[test]
fn tmp_v1_min_max() {
    let mut rng = rand::rng();
    let mut bad = 0;
    for &n in [129usize, 1000, 4096, 10_000, 100_000, 1_000_000].iter() {
        for &low in [1usize, 50, 99].iter() {
            for _ in 0..20 {
                let a: Vec<i64> = (0..n).map(|_| { let d = rng.random_range(0..300) as i64; if rng.random_range(0..100) < low { i64::MIN.wrapping_add(d) } else { i64::MAX.wrapping_sub(d) } }).collect();
                let mut x = a.clone(); let mut e = a.clone(); e.sort_unstable(); x.voracious_sort();
                if x != e { bad += 1; println!("i64 n {} low {}", n, low); }
                let mut u: Vec<u64> = a.iter().map(|&v| v as u64).collect(); let mut ue = u.clone(); ue.sort_unstable(); u.voracious_sort();
                if u != ue { bad += 1; println!("u64 n {} low {}", n, low); }
                let mut w = a.clone(); let mut we = a.clone(); we.sort_unstable(); w.voracious_mt_sort(4);
                if w != we { bad += 1; println!("mt i64 n {} low {}", n, low); }
            }
        }
    }
    println!("v1 min/max bad {}", bad);
}

#[test]
fn tmp_v1_nan() {
    for &n in [10usize, 100, 1000, 100_000].iter() {
        let r = std::panic::catch_unwind(|| {
            let mut v: Vec<f64> = (0..n).map(|i| if i % 7 == 0 { f64::NAN } else { (i * 37 % 101) as f64 }).collect();
            v.voracious_sort();
        });
        println!("v1 f64 NaN n {} {}", n, if r.is_ok() { "ok" } else { "PANIC" });
    }
}
