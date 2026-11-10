//! Generators shared by u32, u64, i32 and i64, added to each type's own list
//! so that every integer type is measured on the same situations: real data
//! shapes (identifiers, timestamps, aligned addresses, sorted logs) and the
//! worst cases of the detectors (late outlier, late extra value, random tail).

use rand::seq::SliceRandom;
use rand::RngExt;
use rayon::prelude::*;

macro_rules! int_extra {
    ($t:ty, $name:ident, $stamp_base:expr) => {
        pub mod $name {
            use super::*;

            // Narrow range (ratings, ages, counters): 0..=100
            pub fn int_narrow(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random_range(0..=100) as $t).collect()
            }

            // Identifiers: shuffled 0..n
            pub fn int_wide(size: usize) -> Vec<$t> {
                let mut array: Vec<$t> = (0..size).map(|i| i as $t).collect();
                array.as_mut_slice().shuffle(&mut rand::rng());
                array
            }

            // Few distinct values: 16 random full-range values
            pub fn few16(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let values: Vec<$t> = (0..16).map(|_| rng.random::<$t>()).collect();
                (0..size).map(|_| values[rng.random_range(0..16)]).collect()
            }

            // Detector worst case: narrow integers, one wide outlier late (at 95 %)
            pub fn int_late_outlier(size: usize) -> Vec<$t> {
                let mut array = int_narrow(size);
                if size > 0 {
                    array[size * 95 / 100] = <$t>::MAX / 3;
                }
                array
            }

            // Detector worst case: 16 distinct values, a 17th one late (at 95 %)
            pub fn few16_late(size: usize) -> Vec<$t> {
                let mut array = few16(size);
                if size > 0 {
                    array[size * 95 / 100] = 12_345;
                }
                array
            }

            // Detector worst case: ascending for 90 %, random tail
            pub fn asc_random_tail(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let cut = size * 9 / 10;
                (0..size)
                    .map(|i| if i < cut { i as $t } else { rng.random_range(0..size.max(1)) as $t })
                    .collect()
            }

            // Concatenation of k independently sorted random sources
            // (sorted logs from several machines).
            fn sorted_runs(size: usize, k: usize) -> Vec<$t> {
                let mut array: Vec<$t> = (0..size).into_par_iter().map(|_| rand::rng().random::<$t>()).collect();
                for r in 0..k {
                    array[r * size / k..(r + 1) * size / k].sort_unstable();
                }
                array
            }

            pub fn runs8(size: usize) -> Vec<$t> {
                sorted_runs(size, 8)
            }

            pub fn runs64(size: usize) -> Vec<$t> {
                sorted_runs(size, 64)
            }

            // Tight clusters: n / 200 random centres, each key within 2^10 of
            // its centre, shuffled. The top bits name the cluster.
            pub fn clusters(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let k = (size / 200).max(1);
                let centres: Vec<$t> = (0..k).map(|_| rng.random::<$t>() >> 1).collect();
                let mut array: Vec<$t> =
                    (0..size).map(|i| centres[i % k].wrapping_add(rng.random_range(0..1024) as $t)).collect();
                array.as_mut_slice().shuffle(&mut rng);
                array
            }

            // Timestamps: a large common prefix (an epoch), increasing with
            // jitter (events logged slightly out of order).
            pub fn timestamps(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                (0..size)
                    .map(|i| ($stamp_base as $t).wrapping_add((i as $t).wrapping_mul(16)).wrapping_add(rng.random_range(0..256) as $t))
                    .collect()
            }

            // Aligned addresses: random multiples of 2^16 (low digits all zero)
            pub fn aligned(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random::<$t>() & !0xFFFF).collect()
            }

            // Common high prefix: a fixed random prefix, 20 random low bits
            pub fn prefix20(size: usize) -> Vec<$t> {
                let prefix = rand::rng().random::<$t>() & !0xF_FFFF;
                (0..size).into_par_iter().map(|_| prefix | (rand::rng().random_range(0..1 << 20) as $t)).collect()
            }

            // Bimodal: half near zero, half near the top of the range
            pub fn bimodal(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|i| {
                        let v = rand::rng().random_range(0..1 << 20) as $t;
                        if i % 2 == 0 { v } else { <$t>::MAX - v }
                    })
                    .collect()
            }

            pub fn generators() -> Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)> {
                vec![
                    (&int_narrow, "-- Int 0..100 :"),
                    (&int_wide, "-- Int 0..n   :"),
                    (&few16, "-- Few 16     :"),
                    (&int_late_outlier, "-- Int late out:"),
                    (&few16_late, "-- Few16 late :"),
                    (&asc_random_tail, "-- Asc rnd tail:"),
                    (&runs8, "-- Runs 8     :"),
                    (&runs64, "-- Runs 64    :"),
                    (&clusters, "-- Clusters   :"),
                    (&timestamps, "-- Timestamps :"),
                    (&aligned, "-- Aligned 2^16:"),
                    (&prefix20, "-- Prefix 20b :"),
                    (&bimodal, "-- Bimodal    :"),
                ]
            }
        }
    };
}

// Epochs: seconds since 1970 for 32-bit keys, nanoseconds for 64-bit keys.
int_extra!(u32, extra_u32, 1_700_000_000u32);
int_extra!(i32, extra_i32, 1_700_000_000i32);
int_extra!(u64, extra_u64, 1_700_000_000_000_000_000u64);
int_extra!(i64, extra_i64, 1_700_000_000_000_000_000i64);
