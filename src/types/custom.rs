use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::lsd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort;
use super::super::{Radixable};
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub struct Custom {
    min: u32,
    max: u32,
}

impl Custom {
    #![allow(dead_code)]
    pub fn new(min: u32, max: u32) -> Custom {
        Custom { min, max }
    }
}

impl Ord for Custom {
    fn cmp(&self, other: &Custom) -> Ordering {
        (self.max - self.min).cmp(&(other.max - other.min))
    }
}

impl PartialOrd for Custom {
    fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
        (self.max - self.min).partial_cmp(&(other.max - other.min))
    }
}

impl Eq for Custom {}

impl PartialEq for Custom {
    fn eq(&self, other: &Self) -> bool {
        self.max - self.min == other.max - other.min
    }
}

impl Radixable<u32> for Custom {
    type Key = u32;

    #[inline]
    fn key(&self) -> u32 {
        self.max - self.min
    }
    fn voracious_sort(&self, arr: &mut [Custom]) {
        lsd_radixsort(arr, 8);
    }
    fn dlsd_sort(&self, arr: &mut [Custom]) {
        dlsd_radixsort(arr, 8);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MyStruct {
    pub value: i32,
    pub _rank: u8,
}

impl PartialOrd for MyStruct {
    fn partial_cmp(&self, other: &MyStruct) -> Option<Ordering> {
        self.value.partial_cmp(&(other.value))
    }
}

impl PartialEq for MyStruct {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Radixable<i32> for MyStruct {
    type Key = i32;

    #[inline]
    fn key(&self) -> i32 {
        self.value
    }
    fn voracious_sort(&self, arr: &mut [MyStruct]) {
        lsd_radixsort(arr, 8);
    }
    fn dlsd_sort(&self, arr: &mut [MyStruct]) {
        dlsd_radixsort(arr, 8);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructWithF64 {
    pub rate: f64,
}

impl PartialOrd for StructWithF64 {
    fn partial_cmp(&self, other: &StructWithF64) -> Option<Ordering> {
        self.rate.partial_cmp(&(other.rate))
    }
}

impl PartialEq for StructWithF64 {
    fn eq(&self, other: &Self) -> bool {
        self.rate == other.rate
    }
}

impl Radixable<f64> for StructWithF64 {
    type Key = f64;

    #[inline]
    fn key(&self) -> f64 {
        self.rate
    }
    fn voracious_sort(&self, arr: &mut [StructWithF64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [StructWithF64]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Craftf32 {
    pub key: usize,
    pub value: f32,
}

impl PartialOrd for Craftf32 {
    fn partial_cmp(&self, other: &Craftf32) -> Option<Ordering> {
        self.value.partial_cmp(&(other.value))
    }
}

impl PartialEq for Craftf32 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Radixable<f32> for Craftf32 {
    type Key = f32;

    #[inline]
    fn key(&self) -> f32 {
        self.value
    }
    fn voracious_sort(&self, arr: &mut [Craftf32]) {
        if arr.len() <= 300 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [Craftf32]) {
        if arr.len() <= 300 {
            arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
}
