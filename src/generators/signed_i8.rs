use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_i8(size: usize) -> Vec<i8> {
    let mut rng = thread_rng();
    let mut array: Vec<i8> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i8 = rng.gen();
        array.push(value);
    }
    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i8(size: usize) -> Vec<i8> {
    let mut array: Vec<i8> = Vec::with_capacity(size);

    for i in 0..size {
        array.push(i as i8);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_desc_sawtooth_i8(size: usize) -> Vec<i8> {
    let mut array: Vec<i8> = Vec::with_capacity(size);

    for i in 0..size {
        array.push((size - 1 - i) as i8);
    }

    array
}

// All equals
pub fn helper_random_array_allequals_i8(size: usize) -> Vec<i8> {
    let mut rng = thread_rng();
    let value: i8 = rng.gen();

    vec![value; size]
}

// Zipf
pub fn helper_random_array_zipf_i8(size: usize) -> Vec<i8> {
    let mut array: Vec<i8> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i8 = 0;
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

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i8(size: usize) -> Vec<i8> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<i8> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i8);
    }
    array
}

pub fn generators_i8() -> Vec<(&'static dyn Fn(usize) -> Vec<i8>, &'static str)>
{
    vec![
        (&helper_random_array_uniform_i8, "-- Unif       :"),
        (&helper_random_array_asc_sawtooth_i8, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i8, "-- Desc Saw   :"),
        (&helper_random_array_allequals_i8, "-- Equal      :"),
        (&helper_random_array_zipf_i8, "-- Zipf       :"),
        (&helper_random_array_normale_10_i8, "-- Normale 10 :"),
    ]
}
