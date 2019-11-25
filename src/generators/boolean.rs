use rand::{thread_rng, Rng};

pub fn helper_random_array_bool_unif(size: usize) -> Vec<bool> {
    let mut rng = thread_rng();
    let mut array: Vec<bool> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u8 = rng.gen_range(0, 2);
        if value == 0 {
            array.push(false);
        } else {
            array.push(true);
        }
    }
    array
}

pub fn helper_random_array_bool_alt(size: usize) -> Vec<bool> {
    let mut array: Vec<bool> = Vec::with_capacity(size);
    let mut is_odd = false;
    for _ in 0..size {
        if is_odd {
            array.push(true);
        } else {
            array.push(false);
        }
        is_odd = !is_odd;
    }
    array
}

pub fn helper_random_array_bool_true(size: usize) -> Vec<bool> {
    vec![true; size]
}

pub fn helper_random_array_bool_false(size: usize) -> Vec<bool> {
    vec![false; size]
}

pub fn helper_random_array_bool_pipe(size: usize) -> Vec<bool> {
    let mut array: Vec<bool> = Vec::with_capacity(size);
    for _ in 0..(size / 2) {
        array.push(true);
    }
    for _ in (size / 2)..size {
        array.push(false);
    }
    array
}

pub fn helper_random_array_bool_pipe_rev(size: usize) -> Vec<bool> {
    let mut array: Vec<bool> = Vec::with_capacity(size);
    for _ in 0..(size / 2) {
        array.push(false);
    }
    for _ in (size / 2)..size {
        array.push(true);
    }
    array
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
