use rand::seq::SliceRandom;
use rand::RngExt;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i8(size: usize) -> Vec<i8> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random::<i8>())
        .collect::<Vec<i8>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i8(size: usize) -> Vec<i8> {
    (0..size).into_par_iter().map(|i| i as i8).collect::<Vec<i8>>()
}

// Ascending sawtooth
pub fn helper_random_array_desc_sawtooth_i8(size: usize) -> Vec<i8> {
    (0..size).into_par_iter().map(|i| (size - 1 - i) as i8).collect::<Vec<i8>>()
}

// All equals
pub fn helper_random_array_allequals_i8(size: usize) -> Vec<i8> {
    vec![rand::rng().random(); size]
}

// Zipf: frequencies in 1 / rank over min(size, domain) random values
pub fn helper_random_array_zipf_i8(size: usize) -> Vec<i8> {
    super::zipf(size, 1 << 8, || rand::rng().random::<i8>())
}

// Fake Zipf: n / 2 copies of a random value, then n / 4, ... (about
// log2(n) distinct values, geometric frequencies)
pub fn helper_random_array_fakezipf_i8(size: usize) -> Vec<i8> {
    let mut array: Vec<i8> = Vec::with_capacity(size);
    let mut rng = rand::rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i8 = 0;
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

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i8(size: usize) -> Vec<i8> {
    let normal = Normal::new(0.0, 1024.0).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut rand::rng()) as i8)
        .collect::<Vec<i8>>()
}

pub fn generators_i8() -> Vec<(&'static dyn Fn(usize) -> Vec<i8>, &'static str)>
{
    vec![
        (&helper_random_array_uniform_i8, "-- Unif       :"),
        (&helper_random_array_asc_sawtooth_i8, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i8, "-- Desc Saw   :"),
        (&helper_random_array_allequals_i8, "-- Equal      :"),
        (&helper_random_array_zipf_i8, "-- Zipf       :"),
        (&helper_random_array_fakezipf_i8, "-- FakeZipf   :"),
        (&helper_random_array_normale_10_i8, "-- Normale 10 :"),
    ]
}

#[cfg(target_pointer_width = "8")]
pub fn generators_isize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i8()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i8>, Vec<isize>>(arr)
                }
            };

            (
                Box::leak(Box::new(new_gen))
                    as &'static dyn Fn(usize) -> Vec<isize>,
                title,
            )
        })
        .collect()
}
