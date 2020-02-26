use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_boolbool(size: usize) -> Vec<(bool, bool)> {
    let mut rng = thread_rng();
    let mut array: Vec<(bool, bool)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u8 = rng.gen_range(0, 2);
        let v2: u8 = rng.gen_range(0, 2);
        array.push((v1 == 1, v2 == 1));
    }
    array
}

pub fn generators_boolbool(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(bool, bool)>, &'static str)> {
    vec![(&helper_random_array_uniform_boolbool, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_u8u8(size: usize) -> Vec<(u8, u8)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u8, u8)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u8 = rng.gen();
        let v2: u8 = rng.gen();
        array.push((v1, v2));
    }
    array
}

pub fn generators_u8u8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(u8, u8)>, &'static str)> {
    vec![(&helper_random_array_uniform_u8u8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_u8bool(size: usize) -> Vec<(u8, bool)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u8, bool)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u8 = rng.gen();
        let v2: u8 = rng.gen_range(0, 2);
        array.push((v1, v2 == 1));
    }
    array
}

pub fn generators_u8bool(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(u8, bool)>, &'static str)> {
    vec![(&helper_random_array_uniform_u8bool, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_usizef64(size: usize) -> Vec<(usize, f64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(usize, f64)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: usize = rng.gen();
        let v2: f64 = rng.gen();
        array.push((v1, v2));
    }
    array
}

pub fn generators_usizef64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(usize, f64)>, &'static str)> {
    vec![(&helper_random_array_uniform_usizef64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_usizef32(size: usize) -> Vec<(usize, f32)> {
    let mut rng = thread_rng();
    let mut array: Vec<(usize, f32)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: usize = rng.gen();
        let v2: f32 = rng.gen();
        array.push((v1, v2));
    }
    array
}

pub fn generators_usizef32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(usize, f32)>, &'static str)> {
    vec![(&helper_random_array_uniform_usizef32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_boolu8(size: usize) -> Vec<(bool, u8)> {
    let mut rng = thread_rng();
    let mut array: Vec<(bool, u8)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u8 = rng.gen_range(0, 2);
        let v2: u8 = rng.gen();
        array.push((v1 == 1, v2));
    }
    array
}

pub fn generators_boolu8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(bool, u8)>, &'static str)> {
    vec![(&helper_random_array_uniform_boolu8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_boolu16(size: usize) -> Vec<(bool, u16)> {
    let mut rng = thread_rng();
    let mut array: Vec<(bool, u16)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u8 = rng.gen_range(0, 2);
        let v2: u16 = rng.gen();
        array.push((v1 == 1, v2));
    }
    array
}

pub fn generators_boolu16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(bool, u16)>, &'static str)> {
    vec![(&helper_random_array_uniform_boolu16, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_u32u32(size: usize) -> Vec<(u32, u32)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u32, u32)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u32 = rng.gen();
        let v2: u32 = rng.gen();
        array.push((v1, v2));
    }
    array
}

pub fn generators_u32u32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(u32, u32)>, &'static str)> {
    vec![(&helper_random_array_uniform_u32u32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u64 = rng.gen();
        let v2: u64 = rng.gen();
        array.push((v1, v2));
    }
    array
}

// Small
pub fn helper_random_array_small_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u64 = rng.gen_range(0, 1_000_000_000);
        let v2: u64 = rng.gen_range(0, 1_000_000_000);
        array.push((v1, v2));
    }
    array
}

// Normal 10
pub fn helper_random_array_normal_10_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1024.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as u64, v2 as u64));
    }
    array
}

// Normal 20
pub fn helper_random_array_normal_20_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as u64, v2 as u64));
    }
    array
}

// Normal 30
pub fn helper_random_array_normal_30_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as u64, v2 as u64));
    }
    array
}

// Normal 40
pub fn helper_random_array_normal_40_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000_0000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as u64, v2 as u64));
    }
    array
}

// Normal 50
pub fn helper_random_array_normal_50_u64u64(size: usize) -> Vec<(u64, u64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(u64, u64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as u64, v2 as u64));
    }
    array
}

pub fn generators_u64u64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(u64, u64)>, &'static str)> {
    vec![
        (&helper_random_array_uniform_u64u64, "-- Unif       :"),
        (&helper_random_array_small_u64u64, "-- Small      :"),
        (&helper_random_array_normal_10_u64u64, "-- Normal 10  :"),
        (&helper_random_array_normal_20_u64u64, "-- Normal 20  :"),
        (&helper_random_array_normal_30_u64u64, "-- Normal 30  :"),
        (&helper_random_array_normal_40_u64u64, "-- Normal 40  :"),
        (&helper_random_array_normal_50_u64u64, "-- Normal 50  :"),
    ]
}

// Uniform
pub fn helper_random_array_uniform_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: i64 = rng.gen();
        let v2: i64 = rng.gen();
        array.push((v1, v2));
    }
    array
}

// Small
pub fn helper_random_array_small_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: i64 = rng.gen_range(0, 1_000_000_000);
        let v2: i64 = rng.gen_range(0, 1_000_000_000);
        array.push((v1, v2));
    }
    array
}

// Normal 10
pub fn helper_random_array_normal_10_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1024.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as i64, v2 as i64));
    }
    array
}

// Normal 20
pub fn helper_random_array_normal_20_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as i64, v2 as i64));
    }
    array
}

// Normal 30
pub fn helper_random_array_normal_30_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as i64, v2 as i64));
    }
    array
}

// Normal 40
pub fn helper_random_array_normal_40_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000_0000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as i64, v2 as i64));
    }
    array
}

// Normal 50
pub fn helper_random_array_normal_50_i64i64(size: usize) -> Vec<(i64, i64)> {
    let mut rng = thread_rng();
    let mut array: Vec<(i64, i64)> = Vec::with_capacity(size);
    let normal = Normal::new(0.0, 1_000_000_000_000_000.0);
    for _ in 0..size {
        let v1: f64 = normal.sample(&mut rng);
        let v2: f64 = normal.sample(&mut rng);
        array.push((v1 as i64, v2 as i64));
    }
    array
}

pub fn generators_i64i64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<(i64, i64)>, &'static str)> {
    vec![
        (&helper_random_array_uniform_i64i64, "-- Unif       :"),
        (&helper_random_array_small_i64i64, "-- Small      :"),
        (&helper_random_array_normal_10_i64i64, "-- Normal 10  :"),
        (&helper_random_array_normal_20_i64i64, "-- Normal 20  :"),
        (&helper_random_array_normal_30_i64i64, "-- Normal 30  :"),
        (&helper_random_array_normal_40_i64i64, "-- Normal 40  :"),
        (&helper_random_array_normal_50_i64i64, "-- Normal 50  :"),
    ]
}
