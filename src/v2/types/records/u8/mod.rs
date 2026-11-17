//! Records sorted by u8 keys: one chain per record size (a record goes to
//! the smallest class that holds it; the last class takes all larger ones).

pub(crate) mod r1;
pub(crate) mod r16;
pub(crate) mod r2;
pub(crate) mod r32;
pub(crate) mod r4;
pub(crate) mod r64;
pub(crate) mod r8;

/// Sorts `arr` by the encoded keys `enc`, through the chain of its size.
#[inline]
pub(crate) fn sort<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unstable: bool) {
    let size = std::mem::size_of::<T>();
    if size <= 1 {
        if unstable {
            r1::sort_unstable(arr, enc)
        } else {
            r1::sort_stable(arr, enc)
        }
    } else if size <= 2 {
        if unstable {
            r2::sort_unstable(arr, enc)
        } else {
            r2::sort_stable(arr, enc)
        }
    } else if size <= 4 {
        if unstable {
            r4::sort_unstable(arr, enc)
        } else {
            r4::sort_stable(arr, enc)
        }
    } else if size <= 8 {
        if unstable {
            r8::sort_unstable(arr, enc)
        } else {
            r8::sort_stable(arr, enc)
        }
    } else if size <= 16 {
        if unstable {
            r16::sort_unstable(arr, enc)
        } else {
            r16::sort_stable(arr, enc)
        }
    } else if size <= 32 {
        if unstable {
            r32::sort_unstable(arr, enc)
        } else {
            r32::sort_stable(arr, enc)
        }
    } else if unstable {
        r64::sort_unstable(arr, enc)
    } else {
        r64::sort_stable(arr, enc)
    }
}
