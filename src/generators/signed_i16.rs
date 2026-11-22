use rand::seq::SliceRandom;
use rand::RngExt;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

// Uniform
pub fn helper_random_array_uniform_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|_| crate::generators::rng().random::<i16>())
        .collect::<Vec<i16>>()
}

// Small
pub fn helper_random_array_small_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|_| crate::generators::rng().random_range(-128..127))
        .collect::<Vec<i16>>()
}

// Ascending sawtooth
pub fn helper_random_array_asc_sawtooth_i16(size: usize) -> Vec<i16> {
    (0..size).into_par_iter().map(|i| i as i16).collect::<Vec<i16>>()
}

// Descending sawtooth
pub fn helper_random_array_desc_sawtooth_i16(size: usize) -> Vec<i16> {
    (0..size).into_par_iter().map(|i| (size - 1 - i) as i16).collect::<Vec<i16>>()
}

// All equals
pub fn helper_random_array_allequals_i16(size: usize) -> Vec<i16> {
    vec![crate::generators::rng().random(); size]
}

// Zipf: frequencies in 1 / rank over min(size, domain) random values
pub fn helper_random_array_zipf_i16(size: usize) -> Vec<i16> {
    super::zipf(size, 1 << 16, || crate::generators::rng().random::<i16>())
}

// Fake Zipf: n / 2 copies of a random value, then n / 4, ... (about
// log2(n) distinct values, geometric frequencies)
pub fn helper_random_array_fakezipf_i16(size: usize) -> Vec<i16> {
    let mut array: Vec<i16> = Vec::with_capacity(size);
    let mut rng = crate::generators::rng();
    let mut quantity = size / 2;
    let mut i = 0;
    let mut value: i16 = 0;
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
fn spread_i16(size: usize, k: usize) -> Vec<i16> {
    let mut rng = crate::generators::rng();
    let values: Vec<i16> = (0..k).map(|_| rng.random::<i16>()).collect();
    (0..size)
        .into_par_iter()
        .map(|_| values[crate::generators::rng().random_range(0..k)])
        .collect()
}

// Two random values
pub fn helper_random_array_binary_i16(size: usize) -> Vec<i16> { spread_i16(size, 2) }

// 16 random values
pub fn helper_random_array_few16_i16(size: usize) -> Vec<i16> { spread_i16(size, 16) }

// 256 random values spread over the domain
pub fn helper_random_array_spread256_i16(size: usize) -> Vec<i16> { spread_i16(size, 256) }

// 4096 random values spread over the domain
pub fn helper_random_array_spread4k_i16(size: usize) -> Vec<i16> { spread_i16(size, 4096) }

// Normal around 0 (both signs), standard deviation `sd`, clamped
fn normal_i16(size: usize, sd: f64) -> Vec<i16> {
    let normal = Normal::new(0.0, sd).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let v: f64 = normal.sample(&mut crate::generators::rng());
            v.clamp(i16::MIN as f64, i16::MAX as f64) as i16
        })
        .collect()
}

// Normal, standard deviation 2^8
pub fn helper_random_array_normal8_i16(size: usize) -> Vec<i16> { normal_i16(size, 255.0) }

// Normal, standard deviation 2^13
pub fn helper_random_array_normal13_i16(size: usize) -> Vec<i16> { normal_i16(size, 8191.0) }

// RootDup (IPS4o): i mod sqrt(n), a repeating ramp of sqrt(n) values
pub fn helper_random_array_rootdup_i16(size: usize) -> Vec<i16> {
    let root = ((size as f64).sqrt() as usize).max(1);
    (0..size).into_par_iter().map(|i| (i % root) as i16).collect()
}

// Audio samples (PCM): 40 % near silence (|x| < 16), the rest Laplacian
// with scale 1000 (speech / music), clamped
pub fn helper_random_array_audio_i16(size: usize) -> Vec<i16> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = crate::generators::rng();
            if rng.random_range(0..10) < 4 {
                rng.random_range(-15..16)
            } else {
                let u: f64 = rng.random_range(-0.5..0.5);
                let v = -1000.0 * u.signum() * (1.0 - 2.0 * u.abs()).ln();
                v.clamp(i16::MIN as f64, i16::MAX as f64) as i16
            }
        })
        .collect()
}

pub fn generators_i16() -> Vec<(&'static dyn Fn(usize) -> Vec<i16>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i16, "-- Unif       :"),
        (&helper_random_array_small_i16, "-- Small      :"),
        (&helper_random_array_asc_sawtooth_i16, "-- Asc Saw    :"),
        (&helper_random_array_desc_sawtooth_i16, "-- Desc Saw   :"),
        (&helper_random_array_allequals_i16, "-- Equal      :"),
        (&helper_random_array_zipf_i16, "-- Zipf       :"),
        (&helper_random_array_fakezipf_i16, "-- FakeZipf   :"),
        (&helper_random_array_binary_i16, "-- Binary     :"),
        (&helper_random_array_few16_i16, "-- Few 16     :"),
        (&helper_random_array_spread256_i16, "-- Spread 256 :"),
        (&helper_random_array_spread4k_i16, "-- Spread 4k  :"),
        (&helper_random_array_normal8_i16, "-- Normal 8   :"),
        (&helper_random_array_normal13_i16, "-- Normal 13  :"),
        (&helper_random_array_rootdup_i16, "-- RootDup    :"),
        (&helper_random_array_audio_i16, "-- Audio      :"),
    ]
}

#[cfg(target_pointer_width = "16")]
pub fn generators_isize() -> Vec<(&'static dyn Fn(usize) -> Vec<isize>, &'static str)> {
    generators_i16()
        .into_iter()
        .map(|(gen, title)| {
            let new_gen = move |size: usize| -> Vec<isize> {
                unsafe {
                    let arr = gen(size);
                    std::mem::transmute::<Vec<i16>, Vec<isize>>(arr)
                }
            };

            (Box::leak(Box::new(new_gen)) as &'static dyn Fn(usize) -> Vec<isize>, title)
        })
        .collect()
}
