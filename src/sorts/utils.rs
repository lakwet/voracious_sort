use super::super::{RadixKey, Radixable};

#[derive(Copy, Clone, Debug, Default)]
pub struct Params {
    pub level: usize,
    pub radix: usize,
    pub offset: usize,
    pub max_level: usize,
    pub radix_range: usize,
}

impl Params {
    pub fn new(
        level: usize,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> Params {
        Params {
            level,
            radix,
            offset,
            max_level,
            radix_range: (2 as usize).pow(radix as u32),
        }
    }
    pub fn new_level(&self, level: usize) -> Params {
        Params { level, ..(*self) }
    }
}

#[inline]
pub fn copy_nonoverlapping<T>(
    source: &mut [T],
    destination: &mut [T],
    length: usize,
) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            source.as_ptr(),
            destination.get_unchecked_mut(0),
            length,
        );
    }
}

pub fn prefix_sums(
    histogram: &[usize],
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let p_sums = histogram.iter().fold(vec![0], |mut acc, count| {
        acc.push(*count as usize + acc.last().unwrap());
        acc
    });

    let mut heads = p_sums.to_vec();
    let _ = heads.split_off(p_sums.len() - 1);
    let tails = p_sums.to_vec().split_off(1);

    (p_sums, heads, tails)
}

pub fn only_one_bucket_filled(histogram: &[usize]) -> bool {
    let mut count = 0;

    for item in histogram {
        if *item > 0 {
            count += 1;
        }

        if count > 1 {
            return false;
        }
    }

    true
}

pub fn offset_from_bits<T, K>(
    _arr: &mut [T],
    biggest: <<T as Radixable<K>>::Key as RadixKey>::Key,
    radix: usize,
    bits: usize,
    zero: <<T as Radixable<K>>::Key as RadixKey>::Key,
    one: <<T as Radixable<K>>::Key as RadixKey>::Key,
) -> (usize, usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    let mut count = 0;
    let mut buf = biggest;

    while buf != zero {
        buf = buf >> one;
        count += 1;
    }

    let offset = if count % radix == 0 {
        bits - count
    } else {
        let q = count / radix;
        let total_bits = (q + 1) * radix;

        if total_bits > bits {
            0
        } else {
            bits - total_bits
        }
    };

    (offset, bits - count)
}

pub fn compute_offset<T: Radixable<K> + Copy, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
) -> (usize, usize) {
    let dummy = arr[0];
    let max = arr.iter().map(|item| item.into_key_type()).max().unwrap();

    offset_from_bits(
        arr,
        max,
        radix,
        dummy.type_size(),
        dummy.default_key(),
        dummy.one(),
    )
}

pub fn compute_max_level(bits: usize, offset: usize, radix: usize) -> usize {
    assert!(bits >= offset);
    let rest = bits - offset;
    if rest % radix != 0 {
        (rest / radix) + 1
    } else {
        rest / radix
    }
}

pub fn get_empty_histograms(
    partial: usize,
    radix_range: usize,
) -> Vec<Vec<usize>> {
    let mut histograms = Vec::new();

    for _ in 0..partial {
        let histogram = vec![0; radix_range];
        histograms.push(histogram);
    }

    histograms
}

pub fn get_histogram<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    mask: <<T as Radixable<K>>::Key as RadixKey>::Key,
    shift: usize,
) -> Vec<usize> {
    let mut histogram = vec![0; p.radix_range];
    let remainder = arr.len() % 4;
    let (arr_fst, arr_remainder) = arr.split_at(arr.len() - remainder);

    arr_fst.chunks_exact(4).for_each(|chunk| {
        let bucket0 = chunk[0].extract(mask, shift);
        let bucket1 = chunk[1].extract(mask, shift);
        let bucket2 = chunk[2].extract(mask, shift);
        let bucket3 = chunk[3].extract(mask, shift);
        histogram[bucket0] += 1;
        histogram[bucket1] += 1;
        histogram[bucket2] += 1;
        histogram[bucket3] += 1;
    });

    arr_remainder.iter().for_each(|item| {
        let bucket = item.extract(mask, shift);
        histogram[bucket] += 1;
    });

    histogram
}

