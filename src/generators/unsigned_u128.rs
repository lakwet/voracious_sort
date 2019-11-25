use rand::{thread_rng, Rng};

// Uniform
pub fn helper_random_array_uniform_u128(size: usize) -> Vec<u128> {
    let mut rng = thread_rng();
    let mut array: Vec<u128> = Vec::with_capacity(size);
    for _ in 0..size {
        let v1: u64 = rng.gen();
        let v2: u64 = rng.gen();
        let value: u128 = ((v1 as u128) << 64) | (v2 as u128);
        array.push(value);
    }
    array
}

pub fn generators_u128(
) -> Vec<(&'static dyn Fn(usize) -> Vec<u128>, &'static str)> {
    vec![(&helper_random_array_uniform_u128, "-- Unif       :")]
}
