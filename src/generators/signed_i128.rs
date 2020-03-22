use rand::{thread_rng, Rng};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i128(size: usize) -> Vec<i128> {
    (0..size).into_par_iter().map(|_| thread_rng().gen()).collect::<Vec<i128>>()
}

// Small values
pub fn helper_random_array_109_i128(size: usize) -> Vec<i128> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000_000, 1_000_000_000))
        .collect::<Vec<i128>>()
}

pub fn generators_i128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i128>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i128, "-- Unif       :"),
        (&helper_random_array_109_i128, "-- +-10^9     :"),
    ]
}

#[cfg(target_pointer_width = "128")]
pub fn generators_isize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i128()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i128>, Vec<isize>>(arr)
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
