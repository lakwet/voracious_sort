//! Helpers of the v2 unit tests (each module tests its own pieces): a
//! deterministic generator, the array sizes and shapes the engines and
//! detectors meet, and the expected orders of the key codecs.

/// xorshift64*: deterministic, no dependency.
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Sizes: empty, tiny, around powers of two and the small-array limits
/// (networks, merges, 128-key fallbacks), and beyond the chunk sizes.
pub(crate) const SIZES: &[usize] = &[
    0, 1, 2, 3, 4, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 1000,
    4095, 4096, 4097, 20_000,
];

/// Sizes around every threshold derived from the cache of this machine:
/// each budget (`cache::BUDGETS`, in bytes) divided by the key widths and
/// by the divisors the chains use (L2 / 12, / 24, / 48...), minus one, the
/// size itself, plus one. Up to `max` keys.
pub(crate) fn cache_sizes(max: usize) -> Vec<usize> {
    use crate::v2::cache::{budget, BUDGETS};
    let mut sizes = Vec::new();
    for &(b, _) in BUDGETS.iter() {
        for d in [1, 2, 4, 8, 12, 16, 24, 32, 48, 64, 128] {
            let n = budget(b) / d;
            sizes.extend([n.saturating_sub(1), n, n + 1]);
        }
    }
    sizes.retain(|&n| n > 64 && n <= max);
    sizes.sort_unstable();
    sizes.dedup();
    sizes
}

/// Sizes beyond the cache-sized pieces (rollercoaster leaves, ping-pong).
pub(crate) const LARGE: &[usize] = &[40_000, 200_000];

/// Raw 64-bit patterns for each shape: random, few distinct values, all
/// equal, ascending, descending, only the extremes (0, 1, MAX - 1, MAX),
/// a narrow range, the high bits only, the low bits only, a sawtooth and a
/// dense range with outliers.
pub(crate) fn shapes(n: usize, seed: u64) -> Vec<(&'static str, Vec<u64>)> {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let random: Vec<u64> = (0..n).map(|_| r.next()).collect();
    let mut asc = random.clone();
    asc.sort_unstable();
    let desc: Vec<u64> = asc.iter().rev().copied().collect();
    let pick = |r: &mut Rng, k: u64| r.next() % k;
    vec![
        ("random", random),
        ("few", (0..n).map(|_| pick(&mut r, 7) * 0x0101_0101_0101).collect()),
        ("equal", vec![0x5A5A_5A5A_5A5A_5A5A; n]),
        ("asc", asc),
        ("desc", desc),
        (
            "extremes",
            (0..n).map(|_| [0, 1, u64::MAX - 1, u64::MAX][pick(&mut r, 4) as usize]).collect(),
        ),
        ("narrow", (0..n).map(|_| 1_000_000 + pick(&mut r, 1000)).collect()),
        ("high", (0..n).map(|_| r.next() & 0xFFFF_0000_0000_0000).collect()),
        ("low", (0..n).map(|_| r.next() & 0xFFFF).collect()),
        ("saw", (0..n).map(|i| (i % 64) as u64).collect()),
        (
            "dense",
            (0..n)
                .map(|_| if pick(&mut r, 10) == 0 { r.next() } else { 300 + pick(&mut r, 512) })
                .collect(),
        ),
    ]
}

/// A 128-bit key from a 64-bit one: equal inputs stay equal, the extremes
/// stay the extremes.
pub(crate) fn wide(x: u64) -> u128 { ((x.rotate_left(29) as u128) << 64) | x as u128 }

/// Whether `a` and `b` hold the same values.
pub(crate) fn same_values<T: Copy + Ord>(a: &[T], b: &[T]) -> bool {
    let (mut x, mut y) = (a.to_vec(), b.to_vec());
    x.sort_unstable();
    y.sort_unstable();
    x == y
}

/// Whether `a` is in order by `key`.
pub(crate) fn sorted_by<T: Copy, K: Ord>(a: &[T], key: impl Fn(T) -> K) -> bool {
    a.windows(2).all(|w| key(w[0]) <= key(w[1]))
}

/// The unsigned image of raw float bits in their total order (the order
/// the float engines give: -NaN, -inf, ..., -0, +0, ..., +inf, +NaN).
pub(crate) fn float_key_32(b: u32) -> u32 { b ^ ((((b as i32) >> 31) as u32) | (1 << 31)) }
pub(crate) fn float_key_64(b: u64) -> u64 { b ^ ((((b as i64) >> 63) as u64) | (1 << 63)) }

/// Unsigned keys built from the raw patterns, two ways: from the low bits
/// and from the high bits (64-bit: byte-swapped; 128-bit: `wide`).
pub(crate) trait FromRaw: Copy + Ord + std::fmt::Debug {
    fn from_raw(x: u64) -> [Self; 2];
}

macro_rules! from_raw {
    ($($u:ty),*) => {$(
        impl FromRaw for $u {
            fn from_raw(x: u64) -> [$u; 2] { [x as $u, (x >> (64 - <$u>::BITS)) as $u] }
        }
    )*};
}
from_raw!(u8, u16, u32);

impl FromRaw for u64 {
    fn from_raw(x: u64) -> [u64; 2] { [x, x.swap_bytes()] }
}

impl FromRaw for u128 {
    fn from_raw(x: u64) -> [u128; 2] { [x as u128, wide(x)] }
}

/// Calls `f` on every test array of `T`: every shape at every size of
/// `SIZES`, then the random, few and dense shapes at the `LARGE` sizes,
/// each built both ways (`FromRaw`).
pub(crate) fn for_each_case<T: FromRaw>(mut f: impl FnMut(&str, Vec<T>)) {
    for &n in SIZES.iter().chain(LARGE.iter()) {
        for (shape, raw) in shapes(n, n as u64) {
            if n > 20_000 && !matches!(shape, "random" | "few" | "dense") {
                continue;
            }
            for way in 0..2 {
                f(shape, raw.iter().map(|&x| T::from_raw(x)[way]).collect());
            }
        }
    }
}
