use super::super::sorts::utils::copy_nonoverlapping;
use super::k_way_merge::merge2;

fn get_cursor<T: PartialOrd>(
    arr: &mut [T],
    min_boundary: usize,
    max_boundary: usize,
    value: T,
) -> usize {
    let mut min = min_boundary;
    let mut max = max_boundary - 1;
    let mut cursor = (max_boundary + min_boundary) / 2;

    if value <= arr[min] {
        return min;
    }
    if value >= arr[max] {
        return max;
    }

    while arr[cursor] != value {
        if arr[cursor] > value {
            max = cursor;
        } else if arr[cursor] < value {
            min = cursor;
        } else {
            break;
        }

        if min + 1 == max {
            return max;
        }

        cursor = (max + min) / 2;
    }

    cursor
}

// return max position for "value"
pub fn binary_search_max<T: PartialOrd + Copy>(
    arr: &mut [T],
    min_boundary: usize,
    max_boundary: usize,
    value: T,
) -> usize {
    let mut cursor = get_cursor(arr, min_boundary, max_boundary, value);

    let mut shifted = false;
    while cursor < max_boundary && arr[cursor] <= value {
        cursor += 1;
        shifted = true;
    }

    if shifted {
        cursor - 1
    } else {
        cursor
    }
}

// return min position for "value"
pub fn binary_search_min<T: PartialOrd + Copy>(
    arr: &mut [T],
    min_boundary: usize,
    max_boundary: usize,
    value: T,
) -> usize {
    let mut cursor = get_cursor(arr, min_boundary, max_boundary, value);

    while cursor > min_boundary && arr[cursor] >= value {
        cursor -= 1;
    }

    if arr[cursor] >= value {
        cursor
    } else {
        cursor + 1
    }
}

pub fn smart_merge2<T: Copy + Clone + PartialOrd>(
    source: &mut [T],
    destination: &mut [T],
    slice1_start: usize,
    slice1_end: usize,
    slice2_start: usize,
    slice2_end: usize,
    dest_start: usize,
) {
    let mut i = slice1_start;
    let mut j = slice2_start;
    let mut position = dest_start;

    loop {
        if i == slice1_end {
            copy_nonoverlapping(
                &mut source[j..],
                &mut destination[position..],
                slice2_end - j,
            );
            return;
        }
        if j == slice2_end {
            copy_nonoverlapping(
                &mut source[i..],
                &mut destination[position..],
                slice1_end - i,
            );
            return;
        }

        if source[i] <= source[j] {
            destination[position] = source[i];
            i += 1;
        } else {
            destination[position] = source[j];
            j += 1;
        }

        position += 1;
    }
}

