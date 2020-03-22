use rand::{thread_rng, Rng};
use rayon::prelude::*;

fn get_charset() -> Vec<char> {
    vec![
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
        'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
        'U', 'V', 'W', 'X', 'Y', 'Z',
    ]
}

pub fn helper_random_array_uniform_char(size: usize) -> Vec<char> {
    (0..size)
        .into_par_iter()
        .map(|_| thread_rng().gen::<char>())
        .collect::<Vec<char>>()
}

pub fn helper_random_array_equal_char(size: usize) -> Vec<char> {
    vec![thread_rng().gen::<char>(); size]
}

pub fn helper_random_array_charset_char(size: usize) -> Vec<char> {
    let charset = get_charset();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let index: usize = thread_rng().gen_range(0, charset.len());
            charset[index]
        })
        .collect::<Vec<char>>()
}

pub fn helper_random_array_charset_den_char(size: usize) -> Vec<char> {
    let mut charset = get_charset();
    charset.push(std::char::from_u32(0x00000db4).unwrap());
    (0..size)
        .into_par_iter()
        .map(|_| {
            let index: usize = thread_rng().gen_range(0, charset.len());
            charset[index]
        })
        .collect::<Vec<char>>()
}

pub fn helper_random_array_charset_vden_char(size: usize) -> Vec<char> {
    let mut charset = get_charset();
    for i in 0..11 {
        let char_u32 = std::char::MAX as u32 - i as u32;
        charset.push(std::char::from_u32(char_u32).unwrap());
    }
    (0..size)
        .into_par_iter()
        .map(|_| {
            let index: usize = thread_rng().gen_range(0, charset.len());
            charset[index]
        })
        .collect::<Vec<char>>()
}

pub fn generators_char(
) -> Vec<(&'static dyn Fn(usize) -> Vec<char>, &'static str)> {
    vec![
        (&helper_random_array_uniform_char, "-- Unif       :"),
        (&helper_random_array_equal_char, "-- Equal      :"),
        (&helper_random_array_charset_char, "-- Charset    :"),
        (&helper_random_array_charset_den_char, "-- Charset Den:"),
        (&helper_random_array_charset_vden_char, "-- Charset VDe:"),
    ]
}
