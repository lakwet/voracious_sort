//! Fuzzy testing (`make test-fuzzy`, ignored by `cargo test`): random arrays
//! built by mixing random choices (type, size, value range, duplicates,
//! sorted runs, reversed parts, special values), sorted by the public API
//! and compared with the standard library sort, for `FUZZ_SECONDS`
//! seconds (default 60). Each case has its own seed: a failure prints it,
//! `FUZZ_SEED=<seed>` replays that case alone.

use voracious_radix_sort::RadixSort;

/// splitmix64: one seed gives the whole case.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 { self.next() % n.max(1) }
}

/// Raw 64-bit values of one random shape.
fn raw(r: &mut Rng) -> Vec<u64> {
    // Sizes: mostly small (where the thresholds are), sometimes large.
    let n = match r.below(10) {
        0..=5 => r.below(300),
        6..=8 => r.below(20_000),
        _ => r.below(2_000_000),
    } as usize;
    let bits = 1 + r.below(64) as u32;
    let mask = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
    let limit = if r.below(2) == 0 { 16 } else { 1 << 20 };
    let distinct = 1 + r.below(limit);
    let pool: Vec<u64> = (0..distinct.min(n as u64 + 1)).map(|_| r.next() & mask).collect();
    let from_pool = r.below(3) == 0;
    let mut v: Vec<u64> = (0..n)
        .map(
            |_| if from_pool { pool[r.below(pool.len() as u64) as usize] } else { r.next() & mask },
        )
        .collect();
    // Sorted runs, reversed parts, a few swaps, a constant block.
    for _ in 0..r.below(4) {
        if n < 2 {
            break;
        }
        let a = r.below(n as u64) as usize;
        let b = a + r.below((n - a) as u64) as usize;
        match r.below(4) {
            0 => v[a..b].sort_unstable(),
            1 => {
                v[a..b].sort_unstable();
                v[a..b].reverse()
            },
            2 => {
                for _ in 0..r.below(8) {
                    let i = r.below(n as u64) as usize;
                    let j = r.below(n as u64) as usize;
                    v.swap(i, j);
                }
            },
            _ => {
                let x = v[a];
                v[a..b].iter_mut().for_each(|y| *y = x);
            },
        }
    }
    v
}

#[allow(unused_assignments)] // the type counter of `ints!`
fn check_case(seed: u64) {
    let mut r = Rng(seed);
    let v = raw(&mut r);
    let fail = |ty: &str| {
        format!("type {} n={} seed={} (FUZZ_SEED={} to replay)", ty, v.len(), seed, seed)
    };
    macro_rules! ints {
        ($($t:ty),*) => {{
            let k = r.below(12);
            let mut i = 0;
            $(
                if k == i {
                    let mut a: Vec<$t> = v.iter().map(|&x| x as $t).collect();
                    let mut e = a.clone();
                    e.sort_unstable();
                    a.voracious_sort();
                    assert!(a == e, "{}", fail(stringify!($t)));
                }
                i += 1;
            )*
            let _ = i;
        }};
    }
    match r.below(5) {
        0 | 1 => ints!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize),
        2 => {
            // f64 from raw bits: every class (NaNs, infinities, subnormals, -0.0).
            let mut a: Vec<f64> = v.iter().map(|&x| f64::from_bits(x)).collect();
            let mut e = a.clone();
            e.sort_unstable_by(|x, y| x.total_cmp(y));
            a.voracious_sort();
            assert!(
                a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())),
                "{}",
                fail("f64")
            );
        },
        3 => {
            let mut a: Vec<f32> = v.iter().map(|&x| f32::from_bits(x as u32)).collect();
            let mut e = a.clone();
            e.sort_unstable_by(|x, y| x.total_cmp(y));
            a.voracious_sort();
            assert!(
                a.iter().map(|x| x.to_bits()).eq(e.iter().map(|x| x.to_bits())),
                "{}",
                fail("f32")
            );
        },
        _ => {
            // Records: (key, original index), stable and unstable.
            let mut a: Vec<(u64, u32)> =
                v.iter().enumerate().map(|(i, &x)| (x, i as u32)).collect();
            let mut e = a.clone();
            e.sort_by_key(|x| x.0);
            let mut u = a.clone();
            a.voracious_sort_by_key(|x| x.0);
            assert!(a == e, "{}", fail("records stable"));
            u.voracious_sort_unstable_by_key(|x| x.0);
            u.sort_by_key(|x| (x.0, x.1));
            e.sort_by_key(|x| (x.0, x.1));
            assert!(u == e, "{}", fail("records unstable"));
        },
    }
}

#[test]
#[ignore]
fn fuzzy() {
    if let Some(seed) = std::env::var("FUZZ_SEED").ok().and_then(|s| s.parse().ok()) {
        check_case(seed);
        return;
    }
    let seconds: u64 =
        std::env::var("FUZZ_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);
    let start = std::time::Instant::now();
    let mut seeds =
        Rng(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            as u64);
    let mut cases = 0u64;
    while start.elapsed().as_secs() < seconds {
        check_case(seeds.next());
        cases += 1;
    }
    println!("fuzzy: {} cases in {} s, no failure", cases, seconds);
}
