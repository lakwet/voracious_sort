use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<i16>())
        .collect::<Vec<i16>>()
}

// Small
pub fn helper_random_array_small_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-128, 127))
        .collect::<Vec<i16>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i16(size: usize) -> Vec<i16> {
    (0..size).into_par_iter().map(|i| i as i16).collect::<Vec<i16>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as i16)
        .collect::<Vec<i16>>()
}

// All equals
pub fn helper_random_array_allequals_i16(size: usize) -> Vec<i16> {
    vec![thread_rng().gen(); size]
}

// Zipf
pub fn helper_random_array_zipf_i16(size: usize) -> Vec<i16> {
    let mut array: Vec<i16> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i16 = 0;
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

pub fn generators_i16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i16>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i16, "-- Unif       :"),
        (&helper_random_array_small_i16, "-- Small      :"),
        (&helper_random_array_asc_sawtooth_i16, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i16, "-- Desc Saw   :"),
        (&helper_random_array_allequals_i16, "-- Equal      :"),
        (&helper_random_array_zipf_i16, "-- Zipf       :"),
    ]
}

#[cfg(target_pointer_width = "16")]
pub fn generators_isize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i16()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i16>, Vec<isize>>(arr)
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
