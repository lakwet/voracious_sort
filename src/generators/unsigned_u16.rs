use rand::seq::SliceRandom;
use rand::RngExt;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_u16(size: usize) -> Vec<u16> {
    (0..size)
        .into_par_iter()
        .map(|_| crate::generators::rng().random::<u16>())
        .collect::<Vec<u16>>()
}

// Small
pub fn helper_random_array_small_u16(size: usize) -> Vec<u16> {
    (0..size)
        .into_par_iter()
        .map(|_| crate::generators::rng().random_range(0..255))
        .collect::<Vec<u16>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_u16(size: usize) -> Vec<u16> {
    (0..size).into_par_iter().map(|i| i as u16).collect::<Vec<u16>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_u16(size: usize) -> Vec<u16> {
    (0..size).into_par_iter().map(|i| (size - 1 - i) as u16).collect::<Vec<u16>>()
}

// All equals
pub fn helper_random_array_allequals_u16(size: usize) -> Vec<u16> {
    vec![crate::generators::rng().random(); size]
}

// Zipf: frequencies in 1 / rank over min(size, domain) random values
pub fn helper_random_array_zipf_u16(size: usize) -> Vec<u16> {
    super::zipf(size, 1 << 16, || crate::generators::rng().random::<u16>())
}

// Fake Zipf: n / 2 copies of a random value, then n / 4, ... (about
// log2(n) distinct values, geometric frequencies)
pub fn helper_random_array_fakezipf_u16(size: usize) -> Vec<u16> {
    let mut array: Vec<u16> = Vec::with_capacity(size);
    let mut rng = crate::generators::rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: u16 = 0;
    while quantity > 2 {
        value = rng.random();

        for _ in 0..quantity {
            array.push(value);
            i += 1;
        }

        quantity /= 2;
    }
    while i < size {
        array.push(value);
        i += 1;
    }

    array.as_mut_slice().shuffle(&mut rng);

    array
}

// Distinct values between Few 4 and Unif (detectors few / heavy, windowed
// against whole-domain counting): `k` random values, drawn uniformly.
fn spread_u16(size: usize, k: usize) -> Vec<u16> {
    let mut rng = crate::generators::rng();
    let values: Vec<u16> = (0..k).map(|_| rng.random::<u16>()).collect();
    (0..size)
        .into_par_iter()
        .map(|_| values[crate::generators::rng().random_range(0..k)])
        .collect()
}

// Two random values
pub fn helper_random_array_binary_u16(size: usize) -> Vec<u16> { spread_u16(size, 2) }

// 16 random values
pub fn helper_random_array_few16_u16(size: usize) -> Vec<u16> { spread_u16(size, 16) }

// 256 random values spread over the domain
pub fn helper_random_array_spread256_u16(size: usize) -> Vec<u16> { spread_u16(size, 256) }

// 4096 random values spread over the domain
pub fn helper_random_array_spread4k_u16(size: usize) -> Vec<u16> { spread_u16(size, 4096) }

// |Normal(0, sd)|, clamped (as Normale for u32)
fn normal_u16(size: usize, sd: f64) -> Vec<u16> {
    let normal = Normal::new(0.0, sd).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let v: f64 = normal.sample(&mut crate::generators::rng());
            v.abs().min(u16::MAX as f64) as u16
        })
        .collect()
}

// Normal, standard deviation 2^8
pub fn helper_random_array_normal8_u16(size: usize) -> Vec<u16> { normal_u16(size, 255.0) }

// Normal, standard deviation 2^13
pub fn helper_random_array_normal13_u16(size: usize) -> Vec<u16> { normal_u16(size, 8191.0) }

// RootDup (IPS4o): i mod sqrt(n), a repeating ramp of sqrt(n) values
pub fn helper_random_array_rootdup_u16(size: usize) -> Vec<u16> {
    let root = ((size as f64).sqrt() as usize).max(1);
    (0..size).into_par_iter().map(|i| (i % root) as u16).collect()
}

// Depth map (millimetres, 640-pixel rows): a smooth walk between 500 and
// 8000, with patches of invalid pixels (0), about 10 %
pub fn helper_random_array_depth_u16(size: usize) -> Vec<u16> {
    let mut rng = crate::generators::rng();
    let mut out = Vec::with_capacity(size);
    let mut d: i32 = rng.random_range(500..8000);
    let mut hole = 0;
    for i in 0..size {
        if i % 640 == 0 {
            d = rng.random_range(500..8000);
        }
        d = (d + rng.random_range(-8..9)).clamp(500, 8000);
        if hole == 0 && rng.random_range(0..400) == 0 {
            hole = rng.random_range(10..80);
        }
        if hole > 0 {
            hole -= 1;
            out.push(0);
        } else {
            out.push(d as u16);
        }
    }
    out
}

pub fn generators_u16() -> Vec<(&'static dyn Fn(usize) -> Vec<u16>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u16, "-- Unif       :"),
        (&helper_random_array_small_u16, "-- Small      :"),
        (&helper_random_array_asc_sawtooth_u16, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_u16, "-- Desc Saw   :"),
        (&helper_random_array_allequals_u16, "-- Equal      :"),
        (&helper_random_array_zipf_u16, "-- Zipf       :"),
        (&helper_random_array_fakezipf_u16, "-- FakeZipf   :"),
        (&helper_random_array_binary_u16, "-- Binary     :"),
        (&helper_random_array_few16_u16, "-- Few 16     :"),
        (&helper_random_array_spread256_u16, "-- Spread 256 :"),
        (&helper_random_array_spread4k_u16, "-- Spread 4k  :"),
        (&helper_random_array_normal8_u16, "-- Normal 8   :"),
        (&helper_random_array_normal13_u16, "-- Normal 13  :"),
        (&helper_random_array_rootdup_u16, "-- RootDup    :"),
        (&helper_random_array_depth_u16, "-- Depth      :"),
    ]
}

#[cfg(target_pointer_width = "16")]
pub fn generators_usize() -> Vec<(&'static dyn Fn(usize) -> Vec<usize>, &'static str)> {
    generators_u16()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<usize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<u16>, Vec<usize>>(arr)
                }
            };

            (Box::leak(Box::new(new_gen)) as &'static dyn Fn(usize) -> Vec<usize>, title)
        })
        .collect()
}
