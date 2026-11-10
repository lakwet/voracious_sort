use rand::seq::SliceRandom;
use rand::RngExt;
use rand_distr::{Distribution, Normal, Pareto};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random::<f32>())
        .collect::<Vec<f32>>()
}

// Small
pub fn helper_random_array_small_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-1_000_000.0..1_000_000.0))
        .collect::<Vec<f32>>()
}

// Asc
pub fn helper_random_array_asc_f32(size: usize) -> Vec<f32> {
    (0..size).into_par_iter().map(|i| i as f32 + 0.5).collect::<Vec<f32>>()
}

// Desc
pub fn helper_random_array_desc_f32(size: usize) -> Vec<f32> {
    (0..size).into_par_iter().map(|i| -(i as f32 + 0.5)).collect::<Vec<f32>>()
}

// Equal
pub fn helper_random_array_equal_f32(size: usize) -> Vec<f32> {
    vec![rand::rng().random(); size]
}

fn helper_pareto(size: usize, arg: f32) -> Vec<f32> {
    let pareto = Pareto::new(0.1, arg).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| pareto.sample(&mut rand::rng()))
        .collect::<Vec<f32>>()
}

// Pareto
pub fn helper_random_array_pareto075_f32(size: usize) -> Vec<f32> {
    helper_pareto(size, 0.75)
}

// Pareto
pub fn helper_random_array_pareto100_f32(size: usize) -> Vec<f32> {
    helper_pareto(size, 1.0)
}

// Pareto
pub fn helper_random_array_pareto200_f32(size: usize) -> Vec<f32> {
    helper_pareto(size, 2.0)
}

fn helper_normal(size: usize, standard_deviation: f32) -> Vec<f32> {
    let normal = Normal::new(0.0, standard_deviation).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut rand::rng()))
        .collect::<Vec<f32>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_f32(size: usize) -> Vec<f32> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_f32(size: usize) -> Vec<f32> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_f32(size: usize) -> Vec<f32> {
    helper_normal(size, 1_000_000_000.0)
}

// Sqrt
pub fn helper_random_array_sqrt_f32(size: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<f32> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: f32 = 0.0;
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
pub fn helper_random_array_almost_asc_f32(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f32(size);
    }
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f32(size);
    }
    let mut array = helper_random_array_asc_f32(size);

    for _ in 0..((size as f64).log2() as usize) {
        let i = rand::rng().random_range(0..size);
        let j = rand::rng().random_range(0..size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_f32(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f32(size);
    }

    let mut array = helper_random_array_desc_f32(size);

    for _ in 0..((size as f64).log2() as usize) {
        let i = rand::rng().random_range(0..size);
        let j = rand::rng().random_range(0..size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_f32(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f32;
    (0..size).into_par_iter().map(|i| i as f32 % limit).collect::<Vec<f32>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_f32(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_f32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f32;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as f32 % limit)
        .collect::<Vec<f32>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_f32(size: usize) -> Vec<f32> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as f32 } else { (size - i) as f32 })
        .collect::<Vec<f32>>()
}

// Push Front
pub fn helper_random_array_push_front_f32(size: usize) -> Vec<f32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as f32).collect::<Vec<f32>>();

    if size > 0 {
        array[size - 1] = 0.0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_f32(size: usize) -> Vec<f32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as f32).collect::<Vec<f32>>();

    if size > 0 {
        array[size - 1] = (size / 2) as f32;
    }

    array
}

// Zipf
pub fn helper_random_array_zipf_f32(size: usize) -> Vec<f32> {
    let mut array: Vec<f32> = Vec::with_capacity(size);
    let mut rng = rand::rng();
    let mut quantity = size / 2;
    let mut value: f32 = 0.0;
    while quantity > 2 {
        value = rng.random();
        for _ in 0..quantity {
            array.push(value);
        }
        quantity /= 2;
    }
    while array.len() < size {
        array.push(value);
    }
    array.as_mut_slice().shuffle(&mut rng);
    array
}

// Integer-valued floats in a narrow range (ratings, ages, counters): 0..=100
pub fn helper_random_array_int_narrow_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0..=100u32) as f32)
        .collect::<Vec<f32>>()
}

// Integer-valued floats in a wide range (ids stored as floats): shuffled 0..n
pub fn helper_random_array_int_wide_f32(size: usize) -> Vec<f32> {
    let mut array = (0..size).map(|i| i as f32).collect::<Vec<f32>>();
    array.as_mut_slice().shuffle(&mut rand::rng());
    array
}

// Power-of-two grid: multiples of 0.25 in [-32, 32) (256 points)
pub fn helper_random_array_grid_quarter_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(-128..128i32) as f32 * 0.25)
        .collect::<Vec<f32>>()
}

// Decimal grid (prices in cents): not exactly representable in binary
pub fn helper_random_array_prices_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0..100_000u32) as f32 / 100.0)
        .collect::<Vec<f32>>()
}

