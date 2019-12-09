use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i32 = rng.gen();
        array.push(value);
    }
    array
}

// Ascending
pub fn helper_random_array_ascending_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i32);
    }
    array
}

// Descending
pub fn helper_random_array_descending_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in ((-(size as i32))..0).rev() {
        array.push(i as i32);
    }
    array
}

// All equals
pub fn helper_random_array_allequals_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let value: i32 = rng.gen();

    vec![value; size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i32 = rng.gen_range(0, 16);
        array.push(value);
    }
    for i in 0..size {
        array[i] = -array[i];
    }
    array
}

// Zipf
pub fn helper_random_array_zipf_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i32 = 0;
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

// Small values
pub fn helper_random_array_small_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i32 = rng.gen_range(-64_000, 64_000);
        array.push(value);
    }
    array
}

// Sqrt
pub fn helper_random_array_sqrt_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<i32> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: i32 = 0;
    for _ in 0..sqrt {
        value = rng.gen();
        for _ in 0..sqrt {
            array.push(value);
            i += 1;
        }
    }
    while i < size {
        array.push(value);
        i += 1;
    }

    array
}

// Almost sorted ascending
pub fn helper_random_array_almost_asc_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i32);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as i32);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i32(size: usize) -> Vec<i32> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<i32> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i32;

    for i in 0..size {
        array.push((i as i32) % limit);
    }

    array
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i32(size: usize) -> Vec<i32> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<i32> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i32;

    for i in 0..size {
        array.push(((size - 1 - i) as i32) % limit);
    }

    array
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    let middle = size / 2;

    for i in 0..middle {
        array.push(i as i32);
    }
    let mut k: i32 = middle as i32 + 1;
    for _ in middle..size {
        array.push(k);

        k -= 1;
    }

    array
}

// Push Front
pub fn helper_random_array_push_front_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i32);
    }
    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_i32(size: usize) -> Vec<i32> {
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i32);
    }
    if size > 0 {
        array[size - 1] = (size / 2) as i32;
    }

    array
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i32);
    }
    array
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000.0);
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i32);
    }
    array
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_i32(size: usize) -> Vec<i32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000.0);
    let mut array: Vec<i32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i32);
    }
    array
}

pub fn generators_i32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i32>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i32, "-- Unif       :"),
        (&helper_random_array_small_i32, "-- Small      :"),
        (&helper_random_array_ascending_i32, "-- Asc        :"),
        (&helper_random_array_descending_i32, "-- Desc       :"),
        (&helper_random_array_allequals_i32, "-- Equal      :"),
        (&helper_random_array_alternating16_i32, "-- Alt16      :"),
        (&helper_random_array_zipf_i32, "-- Zipf       :"),
        (&helper_random_array_almost_asc_i32, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_i32, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_i32, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i32, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_i32, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_i32, "-- Pipe Organ :"),
        (&helper_random_array_push_front_i32, "-- Front      :"),
        (&helper_random_array_push_middle_i32, "-- Middle     :"),
        (&helper_random_array_normale_10_i32, "-- Normale 10 :"),
        (&helper_random_array_normale_20_i32, "-- Normale 20 :"),
        (&helper_random_array_normale_30_i32, "-- Normale 30 :"),
    ]
}
