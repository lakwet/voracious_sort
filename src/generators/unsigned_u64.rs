use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen();
        array.push(value);
    }
    array
}

// Uniform 10^9
pub fn helper_random_array_uniform_10_9_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 1_000_000_000);
        array.push(value);
    }
    array
}

// Ascending
pub fn helper_random_array_ascending_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    array
}

// Asc1pm
pub fn helper_random_array_asc1pm_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    for i in 0..(size / 1000) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Asc1pct
pub fn helper_random_array_asc1pct_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    for i in 0..(size / 100) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Asc10pct
pub fn helper_random_array_asc10pct_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    for i in 0..(size / 10) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Descending
pub fn helper_random_array_descending_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u64);
    }
    array
}

// Desc1pm
pub fn helper_random_array_desc1pm_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u64);
    }
    for i in 0..(size / 1000) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Desc1pct
pub fn helper_random_array_desc1pct_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u64);
    }
    for i in 0..(size / 100) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// Desc10pct
pub fn helper_random_array_desc10pct_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u64);
    }
    for i in 0..(size / 10) {
        let value: u64 = rng.gen();
        array[size - i - 1] = value;
    }

    array
}

// All equals
pub fn helper_random_array_allequals_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let value: u64 = rng.gen();

    vec![value; size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 16);
        array.push(value);
    }
    array
}

// Zipf
pub fn helper_random_array_zipf_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: u64 = 0;
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

// Small size1
pub fn helper_random_array_small_size1_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 255);
        array.push(value);
    }
    array
}

// Small size2
pub fn helper_random_array_small_size2_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 65_535);
        array.push(value);
    }
    array
}

// Small size3
pub fn helper_random_array_small_size3_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 16_777_215);
        array.push(value);
    }
    array
}

// Small size4
pub fn helper_random_array_small_size4_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 2u64.pow(32) - 1);
        array.push(value);
    }
    array
}

// Small size5
pub fn helper_random_array_small_size5_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 2u64.pow(40) - 1);
        array.push(value);
    }
    array
}

// Small size6
pub fn helper_random_array_small_size6_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 2u64.pow(48) - 1);
        array.push(value);
    }
    array
}

// Small size7
pub fn helper_random_array_small_size7_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u64 = rng.gen_range(0, 2u64.pow(56) - 1);
        array.push(value);
    }
    array
}

// Sqrt
pub fn helper_random_array_sqrt_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<u64> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: u64 = 0;
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
pub fn helper_random_array_almost_asc_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut array: Vec<u64> = Vec::with_capacity(size);

    for i in 0..size {
        array.push((size - 1 - i) as u64);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<u64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u64;

    for i in 0..size {
        if limit == 0 {
            array.push(i as u64);
        } else {
            array.push((i as u64) % limit);
        }
    }

    array
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<u64> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u64;

    for i in 0..size {
        if limit == 0 {
            array.push((size - 1 - i) as u64);
        } else {
            array.push(((size - 1 - i) as u64) % limit);
        }
    }

    array
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    let middle = size / 2;

    for i in 0..middle {
        array.push(i as u64);
    }
    let mut k: u64 = middle as u64 + 1;
    for _ in middle..size {
        array.push(k);

        k -= 1;
    }

    array
}

// Push Front
pub fn helper_random_array_push_front_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_u64(size: usize) -> Vec<u64> {
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u64);
    }
    if size > 0 {
        array[size - 1] = (size / 2) as u64;
    }

    array
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000_000.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2_000_000_000_000_000.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 4_000_000_000_000_000_000.0);
    let mut array: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u64);
    }
    array
}

pub fn generators_u64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<u64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u64, "-- Unif       :"),
        (&helper_random_array_uniform_10_9_u64, "-- Unif 10^9  :"),
        (&helper_random_array_small_size1_u64, "-- Small 1    :"),
        (&helper_random_array_small_size2_u64, "-- Small 2    :"),
        (&helper_random_array_small_size3_u64, "-- Small 3    :"),
        (&helper_random_array_small_size4_u64, "-- Small 4    :"),
        (&helper_random_array_small_size5_u64, "-- Small 5    :"),
        (&helper_random_array_small_size6_u64, "-- Small 6    :"),
        (&helper_random_array_small_size7_u64, "-- Small 7    :"),
        (&helper_random_array_ascending_u64, "-- Asc        :"),
        (&helper_random_array_asc1pm_u64, "-- Asc1pm     :"),
        (&helper_random_array_asc1pct_u64, "-- Asc1pct    :"),
        (&helper_random_array_asc10pct_u64, "-- Asc10pct   :"),
        (&helper_random_array_descending_u64, "-- Desc       :"),
        (&helper_random_array_desc1pm_u64, "-- Desc1pm    :"),
        (&helper_random_array_desc1pct_u64, "-- Desc1pct   :"),
        (&helper_random_array_desc10pct_u64, "-- Desc10pct  :"),
        (&helper_random_array_allequals_u64, "-- Equal      :"),
        (&helper_random_array_alternating16_u64, "-- Alt16      :"),
        (&helper_random_array_zipf_u64, "-- Zipf       :"),
        (&helper_random_array_almost_asc_u64, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_u64, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_u64, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_u64, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_u64, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_u64, "-- Pipe Organ :"),
        (&helper_random_array_push_front_u64, "-- Front      :"),
        (&helper_random_array_push_middle_u64, "-- Middle     :"),
        (&helper_random_array_normale_10_u64, "-- Normale 10 :"),
        (&helper_random_array_normale_20_u64, "-- Normale 20 :"),
        (&helper_random_array_normale_30_u64, "-- Normale 30 :"),
        (&helper_random_array_normale_40_u64, "-- Normale 40 :"),
        (&helper_random_array_normale_51_u64, "-- Normale 51 :"),
        (&helper_random_array_normale_63_u64, "-- Normale 63 :"),
    ]
}
