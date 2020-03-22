use rand::{thread_rng, Rng};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u128(size: usize) -> Vec<u128> {
    (0..size).into_par_iter().map(|_| thread_rng().gen()).collect::<Vec<u128>>()
}

// Uniform 10^9
pub fn helper_random_array_uniform_10_9_u128(size: usize) -> Vec<u128> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, 1_000_000_000))
        .collect::<Vec<u128>>()
}

pub fn generators_u128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<u128>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u128, "-- Unif       :"),
        (&helper_random_array_uniform_10_9_u128, "-- Unif 10^9  :"),
    ]
}

#[cfg(target_pointer_width = "128")]
pub fn generators_usize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u128()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u128>, Vec<usize>>(arr)
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
