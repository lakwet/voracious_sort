//! Stable LSD on the records themselves (small records beyond the cache).

use std::mem::MaybeUninit;

/// Stable LSD on the elements themselves: byte digits of the encoded key
/// (histograms `hist` taken in the first read), the digits equal in every
/// key skipped, each pass moving the elements between the array and a
/// buffer (the key read again from each element).
pub(super) fn lsd<T, E: Fn(&T) -> u128>(arr: &mut [T], enc: &E, hist: &[[usize; 256]]) {
    let n = arr.len();
    let mut buf: Vec<MaybeUninit<T>> = Vec::with_capacity(n);
    let mut in_buf = false;
    // SAFETY: each pass moves every element once from one side to the
    // other (the heads cover 0..n exactly); at the end the elements are
    // copied back if they ended in the buffer. Nothing is dropped twice
    // (the buffer is MaybeUninit); `enc` was already called on every
    // element without panicking.
    unsafe {
        let a = arr.as_mut_ptr();
        let b = buf.as_mut_ptr() as *mut T;
        for (d, h) in hist.iter().enumerate() {
            if h.iter().any(|&c| c == n) {
                continue;
            }
            let mut heads = [0usize; 256];
            let mut s = 0;
            for (hd, &c) in heads.iter_mut().zip(h.iter()) {
                *hd = s;
                s += c;
            }
            let (src, dst) = if in_buf { (b as *const T, a) } else { (a as *const T, b) };
            for i in 0..n {
                let x = &*src.add(i);
                let digit = ((enc(x) as u64 >> (8 * d)) & 0xFF) as usize;
                dst.add(heads[digit]).write(std::ptr::read(x));
                heads[digit] += 1;
            }
            in_buf = !in_buf;
        }
        if in_buf {
            std::ptr::copy_nonoverlapping(b, a, n);
        }
    }
}
