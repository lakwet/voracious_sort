use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal, Pareto};
use rayon::prelude::*;

use super::super::types::custom::*;

// Uniform
pub fn helper_random_array_uniform_structbool(size: usize) -> Vec<StructBool> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<bool>();
            StructBool { value, other }
        })
        .collect::<Vec<StructBool>>()
}

pub fn generators_structbool(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructBool>, &'static str)> {
    vec![(&helper_random_array_uniform_structbool, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structchar(size: usize) -> Vec<StructChar> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<char>();
            StructChar { value, other }
        })
        .collect::<Vec<StructChar>>()
}

pub fn generators_structchar(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructChar>, &'static str)> {
    vec![(&helper_random_array_uniform_structchar, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structf32(size: usize) -> Vec<StructF32> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<f32>();
            StructF32 { value, other }
        })
        .collect::<Vec<StructF32>>()
}

// 10^9
pub fn helper_random_array_109_structf32(size: usize) -> Vec<StructF32> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen_range(-1_000_000.0, 1_000_000.0);
            StructF32 { value, other }
        })
        .collect::<Vec<StructF32>>()
}

fn helper_pareto_structf32(size: usize, arg: f32) -> Vec<StructF32> {
    let pareto = Pareto::new(0.1, arg).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| StructF32 {
            value: pareto.sample(&mut thread_rng()),
            other: thread_rng().gen::<isize>(),
        })
        .collect::<Vec<StructF32>>()
}

// Pareto
pub fn helper_random_array_pareto075_structf32(size: usize) -> Vec<StructF32> {
    helper_pareto_structf32(size, 0.75)
}

// Pareto
pub fn helper_random_array_pareto100_structf32(size: usize) -> Vec<StructF32> {
    helper_pareto_structf32(size, 1.0)
}

// Pareto
pub fn helper_random_array_pareto200_structf32(size: usize) -> Vec<StructF32> {
    helper_pareto_structf32(size, 2.0)
}

fn helper_normal_structf32(
    size: usize,
    standard_deviation: f32,
) -> Vec<StructF32> {
    let normal = Normal::new(0.0, standard_deviation).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| StructF32 {
            value: normal.sample(&mut thread_rng()),
            other: thread_rng().gen::<isize>(),
        })
        .collect::<Vec<StructF32>>()
}

// Normale(0, 2^10)
pub fn helper_random_array_normale_10_structf32(size: usize) -> Vec<StructF32> {
    helper_normal_structf32(size, 1024.0)
}

// Normale(0, 2^20)
pub fn helper_random_array_normale_20_structf32(size: usize) -> Vec<StructF32> {
    helper_normal_structf32(size, 1_000_000.0)
}

// Normale(0, 2^30)
pub fn helper_random_array_normale_30_structf32(size: usize) -> Vec<StructF32> {
    helper_normal_structf32(size, 1_000_000_000.0)
}

pub fn generators_structf32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructF32>, &'static str)> {
    vec![
        (&helper_random_array_uniform_structf32, "-- Unif       :"),
        (&helper_random_array_109_structf32, "-- +/-10^9   :"),
        (&helper_random_array_pareto075_structf32, "-- Pareto 0.75:"),
        (&helper_random_array_pareto100_structf32, "-- Pareto 1.00:"),
        (&helper_random_array_pareto200_structf32, "-- Pareto 2.00:"),
        (&helper_random_array_normale_10_structf32, "-- Normale 10 :"),
        (&helper_random_array_normale_20_structf32, "-- Normale 20 :"),
        (&helper_random_array_normale_30_structf32, "-- Normale 30 :"),
    ]
}

// Uniform
pub fn helper_random_array_uniform_structf64(size: usize) -> Vec<StructF64> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<f64>();
            StructF64 { value, other }
        })
        .collect::<Vec<StructF64>>()
}

// 10^9
pub fn helper_random_array_109_structf64(size: usize) -> Vec<StructF64> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen_range(-1_000_000.0, 1_000_000.0);
            StructF64 { value, other }
        })
        .collect::<Vec<StructF64>>()
}

pub fn generators_structf64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructF64>, &'static str)> {
    vec![
        (&helper_random_array_uniform_structf64, "-- Unif       :"),
        (&helper_random_array_109_structf64, "-- +/-10^9   :"),
    ]
}