pub fn smart_pre_merge2<T: Copy + Clone + PartialOrd>(
    source: &mut [T],
    destination: &mut [T],
    slice1_start: usize,
    slice1_end: usize,
    slice2_start: usize,
    slice2_end: usize,
    dest_start: usize,
) {
    let slice1_len = slice1_end - slice1_start;
    let slice2_len = slice2_end - slice2_start;

    // check slices lengths, and handle obvious cases
    if slice1_len == 0 && slice2_len == 0 {
        return;
    }
    if slice1_len == 0 && slice2_len > 0 {
        copy_nonoverlapping(
            &mut source[slice2_start..],
            &mut destination[dest_start..],
            slice2_len,
        );
        return;
    }
    if slice1_len > 0 && slice2_len == 0 {
        copy_nonoverlapping(
            &mut source[slice1_start..],
            &mut destination[dest_start..],
            slice1_len,
        );
        return;
    }
    if slice1_len == 1 && slice2_len == 1 {
        if source[slice1_start] <= source[slice2_start] {
            destination[dest_start] = source[slice1_start];
            destination[dest_start + 1] = source[slice2_start];
        } else {
            destination[dest_start] = source[slice2_start];
            destination[dest_start + 1] = source[slice1_start];
        }
        return;
    }

    // check if slices are already sorted
    if source[slice1_end - 1] <= source[slice2_start] {
        copy_nonoverlapping(
            &mut source[slice1_start..],
            &mut destination[dest_start..],
            slice1_len,
        );
        copy_nonoverlapping(
            &mut source[slice2_start..],
            &mut destination[(dest_start + slice1_len)..],
            slice2_len,
        );
        return;
    }
    if source[slice2_end - 1] <= source[slice1_start] {
        copy_nonoverlapping(
            &mut source[slice2_start..],
            &mut destination[dest_start..],
            slice2_len,
        );
        copy_nonoverlapping(
            &mut source[slice1_start..],
            &mut destination[(dest_start + slice2_len)..],
            slice1_len,
        );
        return;
    }

    // if one slice contains the other
    let contains = source[slice2_start] <= source[slice1_start]
        && source[slice1_end - 1] <= source[slice2_end - 1];
    if contains {
        let p1 = binary_search_max(
            source,
            slice2_start,
            slice2_end,
            source[slice1_start],
        );
        let p2 = binary_search_min(
            source,
            slice2_start,
            slice2_end,
            source[slice1_end - 1],
        );
        copy_nonoverlapping(
            &mut source[slice2_start..],
            &mut destination[dest_start..],
            p1 - slice2_start,
        );
        let len2 = slice2_end - p2;
        let dest_start2 = dest_start + slice1_len + slice2_len - len2;
        copy_nonoverlapping(
            &mut source[p2..],
            &mut destination[dest_start2..],
            len2,
        );

        smart_merge2(
            source,
            destination,
            slice1_start,
            slice1_end,
            p1,
            p2,
            dest_start + p1 - slice2_start,
        );
        return;
    }

    let contains = source[slice1_start] <= source[slice2_start]
        && source[slice2_end - 1] <= source[slice1_end - 1];
    if contains {
        let p1 = binary_search_max(
            source,
            slice1_start,
            slice1_end,
            source[slice2_start],
        );
        let p2 = binary_search_min(
            source,
            slice1_start,
            slice1_end,
            source[slice2_end - 1],
        );
        copy_nonoverlapping(
            &mut source[slice1_start..],
            &mut destination[dest_start..],
            p1 - slice1_start,
        );
        let len2 = slice1_end - p2;
        let dest_start2 = dest_start + slice2_len + slice1_len - len2;
        copy_nonoverlapping(
            &mut source[p2..],
            &mut destination[dest_start2..],
            len2,
        );

        smart_merge2(
            source,
            destination,
            slice2_start,
            slice2_end,
            p1,
            p2,
            dest_start + p1 - slice1_start,
        );
        return;
    }

    // if they are partially overlapping
    let partially_overlapping = source[slice1_start] <= source[slice2_start]
        && source[slice2_start] <= source[slice1_end - 1]
        && source[slice1_end - 1] <= source[slice2_end - 1];
    if partially_overlapping {
        let p1 = binary_search_max(
            source,
            slice1_start,
            slice1_end,
            source[slice2_start],
        );
        copy_nonoverlapping(
            &mut source[slice1_start..],
            &mut destination[dest_start..],
            p1 - slice1_start,
        );
        let p2 = binary_search_min(
            source,
            slice2_start,
            slice2_end,
            source[slice1_end - 1],
        );
        let len2 = slice2_end - p2;
        let dest_start2 = dest_start + slice1_len + slice2_len - len2;
        copy_nonoverlapping(
            &mut source[p2..],
            &mut destination[dest_start2..],
            len2,
        );

        smart_merge2(
            source,
            destination,
            p1,
            slice1_end,
            slice2_start,
            p2,
            dest_start + p1 - slice1_start,
        );
        return;
    }

    let partially_overlapping = source[slice2_start] <= source[slice1_start]
        && source[slice1_start] <= source[slice2_end - 1]
        && source[slice2_end - 1] <= source[slice1_end - 1];
    if partially_overlapping {
        let p1 = binary_search_max(
            source,
            slice2_start,
            slice2_end,
            source[slice1_start],
        );
        copy_nonoverlapping(
            &mut source[slice2_start..],
            &mut destination[dest_start..],
            p1 - slice2_start,
        );
        let p2 = binary_search_min(
            source,
            slice1_start,
            slice1_end,
            source[slice2_end - 1],
        );
        let len2 = slice1_end - p2;
        let dest_start2 = dest_start + slice2_len + slice1_len - len2;
        copy_nonoverlapping(
            &mut source[p2..],
            &mut destination[dest_start2..],
            len2,
        );

        smart_merge2(
            source,
            destination,
            p1,
            slice2_end,
            slice1_start,
            p2,
            dest_start + p1 - slice2_start,
        );
        return;
    }

    panic!("[Smart merge2] Bad implementation.");
}

