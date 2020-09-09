use rand::{thread_rng, Rng};
use rayon::prelude::*;

pub fn helper_random_array_bool_unif(size: usize) -> Vec<bool> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let value: u8 = thread_rng().gen_range(0, 2);
            value != 0
        })
        .collect::<Vec<bool>>()
}

pub fn helper_random_array_bool_alt(size: usize) -> Vec<bool> {
    (0..size)
        .into_par_iter()
        .enumerate()
        .map(|(i, _)| i % 2 == 0)
        .collect::<Vec<bool>>()
}

pub fn helper_random_array_bool_true(size: usize) -> Vec<bool> {
    vec![true; size]
}

pub fn helper_random_array_bool_false(size: usize) -> Vec<bool> {
    vec![false; size]
}

pub fn helper_random_array_bool_pipe(size: usize) -> Vec<bool> {
    let mut trues = vec![false; size / 2];
    let falses = vec![true; size - (size / 2)];

    trues.extend(falses.iter());
    trues
}

pub fn helper_random_array_bool_pipe_rev(size: usize) -> Vec<bool> {
    let trues = vec![false; size / 2];
    let mut falses = vec![true; size - (size / 2)];

    falses.extend(trues.iter());
    falses
}

pub fn generators_bool(
) -> Vec<(&'static dyn Fn(usize) -> Vec<bool>, &'static str)> {
    vec![
        (&helper_random_array_bool_unif, "-- Unif       :"),
        (&helper_random_array_bool_alt, "-- Alt        :"),
        (&helper_random_array_bool_true, "-- True only  :"),
        (&helper_random_array_bool_false, "-- False only :"),
        (&helper_random_array_bool_pipe, "-- Pipe Organ :"),
        (&helper_random_array_bool_pipe_rev, "-- Pipe Rev   :"),
    ]
}
