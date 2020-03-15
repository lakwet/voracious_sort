use rand::{thread_rng, Rng};

use super::super::types::custom::*;

// Uniform
pub fn helper_random_array_uniform_structbool(size: usize) -> Vec<StructBool> {
    let mut rng = thread_rng();
    let mut array: Vec<StructBool> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: bool = rng.gen();
        array.push(StructBool { value, other });
    }
    array
}

pub fn generators_structbool(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructBool>, &'static str)> {
    vec![(&helper_random_array_uniform_structbool, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structchar(size: usize) -> Vec<StructChar> {
    let mut rng = thread_rng();
    let mut array: Vec<StructChar> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: char = rng.gen();
        array.push(StructChar { value, other });
    }
    array
}

pub fn generators_structchar(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructChar>, &'static str)> {
    vec![(&helper_random_array_uniform_structchar, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structf32(size: usize) -> Vec<StructF32> {
    let mut rng = thread_rng();
    let mut array: Vec<StructF32> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: f32 = rng.gen();
        array.push(StructF32 { value, other });
    }
    array
}

pub fn generators_structf32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructF32>, &'static str)> {
    vec![(&helper_random_array_uniform_structf32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structf64(size: usize) -> Vec<StructF64> {
    let mut rng = thread_rng();
    let mut array: Vec<StructF64> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: f64 = rng.gen();
        array.push(StructF64 { value, other });
    }
    array
}

pub fn generators_structf64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructF64>, &'static str)> {
    vec![(&helper_random_array_uniform_structf64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu8(size: usize) -> Vec<StructU8> {
    let mut rng = thread_rng();
    let mut array: Vec<StructU8> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: u8 = rng.gen();
        array.push(StructU8 { value, other });
    }
    array
}

pub fn generators_structu8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU8>, &'static str)> {
    vec![(&helper_random_array_uniform_structu8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu16(size: usize) -> Vec<StructU16> {
    let mut rng = thread_rng();
    let mut array: Vec<StructU16> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: u16 = rng.gen();
        array.push(StructU16 { value, other });
    }
    array
}

pub fn generators_structu16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU16>, &'static str)> {
    vec![(&helper_random_array_uniform_structu16, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu32(size: usize) -> Vec<StructU32> {
    let mut rng = thread_rng();
    let mut array: Vec<StructU32> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: u32 = rng.gen();
        array.push(StructU32 { value, other });
    }
    array
}

pub fn generators_structu32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU32>, &'static str)> {
    vec![(&helper_random_array_uniform_structu32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu64(size: usize) -> Vec<StructU64> {
    let mut rng = thread_rng();
    let mut array: Vec<StructU64> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: u64 = rng.gen();
        array.push(StructU64 { value, other });
    }
    array
}

pub fn generators_structu64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU64>, &'static str)> {
    vec![(&helper_random_array_uniform_structu64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu128(size: usize) -> Vec<StructU128> {
    let mut rng = thread_rng();
    let mut array: Vec<StructU128> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let v1: u64 = rng.gen();
        let v2: u64 = rng.gen();
        let value: u128 = ((v1 as u128) << 64) | (v2 as u128);
        array.push(StructU128 { value, other });
    }
    array
}

pub fn generators_structu128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU128>, &'static str)> {
    vec![(&helper_random_array_uniform_structu128, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi8(size: usize) -> Vec<StructI8> {
    let mut rng = thread_rng();
    let mut array: Vec<StructI8> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: i8 = rng.gen();
        array.push(StructI8 { value, other });
    }
    array
}

pub fn generators_structi8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI8>, &'static str)> {
    vec![(&helper_random_array_uniform_structi8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi16(size: usize) -> Vec<StructI16> {
    let mut rng = thread_rng();
    let mut array: Vec<StructI16> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: i16 = rng.gen();
        array.push(StructI16 { value, other });
    }
    array
}

pub fn generators_structi16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI16>, &'static str)> {
    vec![(&helper_random_array_uniform_structi16, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi32(size: usize) -> Vec<StructI32> {
    let mut rng = thread_rng();
    let mut array: Vec<StructI32> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: i32 = rng.gen();
        array.push(StructI32 { value, other });
    }
    array
}

pub fn generators_structi32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI32>, &'static str)> {
    vec![(&helper_random_array_uniform_structi32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi64(size: usize) -> Vec<StructI64> {
    let mut rng = thread_rng();
    let mut array: Vec<StructI64> = Vec::with_capacity(size);
    for _ in 0..size {
        let other: isize = rng.gen();
        let value: i64 = rng.gen();
        array.push(StructI64 { value, other });
    }
    array
}

pub fn generators_structi64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI64>, &'static str)> {
    vec![(&helper_random_array_uniform_structi64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi128(size: usize) -> Vec<StructI128> {
    let mut rng = thread_rng();
    let mut array: Vec<StructI128> = Vec::with_capacity(size);
    for _ in 0..size {
        unsafe {
            let other: isize = rng.gen();
            let v1: u64 = rng.gen();
            let v2: u64 = rng.gen();
            let value: u128 = ((v1 as u128) << 64) | (v2 as u128);
            let value: i128 = std::mem::transmute::<u128, i128>(value);
            array.push(StructI128 { value, other });
        }
    }
    array
}

pub fn generators_structi128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI128>, &'static str)> {
    vec![(&helper_random_array_uniform_structi128, "-- Unif       :")]
}
