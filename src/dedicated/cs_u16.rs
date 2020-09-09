const UNROLL_SIZE: usize = 4;

pub fn cs_u16(arr: &mut [u16]) {
    let size = arr.len();
    if size < 2 {
        return;
    }

    let mut histogram = vec![0; 65536];

    let remainder = size % UNROLL_SIZE;
    let (arr_main, arr_remainder) = arr.split_at_mut(size - remainder);

    arr_main.chunks_exact(UNROLL_SIZE).for_each(|chunk| {
        histogram[chunk[0] as usize] += 1;
        histogram[chunk[1] as usize] += 1;
        histogram[chunk[2] as usize] += 1;
        histogram[chunk[3] as usize] += 1;
    });
    arr_remainder.iter().for_each(|item| {
        histogram[*item as usize] += 1;
    });

    let mut position = 0;
    histogram.iter().enumerate().for_each(|(value, count)| {
        if *count > 0 {
            let v = value as u16;
            let quotient = *count / 4;
            let remainder = count % 4;
            for _ in 0..quotient {
                unsafe {
                    *arr.get_unchecked_mut(position) = v;
                    *arr.get_unchecked_mut(position + 1) = v;
                    *arr.get_unchecked_mut(position + 2) = v;
                    *arr.get_unchecked_mut(position + 3) = v;
                    position += 4;
                }
            }
            for _ in 0..remainder {
                unsafe {
                    *arr.get_unchecked_mut(position) = v;
                    position += 1;
                }
            }
        }
    });
}
