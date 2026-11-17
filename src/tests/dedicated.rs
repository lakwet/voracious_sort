use super::super::dedicated::cs_u16::cs_u16;
use super::super::dedicated::lsd_f32::lsd_f32;
use super::super::dedicated::lsd_u32::lsd_u32;

use super::super::generators::float_32::*;
use super::super::generators::unsigned_u16::*;
use super::super::generators::unsigned_u32::*;

use super::sorts::helper_sort;

#[test]
fn test_ded_sort_lsd_f32() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| lsd_f32(a), generators_f32(), *size);
    }
}

#[test]
fn test_ded_sort_lsd_u32() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| lsd_u32(a), generators_u32(), *size);
    }
}

#[test]
fn test_ded_sort_counting_sort_u16() {
    for size in [0, 1, 100_000].iter() {
        helper_sort(false, &|a| cs_u16(a), generators_u16(), *size);
    }
}
