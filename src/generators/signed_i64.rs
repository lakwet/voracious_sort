use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i64(size: usize) -> Vec<i64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<i64>())
        .collect::<Vec<i64>>()
}

// Small values
pub fn helper_random_array_109_i64(size: usize) -> Vec<i64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(-1_000_000_000, 1_000_000_000))
        .collect::<Vec<i64>>()
}

// Ascending
pub fn helper_random_array_ascending_i64(size: usize) -> Vec<i64> {
    (0..size).into_par_iter().map(|i| i as i64).collect::<Vec<i64>>()
}

fn helper_asc_xth(size: usize, frac: usize) -> Vec<i64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i > size - (size / frac) {
                thread_rng().gen::<i64>()
            } else {
                i as i64
            }
        })
        .collect::<Vec<i64>>()
}

// Asc1pm
pub fn helper_random_array_asc1pm_i64(size: usize) -> Vec<i64> {
    helper_asc_xth(size, 1000)
}

// Asc1pct
pub fn helper_random_array_asc1pct_i64(size: usize) -> Vec<i64> {
    helper_asc_xth(size, 100)
}

// Asc10pct
pub fn helper_random_array_asc10pct_i64(size: usize) -> Vec<i64> {
    helper_asc_xth(size, 10)
}

// Descending
pub fn helper_random_array_descending_i64(size: usize) -> Vec<i64> {
    (0..size).into_par_iter().map(|i| -(i as i64)).collect::<Vec<i64>>()
}

fn helper_desc_xth(size: usize, frac: usize) -> Vec<i64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i > size - (size / frac) {
                thread_rng().gen::<i64>()
            } else {
                -(i as i64)
            }
        })
        .collect::<Vec<i64>>()
}

// Desc1pm
pub fn helper_random_array_desc1pm_i64(size: usize) -> Vec<i64> {
    helper_desc_xth(size, 1000)
}

// Desc1pct
pub fn helper_random_array_desc1pct_i64(size: usize) -> Vec<i64> {
    helper_desc_xth(size, 100)
}

// Desc10pct
pub fn helper_random_array_desc10pct_i64(size: usize) -> Vec<i64> {
    helper_desc_xth(size, 10)
}

// All equals
pub fn helper_random_array_allequals_i64(size: usize) -> Vec<i64> {
    vec![thread_rng().gen(); size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_i64(size: usize) -> Vec<i64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i % 2 == 0 {
                thread_rng().gen_range(0, 16) as i64
            } else {
                -(thread_rng().gen_range(0, 16) as i64)
            }
        })
        .collect::<Vec<i64>>()
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

    array.as_mut_slice().shuffle(&mut rng);

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
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i64(size);
    }

    let mut array = helper_random_array_ascending_i64(size);
    let mut rng = thread_rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_i64(size: usize) -> Vec<i64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i64(size);
    }

    let mut array = helper_random_array_descending_i64(size);
    let mut rng = thread_rng();

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i64(size: usize) -> Vec<i64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i64;
    (0..size).into_par_iter().map(|i| i as i64 % limit).collect::<Vec<i64>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i64(size: usize) -> Vec<i64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_i64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as i64;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as i64 % limit)
        .collect::<Vec<i64>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_i64(size: usize) -> Vec<i64> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as i64 } else { (size - i) as i64 })
        .collect::<Vec<i64>>()
}

// Push Front
pub fn helper_random_array_push_front_i64(size: usize) -> Vec<i64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as i64).collect::<Vec<i64>>();

    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_i64(size: usize) -> Vec<i64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as i64).collect::<Vec<i64>>();

    if size > 0 {
        array[size - 1] = (size / 2) as i64;
    }

    array
}

fn helper_normal(size: usize, range: f32) -> Vec<i64> {
    let normal = Normal::new(0.0, range).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| normal.sample(&mut thread_rng()) as i64)
        .collect::<Vec<i64>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 1_000_000_000.0)
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 1_000_000_000_000.0)
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 2_000_000_000_000_000.0)
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_i64(size: usize) -> Vec<i64> {
    helper_normal(size, 4_000_000_000_000_000_000.0)
}

pub fn generators_i64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i64, "-- Unif       :"),
        (&helper_random_array_109_i64, "-- +-10^9     :"),
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

#[cfg(target_pointer_width = "64")]
pub fn generators_isize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i64()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i64>, Vec<isize>>(arr)
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
