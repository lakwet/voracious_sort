use rayon;

use super::super::sorts::utils::{copy_nonoverlapping, swap};

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
            swap(arr, position, j);
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
    if start < middle && middle < end  && arr[middle - 1] <= arr[middle] {
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
            swap(arr, position, j as usize);
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

fn kway_merge_mt_helper<T: Copy + Clone + PartialOrd + Send>(
    pool: &rayon::ThreadPool,
    arr: &mut [T],
    buffer: &mut [T],
    separators: &mut Vec<usize>,
) {
    pool.scope(|s| {
        let half = (separators.len() - 1) / 2;
        let mut offset = 0;
        let mut rest = arr;
        let mut rest_buffer = buffer;
        let mut parts = Vec::new();
        let mut buffer_parts = Vec::new();
        for i in 0..half {
            let i2 = i * 2;
            let sep1 = separators[i2];
            let sep3 = separators[i2 + 2];
            let (part, snd) = rest.split_at_mut(sep3 - offset);
            rest = snd;
            parts.push(part);
            let (part, snd) = rest_buffer.split_at_mut(sep3 - offset);
            rest_buffer = snd;
            buffer_parts.push(part);
            offset += sep3 - sep1;
        }
        for (i, (part, buffer_part)) in parts.into_iter().zip(buffer_parts.into_iter()).enumerate() {
            let i2 = i * 2;
            let sep1 = separators[i2];
            let sep2 = separators[i2 + 1];
            let sep3 = separators[i2 + 2];
            s.spawn(move|_| {
                merge2(
                    part,
                    buffer_part,
                    0,
                    sep2 - sep1,
                    sep3 - sep1,
                );
            });
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    });
}

pub fn k_way_merge_mt<T: Copy + Clone + PartialOrd + Send>(
    arr: &mut [T],
    buffer: &mut [T],
    separators: &mut Vec<usize>,
    thread_n: usize,
) {
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

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_n)
        .build()
        .unwrap();

    while separators.len() > 2 {
        kway_merge_mt_helper(&pool, arr, buffer, separators);
    }
}
