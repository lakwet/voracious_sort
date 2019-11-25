use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_i16(size: usize) -> Vec<i16> {
    let mut rng = thread_rng();
    let mut array: Vec<i16> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i16 = rng.gen();
        array.push(value);
    }
    array
}

// Small
pub fn helper_random_array_small_i16(size: usize) -> Vec<i16> {
    let mut rng = thread_rng();
    let mut array: Vec<i16> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i16 = rng.gen_range(-128, 127);
        array.push(value);
    }
    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i16(size: usize) -> Vec<i16> {
    let mut array: Vec<i16> = Vec::with_capacity(size);

    for i in 0..size {
        array.push(i as i16);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_desc_sawtooth_i16(size: usize) -> Vec<i16> {
    let mut array: Vec<i16> = Vec::with_capacity(size);

    for i in 0..size {
        array.push((size - 1 - i) as i16);
    }

    array
}

// All equals
pub fn helper_random_array_allequals_i16(size: usize) -> Vec<i16> {
    let mut rng = thread_rng();
    let value: i16 = rng.gen();

    vec![value; size]
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

    rng.shuffle(array.as_mut_slice());

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
