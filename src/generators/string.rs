use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn helper_random_array_uniform_string(
    size: usize,
    string_size: usize,
) -> Vec<String> {
    let mut rng = thread_rng();
    let mut array: Vec<String> = Vec::with_capacity(size);

    for _ in 0..size {
        let s = rng
            .sample_iter(&Alphanumeric)
            .take(string_size)
            .collect::<String>();
        array.push(s);
    }

    array
}

pub fn generators_string(
) -> Vec<(&'static dyn Fn(usize, usize) -> Vec<String>, &'static str)> {
    vec![(&helper_random_array_uniform_string, "-- Unif       :")]
}