pub fn get_partial_histograms<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    partial: usize,
) -> Vec<Vec<usize>> {
    if partial > 5 {
        panic!("[Partial histogram] Array size can't be that huge !");
    }

    let dummy = arr[0];
    let mut histograms = get_empty_histograms(partial, p.radix_range);
    let default_mask = dummy.default_mask(p.radix);
    let shift = dummy.usize_to_keytype(p.radix);
    let bits = dummy.type_size();
    let fs = if p.radix * partial > bits - p.offset {
        0
    } else {
        bits - p.offset - p.radix * partial
    };
    let fs = dummy.usize_to_keytype(fs);

    let remainder = arr.len() % 4;
    let (arr_fst, arr_remainder) = arr.split_at(arr.len() - remainder);

    if partial == 1 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let v0 = chunk[0].into_key_type() >> fs;
            let v1 = chunk[1].into_key_type() >> fs;
            let v2 = chunk[2].into_key_type() >> fs;
            let v3 = chunk[3].into_key_type() >> fs;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let v = item.into_key_type() >> fs;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if partial == 2 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type() >> fs;
            let mut v1 = chunk[1].into_key_type() >> fs;
            let mut v2 = chunk[2].into_key_type() >> fs;
            let mut v3 = chunk[3].into_key_type() >> fs;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type() >> fs;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if partial == 3 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type() >> fs;
            let mut v1 = chunk[1].into_key_type() >> fs;
            let mut v2 = chunk[2].into_key_type() >> fs;
            let mut v3 = chunk[3].into_key_type() >> fs;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type() >> fs;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if partial == 4 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type() >> fs;
            let mut v1 = chunk[1].into_key_type() >> fs;
            let mut v2 = chunk[2].into_key_type() >> fs;
            let mut v3 = chunk[3].into_key_type() >> fs;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type() >> fs;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if partial == 5 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type() >> fs;
            let mut v1 = chunk[1].into_key_type() >> fs;
            let mut v2 = chunk[2].into_key_type() >> fs;
            let mut v3 = chunk[3].into_key_type() >> fs;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type() >> fs;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    }

    histograms
}

pub fn get_full_histograms<T, K>(arr: &mut [T], p: &Params) -> Vec<Vec<usize>>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut histograms = get_empty_histograms(p.max_level, p.radix_range);
    let default_mask = dummy.default_mask(p.radix);
    let shift = dummy.usize_to_keytype(p.radix);

    let remainder = arr.len() % 4;
    let (arr_fst, arr_remainder) = arr.split_at(arr.len() - remainder);

    if p.max_level == 1 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let v0 = chunk[0].into_key_type();
            let v1 = chunk[1].into_key_type();
            let v2 = chunk[2].into_key_type();
            let v3 = chunk[3].into_key_type();
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        });
        arr_remainder.iter().for_each(|item| {
            let v = item.into_key_type();
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 2 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 3 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 4 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 5 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 6 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 7 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 8 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 9 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 10 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 11 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 12 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[11][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[11][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 13 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[12][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[11][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[12][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[11][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 14 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[13][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[12][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[11][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[13][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[12][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[11][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 15 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[14][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[13][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[12][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[11][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[13][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[12][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[11][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else if p.max_level == 16 {
        arr_fst.chunks_exact(4).for_each(|chunk| {
            let mut v0 = chunk[0].into_key_type();
            let mut v1 = chunk[1].into_key_type();
            let mut v2 = chunk[2].into_key_type();
            let mut v3 = chunk[3].into_key_type();
            histograms[15][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[15][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[15][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[15][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[14][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[14][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[13][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[13][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[12][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[12][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[11][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[11][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[10][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[10][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[9][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[9][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[8][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[8][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
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
        });
        arr_remainder.iter().for_each(|item| {
            let mut v = item.into_key_type();
            histograms[15][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[14][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[13][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[12][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[11][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[10][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[9][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[8][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        });
    } else {
        panic!("[Get full histogram] Too small radix.");
    }

    histograms
}
