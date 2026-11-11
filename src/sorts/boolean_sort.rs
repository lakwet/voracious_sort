/// Bytes summed per lane before the 8-bit lanes are flushed (255 ones at
/// most per lane).
const LANES: usize = 32;
const ROUNDS: usize = 255;

/// Number of `true` among `bytes[..n]` and of descents (`true` followed by
/// `false`) among the pairs `(bytes[i], bytes[i + 1])`, in one pass: the
/// bytes (0 or 1) are added into 32 lanes of 8-bit counters, flushed every
/// 255 rounds, which the compiler vectorizes.
#[inline]
fn count(bytes: &[u8]) -> (usize, usize) {
    let n = bytes.len();
    let (a, b) = (&bytes[..n - 1], &bytes[1..]);
    let (mut ones, mut descents) = (0usize, 0usize);
    let mut ca = a.chunks_exact(LANES);
    let mut cb = b.chunks_exact(LANES);
    loop {
        let mut acc_one = [0u8; LANES];
        let mut acc_desc = [0u8; LANES];
        let mut rounds = 0;
        while rounds < ROUNDS {
            let (Some(x), Some(y)) = (ca.next(), cb.next()) else { break };
            for k in 0..LANES {
                acc_one[k] += x[k];
                acc_desc[k] += x[k] & (y[k] ^ 1);
            }
            rounds += 1;
        }
        ones += acc_one.iter().map(|&v| v as usize).sum::<usize>();
        descents += acc_desc.iter().map(|&v| v as usize).sum::<usize>();
        if rounds < ROUNDS {
            break;
        }
    }
    for (&x, &y) in ca.remainder().iter().zip(cb.remainder()) {
        ones += x as usize;
        descents += (x & (y ^ 1)) as usize;
    }
    (ones + bytes[n - 1] as usize, descents)
}

/// # Boolean sort
///
/// A dedicated sort for booleans: one vectorized pass counts the `true`
/// values and the descents (`true` before `false`); a sorted array (no
/// descent) is left untouched, otherwise the array is rewritten with two
/// fills (`false` then `true`).
pub fn boolean_sort(arr: &mut [bool]) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    // SAFETY: a `bool` is one byte holding 0 or 1.
    let bytes = unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, n) };
    let (trues, descents) = count(bytes);
    if descents == 0 {
        return;
    }
    let falses = n - trues;
    arr[..falses].fill(false);
    arr[falses..].fill(true);
}
