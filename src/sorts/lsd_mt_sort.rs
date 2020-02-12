use rayon;

use super::super::algo::k_way_merge::k_way_merge_mt;
use super::super::Radixable;
use super::msd_sort::copy_by_histogram;
// use super::lsd_sort::lsd_radixsort_body;
use super::utils::{
    copy_nonoverlapping, only_one_bucket_filled, prefix_sums, Params,
    split_into_chunks,
};

pub fn lsd_radixsort_body<T>(arr: &mut [T], buffer: &mut [T], p: Params)
where
    T: Radixable + Copy + PartialOrd,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let size = arr.len();
    let dummy = arr[0];

    let mut index = 0;

    // let start = Instant::now();
    let histograms = dummy.get_full_histograms(arr, &p);
    // println!("time for histograms {}ns", start.elapsed().as_nanos() as u64);

    let mut t1 = arr;
    let mut t2 = buffer;
    // let mut t2 = t2.as_mut_slice();

    for level in (p.level..p.max_level).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));
        let (_, mut heads, _) = prefix_sums(&histograms[level]);

        copy_by_histogram(
            source.len(),
            &mut source,
            &mut destination,
            &mut heads,
            mask,
            shift,
        );

        index = 1 - index;

        if index == 1 {
            t1 = source;
            t2 = destination;
        } else {
            t2 = source;
            t1 = destination;
        }
    }

    if index == 1 {
        copy_nonoverlapping(t2, t1, size);
    }
}

fn lsd_radixsort_aux<T>(arr: &mut [T], buffer: &mut Vec<T>, radix: usize, thread_n: usize)
-> Vec<usize>
where
    T: Radixable + Copy + PartialOrd,
{
    let mut parts = split_into_chunks(arr, thread_n);
    let mut buffer_parts = split_into_chunks(buffer, thread_n);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_n)
        .build()
        .unwrap();

    pool.scope(|s| {
        for (mut part, mut buffer_part) in parts.iter_mut().zip(buffer_parts.iter_mut()) {
            s.spawn(move|_| {
                let dummy = part[0];
                let (offset, _) = dummy.compute_offset(part, radix);
                let max_level = dummy.compute_max_level(offset, radix);
                let p = Params::new(0, radix, offset, max_level);

                lsd_radixsort_body(&mut part, &mut buffer_part, p);
            });
        }
    });

    let sizes: Vec<usize> = parts.iter().map(|part| part.len()).collect();

    let mut separators = vec![0];
    for size in sizes.iter() {
        separators.push(separators.last().unwrap() + size);
    }

    separators
}

pub fn lsd_mt_radixsort<T>(arr: &mut [T], radix: usize, thread_n: usize)
where
    T: Radixable + Copy + PartialOrd,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut buffer: Vec<T> = vec![arr[0]; arr.len()];

    // let start = Instant::now();
    let mut separators = lsd_radixsort_aux(arr, &mut buffer, radix, thread_n);
    // println!("body: {}us", start.elapsed().as_micros() as u64);

    // let start = Instant::now();
    k_way_merge_mt(arr, &mut buffer, &mut separators, thread_n);
    // println!("K way merge: {}us", start.elapsed().as_micros() as u64);
}
