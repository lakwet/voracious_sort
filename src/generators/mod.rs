#[allow(dead_code)] pub mod boolean;
#[allow(dead_code)] pub mod char;
#[allow(dead_code)] pub mod float_32;
#[allow(dead_code)] pub mod float_64;
#[allow(dead_code)] pub mod float_extra;
#[allow(dead_code)] pub mod int128_extra;
#[allow(dead_code)] pub mod int_extra;
#[allow(dead_code)] pub mod narrow_extra;
#[allow(dead_code)] pub mod records;
#[allow(dead_code)] pub mod signed_i128;
#[allow(dead_code)] pub mod signed_i16;
#[allow(dead_code)] pub mod signed_i32;
#[allow(dead_code)] pub mod signed_i64;
#[allow(dead_code)] pub mod signed_i8;
// #[allow(dead_code)] pub mod string;
#[allow(dead_code)] pub mod unsigned_u128;
#[allow(dead_code)] pub mod unsigned_u16;
#[allow(dead_code)] pub mod unsigned_u32;
#[allow(dead_code)] pub mod unsigned_u64;
#[allow(dead_code)] pub mod unsigned_u8;

use rand_distr::Distribution;
use rayon::prelude::*;

/// RNG of the generators: the thread RNG, or, once `seed` was called on
/// the thread, a seeded generator (reproducible inputs for the decision
/// trace; generate in a one-thread rayon pool, `seed` called inside it).
pub struct GenRng(Option<rand::rngs::ThreadRng>);

thread_local! {
    static SEEDED: std::cell::RefCell<Option<rand::rngs::Xoshiro256PlusPlus>> = const { std::cell::RefCell::new(None) };
}

/// From now on, the generators of this thread draw from `seed`.
pub fn seed(seed: u64) {
    use rand::SeedableRng;
    SEEDED.with(|r| *r.borrow_mut() = Some(rand::rngs::Xoshiro256PlusPlus::seed_from_u64(seed)));
}

pub fn rng() -> GenRng {
    GenRng(if SEEDED.with(|r| r.borrow().is_some()) { None } else { Some(rand::rng()) })
}

impl rand::TryRng for GenRng {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        match &mut self.0 {
            Some(t) => t.try_next_u32(),
            None => SEEDED.with(|r| r.borrow_mut().as_mut().unwrap().try_next_u32()),
        }
    }
    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        match &mut self.0 {
            Some(t) => t.try_next_u64(),
            None => SEEDED.with(|r| r.borrow_mut().as_mut().unwrap().try_next_u64()),
        }
    }
    #[inline]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        match &mut self.0 {
            Some(t) => t.try_fill_bytes(dst),
            None => SEEDED.with(|r| r.borrow_mut().as_mut().unwrap().try_fill_bytes(dst)),
        }
    }
}

/// Zipf distribution (exponent 1, word frequencies in a text): `min(size,
/// domain)` random values, the value of rank k drawn with a probability in
/// 1 / k (the most frequent about 1 / ln(n) of the keys at n distinct
/// values, half of the values seen once or twice).
pub fn zipf<T: Copy + Send + Sync>(size: usize, domain: usize, value: impl Fn() -> T) -> Vec<T> {
    if size == 0 {
        return Vec::new();
    }
    let n = size.min(domain);
    let table: Vec<T> = (0..n).map(|_| value()).collect();
    let z = rand_distr::Zipf::new(n as f64, 1.0).unwrap();
    (0..size)
        .into_par_iter()
        .map(|_| {
            let k = z.sample(&mut crate::generators::rng()) as usize;
            table[k.clamp(1, n) - 1]
        })
        .collect()
}
