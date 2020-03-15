use rayon::prelude::*;

use std::sync::mpsc::channel;

#[inline]
pub fn swap_range<T>(a: &mut [T], len: usize, i: usize, j: usize) {
    unsafe {
        let pa: *mut T = a.get_unchecked_mut(i);
        let pb: *mut T = a.get_unchecked_mut(j);
        std::ptr::swap_nonoverlapping(pa, pb, len);
    }
}



struct SafePtr<T: ?Sized>(*mut T);
unsafe impl<T: ?Sized> Send for SafePtr<T> {}
unsafe impl<T: ?Sized> Sync for SafePtr<T> {}

#[inline]
pub fn swap_range_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    len: usize,
    i: usize,
    j: usize,
) {
    let ptr = SafePtr(arr.as_mut_ptr());

    let indices = vec![0; len];

    indices
        .par_iter()
        .enumerate()
        .for_each(|(offset, _)| unsafe {
            let SafePtr(ptr) = ptr;
            let p1: *mut _ = ptr.add(i + offset);
            let p2: *mut _ = ptr.add(j + offset);
            std::ptr::swap(p1, p2);
        });
}

pub fn perform_swaps<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    swaps: Vec<(usize, usize, usize)>,
    offset: usize,
) {
    for (len, i1, i2) in swaps.iter() {
        swap_range(arr, *len, *i1 - offset, *i2 - offset);
    }
}

pub fn perform_swaps_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    swaps: Vec<(usize, usize, usize)>,
    offset: usize,
) {
    let ptr = SafePtr(arr.as_mut_ptr());

    swaps.par_iter().for_each(|(len, i, j)| unsafe {
        let SafePtr(ptr) = ptr;
        let p1: *mut _ = ptr.add(i - offset);
        let p2: *mut _ = ptr.add(j - offset);
        std::ptr::swap_nonoverlapping(p1, p2, *len);
    });
}

pub fn aggregate_histograms(histograms: &Vec<Vec<usize>>) -> Vec<usize> {
    let mut global_histogram = vec![0; histograms[0].len()];

    histograms.iter().for_each(|histogram| {
        histogram.iter().enumerate().for_each(|(i, v)| {
            global_histogram[i] += v;
        });
    });

    global_histogram
}


pub fn split_into_chunks<T, K>(arr: &mut [T], chunk_n: usize) -> Vec<&mut [T]>
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let part_size = arr.len() / chunk_n;

    let mut parts = Vec::new();
    let mut rest = arr;
    for _ in 0..(chunk_n - 1) {
        let (fst, snd) = rest.split_at_mut(part_size);
        rest = snd;
        parts.push(fst);
    }
    if rest.len() > 0 {
        parts.push(rest);
    }

    parts
}

pub fn get_histogram_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    mask: <<T as Radixable<K>>::Key as RadixKey>::Key,
    shift: usize,
    pool: &rayon::ThreadPool,
    chunk_n: usize,
) -> Vec<usize> {
    let parts = split_into_chunks(arr, chunk_n);
    let mut histograms: Vec<Vec<usize>> = Vec::new();
    let mut receivers = Vec::new();

    pool.scope(|s| {
        for part in parts.into_iter() {
            let (sender, receiver) = channel();
            receivers.push(receiver);
            s.spawn(move |_| {
                let h = get_histogram(part, p, mask, shift);
                sender.send(h).unwrap();
            });
        }
    });

    for receiver in receivers.iter() {
        histograms.push(receiver.recv().unwrap());
    }

    aggregate_histograms(&histograms)
}


pub fn get_next_two_histograms<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
) -> Vec<Vec<usize>> {
    let dummy = arr[0];
    let shift = dummy.usize_to_keytype(p.radix);
    let (_, fst_shift) = dummy.get_mask_and_shift_from_left(&p);
    let fst_shift = dummy.usize_to_keytype(fst_shift - p.radix);

    let mut histograms = get_empty_histograms(p, 2);
    let default_mask = dummy.default_mask(p.radix);

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    let offset = quotient * 4;

    for q in 0..quotient {
        unsafe {
            let i = q * 4;
            let mut v0 = arr.get_unchecked(i).into_key_type();
            let mut v1 = arr.get_unchecked(i + 1).into_key_type();
            let mut v2 = arr.get_unchecked(i + 2).into_key_type();
            let mut v3 = arr.get_unchecked(i + 3).into_key_type();
            v0 = v0 >> fst_shift;
            v1 = v1 >> fst_shift;
            v2 = v2 >> fst_shift;
            v3 = v3 >> fst_shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        }
    }
    for i in 0..remainder {
        unsafe {
            let mut v = arr.get_unchecked(offset + i).into_key_type();
            v = v >> fst_shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        }
    }

    histograms
}
