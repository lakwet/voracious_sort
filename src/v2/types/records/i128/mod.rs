//! Records sorted by i128 keys: one chain per record size (a record goes to
//! the smallest class that holds it; the last class takes all larger ones).

pub(crate) mod r16;
pub(crate) mod r32;
pub(crate) mod r64;

/// Sorts `arr` by the encoded keys `enc`, through the chain of its size.
#[inline]
pub(crate) fn sort<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unstable: bool) {
    let size = std::mem::size_of::<T>();
    if size <= 16 {
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
