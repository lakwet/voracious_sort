use super::super::{Radixable, RadixableForContainer};
use super::utils::Params;

fn counting_sort_aux<T>(arr: &mut [T], p: Params)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    let mut histogram = vec![0; p.radix_range];
    let mask = arr.get_default_mask(&p);

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    for q in 0..quotient {
        unsafe {
            let i = q * 4;
            let bucket0 = arr.get_unchecked(i).get_key(mask, 0);
            let bucket1 = arr.get_unchecked(i + 1).get_key(mask, 0);
            let bucket2 = arr.get_unchecked(i + 2).get_key(mask, 0);
            let bucket3 = arr.get_unchecked(i + 3).get_key(mask, 0);
            histogram[bucket0] += 1;
            histogram[bucket1] += 1;
            histogram[bucket2] += 1;
            histogram[bucket3] += 1;
        }
    }
    let offset = quotient * 4;
    for i in 0..remainder {
        unsafe {
            let bucket = arr.get_unchecked(offset + i).get_key(mask, 0);
            histogram[bucket] += 1;
        }
    }

    let dummy = arr[0];

    let mut position = 0;
    histogram.iter().enumerate().for_each(|(value, count)| {
        let quotient = *count / 4;
        let remainder = count % 4;
        for _ in 0..quotient {
            unsafe {
                *arr.get_unchecked_mut(position) = dummy.to_generic(value);
                *arr.get_unchecked_mut(position + 1) = dummy.to_generic(value);
                *arr.get_unchecked_mut(position + 2) = dummy.to_generic(value);
                *arr.get_unchecked_mut(position + 3) = dummy.to_generic(value);
            }
            position += 4;
        }
        for _ in 0..remainder {
            unsafe {
                *arr.get_unchecked_mut(position) = dummy.to_generic(value);
                position += 1;
            }
        }
    });
}

pub fn counting_sort<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    let offset = 0;
    let level = 0;
    let max_level = 1;
    let params = Params::new(level, radix, offset, max_level);

    counting_sort_aux(arr, params);
}
