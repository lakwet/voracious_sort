//! Records sorted by a key: two chains, every key type (each key arrives
//! encoded as a u128 of the same order, `SortKey`): records of at most 8
//! bytes (`small`), larger ones (`large`). They were one copy per key type
//! and record class (r1 ... r64), all identical; factored back (2.1.0).

pub(crate) mod large;
pub(crate) mod small;

/// Sorts `arr` by the encoded keys `enc`, through the chain of its size.
#[inline]
pub(crate) fn sort<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, unstable: bool) {
    if std::mem::size_of::<T>() <= 8 {
        if unstable {
            small::sort_unstable(arr, enc)
        } else {
            small::sort_stable(arr, enc)
        }
    } else if unstable {
        large::sort_unstable(arr, enc)
    } else {
        large::sort_stable(arr, enc)
    }
}
