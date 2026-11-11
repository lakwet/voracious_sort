use rand::seq::SliceRandom;
use rand::RngExt;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random::<f64>())
        .collect::<Vec<f64>>()
}

// Small
pub fn helper_random_array_small_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-1_000_000_000.0..1_000_000_000.0))
        .collect::<Vec<f64>>()
}

// Small plus
pub fn helper_random_array_small_plus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0.0..1_000_000_000.0))
        .collect::<Vec<f64>>()
}

// Small minus
pub fn helper_random_array_small_minus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-1_000_000_000.0..0.0))
        .collect::<Vec<f64>>()
}

// Very small
pub fn helper_random_array_very_small_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-1_000_000.0..1_000_000.0))
        .collect::<Vec<f64>>()
}

// Very small plus
pub fn helper_random_array_very_small_plus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0.0..1_000_000.0))
        .collect::<Vec<f64>>()
}

// Very small minus
pub fn helper_random_array_very_small_minus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-1_000_000.0..0.0))
        .collect::<Vec<f64>>()
}

// Asc
pub fn helper_random_array_asc_f64(size: usize) -> Vec<f64> {
    (0..size).into_par_iter().map(|i| i as f64 + 0.5).collect::<Vec<f64>>()
}

// Desc
pub fn helper_random_array_desc_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|i| ((size - i) as f64) + 0.5)
        .collect::<Vec<f64>>()
}

// Equal
pub fn helper_random_array_equal_f64(size: usize) -> Vec<f64> {
    vec![rand::rng().random(); size]
}

// Zipf: frequencies in 1 / rank over min(size, domain) random values
pub fn helper_random_array_zipf_f64(size: usize) -> Vec<f64> {
    super::zipf(size, usize::MAX, || rand::rng().random::<f64>())
}

// Fake Zipf: n / 2 copies of a random value, then n / 4, ... (about
// log2(n) distinct values, geometric frequencies)
pub fn helper_random_array_fakezipf_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    let mut rng = rand::rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: f64 = 0.0;
    while quantity > 2 {
        value = rng.random();

        for _ in 0..quantity {
            array.push(value);
            i += 1;
        }

        quantity = quantity / 2;
    }
    while i < size {
        array.push(value);
        i += 1;
    }

    array.as_mut_slice().shuffle(&mut rng);

    array
}

fn helper_normal(size: usize, range: f64) -> Vec<f64> {
    let normal = Normal::new(0.0, range).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut rand::rng()))
        .collect::<Vec<f64>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 1_000_000_000.0)
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 1_000_000_000_000.0)
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 2_000_000_000_000_000.0)
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_f64(size: usize) -> Vec<f64> {
    helper_normal(size, 4_000_000_000_000_000_000.0)
}

// Sqrt
pub fn helper_random_array_sqrt_f64(size: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<f64> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: f64 = 0.0;
    for _ in 0..sqrt {
        value = rng.random();
        for _ in 0..sqrt {
            array.push(value);
            i += 1;
        }
    }
    while i < size {
        array.push(value);
        i += 1;
    }

    array
}

// Almost sorted ascending
pub fn helper_random_array_almost_asc_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f64(size);
    }

    let mut array = helper_random_array_asc_f64(size);
    let mut rng = rand::rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.random_range(0..size);
        let j = rng.random_range(0..size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f64(size);
    }

    let mut array = helper_random_array_desc_f64(size);
    let mut rng = rand::rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.random_range(0..size);
        let j = rng.random_range(0..size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f64;
    (0..size).into_par_iter().map(|i| i as f64 % limit).collect::<Vec<f64>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f64;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as f64 % limit)
        .collect::<Vec<f64>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_f64(size: usize) -> Vec<f64> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as f64 } else { (size - i) as f64 })
        .collect::<Vec<f64>>()
}

// Push Front
pub fn helper_random_array_push_front_f64(size: usize) -> Vec<f64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as f64).collect::<Vec<f64>>();

    if size > 0 {
        array[size - 1] = 0.0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_f64(size: usize) -> Vec<f64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as f64).collect::<Vec<f64>>();

    if size > 0 {
        array[size - 1] = (size / 2) as f64;
    }

    array
}

// Integer-valued floats in a narrow range (ratings, ages, counters): 0..=100
pub fn helper_random_array_int_narrow_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0..=100u32) as f64)
        .collect::<Vec<f64>>()
}

// Integer-valued floats in a wide range (ids stored as floats): shuffled 0..n
pub fn helper_random_array_int_wide_f64(size: usize) -> Vec<f64> {
    let mut array = (0..size).map(|i| i as f64).collect::<Vec<f64>>();
    array.as_mut_slice().shuffle(&mut rand::rng());
    array
}

// Power-of-two grid: multiples of 0.25 in [-32, 32) (256 points)
pub fn helper_random_array_grid_quarter_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-128..128i32) as f64 * 0.25)
        .collect::<Vec<f64>>()
}

// Decimal grid (prices in cents): not exactly representable in binary
pub fn helper_random_array_prices_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0..100_000u32) as f64 / 100.0)
        .collect::<Vec<f64>>()
}

// Few distinct values: 16 random values
pub fn helper_random_array_few16_f64(size: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let values: Vec<f64> = (0..16).map(|_| rng.random_range(-1000.0..1000.0)).collect();
    (0..size).map(|_| values[rng.random_range(0..16)]).collect::<Vec<f64>>()
}

// Sparse data: 90 % zeros, 10 % random values in [-1000, 1000)
pub fn helper_random_array_sparse90_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::rng();
            if rng.random_range(0..10) == 0 { rng.random_range(-1000.0..1000.0) } else { 0.0 }
        })
        .collect()
}

