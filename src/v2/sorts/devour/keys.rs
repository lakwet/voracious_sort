//! Integers seen as their unsigned bits, and the key maps (identity for
//! unsigned integers, flipped sign bit for signed ones).

/// Unsigned key bits seen through `u64` (the counting sorts).
pub(crate) trait Key: Copy {
    const MAX_U64: u64;
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
}

macro_rules! key {
    ($u:ty) => {
        impl Key for $u {
            const MAX_U64: u64 = <$u>::MAX as u64;
            #[inline(always)]
            fn to_u64(self) -> u64 {
                self as u64
            }
            #[inline(always)]
            fn from_u64(v: u64) -> Self {
                v as $u
            }
        }
    };
}
key!(u8);
key!(u16);
key!(u32);
key!(u64);

/// The unsigned integer of the same size.
pub(crate) trait BitsOf {
    type Bits: Key + Ord;
}

macro_rules! bits_of {
    ($($t:ty => $u:ty),*) => {
        $(impl BitsOf for $t {
            type Bits = $u;
        })*
    };
}
bits_of!(u8 => u8, i8 => u8, u16 => u16, i16 => u16, u32 => u32, i32 => u32, u64 => u64, i64 => u64);

/// The integers as their unsigned bits.
#[inline(always)]
pub(crate) fn bits_of<T: BitsOf>(arr: &mut [T]) -> &mut [T::Bits] {
    // SAFETY: `BitsOf` pairs integers of the same size and alignment, every
    // bit pattern valid for both; the slice reborrows `arr` exclusively.
    unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut T::Bits, arr.len()) }
}

macro_rules! key_maps {
    ($($id:ident, $flip:ident, $u:ty);*) => {
        $(
            #[inline(always)]
            pub(crate) fn $id(k: $u) -> $u {
                k
            }
            #[inline(always)]
            pub(crate) fn $flip(k: $u) -> $u {
                k ^ (1 << (<$u>::BITS - 1))
            }
        )*
    };
}
key_maps!(id8, flip8, u8; id16, flip16, u16; id32, flip32, u32; id64, flip64, u64);
