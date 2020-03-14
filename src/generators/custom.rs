use rand::{thread_rng, Rng};

use super::super::types::custom::{Custom, MyStruct, StructWithF64, Craftf32};

// Uniform
pub fn helper_random_array_uniform_custom(size: usize) -> Vec<Custom> {
    let mut rng = thread_rng();
    let mut array: Vec<Custom> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u32 = rng.gen();
        let v2: u32 = rng.gen();
        let min = u32::min(v1, v2);
        let max = u32::max(v1, v2);
        array.push(Custom::new(min, max));
    }
    array
}

pub fn generators_custom(
) -> Vec<(&'static dyn Fn(usize) -> Vec<Custom>, &'static str)> {
    vec![(&helper_random_array_uniform_custom, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_my_struct(size: usize) -> Vec<MyStruct> {
    let mut rng = thread_rng();
    let mut array: Vec<MyStruct> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: i32 = rng.gen();
        let _rank: u8 = rng.gen();
        array.push(MyStruct { value, _rank });
    }
    array
}

pub fn generators_mystruct(
) -> Vec<(&'static dyn Fn(usize) -> Vec<MyStruct>, &'static str)> {
    vec![(&helper_random_array_uniform_my_struct, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structwithf64(
    size: usize,
) -> Vec<StructWithF64> {
    let mut rng = thread_rng();
    let mut array: Vec<StructWithF64> = Vec::with_capacity(size);
    for _ in 0..size {
        let value: f64 = rng.gen();
        array.push(StructWithF64 { rate: value });
    }
    array
}

pub fn generators_structwithf64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructWithF64>, &'static str)> {
    vec![(
        &helper_random_array_uniform_structwithf64,
        "-- Unif       :",
    )]
}

// Uniform
pub fn helper_random_array_uniform_craftf32(
    size: usize,
) -> Vec<Craftf32> {
    let mut rng = thread_rng();
    let mut array: Vec<Craftf32> = Vec::with_capacity(size);
    for _ in 0..size {
        let key: usize = rng.gen();
        let value: f32 = rng.gen();
        array.push(Craftf32 { key, value });
    }
    array
}

pub fn generators_craftf32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<Craftf32>, &'static str)> {
    vec![(
        &helper_random_array_uniform_craftf32,
        "-- Unif       :",
    )]
}
