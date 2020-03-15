use super::super::sorts::utils::copy_nonoverlapping;

fn forward_merge2<T: Copy + Clone + PartialOrd>(
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

fn backward_merge2<T: Copy + Clone + PartialOrd>(
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

pub fn merge2<T: Copy + Clone + PartialOrd>(
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

pub fn k_way_merge<T>(arr: &mut [T], separators: &mut Vec<usize>)
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

    let mut copy: Vec<T> = vec![arr[0]; (arr.len() / 2) + 2];
    while separators.len() > 2 {
        let half = (separators.len() - 1) / 2;
        for i in 0..half {
            let i2 = i * 2;
            merge2(
                arr,
                &mut copy,
                separators[i2],
                separators[i2 + 1],
                separators[i2 + 2],
            );
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    }
}
