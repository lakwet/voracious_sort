use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

pub fn helper_random_array_uniform_string(
    size: usize,
    string_size: usize,
) -> Vec<String> {
    (0..size).into_par_iter().map(|_|
        thread_rng().sample_iter(&Alphanumeric)
            .take(string_size)
            .collect::<String>()
    ).collect::<Vec<u64>>()
}

pub fn generators_string(
) -> Vec<(&'static dyn Fn(usize, usize) -> Vec<String>, &'static str)> {
    vec![(&helper_random_array_uniform_string, "-- Unif       :")]
}
