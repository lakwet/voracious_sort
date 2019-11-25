pub fn offset_from_bits<T>(
    biggest: T,
    radix: usize,
    bits: usize,
    zero: T,
    one: T,
) -> (usize, usize)
where
    T: std::ops::Shr<Output = T> + PartialEq + Copy,
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
