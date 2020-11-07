use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u64(size: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<u64>())
        .collect::<Vec<u64>>()
}

// Uniform 10^9
pub fn helper_random_array_uniform_10_9_u64(size: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, 1_000_000_000))
        .collect::<Vec<u64>>()
}

// Ascending
pub fn helper_random_array_ascending_u64(size: usize) -> Vec<u64> {
    (0..size).into_par_iter().map(|i| i as u64).collect::<Vec<u64>>()
}

fn helper_asc_xth_start(size: usize, frac: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i < size / frac {
                thread_rng().gen::<u64>()
            } else {
                i as u64
            }
        })
        .collect::<Vec<u64>>()
}

// Asc1pmst
pub fn helper_random_array_asc1pmst_u64(size: usize) -> Vec<u64> {
    helper_asc_xth_start(size, 1000)
}

// Asc1pctst
pub fn helper_random_array_asc1pctst_u64(size: usize) -> Vec<u64> {
    helper_asc_xth_start(size, 100)
}

// Asc10pctst
pub fn helper_random_array_asc10pctst_u64(size: usize) -> Vec<u64> {
    helper_asc_xth_start(size, 10)
}

fn helper_asc_xth(size: usize, frac: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i > size - (size / frac) {
                thread_rng().gen::<u64>()
            } else {
                i as u64
            }
        })
        .collect::<Vec<u64>>()
}

// Asc1pm
pub fn helper_random_array_asc1pm_u64(size: usize) -> Vec<u64> {
    helper_asc_xth(size, 1000)
}

// Asc1pct
pub fn helper_random_array_asc1pct_u64(size: usize) -> Vec<u64> {
    helper_asc_xth(size, 100)
}

// Asc10pct
pub fn helper_random_array_asc10pct_u64(size: usize) -> Vec<u64> {
    helper_asc_xth(size, 10)
}

// Descending
pub fn helper_random_array_descending_u64(size: usize) -> Vec<u64> {
    (0..size).into_par_iter().map(|i| (size - i) as u64).collect::<Vec<u64>>()
}

fn helper_desc_xthst(size: usize, frac: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i < size / frac {
                thread_rng().gen::<u64>()
            } else {
                (size - i) as u64
            }
        })
        .collect::<Vec<u64>>()
}

// Desc1pmst
pub fn helper_random_array_desc1pmst_u64(size: usize) -> Vec<u64> {
    helper_desc_xthst(size, 1000)
}

// Desc1pctst
pub fn helper_random_array_desc1pctst_u64(size: usize) -> Vec<u64> {
    helper_desc_xthst(size, 100)
}

// Desc10pctst
pub fn helper_random_array_desc10pctst_u64(size: usize) -> Vec<u64> {
    helper_desc_xthst(size, 10)
}

fn helper_desc_xth(size: usize, frac: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|i| {
            if i > size - (size / frac) {
                thread_rng().gen::<u64>()
            } else {
                (size - i) as u64
            }
        })
        .collect::<Vec<u64>>()
}

// Desc1pm
pub fn helper_random_array_desc1pm_u64(size: usize) -> Vec<u64> {
    helper_desc_xth(size, 1000)
}

// Desc1pct
pub fn helper_random_array_desc1pct_u64(size: usize) -> Vec<u64> {
    helper_desc_xth(size, 100)
}

// Desc10pct
pub fn helper_random_array_desc10pct_u64(size: usize) -> Vec<u64> {
    helper_desc_xth(size, 10)
}

// All equals
pub fn helper_random_array_allequals_u64(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let value: u64 = rng.gen();

    vec![value; size]
}

// Alternating 16 values
pub fn helper_random_array_alternating16_u64(size: usize) -> Vec<u64> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, 16))
        .collect::<Vec<u64>>()
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

    array.as_mut_slice().shuffle(&mut rng);

    array
}

fn helper_small(size: usize, range: u64) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }

    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen_range(0, range))
        .collect::<Vec<u64>>()
}

// Small size1
pub fn helper_random_array_small_size1_u64(size: usize) -> Vec<u64> {
    helper_small(size, 255)
}

// Small size2
pub fn helper_random_array_small_size2_u64(size: usize) -> Vec<u64> {
    helper_small(size, 65_535)
}

// Small size3
pub fn helper_random_array_small_size3_u64(size: usize) -> Vec<u64> {
    helper_small(size, 16_777_215)
}

// Small size4
pub fn helper_random_array_small_size4_u64(size: usize) -> Vec<u64> {
    helper_small(size, 2u64.pow(32) - 1)
}