// Heavy hitters: 60 % of the keys among 4 values, the rest random
pub fn helper_random_array_heavy4_f64(size: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let values: Vec<f64> = (0..4).map(|_| rng.random_range(-1000.0..1000.0)).collect();
    (0..size)
        .map(|_| if rng.random_range(0..10) < 6 { values[rng.random_range(0..4)] } else { rng.random_range(-1000.0..1000.0) })
        .collect()
}

// Detector worst case: narrow integers, one wide outlier late (at 95 %)
pub fn helper_random_array_int_late_outlier_f64(size: usize) -> Vec<f64> {
    let mut array = helper_random_array_int_narrow_f64(size);
    if size > 0 {
        array[size * 95 / 100] = 1_000_000.0;
    }
    array
}

// Detector worst case: 16 distinct values, a 17th one late (at 95 %)
pub fn helper_random_array_few16_late_f64(size: usize) -> Vec<f64> {
    let mut array = helper_random_array_few16_f64(size);
    if size > 0 {
        array[size * 95 / 100] = 12_345.5;
    }
    array
}

// Detector worst case: ascending for 90 %, random tail
pub fn helper_random_array_asc_random_tail_f64(size: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let cut = size * 9 / 10;
    (0..size)
        .map(|i| if i < cut { i as f64 } else { rng.random_range(0.0..size as f64) })
        .collect::<Vec<f64>>()
}

// Concatenation of k independently sorted random sources (e.g. sorted logs
// from several machines): k sorted runs with randomly interleaved values.
fn helper_sorted_runs_f64(size: usize, k: usize) -> Vec<f64> {
    let mut array = helper_random_array_uniform_f64(size);
    let k = k.max(1);
    for r in 0..k {
        let (a, b) = (r * size / k, (r + 1) * size / k);
        array[a..b].sort_unstable_by(|x, y| x.partial_cmp(y).unwrap());
    }
    array
}

pub fn helper_random_array_sorted_runs8_f64(size: usize) -> Vec<f64> {
    helper_sorted_runs_f64(size, 8)
}

pub fn helper_random_array_sorted_runs64_f64(size: usize) -> Vec<f64> {
    helper_sorted_runs_f64(size, 64)
}

// Tight clusters (measurements around n / 200 random values, relative noise
// 1e-9), shuffled: the top bits name the cluster, its keys differ far lower.
pub fn helper_random_array_clusters_f64(size: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let k = (size / 200).max(1);
    let centers: Vec<f64> = (0..k).map(|_| rng.random::<f64>() * 1000.0).collect();
    let mut array: Vec<f64> =
        (0..size).map(|i| centers[i % k] * (1.0 + rng.random::<f64>() * 1e-9)).collect();
    array.as_mut_slice().shuffle(&mut rng);
    array
}

pub fn generators_f64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<f64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_f64, "-- Unif       :"),
        (&helper_random_array_small_f64, "-- Small 10^9 :"),
        (&helper_random_array_small_plus_f64, "-- Small+10^9 :"),
        (&helper_random_array_small_minus_f64, "-- Small-10^9 :"),
        (&helper_random_array_very_small_f64, "-- Small 10^6 :"),
        (&helper_random_array_very_small_plus_f64, "-- Small+10^6 :"),
        (&helper_random_array_very_small_minus_f64, "-- Small-10^6 :"),
        (&helper_random_array_asc_f64, "-- Asc        :"),
        (&helper_random_array_desc_f64, "-- Desc       :"),
        (&helper_random_array_equal_f64, "-- Equal      :"),
        (&helper_random_array_zipf_f64, "-- Zipf       :"),
        (&helper_random_array_fakezipf_f64, "-- FakeZipf   :"),
        (&helper_random_array_almost_asc_f64, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_f64, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_f64, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_f64, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_f64, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_f64, "-- Pipe Organ :"),
        (&helper_random_array_push_front_f64, "-- Front      :"),
        (&helper_random_array_push_middle_f64, "-- Middle     :"),
        (&helper_random_array_normale_10_f64, "-- Normale 10 :"),
        (&helper_random_array_normale_20_f64, "-- Normale 20 :"),
        (&helper_random_array_normale_30_f64, "-- Normale 30 :"),
        (&helper_random_array_normale_40_f64, "-- Normale 40 :"),
        (&helper_random_array_normale_51_f64, "-- Normale 51 :"),
        (&helper_random_array_normale_63_f64, "-- Normale 63 :"),
        (&helper_random_array_int_narrow_f64, "-- Int 0..100 :"),
        (&helper_random_array_int_wide_f64, "-- Int 0..n   :"),
        (&helper_random_array_grid_quarter_f64, "-- Grid 0.25  :"),
        (&helper_random_array_prices_f64, "-- Prices     :"),
        (&helper_random_array_few16_f64, "-- Few 16     :"),
        (&helper_random_array_sparse90_f64, "-- Sparse 90% :"),
        (&helper_random_array_heavy4_f64, "-- Heavy 4    :"),
        (&helper_random_array_int_late_outlier_f64, "-- Int late out:"),
        (&helper_random_array_few16_late_f64, "-- Few16 late :"),
        (&helper_random_array_asc_random_tail_f64, "-- Asc rnd tail:"),
        (&helper_random_array_sorted_runs8_f64, "-- Runs 8     :"),
        (&helper_random_array_sorted_runs64_f64, "-- Runs 64    :"),
        (&helper_random_array_clusters_f64, "-- Clusters   :"),
    ]
}
