use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i64 = rng.gen();
        array.push(value);
    }
    array
}

// Ascending
pub fn helper_random_array_ascending_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    array
}

// Asc1pm
pub fn helper_random_array_asc1pm_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    for i in 0..(size / 1000) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Asc1pct
pub fn helper_random_array_asc1pct_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    for i in 0..(size / 100) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Asc10pct
pub fn helper_random_array_asc10pct_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    for i in 0..(size / 10) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Descending
pub fn helper_random_array_descending_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in ((-(size as i64))..0).rev() {
        array.push(i as i64);
    }
    array
}

// Desc1pm
pub fn helper_random_array_desc1pm_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as i64);
    }
    for i in 0..(size / 1000) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Desc1pct
pub fn helper_random_array_desc1pct_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as i64);
    }
    for i in 0..(size / 100) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Desc10pct
pub fn helper_random_array_desc10pct_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as i64);
    }
    for i in 0..(size / 10) {
        let value: i64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// All equals
pub fn helper_random_array_allequals_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let value: i64 = rng.gen();

    vec![value; size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i64 = rng.gen_range(0, 16);
        array.push(value);
    }
    for i in 0..size {
        array[i] = -array[i];
    }
    array
}

// Zipf
pub fn helper_random_array_zipf_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i64 = 0;
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
pub fn helper_random_array_small_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i64 = rng.gen_range(-1_000_000_000, 1_000_000_000);
        array.push(value);
    }
    array
}

// Sqrt
pub fn helper_random_array_sqrt_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<i64> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: i64 = 0;
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
pub fn helper_random_array_almost_asc_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let mut array: Vec<i64> = Vec::with_capacity(size);

    for i in 0..size {
        array.push((size - 1 - i) as i64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i64;

    for i in 0..size {
        array.push((i as i64) % limit);
    }

    array
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i64;

    for i in 0..size {
        array.push(((size - 1 - i) as i64) % limit);
    }

    array
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    let middle = size / 2;

    for i in 0..middle {
        array.push(i as i64);
    }
    let mut k: i64 = middle as i64 + 1;
    for _ in middle..size {
        array.push(k);

        k -= 1;
    }

    array
}

// Push Front
pub fn helper_random_array_push_front_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    array[size - 1] = 0;

    array
}

// Push middle
pub fn helper_random_array_push_middle_i64(size: usize) -> Vec<i64> {
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as i64);
    }
    array[size - 1] = (size / 2) as i64;

    array
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000_000.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2_000_000_000_000_000.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_i64(size: usize) -> Vec<i64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 4_000_000_000_000_000_000.0);
    let mut array: Vec<i64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as i64);
    }
    array
}

pub fn generators_i64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i64, "-- Unif       :"),
        (&helper_random_array_small_i64, "-- Small      :"),
        (&helper_random_array_ascending_i64, "-- Asc        :"),
        (&helper_random_array_asc1pm_i64, "-- Asc1pm     :"),
        (&helper_random_array_asc1pct_i64, "-- Asc1pct    :"),
        (&helper_random_array_asc10pct_i64, "-- Asc10pct   :"),
        (&helper_random_array_descending_i64, "-- Desc       :"),
        (&helper_random_array_desc1pm_i64, "-- Desc1pm    :"),
        (&helper_random_array_desc1pct_i64, "-- Desc1pct   :"),
        (&helper_random_array_desc10pct_i64, "-- Desc10pct  :"),
        (&helper_random_array_allequals_i64, "-- Equal      :"),
        (&helper_random_array_alternating16_i64, "-- Alt16      :"),
        (&helper_random_array_zipf_i64, "-- Zipf       :"),
        (&helper_random_array_almost_asc_i64, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_i64, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_i64, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i64, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_i64, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_i64, "-- Pipe Organ :"),
        (&helper_random_array_push_front_i64, "-- Front      :"),
        (&helper_random_array_push_middle_i64, "-- Middle     :"),
        (&helper_random_array_normale_10_i64, "-- Normale 10 :"),
        (&helper_random_array_normale_20_i64, "-- Normale 20 :"),
        (&helper_random_array_normale_30_i64, "-- Normale 30 :"),
        (&helper_random_array_normale_40_i64, "-- Normale 40 :"),
        (&helper_random_array_normale_51_i64, "-- Normale 51 :"),
        (&helper_random_array_normale_63_i64, "-- Normale 63 :"),
    ]
}
