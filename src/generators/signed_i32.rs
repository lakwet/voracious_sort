use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i32(size: usize) -> Vec<i32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<i32>())
        .collect::<Vec<i32>>()
}

// 10^9 values
pub fn helper_random_array_109_i32(size: usize) -> Vec<i32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000_000, 1_000_000_000))
        .collect::<Vec<i32>>()
}

// Small values
pub fn helper_random_array_small_i32(size: usize) -> Vec<i32> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-64_000, 64_000))
        .collect::<Vec<i32>>()
}

// Ascending
pub fn helper_random_array_ascending_i32(size: usize) -> Vec<i32> {
    (0..(size as i32)).into_par_iter().collect::<Vec<i32>>()
}

// Descending
pub fn helper_random_array_descending_i32(size: usize) -> Vec<i32> {
    (0..size).into_par_iter().map(|i| -(i as i32)).collect::<Vec<i32>>()
}

// All equals
pub fn helper_random_array_allequals_i32(size: usize) -> Vec<i32> {
    vec![thread_rng().gen(); size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_i32(size: usize) -> Vec<i32> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i % 2 == 0 {
                thread_rng().gen_range(0, 16) as i32
            } else {
                -(thread_rng().gen_range(0, 16) as i32)
            }
        })
        .collect::<Vec<i32>>()
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

    array.as_mut_slice().shuffle(&mut rng);

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
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i32(size);
    }

    let mut array = helper_random_array_ascending_i32(size);

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = thread_rng().gen_range(0, size);
        let j = thread_rng().gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_i32(size: usize) -> Vec<i32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i32(size);
    }

    let mut array = helper_random_array_descending_i32(size);

    for _ in 0..((size as f64).sqrt() as usize) {
        let i = thread_rng().gen_range(0, size);
        let j = thread_rng().gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i32(size: usize) -> Vec<i32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i32;
    (0..size).into_par_iter().map(|i| i as i32 % limit).collect::<Vec<i32>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i32(size: usize) -> Vec<i32> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i32(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i32;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as i32 % limit)
        .collect::<Vec<i32>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_i32(size: usize) -> Vec<i32> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as i32 } else { (size - i) as i32 })
        .collect::<Vec<i32>>()
}

// Push Front
pub fn helper_random_array_push_front_i32(size: usize) -> Vec<i32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as i32).collect::<Vec<i32>>();

    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_i32(size: usize) -> Vec<i32> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as i32).collect::<Vec<i32>>();

    if size > 0 {
        array[size - 1] = (size / 2) as i32;
    }

    array
}

fn helper_normal(size: usize, range: f32) -> Vec<i32> {
    let normal = Normal::new(0.0, range).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut thread_rng()) as i32)
        .collect::<Vec<i32>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i32(size: usize) -> Vec<i32> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_i32(size: usize) -> Vec<i32> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_i32(size: usize) -> Vec<i32> {
    helper_normal(size, 1_000_000_000.0)
}

pub fn generators_i32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i32>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i32, "-- Unif       :"),
        (&helper_random_array_109_i32, "-- +-10^9     :"),
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

#[cfg(target_pointer_width = "32")]
pub fn generators_isize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i32()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i32>, Vec<isize>>(arr)
                }
            };

            (
                Box::leak(Box::new(new_gen))
                    as &'static dyn Fn(usize) -> Vec<isize>,
                title,
            )
        })
        .collect()
}