// Uniform
pub fn helper_random_array_uniform_structu8(size: usize) -> Vec<StructU8> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<u8>();
            StructU8 { value, other }
        })
        .collect::<Vec<StructU8>>()
}

pub fn generators_structu8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU8>, &'static str)> {
    vec![(&helper_random_array_uniform_structu8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu16(size: usize) -> Vec<StructU16> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<u16>();
            StructU16 { value, other }
        })
        .collect::<Vec<StructU16>>()
}

pub fn generators_structu16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU16>, &'static str)> {
    vec![(&helper_random_array_uniform_structu16, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu32(size: usize) -> Vec<StructU32> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<u32>();
            StructU32 { value, other }
        })
        .collect::<Vec<StructU32>>()
}

pub fn generators_structu32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU32>, &'static str)> {
    vec![(&helper_random_array_uniform_structu32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu64(size: usize) -> Vec<StructU64> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<u64>();
            StructU64 { value, other }
        })
        .collect::<Vec<StructU64>>()
}

pub fn generators_structu64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU64>, &'static str)> {
    vec![(&helper_random_array_uniform_structu64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structu128(size: usize) -> Vec<StructU128> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value: u128 = rng.gen::<u128>();
            StructU128 { value, other }
        })
        .collect::<Vec<StructU128>>()
}

pub fn generators_structu128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructU128>, &'static str)> {
    vec![(&helper_random_array_uniform_structu128, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi8(size: usize) -> Vec<StructI8> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<i8>();
            StructI8 { value, other }
        })
        .collect::<Vec<StructI8>>()
}

pub fn generators_structi8(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI8>, &'static str)> {
    vec![(&helper_random_array_uniform_structi8, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi16(size: usize) -> Vec<StructI16> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<i16>();
            StructI16 { value, other }
        })
        .collect::<Vec<StructI16>>()
}

pub fn generators_structi16(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI16>, &'static str)> {
    vec![(&helper_random_array_uniform_structi16, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi32(size: usize) -> Vec<StructI32> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<i32>();
            StructI32 { value, other }
        })
        .collect::<Vec<StructI32>>()
}

pub fn generators_structi32(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI32>, &'static str)> {
    vec![(&helper_random_array_uniform_structi32, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi64(size: usize) -> Vec<StructI64> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value = rng.gen::<i64>();
            StructI64 { value, other }
        })
        .collect::<Vec<StructI64>>()
}

pub fn generators_structi64(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI64>, &'static str)> {
    vec![(&helper_random_array_uniform_structi64, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structi128(size: usize) -> Vec<StructI128> {
    (0..size)
        .into_par_iter()
        .map(|_| {
            let mut rng = thread_rng();
            let other = rng.gen::<isize>();
            let value: i128 = rng.gen::<i128>();
            StructI128 { value, other }
        })
        .collect::<Vec<StructI128>>()
}

pub fn generators_structi128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructI128>, &'static str)> {
    vec![(&helper_random_array_uniform_structi128, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structisize(
    size: usize,
) -> Vec<StructIsize> {
    (0..size)
        .into_par_iter()
        .map(|_| StructIsize {
            value: thread_rng().gen::<isize>(),
            other: thread_rng().gen::<isize>(),
        })
        .collect::<Vec<StructIsize>>()
}

pub fn generators_structisize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructIsize>, &'static str)> {
    vec![(&helper_random_array_uniform_structisize, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structusize(
    size: usize,
) -> Vec<StructUsize> {
    (0..size)
        .into_par_iter()
        .map(|_| StructUsize {
            value: thread_rng().gen::<usize>(),
            other: thread_rng().gen::<isize>(),
        })
        .collect::<Vec<StructUsize>>()
}

pub fn generators_structusize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructUsize>, &'static str)> {
    vec![(&helper_random_array_uniform_structusize, "-- Unif       :")]
}

// Uniform
pub fn helper_random_array_uniform_structusizeusize(
  size: usize,
) -> Vec<StructUsizeUsize> {
  (0..size)
      .into_par_iter()
      .map(|_| StructUsizeUsize {
          a: thread_rng().gen::<usize>(),
          b: thread_rng().gen::<usize>(),
      })
      .collect::<Vec<StructUsizeUsize>>()
}

pub fn generators_structusizeusize(
) -> Vec<(&'static dyn Fn(usize) -> Vec<StructUsizeUsize>, &'static str)> {
  vec![(&helper_random_array_uniform_structusizeusize, "-- Unif       :")]
}
