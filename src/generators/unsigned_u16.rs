use rand::RngExt;
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u16(size: usize) -> Vec<u16> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random::<u16>())
        .collect::<Vec<u16>>()
}

// Small
pub fn helper_random_array_small_u16(size: usize) -> Vec<u16> {
    (0..size)
        .into_par_iter()
        .map(|_| rand::rng().random_range(0..255))
        .collect::<Vec<u16>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_u16(size: usize) -> Vec<u16> {
    (0..size).into_par_iter().map(|i| i as u16).collect::<Vec<u16>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u16(size: usize) -> Vec<u16> {
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as u16)
        .collect::<Vec<u16>>()
}

// All equals
pub fn helper_random_array_allequals_u16(size: usize) -> Vec<u16> {
    vec![rand::rng().random(); size]
}

// Zipf: frequencies in 1 / rank over min(size, domain) random values
pub fn helper_random_array_zipf_u16(size: usize) -> Vec<u16> {
    super::zipf(size, 1 << 16, || rand::rng().random::<u16>())
}

pub fn generators_u16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<u16>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u16, "-- Unif       :"),
        (&helper_random_array_small_u16, "-- Small      :"),
        (&helper_random_array_asc_sawtooth_u16, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_u16, "-- Desc Saw   :"),
        (&helper_random_array_allequals_u16, "-- Equal      :"),
        (&helper_random_array_zipf_u16, "-- Zipf       :"),
    ]
}

#[cfg(target_pointer_width = "16")]
pub fn generators_usize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u16()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u16>, Vec<usize>>(arr)
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
