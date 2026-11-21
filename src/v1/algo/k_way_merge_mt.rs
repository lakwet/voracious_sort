use rayon::{ThreadPoolBuilder, ThreadPool};

use super::k_way_merge::merge2;

fn kway_merge_mt_helper<T: Copy + PartialOrd + Send>(
    pool: &ThreadPool,
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
        for (i, (part, buffer_part)) in
            parts.into_iter().zip(buffer_parts.into_iter()).enumerate()
        {
            let i2 = i * 2;
            let sep1 = separators[i2];
            let sep2 = separators[i2 + 1];
            let sep3 = separators[i2 + 2];
            s.spawn(move |_| {
                merge2(part, buffer_part, 0, sep2 - sep1, sep3 - sep1);
            });
        }
        for i in 0..half {
            separators.remove(i + 1);
        }
    });
}

pub fn k_way_merge_mt<T: Copy + PartialOrd + Send>(
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

    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_n)
        .build()
        .unwrap();

    while separators.len() > 2 {
        kway_merge_mt_helper(&pool, arr, buffer, separators);
    }
}

pub fn k_way_merge_mt_with_buffer<T: Copy + PartialOrd + Send>(
    arr: &mut [T],
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

    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_n)
        .build()
        .unwrap();

    let mut buffer: Vec<T> = arr.to_vec();
    let buffer = buffer.as_mut_slice();
    while separators.len() > 2 {
        kway_merge_mt_helper(&pool, arr, buffer, separators);
    }
}
