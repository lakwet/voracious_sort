use std::ops::{BitAnd, BitOrAssign, Shl, Shr};

pub trait RadixKey {
    type Key: Copy
        + Shl<Output = Self::Key>
        + Shr<Output = Self::Key>
        + BitAnd<Output = Self::Key>
        + BitOrAssign
        + PartialEq
        + PartialOrd
        + Ord
        + Send
        + Sync
        + std::fmt::Display;
    fn into_keytype(&self) -> Self::Key;
    fn type_size(&self) -> usize;
    fn usize_to_keytype(&self, item: usize) -> Self::Key;
    fn keytype_to_usize(&self, item: Self::Key) -> usize;
    fn default_key(&self) -> Self::Key;
    fn one(&self) -> Self::Key;
}

impl RadixKey for bool {
    type Key = u8;
    #[inline]
    fn into_keytype(&self) -> Self::Key {
        if *self {
            1
        } else {
            0
        }
    }
    #[inline]
    fn type_size(&self) -> usize { 8 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u8 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for char {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u32 }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for f32 {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key {
        let submask = 0x8000_0000;
        let casted = (*self).to_bits();

        if casted & submask == submask {
            casted ^ 0xFFFF_FFFF
        } else {
            casted ^ submask
        }
    }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for f64 {
    type Key = u64;
    #[inline]
    fn into_keytype(&self) -> Self::Key {
        let submask = 0x8000_0000_0000_0000;
        let casted = (*self).to_bits();

        if casted & submask == submask {
            casted ^ 0xFFFF_FFFF_FFFF_FFFF
        } else {
            casted ^ submask
        }
    }
    #[inline]
    fn type_size(&self) -> usize { 64 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u64 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for u8 {
    type Key = u8;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self }
    #[inline]
    fn type_size(&self) -> usize { 8 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u8 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "8")]
impl RadixKey for usize {
    type Key = u8;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u8 }
    #[inline]
    fn type_size(&self) -> usize { 8 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u8 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for u16 {
    type Key = u16;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self }
    #[inline]
    fn type_size(&self) -> usize { 16 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u16 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "16")]
impl RadixKey for usize {
    type Key = u16;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u16 }
    #[inline]
    fn type_size(&self) -> usize { 16 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u16 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for u32 {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "32")]
impl RadixKey for usize {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u32 }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for u64 {
    type Key = u64;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self }
    #[inline]
    fn type_size(&self) -> usize { 64 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u64 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "64")]
impl RadixKey for usize {
    type Key = u64;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u64 }
    #[inline]
    fn type_size(&self) -> usize { 64 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u64 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for u128 {
    type Key = u128;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self }
    #[inline]
    fn type_size(&self) -> usize { 128 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u128 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "128")]
impl RadixKey for usize {
    type Key = u128;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u128 }
    #[inline]
    fn type_size(&self) -> usize { 128 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u128 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for i8 {
    type Key = u8;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u8 ^ 0x80 }
    #[inline]
    fn type_size(&self) -> usize { 8 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u8 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "8")]
impl RadixKey for isize {
    type Key = u8;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u8 ^ 0x80 }
    #[inline]
    fn type_size(&self) -> usize { 8 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u8 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for i16 {
    type Key = u16;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u16 ^ 0x8000 }
    #[inline]
    fn type_size(&self) -> usize { 16 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u16 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "16")]
impl RadixKey for isize {
    type Key = u16;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u16 ^ 0x8000 }
    #[inline]
    fn type_size(&self) -> usize { 16 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u16 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for i32 {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u32 ^ 0x8000_0000 }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "32")]
impl RadixKey for isize {
    type Key = u32;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u32 ^ 0x8000_0000 }
    #[inline]
    fn type_size(&self) -> usize { 32 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u32 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for i64 {
    type Key = u64;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u64 ^ 0x8000_0000_0000_0000 }
    #[inline]
    fn type_size(&self) -> usize { 64 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u64 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "64")]
impl RadixKey for isize {
    type Key = u64;
    #[inline]
    fn into_keytype(&self) -> Self::Key { *self as u64 ^ 0x8000_0000_0000_0000 }
    #[inline]
    fn type_size(&self) -> usize { 64 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u64 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

impl RadixKey for i128 {
    type Key = u128;
    #[inline]
    fn into_keytype(&self) -> Self::Key {
        *self as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000
    }
    #[inline]
    fn type_size(&self) -> usize { 128 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u128 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}

#[cfg(target_pointer_width = "128")]
impl RadixKey for isize {
    type Key = u128;
    #[inline]
    fn into_keytype(&self) -> Self::Key {
        *self as u128 ^ 0x8000_0000_0000_0000_0000_0000_0000_0000
    }
    #[inline]
    fn type_size(&self) -> usize { 128 }
    #[inline]
    fn usize_to_keytype(&self, item: usize) -> Self::Key { item as u128 }
    #[inline]
    fn keytype_to_usize(&self, item: Self::Key) -> usize { item as usize }
    #[inline]
    fn default_key(&self) -> Self::Key { 0 }
    #[inline]
    fn one(&self) -> Self::Key { 1 }
}
