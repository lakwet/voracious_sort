use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u32(size: usize) -> Vec<u32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<u32>())
        .collect::<Vec<u32>>()
}

// Uniform 10^9
pub fn helper_random_array_uniform_10_9_u32(size: usize) -> Vec<u32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, 1_000_000_000))
        .collect::<Vec<u32>>()
}

// Ascending
pub fn helper_random_array_ascending_u32(size: usize) -> Vec<u32> {
    (0..(size as u32)).into_par_iter().collect::<Vec<u32>>()
}

// Descending
pub fn helper_random_array_descending_u32(size: usize) -> Vec<u32> {
    (0..size).into_par_iter().map(|i| (size - i) as u32).collect::<Vec<u32>>()
}

// All equals
pub fn helper_random_array_allequals_u32(size: usize) -> Vec<u32> {
    vec![thread_rng().gen(); size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_u32(size: usize) -> Vec<u32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, 16))
        .collect::<Vec<u32>>()
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

    array.as_mut_slice().shuffle(&mut rng);

    array
}

fn helper_small(size: usize, range: u32) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }

    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, range))
        .collect::<Vec<u32>>()
}

// Small size1
pub fn helper_random_array_small_size1_u32(size: usize) -> Vec<u32> {
    helper_small(size, 255)
}

// Small size2
pub fn helper_random_array_small_size2_u32(size: usize) -> Vec<u32> {
    helper_small(size, 65_535)
}

// Small size3
pub fn helper_random_array_small_size3_u32(size: usize) -> Vec<u32> {
    helper_small(size, 16_777_215)
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
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u32(size);
    }

    let mut rng = thread_rng();
    let mut array = helper_random_array_ascending_u32(size);

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_u32(size: usize) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u32(size);
    }

    let mut rng = thread_rng();
    let mut array = helper_random_array_descending_u32(size);
    array.reverse();

    for _ in 0..((size as f64).log2() as usize) {
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
    if size < 4 {
        return helper_random_array_uniform_u32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u32;
    (0..size).into_par_iter().map(|i| i as u32 % limit).collect::<Vec<u32>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u32(size: usize) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u32;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as u32 % limit)
        .collect::<Vec<u32>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_u32(size: usize) -> Vec<u32> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as u32 } else { (size - i) as u32 })
        .collect::<Vec<u32>>()
}

// Push Front
pub fn helper_random_array_push_front_u32(size: usize) -> Vec<u32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as u32).collect::<Vec<u32>>();

    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_u32(size: usize) -> Vec<u32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as u32).collect::<Vec<u32>>();

    if size > 0 {
        array[size - 1] = (size / 2) as u32;
    }

    array
}

fn helper_normal(size: usize, bound: f64) -> Vec<u32> {
    let normal = Normal::new(0.0, bound).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let v: f64 = normal.sample(&mut thread_rng());
            v.abs() as u32
        })
        .collect::<Vec<u32>>()
}

// Normale(0, 2^8)
pub fn helper_random_array_normale_8_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 255.0)
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^13)
pub fn helper_random_array_normale_13_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 8191.0)
}

// Normale(0, 2^16)
pub fn helper_random_array_normale_16_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 65_535.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^24)
pub fn helper_random_array_normale_24_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 16_777_215.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_u32(size: usize) -> Vec<u32> {
    helper_normal(size, 1_000_000_000.0)
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
        (&helper_random_array_normale_8_u32, "-- Normale  8 :"),
        (&helper_random_array_normale_10_u32, "-- Normale 10 :"),
        (&helper_random_array_normale_13_u32, "-- Normale 13 :"),
        (&helper_random_array_normale_16_u32, "-- Normale 16 :"),
        (&helper_random_array_normale_20_u32, "-- Normale 20 :"),
        (&helper_random_array_normale_24_u32, "-- Normale 24 :"),
        (&helper_random_array_normale_30_u32, "-- Normale 30 :"),
    ]
}

#[cfg(target_pointer_width = "32")]
pub fn generators_usize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u32()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u32>, Vec<usize>>(arr)
                }
            };

            (
                Box::leak(Box::new(new_gen))
                    as &'static dyn Fn(usize) -> Vec<usize>,
                title,
            )
        })
        .collect()
}
