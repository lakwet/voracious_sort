use rand::RngExt;
use rayon::prelude::*;

pub fn helper_random_array_bool_unif(size: usize) -> Vec<bool> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let value: u8 = rand::rng().random_range(0..2);
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

// Biased: 1 % of true (flags rarely set)
pub fn helper_random_array_bool_1pct(size: usize) -> Vec<bool> {
    (0..size).into_par_iter().map(|_| rand::rng().random_range(0..100) == 0).collect()
}

// Biased: 99 % of true
pub fn helper_random_array_bool_99pct(size: usize) -> Vec<bool> {
    (0..size).into_par_iter().map(|_| rand::rng().random_range(0..100) != 0).collect()
}

// Sparse: 16 true at random positions
pub fn helper_random_array_bool_sparse(size: usize) -> Vec<bool> {
    let mut array = vec![false; size];
    let mut rng = rand::rng();
    for _ in 0..16.min(size) {
        array[rng.random_range(0..size)] = true;
    }
    array
}

// Sorted (false then true), then 1 % of the positions swapped
pub fn helper_random_array_bool_near_sorted(size: usize) -> Vec<bool> {
    let mut array = helper_random_array_bool_pipe(size);
    let mut rng = rand::rng();
    for _ in 0..size / 100 {
        let (i, j) = (rng.random_range(0..size), rng.random_range(0..size));
        array.swap(i, j);
    }
    array
}

// Sorted for 90 %, random tail
pub fn helper_random_array_bool_random_tail(size: usize) -> Vec<bool> {
    let mut array = helper_random_array_bool_unif(size);
    let head = size * 9 / 10;
    let trues = array[..head].iter().filter(|&&b| b).count();
    array[..head - trues].fill(false);
    array[head - trues..head].fill(true);
    array
}

// Runs: blocks of 1 to 1000 equal values
pub fn helper_random_array_bool_runs(size: usize) -> Vec<bool> {
    let mut array = Vec::with_capacity(size);
    let mut rng = rand::rng();
    let mut value = false;
    while array.len() < size {
        let len = rng.random_range(1..=1000).min(size - array.len());
        array.extend(std::iter::repeat(value).take(len));
        value = !value;
    }
    array
}

// Pipe organ: false, then true, then false
pub fn helper_random_array_bool_organ(size: usize) -> Vec<bool> {
    (0..size).map(|i| i >= size / 3 && i < 2 * size / 3).collect()
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
        (&helper_random_array_bool_1pct, "-- 1 % true   :"),
        (&helper_random_array_bool_99pct, "-- 99 % true  :"),
        (&helper_random_array_bool_sparse, "-- Sparse 16  :"),
        (&helper_random_array_bool_near_sorted, "-- Near sorted:"),
        (&helper_random_array_bool_random_tail, "-- Rnd tail   :"),
        (&helper_random_array_bool_runs, "-- Runs       :"),
        (&helper_random_array_bool_organ, "-- F T F      :"),
    ]
}
