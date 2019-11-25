use super::super::types::{Radixable, RadixableForContainer};

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
pub fn swap<T>(a: &mut [T], i: usize, j: usize) {
    unsafe {
        let pa: *mut T = a.get_unchecked_mut(i);
        let pb: *mut T = a.get_unchecked_mut(j);
        std::ptr::swap_nonoverlapping(pa, pb, 1);
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
    heads.split_off(p_sums.len() - 1);
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

pub fn get_empty_histograms(p: &Params, partial: usize) -> Vec<Vec<usize>> {
    let mut histograms = Vec::new();

    for _ in 0..partial {
        let histogram = vec![0; p.radix_range];
        histograms.push(histogram);
    }

    histograms
}

pub fn get_histogram<T>(
    a: &mut [T],
    p: &Params,
    mask: <[T] as RadixableForContainer>::KeyType,
    shift: usize,
) -> Vec<usize>
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>,
    [T]: RadixableForContainer,
{
    let mut histogram = vec![0; p.radix_range];

    let quotient = a.len() / 4;
    let remainder = a.len() % 4;

    for q in 0..quotient {
        let i = q * 4;
        unsafe {
            let bucket0 = a.get_unchecked(i).get_key(mask, shift);
            let bucket1 = a.get_unchecked(i + 1).get_key(mask, shift);
            let bucket2 = a.get_unchecked(i + 2).get_key(mask, shift);
            let bucket3 = a.get_unchecked(i + 3).get_key(mask, shift);
            histogram[bucket0] += 1;
            histogram[bucket1] += 1;
            histogram[bucket2] += 1;
            histogram[bucket3] += 1;
        }
    }

    let offset = quotient * 4;
    for i in 0..remainder {
        unsafe {
            let bucket = a.get_unchecked(offset + i).get_key(mask, shift);
            histogram[bucket] += 1;
        }
    }

    histogram
}

pub fn _get_full_histogram<T>(arr: &mut [T], p: &Params) -> Vec<Vec<usize>>
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>,
    [T]: RadixableForContainer,
{
    let mut histograms = Vec::new();
    for level in 0..p.max_level {
        let mut histogram = vec![0; p.radix_range];
        let (mask, shift) = arr.get_mask_and_shift(&p.new_level(level));

        arr.iter().for_each(|element| {
            histogram[element.get_key(mask, shift)] += 1;
        });

        histograms.push(histogram);
    }

    histograms
}

pub fn get_full_histogram_except_for_last_level<T>(
    arr: &mut [T],
    p: &Params,
) -> Vec<Vec<usize>>
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer,
{
    let mut histograms = Vec::new();
    for level in 0..(p.max_level - 1) {
        let mut histogram = vec![0; p.radix_range];
        let (mask, shift) = arr.get_mask_and_shift(&p.new_level(level));

        arr.iter().for_each(|element| {
            histogram[element.get_key(mask, shift)] += 1;
        });

        histograms.push(histogram);
    }

    histograms
}

pub fn get_partial_histograms_fast<T>(
    arr: &mut [T],
    p: &Params,
    partial: usize,
) -> Vec<Vec<usize>>
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType> + Copy,
    [T]: RadixableForContainer<T = T>,
{
    if partial > 5 {
        panic!("[RadixableForContainer] Array size can't be that huge !");
    }

    let mut histograms = get_empty_histograms(p, partial);
    let default_mask = arr.get_default_mask(p);
    let shift = arr.usize_into_key_type(p.radix);
    let bits = arr.element_bit_size();
    let fs = if p.radix * partial > bits - p.offset {
        0
    } else {
        bits - p.offset - p.radix * partial
    };

    if partial == 1 {
        for element in arr.iter() {
            let value =
                arr.into_key_type(*element) >> arr.usize_into_key_type(fs);

            histograms[0][arr.from_key_type(value & default_mask)] += 1;
        }
    } else if partial == 2 {
        for element in arr.iter() {
            let mut value =
                arr.into_key_type(*element) >> arr.usize_into_key_type(fs);

            histograms[1][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][arr.from_key_type(value & default_mask)] += 1;
        }
    } else if partial == 3 {
        for element in arr.iter() {
            let mut value =
                arr.into_key_type(*element) >> arr.usize_into_key_type(fs);

            histograms[2][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][arr.from_key_type(value & default_mask)] += 1;
        }
    } else if partial == 4 {
        for element in arr.iter() {
            let mut value =
                arr.into_key_type(*element) >> arr.usize_into_key_type(fs);

            histograms[3][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[2][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][arr.from_key_type(value & default_mask)] += 1;
        }
    } else if partial == 5 {
        for element in arr.iter() {
            let mut value =
                arr.into_key_type(*element) >> arr.usize_into_key_type(fs);

            histograms[4][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[3][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[2][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][arr.from_key_type(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][arr.from_key_type(value & default_mask)] += 1;
        }
    }

    histograms
}

pub fn get_full_histograms_fast<T>(arr: &mut [T], p: &Params) -> Vec<Vec<usize>>
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>,
    [T]: RadixableForContainer<T = T>,
{
    let mut histograms = get_empty_histograms(p, p.max_level);
    let default_mask = arr.get_default_mask(p);
    let shift = arr.usize_into_key_type(p.radix);

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    let offset = quotient * 4;

    if p.max_level == 1 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let v0 =
                    arr.into_key_type(*arr.get_unchecked(i)) & default_mask;
                let v1 =
                    arr.into_key_type(*arr.get_unchecked(i + 1)) & default_mask;
                let v2 =
                    arr.into_key_type(*arr.get_unchecked(i + 2)) & default_mask;
                let v3 =
                    arr.into_key_type(*arr.get_unchecked(i + 3)) & default_mask;
                histograms[0][arr.from_key_type(v0)] += 1;
                histograms[0][arr.from_key_type(v1)] += 1;
                histograms[0][arr.from_key_type(v2)] += 1;
                histograms[0][arr.from_key_type(v3)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let v = arr.into_key_type(*arr.get_unchecked(offset + i))
                    & default_mask;
                histograms[0][arr.from_key_type(v)] += 1;
            }
        }
    } else if p.max_level == 2 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 3 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 4 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[3][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[3][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 5 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[4][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[4][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 6 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[5][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[5][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 7 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[6][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[5][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[6][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[5][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 8 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.into_key_type(*arr.get_unchecked(i));
                let mut v1 = arr.into_key_type(*arr.get_unchecked(i + 1));
                let mut v2 = arr.into_key_type(*arr.get_unchecked(i + 2));
                let mut v3 = arr.into_key_type(*arr.get_unchecked(i + 3));
                histograms[7][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[7][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[7][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[7][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[6][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[6][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[5][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[5][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[4][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[3][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[2][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[1][arr.from_key_type(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][arr.from_key_type(v0 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v1 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v2 & default_mask)] += 1;
                histograms[0][arr.from_key_type(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.into_key_type(*arr.get_unchecked(offset + i));
                histograms[7][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[6][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[5][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][arr.from_key_type(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][arr.from_key_type(v & default_mask)] += 1;
            }
        }
    }

    histograms
}
