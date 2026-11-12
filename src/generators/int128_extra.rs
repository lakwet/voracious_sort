//! Generators for u128 and i128, added to each type's own list: the shapes
//! 128-bit keys take in practice (identifiers, timestamps, addresses, fixed
//! point amounts, composite keys) and the classic ones (presorted, few
//! values, runs).

use rand::seq::SliceRandom;
use rand::RngExt;
use rayon::prelude::*;

macro_rules! int128_extra {
    ($t:ty, $name:ident) => {
        pub mod $name {
            use super::*;

            fn random(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random::<u128>() as $t).collect()
            }

            // Random UUID (v4): 122 random bits, version and variant fixed.
            pub fn uuid_v4(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let r = rand::rng().random::<u128>();
                        ((r & !(0xF << 76) & !(0x3 << 62)) | (0x4 << 76) | (0x2 << 62)) as $t
                    })
                    .collect()
            }

            // Time-ordered UUID (v7): 48-bit milliseconds over one day, then
            // 74 random bits; collected out of order.
            pub fn uuid_v7(size: usize) -> Vec<$t> {
                const DAY_MS: u128 = 86_400_000;
                let start: u128 = 1_700_000_000_000;
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = rand::rng();
                        let ms = start + rng.random_range(0..DAY_MS);
                        ((ms << 80) | (0x7 << 76) | (rng.random::<u128>() & ((1 << 76) - 1) & !(0x3 << 62)) | (0x2 << 62)) as $t
                    })
                    .collect()
            }

            // UUID v7 as generated: increasing milliseconds, random tails.
            pub fn uuid_v7_sorted(size: usize) -> Vec<$t> {
                let start: u128 = 1_700_000_000_000;
                (0..size)
                    .into_par_iter()
                    .map(|i| {
                        let ms = start + i as u128 / 16;
                        ((ms << 80) | (0x7 << 76) | (rand::rng().random::<u128>() & ((1 << 62) - 1)) | (0x2 << 62)) as $t
                    })
                    .collect()
            }

            // Nanosecond timestamps (~2^61), increasing with jitter.
            pub fn timestamps_ns(size: usize) -> Vec<$t> {
                let start: u128 = 1_700_000_000_000_000_000;
                (0..size)
                    .into_par_iter()
                    .map(|i| (start + i as u128 * 1000 + rand::rng().random_range(0..5000)) as $t)
                    .collect()
            }

            // Small values stored in 128 bits.
            pub fn small32(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random::<u32>() as $t).collect()
            }

            pub fn small64(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random::<u64>() as $t).collect()
            }

            // Fixed-point amounts: up to 10^6 units with 18 decimals (< 2^80).
            pub fn fixed_point(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| rand::rng().random_range(0..1_000_000_000_000_000_000_000_000u128) as $t)
                    .collect()
            }

            // Composite keys: high half among 64 values (tenant), low half random.
            pub fn composite(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let highs: Vec<u128> = (0..64).map(|_| (rng.random::<u64>() as u128) << 64).collect();
                (0..size).map(|_| (highs[rng.random_range(0..64)] | rng.random::<u64>() as u128) as $t).collect()
            }

            // IPv6 addresses: 1024 /48 prefixes, a small subnet, random interface ids.
            pub fn ipv6(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let prefixes: Vec<u128> = (0..1024).map(|_| (rng.random::<u64>() as u128 >> 16) << 80).collect();
                (0..size)
                    .map(|_| {
                        let subnet = (rng.random_range(0..16u128)) << 64;
                        (prefixes[rng.random_range(0..1024)] | subnet | rng.random::<u64>() as u128) as $t
                    })
                    .collect()
            }

            // Tight clusters: n / 200 random centres, each key within 2^10.
            pub fn clusters(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let k = (size / 200).max(1);
                let centres: Vec<u128> = (0..k).map(|_| rng.random::<u128>() >> 1).collect();
                let mut array: Vec<$t> = (0..size).map(|i| (centres[i % k] + rng.random_range(0..1024u128)) as $t).collect();
                array.as_mut_slice().shuffle(&mut rng);
                array
            }

            // Bimodal: half near zero, half near the top of the domain.
            pub fn bimodal(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|i| {
                        let v = rand::rng().random_range(0..1u128 << 40) as $t;
                        if i % 2 == 0 { v } else { <$t>::MAX - v }
                    })
                    .collect()
            }

            pub fn asc(size: usize) -> Vec<$t> {
                let mut a = random(size);
                a.sort_unstable();
                a
            }

            pub fn desc(size: usize) -> Vec<$t> {
                let mut a = asc(size);
                a.reverse();
                a
            }

            pub fn equal(size: usize) -> Vec<$t> {
                vec![rand::rng().random::<u128>() as $t; size]
            }

            // Sorted, then 1 % of the keys swapped at random.
            pub fn almost_asc(size: usize) -> Vec<$t> {
                let mut a = asc(size);
                let mut rng = rand::rng();
                for _ in 0..size / 100 {
                    let (i, j) = (rng.random_range(0..size), rng.random_range(0..size));
                    a.swap(i, j);
                }
                a
            }

            // Sorted for 90 %, random tail.
            pub fn asc_random_tail(size: usize) -> Vec<$t> {
                let mut a = random(size);
                let cut = size * 9 / 10;
                a[..cut].sort_unstable();
                a
            }

            // Concatenation of 8 sorted random runs.
            pub fn runs8(size: usize) -> Vec<$t> {
                let mut a = random(size);
                for r in 0..8 {
                    a[r * size / 8..(r + 1) * size / 8].sort_unstable();
                }
                a
            }

            pub fn pipe_organ(size: usize) -> Vec<$t> {
                let mut a = random(size);
                let h = size / 2;
                a[..h].sort_unstable();
                a[h..].sort_unstable_by(|x, y| y.cmp(x));
                a
            }

            // 16 distinct random values.
            pub fn few16(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let values: Vec<$t> = (0..16).map(|_| rng.random::<u128>() as $t).collect();
                (0..size).map(|_| values[rng.random_range(0..16)]).collect()
            }

            // 60 % of the keys among 4 values, the rest random.
            pub fn heavy4(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let values: Vec<$t> = (0..4).map(|_| rng.random::<u128>() as $t).collect();
                (0..size)
                    .map(|_| if rng.random_range(0..10) < 6 { values[rng.random_range(0..4)] } else { rng.random::<u128>() as $t })
                    .collect()
            }

            // 90 % zeros, 10 % random.
            pub fn sparse90(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = rand::rng();
                        if rng.random_range(0..10) == 0 { rng.random::<u128>() as $t } else { 0 }
                    })
                    .collect()
            }

            // Zipf over 10^6 random values (exponent 1).
            pub fn zipf(size: usize) -> Vec<$t> {
                super::super::zipf(size, 1_000_000, || rand::rng().random::<u128>() as $t)
            }

            pub fn generators() -> Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)> {
                vec![
                    (&uuid_v4, "-- UUID v4    :"),
                    (&uuid_v7, "-- UUID v7    :"),
                    (&uuid_v7_sorted, "-- UUID v7 asc:"),
                    (&timestamps_ns, "-- Stamps ns  :"),
                    (&small32, "-- Small 32   :"),
                    (&small64, "-- Small 64   :"),
                    (&fixed_point, "-- Fixed point:"),
                    (&composite, "-- Composite  :"),
                    (&ipv6, "-- IPv6       :"),
                    (&clusters, "-- Clusters   :"),
                    (&bimodal, "-- Bimodal    :"),
                    (&asc, "-- Asc        :"),
                    (&desc, "-- Desc       :"),
                    (&equal, "-- Equal      :"),
                    (&almost_asc, "-- Almost Asc :"),
                    (&asc_random_tail, "-- Asc rnd tail:"),
                    (&runs8, "-- Runs 8     :"),
                    (&pipe_organ, "-- Pipe Organ :"),
                    (&few16, "-- Few 16     :"),
                    (&heavy4, "-- Heavy 4    :"),
                    (&sparse90, "-- Sparse 90% :"),
                    (&zipf, "-- Zipf       :"),
                ]
            }
        }
    };
}

int128_extra!(u128, extra_u128);
int128_extra!(i128, extra_i128);
