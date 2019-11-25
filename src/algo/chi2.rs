#[allow(dead_code)]
enum Accuracy {
    A05,
    A01,
    A005,
    A001,
}

const CHI2_TABLE: [[f64; 4]; 5] = [
    [80.232, 89.591, 93.186, 100.888],    // df = 61
    [82.529, 92.010, 95.649, 103.442],    // df = 63
    [153.198, 165.841, 170.634, 180.799], // df = 126
    [154.302, 166.987, 171.796, 181.993], // df = 127
    [293.2478, 310.4574, 316.9194, 330.5197], // df = 255
]; // p = 0.05    0.01    0.005    0.001

fn chi2_table(degree_freedom: usize, accuracy: Accuracy) -> f64 {
    let table_df = match degree_freedom {
        61 => CHI2_TABLE[0],
        63 => CHI2_TABLE[1],
        126 => CHI2_TABLE[2],
        127 => CHI2_TABLE[3],
        255 => CHI2_TABLE[4],
        _ => panic!("[Chi2 test] Bad parameters"),
    };

    match accuracy {
        Accuracy::A05 => table_df[0],
        Accuracy::A01 => table_df[1],
        Accuracy::A005 => table_df[2],
        Accuracy::A001 => table_df[3],
    }
}

fn chi2_p(n: usize, histogram: &[usize], range: usize) -> f64 {
    let expected = n as f64 / range as f64;

    histogram.iter().fold(0.0, |acc, x| {
        acc + ((*x as f64 - expected).powi(2) / expected)
    })
}

pub fn is_uniform(histogram: &[usize], range: usize) -> bool {
    let n: usize = histogram.iter().sum();
    chi2_p(n, histogram, range) <= chi2_table(range - 1, Accuracy::A001)
}

pub fn range_for_chi2(
    type_level: usize,
    offset: usize,
    max_level: usize,
    radix: usize,
) -> usize {
    let bits = radix * type_level;
    let spare = (max_level - 1) * radix;

    if bits - spare - offset == radix - 1 {
        2usize.pow(radix as u32 - 1)
    } else if bits - spare - offset == radix - 2 {
        2usize.pow(radix as u32 - 2)
    } else {
        2usize.pow(radix as u32)
    }
}
