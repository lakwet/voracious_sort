use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal, Pareto};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<f32>())
        .collect::<Vec<f32>>()
}

// Small
pub fn helper_random_array_small_f32(size: usize) -> Vec<f32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000.0, 1_000_000.0))
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
    vec![thread_rng().gen(); size]
}

fn helper_pareto(size: usize, arg: f32) -> Vec<f32> {
    let pareto = Pareto::new(0.1, arg).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| pareto.sample(&mut thread_rng()))
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
        .map(|_| normal.sample(&mut thread_rng()))
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
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<f32> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: f32 = 0.0;
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
        let i = thread_rng().gen_range(0, size);
        let j = thread_rng().gen_range(0, size);
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
        let i = thread_rng().gen_range(0, size);
        let j = thread_rng().gen_range(0, size);
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
    ]
}
