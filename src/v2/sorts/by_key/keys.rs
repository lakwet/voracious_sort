//! Keys of the records, encoded into an unsigned integer of the same order.

/// A key encoded into an unsigned integer with the same order.
pub trait SortKey: Copy {
    fn encode(self) -> u128;

    /// Sorts records by their encoded keys `enc`: the chain of this key type
    /// and the record size (`types::records`). Other key types: the chains
    /// of u128 keys.
    #[doc(hidden)]
    #[inline]
    fn sort_records<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unstable: bool) {
        crate::v2::types::records::u128::sort(arr, enc, unstable);
    }
}

/// The records chains of `$k` keys.
macro_rules! records_of {
    ($k:ident) => {
        #[inline]
        fn sort_records<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unstable: bool) {
            crate::v2::types::records::$k::sort(arr, enc, unstable);
        }
    };
}

macro_rules! key_unsigned {
    ($($t:ident),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                self as u128
            }
            records_of!($t);
        }
    )*};
}
key_unsigned!(u8, u16, u32, u64, u128);

macro_rules! key_signed {
    ($($t:ident => $u:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                ((self as $u) ^ (1 << (<$u>::BITS - 1))) as u128
            }
            records_of!($t);
        }
    )*};
}
key_signed!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128);

// `usize` / `isize`: the chains of the integer of the same width.
impl SortKey for usize {
    #[inline(always)]
    fn encode(self) -> u128 { self as u128 }
    #[cfg(target_pointer_width = "64")]
    records_of!(u64);
    #[cfg(target_pointer_width = "32")]
    records_of!(u32);
    #[cfg(target_pointer_width = "16")]
    records_of!(u16);
}

impl SortKey for isize {
    #[inline(always)]
    fn encode(self) -> u128 { ((self as usize) ^ (1 << (usize::BITS - 1))) as u128 }
    #[cfg(target_pointer_width = "64")]
    records_of!(i64);
    #[cfg(target_pointer_width = "32")]
    records_of!(i32);
    #[cfg(target_pointer_width = "16")]
    records_of!(i16);
}

impl SortKey for char {
    #[inline(always)]
    fn encode(self) -> u128 { self as u32 as u128 }
    records_of!(char);
}

impl SortKey for bool {
    #[inline(always)]
    fn encode(self) -> u128 { self as u128 }
    records_of!(bool);
}

// Floats in total order (`total_cmp`): -0.0 before +0.0, negative NaNs
// first and positive NaNs last (the keys are only compared encoded).
impl SortKey for f32 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i32) >> 31) as u32) | (1 << 31))) as u128
    }
    records_of!(f32);
}

impl SortKey for f64 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i64) >> 63) as u64) | (1 << 63))) as u128
    }
    records_of!(f64);
}
