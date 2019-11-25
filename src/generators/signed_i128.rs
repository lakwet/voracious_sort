use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_i128(size: usize) -> Vec<i128> {
    let mut rng = thread_rng();
    let mut array: Vec<i128> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u64 = rng.gen();
        let v2: u64 = rng.gen();
        let value: i128 = (((v1 as u128) << 64) | (v2 as u128)) as i128;
        array.push(value);
    }
    array
}

pub fn generators_i128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<i128>, &'static str)> {
    vec![(&helper_random_array_uniform_i128, "-- Unif       :")]
}
