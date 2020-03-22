// use super::super::{RadixKey, Radixable};

#[inline]
fn boolean_sort_aux(arr: &mut [bool], shift: usize, count: usize, value: bool) {
    let quotient = count / 4;
    let remainder = count % 4;
    for q in 0..quotient {
        unsafe {
            let i = shift + (q * 4);
            *arr.get_unchecked_mut(i) = value;
            *arr.get_unchecked_mut(i + 1) = value;
            *arr.get_unchecked_mut(i + 2) = value;
            *arr.get_unchecked_mut(i + 3) = value;
        }
    }
    let offset = quotient * 4;
    for i in 0..remainder {
        unsafe {
            *arr.get_unchecked_mut(shift + offset + i) = value;
        }
    }
}

/// # Boolean sort
///
/// A dedicated sort for boolean.
pub fn boolean_sort(arr: &mut [bool]) {
    let mut count_false = 0;

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;

    for q in 0..quotient {
        unsafe {
            let i = q * 4;
            let b0 = arr.get_unchecked(i);
            let b1 = arr.get_unchecked(i + 1);
            let b2 = arr.get_unchecked(i + 2);
            let b3 = arr.get_unchecked(i + 3);
            count_false += if !b0 { 1 } else { 0 };
            count_false += if !b1 { 1 } else { 0 };
            count_false += if !b2 { 1 } else { 0 };
            count_false += if !b3 { 1 } else { 0 };
        }
    }

    let offset = quotient * 4;
    for i in 0..remainder {
        unsafe {
            if !arr.get_unchecked(offset + i) {
                count_false += 1;
            }
        }
    }

    if count_false == arr.len() || count_false == 0 {
        return;
    }

    boolean_sort_aux(arr, 0, count_false, false);
    boolean_sort_aux(arr, count_false, arr.len() - count_false, true);
}
