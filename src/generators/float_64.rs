use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<f64>())
        .collect::<Vec<f64>>()
}

// Small
pub fn helper_random_array_small_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000_000.0, 1_000_000_000.0))
        .collect::<Vec<f64>>()
}

// Small plus
pub fn helper_random_array_small_plus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0.0, 1_000_000_000.0))
        .collect::<Vec<f64>>()
}

// Small minus
pub fn helper_random_array_small_minus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000_000.0, 0.0))
        .collect::<Vec<f64>>()
}

// Very small
pub fn helper_random_array_very_small_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000.0, 1_000_000.0))
        .collect::<Vec<f64>>()
}

// Very small plus
pub fn helper_random_array_very_small_plus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0.0, 1_000_000.0))
        .collect::<Vec<f64>>()
}

// Very small minus
pub fn helper_random_array_very_small_minus_f64(size: usize) -> Vec<f64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000.0, 0.0))
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
    vec![thread_rng().gen(); size]
}

// Zipf
pub fn helper_random_array_zipf_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: f64 = 0.0;
    while quantity > 2 {
        value = rng.gen();

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
        .map(|_| normal.sample(&mut thread_rng()))
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
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<f64> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: f64 = 0.0;
    for _ in 0..sqrt {
        value = rng.gen();
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
    let mut rng = thread_rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
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
    let mut rng = thread_rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
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
    ]
}
