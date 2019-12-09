use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen();
        array.push(value);
    }
    array
}

// Uniform 10^9
pub fn helper_random_array_uniform_10_9_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen_range(0, 1_000_000_000);
        array.push(value);
    }
    array
}

// Ascending
pub fn helper_random_array_ascending_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u32);
    }
    array
}

// Descending
pub fn helper_random_array_descending_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u32);
    }
    array
}

// All equals
pub fn helper_random_array_allequals_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let value: u32 = rng.gen();

    vec![value; size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen_range(0, 16);
        array.push(value);
    }
    array
}

// Zipf
pub fn helper_random_array_zipf_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    let mut rng = thread_rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: u32 = 0;
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
pub fn helper_random_array_small_size1_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen_range(0, 255);
        array.push(value);
    }
    array
}

// Small size2
pub fn helper_random_array_small_size2_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen_range(0, 65_535);
        array.push(value);
    }
    array
}

// Small size3
pub fn helper_random_array_small_size3_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: u32 = rng.gen_range(0, 16_777_215);
        array.push(value);
    }
    array
}

// Sqrt
pub fn helper_random_array_sqrt_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let sqrt = (size as f64).sqrt() as usize;
    let mut array: Vec<u32> = Vec::with_capacity(size);

    let mut i = 0;
    let mut value: u32 = 0;
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
pub fn helper_random_array_almost_asc_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u32);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push((size - 1 - i) as u32);
    }

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_u32(size: usize) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<u32> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u32;

    for i in 0..size {
        if limit == 0 {
            array.push(i as u32);
        } else {
            array.push((i as u32) % limit);
        }
    }

    array
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u32(size: usize) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }

    let mut array: Vec<u32> = Vec::with_capacity(size);
    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u32;

    for i in 0..size {
        if limit == 0 {
            array.push((size - 1 - i) as u32);
        } else {
            array.push(((size - 1 - i) as u32) % limit);
        }
    }

    array
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    let middle = size / 2;

    for i in 0..middle {
        array.push(i as u32);
    }
    let mut k: u32 = middle as u32 + 1;
    for _ in middle..size {
        array.push(k);

        k -= 1;
    }

    array
}

// Push Front
pub fn helper_random_array_push_front_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u32);
    }
    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_u32(size: usize) -> Vec<u32> {
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for i in 0..size {
        array.push(i as u32);
    }
    if size > 0 {
        array[size - 1] = (size / 2) as u32;
    }

    array
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1024.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1_000_000_000_000.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2_000_000_000_000_000.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_u32(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 4_000_000_000_000_000_000.0);
    let mut array: Vec<u32> = Vec::with_capacity(size);
    for _ in 0..size {
        let v: f64 = normal.sample(&mut rng);
        array.push(v as u32);
    }
    array
}

pub fn generators_u32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<u32>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u32, "-- Unif       :"),
        (&helper_random_array_uniform_10_9_u32, "-- Unif 10^9  :"),
        (&helper_random_array_small_size1_u32, "-- Small1     :"),
        (&helper_random_array_small_size2_u32, "-- Small2     :"),
        (&helper_random_array_small_size3_u32, "-- Small3     :"),
        (&helper_random_array_ascending_u32, "-- Asc        :"),
        (&helper_random_array_descending_u32, "-- Desc       :"),
        (&helper_random_array_allequals_u32, "-- Equal      :"),
        (&helper_random_array_alternating16_u32, "-- Alt16      :"),
        (&helper_random_array_zipf_u32, "-- Zipf       :"),
        (&helper_random_array_almost_asc_u32, "-- Almost Asc :"),
        (&helper_random_array_almost_desc_u32, "-- Almost Desc:"),
        (&helper_random_array_asc_sawtooth_u32, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_u32, "-- Desc Saw   :"),
        (&helper_random_array_sqrt_u32, "-- Sqrt       :"),
        (&helper_random_array_pipe_organ_u32, "-- Pipe Organ :"),
        (&helper_random_array_push_front_u32, "-- Front      :"),
        (&helper_random_array_push_middle_u32, "-- Middle     :"),
        (&helper_random_array_normale_10_u32, "-- Normale 10 :"),
        (&helper_random_array_normale_20_u32, "-- Normale 20 :"),
        (&helper_random_array_normale_30_u32, "-- Normale 30 :"),
        (&helper_random_array_normale_40_u32, "-- Normale 40 :"),
        (&helper_random_array_normale_51_u32, "-- Normale 51 :"),
        (&helper_random_array_normale_63_u32, "-- Normale 63 :"),
    ]
}
