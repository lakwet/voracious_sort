//! Temporary: v1 voracious_sort timing (before/after the key-order fix).
use super::super::RadixSort;

fn rnd(n: usize, seed: u64) -> Vec<u64> {
    let mut s = seed | 1;
    (0..n).map(|_| { s ^= s << 13; s ^= s >> 7; s ^= s << 17; s }).collect()
}

fn time<T: Copy>(a: &[T], f: &dyn Fn(&mut Vec<T>)) -> f64 {
    let n = a.len();
    let reps = (2_000_000 / n).max(3);
    let mut best = f64::MAX;
    for _ in 0..5 {
        let mut total = 0.0;
        for _ in 0..reps {
            let mut v = a.to_vec();
            let s = std::time::Instant::now();
            f(&mut v);
            total += s.elapsed().as_nanos() as f64;
            std::hint::black_box(&v);
        }
        best = best.min(total / (reps * n) as f64);
    }
    best
}

#[test]
fn tmp_v1perf() {
    let mut out = String::new();
    for &n in &[1_000usize, 10_000, 100_000, 1_000_000] {
        let r = rnd(n, 7);
        let f64s: Vec<f64> = r.iter().map(|&x| (x >> 11) as f64 / 1e3 - 4e12).collect();
        let f32s: Vec<f32> = r.iter().map(|&x| ((x >> 40) as f32) - 8e6).collect();
        let i64s: Vec<i64> = r.iter().map(|&x| x as i64).collect();
        let u32s: Vec<u32> = r.iter().map(|&x| x as u32).collect();
        let mut asc: Vec<f64> = f64s.clone(); asc.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut almost = asc.clone(); for i in (0..n).step_by(97) { almost.swap(i, (i * 31 + 7) % n); }
        let line = format!("n {:>8} | f64 {:.2} f32 {:.2} u64 {:.2} i64 {:.2} u32 {:.2} | f64 asc {:.2} almost {:.2} | stable f64 {:.2} i64 {:.2}",
            n,
            time(&f64s, &|v| v.voracious_sort()),
            time(&f32s, &|v| v.voracious_sort()),
            time(&r, &|v| v.voracious_sort()),
            time(&i64s, &|v| v.voracious_sort()),
            time(&u32s, &|v| v.voracious_sort()),
            time(&asc, &|v| v.voracious_sort()),
            time(&almost, &|v| v.voracious_sort()),
            time(&f64s, &|v| v.voracious_stable_sort()),
            time(&i64s, &|v| v.voracious_stable_sort()));
        println!("{}", line);
        out += &line; out += "\n";
    }
}
