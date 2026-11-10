const UNROLL_SIZE: usize = 4;

pub fn cs_u16(arr: &mut [u16]) {
    let size = arr.len();
    if size < 2 {
        return;
    }

    // Counts: u32 in the hot loop, added to usize totals every 2^31 keys, so
    // that a value present more than 4 billion times cannot overflow (the
    // counters were inferred as i32 before: wrong result beyond 2^31 equal
    // keys).
    let mut histogram = vec![0usize; 65536];
    let mut block_counts = vec![0u32; 65536];
    for block in arr.chunks(1 << 31) {
        let remainder = block.len() % UNROLL_SIZE;
        let (block_main, block_remainder) = block.split_at(block.len() - remainder);
        block_main.chunks_exact(UNROLL_SIZE).for_each(|chunk| {
            block_counts[chunk[0] as usize] += 1;
            block_counts[chunk[1] as usize] += 1;
            block_counts[chunk[2] as usize] += 1;
            block_counts[chunk[3] as usize] += 1;
        });
        block_remainder.iter().for_each(|item| {
            block_counts[*item as usize] += 1;
        });
        for (total, c) in histogram.iter_mut().zip(block_counts.iter_mut()) {
            *total += *c as usize;
            *c = 0;
        }
    }

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
