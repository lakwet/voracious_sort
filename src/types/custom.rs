use std::cmp::Ordering;

use super::super::Radixable;

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructBool {
    pub value: bool,
    pub other: isize,
}
impl PartialOrd for StructBool {
    fn partial_cmp(&self, other: &StructBool) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructBool {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<bool> for StructBool {
    type Key = bool;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructChar {
    pub value: char,
    pub other: isize,
}
impl PartialOrd for StructChar {
    fn partial_cmp(&self, other: &StructChar) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructChar {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<char> for StructChar {
    type Key = char;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructF32 {
    pub value: f32,
    pub other: isize,
}
impl PartialOrd for StructF32 {
    fn partial_cmp(&self, other: &StructF32) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructF32 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<f32> for StructF32 {
    type Key = f32;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructF64 {
    pub value: f64,
    pub other: isize,
}
impl PartialOrd for StructF64 {
    fn partial_cmp(&self, other: &StructF64) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructF64 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<f64> for StructF64 {
    type Key = f64;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructU8 {
    pub value: u8,
    pub other: isize,
}
impl PartialOrd for StructU8 {
    fn partial_cmp(&self, other: &StructU8) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructU8 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<u8> for StructU8 {
    type Key = u8;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructU16 {
    pub value: u16,
    pub other: isize,
}
impl PartialOrd for StructU16 {
    fn partial_cmp(&self, other: &StructU16) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructU16 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<u16> for StructU16 {
    type Key = u16;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructU32 {
    pub value: u32,
    pub other: isize,
}
impl PartialOrd for StructU32 {
    fn partial_cmp(&self, other: &StructU32) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructU32 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<u32> for StructU32 {
    type Key = u32;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructU64 {
    pub value: u64,
    pub other: isize,
}
impl PartialOrd for StructU64 {
    fn partial_cmp(&self, other: &StructU64) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructU64 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<u64> for StructU64 {
    type Key = u64;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructU128 {
    pub value: u128,
    pub other: isize,
}
impl PartialOrd for StructU128 {
    fn partial_cmp(&self, other: &StructU128) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructU128 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<u128> for StructU128 {
    type Key = u128;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructI8 {
    pub value: i8,
    pub other: isize,
}
impl PartialOrd for StructI8 {
    fn partial_cmp(&self, other: &StructI8) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructI8 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<i8> for StructI8 {
    type Key = i8;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructI16 {
    pub value: i16,
    pub other: isize,
}
impl PartialOrd for StructI16 {
    fn partial_cmp(&self, other: &StructI16) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructI16 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<i16> for StructI16 {
    type Key = i16;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructI32 {
    pub value: i32,
    pub other: isize,
}
impl PartialOrd for StructI32 {
    fn partial_cmp(&self, other: &StructI32) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructI32 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<i32> for StructI32 {
    type Key = i32;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructI64 {
    pub value: i64,
    pub other: isize,
}
impl PartialOrd for StructI64 {
    fn partial_cmp(&self, other: &StructI64) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructI64 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<i64> for StructI64 {
    type Key = i64;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructI128 {
    pub value: i128,
    pub other: isize,
}
impl PartialOrd for StructI128 {
    fn partial_cmp(&self, other: &StructI128) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructI128 {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<i128> for StructI128 {
    type Key = i128;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructIsize {
    pub value: isize,
    pub other: isize,
}
impl PartialOrd for StructIsize {
    fn partial_cmp(&self, other: &StructIsize) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructIsize {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<isize> for StructIsize {
    type Key = isize;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}

// Struct

#[derive(Copy, Clone, Debug)]
pub struct StructUsize {
    pub value: usize,
    pub other: isize,
}
impl PartialOrd for StructUsize {
    fn partial_cmp(&self, other: &StructUsize) -> Option<Ordering> {
        (self.value).partial_cmp(&(other.value))
    }
}
impl PartialEq for StructUsize {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl Radixable<usize> for StructUsize {
    type Key = usize;
    #[inline]
    fn key(&self) -> Self::Key { self.value }
}
