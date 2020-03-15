use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: f64 = rng.gen();
        array.push(value);
    }
    array
}

// Small
pub fn helper_random_array_small_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: f64 = rng.gen_range(-1_000_000_000.0, 1_000_000_000.0);
        array.push(value);
    }
    array
}

// Asc
pub fn helper_random_array_asc_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((i as f64) + 0.5);
    }
    array
}

// Desc
pub fn helper_random_array_desc_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(((size - i) as f64) + 0.5);
    }
    array
}

// Equal
pub fn helper_random_array_equal_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let value: f64 = rng.gen();
    vec![value; size]
}

// Zipf
pub fn helper_random_array_zipf_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: f64 = 0.0;
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
pub fn helper_random_array_normale_10_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000_000.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2_000_000_000_000_000.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 4_000_000_000_000_000_000.0);
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v);
    }
    array
}

// Sqrt
pub fn helper_random_array_sqrt_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<f64> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: f64 = 0.0;
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
pub fn helper_random_array_almost_asc_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as f64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_f64(size: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut array: Vec<f64> = Vec::with_capacity(size);

    for i in 0..size {
        array.push((size - 1 - i) as f64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<f64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f64;

    for i in 0..size {
        array.push((i as f64) % limit);
    }

    array
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_f64(size: usize) -> Vec<f64> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<f64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as f64;

    for i in 0..size {
        array.push(((size - 1 - i) as f64) % limit);
    }

    array
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    let middle = size / 2;

    for i in 0..middle {
        array.push(i as f64);
    }
    let mut k: f64 = middle as f64 + 1.0;
    for _ in middle..size {
        array.push(k);

        k -= 1.0;
    }

    array
}

// Push Front
pub fn helper_random_array_push_front_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as f64);
    }
    if size > 0 {
        array[size - 1] = 0.0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_f64(size: usize) -> Vec<f64> {
    let mut array: Vec<f64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as f64);
    }
    if size > 0 {
        array[size - 1] = (size / 2) as f64;
    }

    array
}

pub fn generators_f64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<f64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_f64, "-- Unif       :"),
        (&helper_random_array_small_f64, "-- Small      :"),
        (&helper_random_array_asc_f64, "-- Asc        :"),
        (&helper_random_array_desc_f64, "-- Desc       :"),
        (&helper_random_array_equal_f64, "-- Equal      :"),
        (&helper_random_array_zipf_f64, "-- Zipf       :"),
        (&helper_random_array_almost_asc_f64, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_f64, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_f64, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_f64, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_f64, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_f64, "-- Pipe Organ :"),
        (&helper_random_array_push_front_f64, "-- Front      :"),
        (&helper_random_array_push_middle_f64, "-- Middle     :"),
        (&helper_random_array_normale_10_f64, "-- Normale 10 :"),
        (&helper_random_array_normale_20_f64, "-- Normale 20 :"),
        (&helper_random_array_normale_30_f64, "-- Normale 30 :"),
        (&helper_random_array_normale_40_f64, "-- Normale 40 :"),
        (&helper_random_array_normale_51_f64, "-- Normale 51 :"),
        (&helper_random_array_normale_63_f64, "-- Normale 63 :"),
    ]
}