fn sort_slices(slices: &mut Vec<(usize, usize)>) {
    slices.sort_unstable_by(|(start1, end1), (start2, end2)| {
        let size1 = end1 - start1;
        let size2 = end2 - start2;

        size1.cmp(&size2)
    });
}

/*
 * => Use two buffer arrays, and switch each time between an array being the source
 * and the other array being the destination
 * => Sort slices by size and merge them two by two at a time (the idea is
 * to merge slices with roughly the same size)
 * => Two slices overlapping, use binary search to remove front and back of each
 * slices if it is possible (already sorted)
 * => Check if slices are already in sorted order
 * => Two slices overlapping, split each slice in two and merge the small values
 * together, and the big values together.
 * => If odd number of slices, do not merge the biggest one (if slices are sorted
 * by size)
 * => Experimental: use counting sort for specifics cases
 */
pub fn smart_k_way_merge<T>(arr: &mut [T], separators: &mut Vec<usize>)
where
    T: Copy + Clone + PartialOrd,
{
    if separators.len() <= 2 {
        return;
    }

    if separators.len() == 3 {
        let min_length =
            if separators[1] - separators[0] <= separators[2] - separators[1] {
                separators[1] - separators[0]
            } else {
                separators[2] - separators[1]
            };

        merge2(
            arr,
            &mut vec![arr[0]; min_length],
            separators[0],
            separators[1],
            separators[2],
        );
        return;
    }

    let size = arr.len();
    let mut slices = Vec::with_capacity(separators.len() - 1);
    for i in 0..separators.len() - 1 {
        slices.push((separators[i], separators[i + 1]));
    }

    let mut t2_vec = vec![arr[0]; size];
    let mut t1 = arr;
    let mut t2 = t2_vec.as_mut_slice();
    let mut index = 0;

    while slices.len() > 1 {
        let (source, destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let mut dest_start = 0;
        let mut slices_buffer = Vec::new();
        let half = slices.len() / 2;

        sort_slices(&mut slices);

        if slices.len() == 2 && index == 0 {
            slices.sort_unstable();
            let (start1, end1) = slices[0];
            let (_start2, end2) = slices[1];
            merge2(source, destination, start1, end1, end2);
            break;
        }

        for i in 0..half {
            let i2 = i * 2;
            let (start1, end1) = slices[i2];
            let (start2, end2) = slices[i2 + 1];
            smart_pre_merge2(
                source,
                destination,
                start1,
                end1,
                start2,
                end2,
                dest_start,
            );
            let boundary = end1 - start1 + end2 - start2;
            slices_buffer.push((dest_start, dest_start + boundary));
            dest_start += boundary;
        }

        if slices.len() % 2 == 1 {
            let (start, end) = *slices.last().unwrap();
            copy_nonoverlapping(
                &mut source[start..],
                &mut destination[dest_start..],
                end - start,
            );
            slices_buffer.push((dest_start, dest_start + end - start));
        }

        slices = slices_buffer;
        index = 1 - index;

        if index == 1 {
            t1 = source;
            t2 = destination;
        } else {
            t2 = source;
            t1 = destination;
        }
    }
}
