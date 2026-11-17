//! The classic merge: the shorter run copied out, merged from its side.

fn forward_merge2<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut [T],
    start: usize,
    middle: usize,
    end: usize,
) {
    if start < middle && middle < end && arr[middle - 1] <= arr[middle] {
        return;
    }

    copy_nonoverlapping(&mut arr[start..], copy, middle - start);

    let mut i = 0;
    let mut j = middle;
    let mut position = start;

    loop {
        if i == middle - start {
            return;
        }
        if j == end {
            let size = middle - start - i;
            let (_, mut rest_copy) = copy.split_at_mut(i);
            let (_, mut rest_arr) = arr.split_at_mut(position);
            copy_nonoverlapping(&mut rest_copy, &mut rest_arr, size);
            return;
        }

        if copy[i] <= arr[j] {
            arr[position] = copy[i];
            i += 1;
        } else {
            arr.swap(position, j);
            j += 1;
        }
        position += 1;
    }
}

fn backward_merge2<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut [T],
    start: usize,
    middle: usize,
    end: usize,
) {
    if start < middle && middle < end && arr[middle - 1] <= arr[middle] {
        return;
    }

    copy_nonoverlapping(&mut arr[middle..], copy, end - middle);

    let mut i: isize = (end - middle - 1) as isize;
    let mut j: isize = (middle - 1) as isize;
    let mut position = end - 1;

    loop {
        if i == -1 {
            return;
        }
        if j == start as isize - 1 {
            let (mut rest_copy, _) = copy.split_at_mut(i as usize + 1);
            let (_, mut rest_arr) = arr.split_at_mut(start);
            copy_nonoverlapping(&mut rest_copy, &mut rest_arr, i as usize + 1);
            return;
        }

        if copy[i as usize] >= arr[j as usize] {
            arr[position] = copy[i as usize];
            i -= 1;
        } else {
            arr.swap(position, j as usize);
            j -= 1;
        }
        position -= 1;
    }
}

pub(super) fn merge2<T: Copy + PartialOrd>(
    arr: &mut [T],
    copy: &mut [T],
    start: usize,
    middle: usize,
    end: usize,
) {
    if middle - start <= end - middle {
        forward_merge2(arr, copy, start, middle, end);
    } else {
        backward_merge2(arr, copy, start, middle, end);
    }
}

/// `length` elements of `source` copied to the start of `destination`.
#[inline]
fn copy_nonoverlapping<T: Copy>(source: &mut [T], destination: &mut [T], length: usize) {
    destination[..length].copy_from_slice(&source[..length]);
}