// Few distinct values: 16 random values
pub fn helper_random_array_few16_f32(size: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let values: Vec<f32> = (0..16).map(|_| rng.random_range(-1000.0..1000.0)).collect();
    (0..size).map(|_| values[rng.random_range(0..16)]).collect::<Vec<f32>>()
}

// Detector worst case: narrow integers, one wide outlier late (at 95 %)
pub fn helper_random_array_int_late_outlier_f32(size: usize) -> Vec<f32> {
    let mut array = helper_random_array_int_narrow_f32(size);
    if size > 0 {
        array[size * 95 / 100] = 1_000_000.0;
    }
    array
}

// Detector worst case: 16 distinct values, a 17th one late (at 95 %)
pub fn helper_random_array_few16_late_f32(size: usize) -> Vec<f32> {
    let mut array = helper_random_array_few16_f32(size);
    if size > 0 {
        array[size * 95 / 100] = 12_345.5;
    }
    array
}

// Detector worst case: ascending for 90 %, random tail
pub fn helper_random_array_asc_random_tail_f32(size: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let cut = size * 9 / 10;
    (0..size)
        .map(|i| if i < cut { i as f32 } else { rng.random_range(0.0..size as f32) })
        .collect::<Vec<f32>>()
}

// Concatenation of k independently sorted random sources (e.g. sorted logs
// from several machines): k sorted runs with randomly interleaved values.
fn helper_sorted_runs_f32(size: usize, k: usize) -> Vec<f32> {
    let mut array = helper_random_array_uniform_f32(size);
    let k = k.max(1);
    for r in 0..k {
        let (a, b) = (r * size / k, (r + 1) * size / k);
        array[a..b].sort_unstable_by(|x, y| x.partial_cmp(y).unwrap());
    }
    array
}

pub fn helper_random_array_sorted_runs8_f32(size: usize) -> Vec<f32> {
    helper_sorted_runs_f32(size, 8)
}

pub fn helper_random_array_sorted_runs64_f32(size: usize) -> Vec<f32> {
    helper_sorted_runs_f32(size, 64)
}

// Tight clusters (measurements around n / 200 random values, relative noise
// 1e-5, a few ulps of f32), shuffled.
pub fn helper_random_array_clusters_f32(size: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let k = (size / 200).max(1);
    let centers: Vec<f32> = (0..k).map(|_| rng.random::<f32>() * 1000.0).collect();
    let mut array: Vec<f32> =
        (0..size).map(|i| centers[i % k] * (1.0 + rng.random::<f32>() * 1e-5)).collect();
    array.as_mut_slice().shuffle(&mut rng);
    array
}

pub fn generators_f32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<f32>, &'static str)> {
    vec![
        (&helper_random_array_uniform_f32, "-- Unif       :"),
        (&helper_random_array_small_f32, "-- Small      :"),
        (&helper_random_array_asc_f32, "-- Asc        :"),
        (&helper_random_array_desc_f32, "-- Desc       :"),
        (&helper_random_array_equal_f32, "-- Equal      :"),
        (&helper_random_array_almost_asc_f32, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_f32, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_f32, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_f32, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_f32, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_f32, "-- Pipe Organ :"),
        (&helper_random_array_push_front_f32, "-- Front      :"),
        (&helper_random_array_push_middle_f32, "-- Middle     :"),
        (&helper_random_array_pareto075_f32, "-- Pareto 0.75:"),
        (&helper_random_array_pareto100_f32, "-- Pareto 1.00:"),
        (&helper_random_array_pareto200_f32, "-- Pareto 2.00:"),
        (&helper_random_array_normale_10_f32, "-- Normale 10 :"),
        (&helper_random_array_normale_20_f32, "-- Normale 20 :"),
        (&helper_random_array_normale_30_f32, "-- Normale 30 :"),
        (&helper_random_array_zipf_f32, "-- Zipf       :"),
        (&helper_random_array_int_narrow_f32, "-- Int 0..100 :"),
        (&helper_random_array_int_wide_f32, "-- Int 0..n   :"),
        (&helper_random_array_grid_quarter_f32, "-- Grid 0.25  :"),
        (&helper_random_array_prices_f32, "-- Prices     :"),
        (&helper_random_array_few16_f32, "-- Few 16     :"),
        (&helper_random_array_int_late_outlier_f32, "-- Int late out:"),
        (&helper_random_array_few16_late_f32, "-- Few16 late :"),
        (&helper_random_array_asc_random_tail_f32, "-- Asc rnd tail:"),
        (&helper_random_array_sorted_runs8_f32, "-- Runs 8     :"),
        (&helper_random_array_sorted_runs64_f32, "-- Runs 64    :"),
        (&helper_random_array_clusters_f32, "-- Clusters   :"),
    ]
}
