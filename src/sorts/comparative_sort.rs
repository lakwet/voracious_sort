use super::super::{RadixKey, Radixable};
use super::utils::Params;

const TRY_THRESHOLD: u8 = 32;

/// # Insertion sort
///
/// An implementation of the
/// [Insertion sort](https://en.wikipedia.org/wiki/Insertion_sort)
/// algorithm.
pub fn insertion_sort<T: PartialOrd>(arr: &mut [T]) {
    for i in 1..arr.len() {
        if arr[i - 1] > arr[i] {
            let mut j = i;
            while j > 0 && arr[j - 1] > arr[j] {
                arr.swap(j - 1, j);
                j -= 1;
            }
        }
    }
}

fn insertion_sort_start_at<T: PartialOrd>(arr: &mut [T], start: usize) {
    for i in start..arr.len() {
        if arr[i - 1] > arr[i] {
            let mut j = i;
            while j > 0 && arr[j - 1] > arr[j] {
                arr.swap(j - 1, j);
                j -= 1;
            }
        }
    }
}

fn find_end_of_bucket<T, K>(arr: &mut [T], start: usize, p: &Params) -> usize
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let dummy = arr[0];
    let mask = dummy.mask_for_high_bits(p.radix, p.offset, p.max_level);
    let high_bits = arr[start].into_key_type() & mask;

    let mut jump = 32;
    let mut i = start;
    let mut j = start + jump;

    if j >= arr.len() {
        j = arr.len() - 1;
    }

    while high_bits == (arr[j].into_key_type() & mask) {
        jump *= 2;
        i = j;
        j += jump;
        if j >= arr.len() {
            j = arr.len() - 1;

            if high_bits == arr[j].into_key_type() & mask {
                return j + 1;
            }
        }
    }

    loop {
        let mid = (i + j) / 2;
        let t_high_bits = arr[mid].into_key_type() & mask;
        if high_bits == t_high_bits {
            if j == i + 1 {
                return j;
            }
            i = mid;
        } else if t_high_bits > high_bits {
            j = mid;
        }
    }
}

fn find_start_of_bucket<T, K>(arr: &mut [T], start: usize, p: &Params) -> usize
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let dummy = arr[0];
    let mask = dummy.mask_for_high_bits(p.radix, p.offset, p.max_level);
    let high_bits = arr[start].into_key_type() & mask;

    let mut jump = 32;
    let mut i = start;

    let mut j = if jump > start { 0 } else { start - jump };

    while high_bits == (arr[j].into_key_type() & mask) {
        jump *= 2;
        i = j;
        j = if jump > j { 0 } else { j - jump };
        if j == 0 && high_bits == arr[j].into_key_type() & mask {
            return 0;
        }
    }

    loop {
        let mid = (i + j) / 2;
        let t_high_bits = arr[mid].into_key_type() & mask;

        if high_bits == t_high_bits {
            i = mid;
        } else if high_bits > t_high_bits {
            if j == i - 1 {
                return i;
            }
            j = mid;
        }
    }
}

pub fn insertion_sort_try<T, K>(arr: &mut [T], p: &Params) -> Vec<(usize, usize)>
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let dummy = arr[0];
    let mask = dummy.mask_for_high_bits(p.radix, p.offset, p.max_level);

    let mut unsorted_parts = Vec::new();

    let mut i = 1;
    let mut high_bits = arr[0].into_key_type() & mask;
    let mut misplaced_count = 0;
    loop {
        if arr[i - 1] > arr[i] {
            let current_high_bits = arr[i].into_key_type() & mask;
            if current_high_bits == high_bits {
                misplaced_count += 1;
            } else {
                high_bits = current_high_bits;
                misplaced_count = 1;
            }

            let mut j = i;
            while j > 0 && arr[j - 1] > arr[j] {
                arr.swap(j - 1, j);
                j -= 1;
            }

            if misplaced_count > TRY_THRESHOLD {
                let end = find_end_of_bucket(arr, i, p);

                if end - i <= TRY_THRESHOLD as usize {
                    insertion_sort_start_at(&mut arr[..end], i + 1);
                } else {
                    let start = find_start_of_bucket(arr, i, p);
                    unsorted_parts.push((start, end));
                }

                i = end;
            }
        }

        i += 1;
        if i >= arr.len() {
            break;
        }
    }

    unsorted_parts
}
