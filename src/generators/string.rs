use rand::distr::Alphanumeric;
use rand::RngExt;
use rayon::prelude::*;

pub fn helper_random_array_uniform_string(
    size: usize,
    string_size: usize,
) -> Vec<String> {
    (0..size).into_par_iter().map(|_|
        rand::rng().sample_iter(&Alphanumeric)
            .take(string_size)
            .map(char::from)
            .collect::<String>()
    ).collect::<Vec<String>>()
}

pub fn generators_string(
) -> Vec<(&'static dyn Fn(usize, usize) -> Vec<String>, &'static str)> {
    vec![(&helper_random_array_uniform_string, "-- Unif       :")]
}
