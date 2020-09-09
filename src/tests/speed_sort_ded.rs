use super::super::{Radixable, RadixKey};

use super::super::dedicated::lsd_f32::lsd_f32;
use super::super::dedicated::lsd_u32::lsd_u32;
use super::super::dedicated::cs_u16::cs_u16;

use super::super::generators::float_32::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;

use super::test_helpers::helper_sort_aux;

fn speed_dedicated<T, K>(
    name: &str,
    sort: &dyn Fn(&mut [T]) -> (),
    generators: Vec<(&dyn Fn(usize) -> Vec<T>, &'static str)>,
) where
    T: Radixable<K> + std::fmt::Debug,
    K: RadixKey,
{
    let runs = 3;
    let with_check = true;

    let sizes: Vec<usize> = vec![
        100,
        1000,
        10000,
        100_000,
        1_000_000,
        5_000_000,
        10_000_000,
        50_000_000,
        100_000_000,
        500_000_000,
        1_000_000_000,
    ];

    println!("Number of iterations: {}", runs);
    println!("=== Dedicated {} ===", name);
    for size in sizes.iter() {
        println!("Array size: {}", size);
        for (generator, gen_name) in generators.iter() {
            print!("{}", gen_name);
            helper_sort_aux(&|arr: &mut [T]| sort(arr), runs, *size, generator, with_check);
            println!();
        }
    }
}

#[test]
fn speed_dedicated_lsd_f32() {
    speed_dedicated("LSD f32", &lsd_f32, generators_f32());
}

#[test]
fn speed_dedicated_lsd_u32() {
    speed_dedicated("LSD u32", &lsd_u32, generators_u32());
}

#[test]
fn speed_dedicated_cs_u16() {
    speed_dedicated("CS u16", &cs_u16, generators_u16());
}
