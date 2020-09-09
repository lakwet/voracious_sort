use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u8(size: usize) -> Vec<u8> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<u8>())
        .collect::<Vec<u8>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_u8(size: usize) -> Vec<u8> {
    (0..size).into_par_iter().map(|i| i as u8).collect::<Vec<u8>>()
}

// Ascending sawtooth
pub fn helper_random_array_desc_sawtooth_u8(size: usize) -> Vec<u8> {
    (0..size).into_par_iter().map(|i| (size - 1 - i) as u8).collect::<Vec<u8>>()
}

// All equals
pub fn helper_random_array_allequals_u8(size: usize) -> Vec<u8> {
    vec![thread_rng().gen(); size]
}

// Zipf
pub fn helper_random_array_zipf_u8(size: usize) -> Vec<u8> {
    let mut array: Vec<u8> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: u8 = 0;
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

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_u8(size: usize) -> Vec<u8> {
    let normal = Normal::new(0.0, 1024.0).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut thread_rng()) as u8)
        .collect::<Vec<u8>>()
}

pub fn generators_u8() -> Vec<(&'static dyn Fn(usize) -> Vec<u8>, &'static str)>
{
    vec![
        (&helper_random_array_uniform_u8, "-- Unif       :"),
        (&helper_random_array_asc_sawtooth_u8, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_u8, "-- Desc Saw   :"),
        (&helper_random_array_allequals_u8, "-- Equal      :"),
        (&helper_random_array_zipf_u8, "-- Zipf       :"),
        (&helper_random_array_normale_10_u8, "-- Normale 10 :"),
    ]
}

#[cfg(target_pointer_width = "8")]
pub fn generators_usize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u8()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u8>, Vec<usize>>(arr)
                }
            };

            (
                Box::leak(Box::new(new_gen))
                    as &'static dyn Fn(usize) -> Vec<usize>,
                title,
            )
        })
        .collect()
}