// Small size5
pub fn helper_random_array_small_size5_u64(size: usize) -> Vec<u64> {
    helper_small(size, 2u64.pow(40) - 1)
}

// Small size6
pub fn helper_random_array_small_size6_u64(size: usize) -> Vec<u64> {
    helper_small(size, 2u64.pow(48) - 1)
}

// Small size7
pub fn helper_random_array_small_size7_u64(size: usize) -> Vec<u64> {
    helper_small(size, 2u64.pow(56) - 1)
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
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let mut rng = thread_rng();
    let mut array = helper_random_array_ascending_u64(size);

    for _ in 0..((size as f64).log2() as usize) {
        let i = rng.gen_range(0, size);
        let j = rng.gen_range(0, size);
        array.swap(i, j);
    }

    array
}

// Almost sorted descending
pub fn helper_random_array_almost_desc_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let mut rng = thread_rng();
    let mut array = helper_random_array_descending_u64(size);
    array.reverse();

    for _ in 0..((size as f64).log2() as usize) {
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
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u64;
    (0..size).into_par_iter().map(|i| i as u64 % limit).collect::<Vec<u64>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 0.9)) as u64;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as u64 % limit)
        .collect::<Vec<u64>>()
}

// Ascending sawtooth killer
pub fn helper_random_array_asc_sawtooth_killer_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 1.1)) as u64;
    (0..size).into_par_iter().map(|i| i as u64 % limit).collect::<Vec<u64>>()
}

// Descending sawtooth killer
pub fn helper_random_array_desc_sawtooth_killer_u64(size: usize) -> Vec<u64> {
    if size == 0 {
        return Vec::new();
    }
    if size < 4 {
        return helper_random_array_uniform_u64(size);
    }

    let limit = (size as f64 / ((size as f64).log2() * 1.1)) as u64;
    (0..size)
        .into_par_iter()
        .map(|i| (size - 1 - i) as u64 % limit)
        .collect::<Vec<u64>>()
}

// Pipe Organ
pub fn helper_random_array_pipe_organ_u64(size: usize) -> Vec<u64> {
    let middle = size / 2;
    (0..size)
        .into_par_iter()
        .map(|i| if i < middle { i as u64 } else { (size - i) as u64 })
        .collect::<Vec<u64>>()
}

// Push Front
pub fn helper_random_array_push_front_u64(size: usize) -> Vec<u64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as u64).collect::<Vec<u64>>();

    if size > 0 {
        array[size - 1] = 0;
    }

    array
}

// Push middle
pub fn helper_random_array_push_middle_u64(size: usize) -> Vec<u64> {
    let mut array =
        (0..size).into_par_iter().map(|i| i as u64).collect::<Vec<u64>>();

    if size > 0 {
        array[size - 1] = (size / 2) as u64;
    }

    array
}

fn helper_normal(size: usize, bound: f64) -> Vec<u64> {
    let normal = Normal::new(0.0, bound).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let v: f64 = normal.sample(&mut thread_rng());
            v.abs() as u64
        })
        .collect::<Vec<u64>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 1_000_000_000.0)
}

// Normale(0, 2^40)
pub fn helper_random_array_normale_40_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 1_000_000_000_000.0)
}

// Normale(0, 2^51)
pub fn helper_random_array_normale_51_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 2_000_000_000_000_000.0)
}

// Normale(0, 2^63)
pub fn helper_random_array_normale_63_u64(size: usize) -> Vec<u64> {
    helper_normal(size, 4_000_000_000_000_000_000.0)
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
        (&helper_random_array_asc1pmst_u64, "-- Asc1pmst   :"),
        (&helper_random_array_asc1pctst_u64, "-- Asc1pctst  :"),
        (&helper_random_array_asc10pctst_u64, "-- Asc10pctst :"),
        (&helper_random_array_asc1pm_u64, "-- Asc1pm     :"),
        (&helper_random_array_asc1pct_u64, "-- Asc1pct    :"),
        (&helper_random_array_asc10pct_u64, "-- Asc10pct   :"),
        (&helper_random_array_descending_u64, "-- Desc       :"),
        (&helper_random_array_desc1pmst_u64, "-- Desc1pmst  :"),
        (&helper_random_array_desc1pctst_u64, "-- Desc1pctst :"),
        (&helper_random_array_desc10pctst_u64, "-- Desc10pctst:"),
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
        (&helper_random_array_asc_sawtooth_killer_u64, "-- Asc Killer :"),
        (&helper_random_array_desc_sawtooth_killer_u64, "-- Desc Killer:"),
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

#[cfg(target_pointer_width = "64")]
pub fn generators_usize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u64()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u64>, Vec<usize>>(arr)
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
