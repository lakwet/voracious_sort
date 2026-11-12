//! Keys of the records, encoded into an unsigned integer of the same order.

/// A key encoded into an unsigned integer with the same order.
pub trait SortKey: Copy {
    fn encode(self) -> u128;
}

macro_rules! key_unsigned {
    ($($t:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                self as u128
            }
        }
    )*};
}
key_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! key_signed {
    ($($t:ty => $u:ty),*) => {$(
        impl SortKey for $t {
            #[inline(always)]
            fn encode(self) -> u128 {
                ((self as $u) ^ (1 << (<$u>::BITS - 1))) as u128
            }
        }
    )*};
}
key_signed!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128, isize => usize);

impl SortKey for char {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u32 as u128
    }
}

impl SortKey for bool {
    #[inline(always)]
    fn encode(self) -> u128 {
        self as u128
    }
}

// Floats in total order (`total_cmp`): -0.0 before +0.0, NaN not handled.
impl SortKey for f32 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i32) >> 31) as u32) | (1 << 31))) as u128
    }
}

impl SortKey for f64 {
    #[inline(always)]
    fn encode(self) -> u128 {
        let b = self.to_bits();
        (b ^ ((((b as i64) >> 63) as u64) | (1 << 63))) as u128
    }
}
