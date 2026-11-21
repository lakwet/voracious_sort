//! The records of the generators sortable by voracious 1.x (its trait on
//! the record, `Radixable`), for the benchmarks and the v1 tests.

use crate::generators::records::Rec;

macro_rules! v1_rec {
    ($($k:ty),*) => {$(
        impl<const W: usize> crate::v1::Radixable<$k> for Rec<$k, W> {
            type Key = $k;
            #[inline]
            fn key(&self) -> $k {
                self.key
            }
        }
    )*};
}
v1_rec!(bool, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64, char);
