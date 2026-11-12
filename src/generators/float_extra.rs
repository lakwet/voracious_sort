//! Generators shared by f32 and f64, appended to each type's own list:
//! shapes missing from one or both lists (heavy tails, timestamps, values at
//! both ends, magnitudes spread over many exponents, signed zeros).

use rand::RngExt;
use rand_distr::{Distribution, Pareto};
use rayon::prelude::*;

macro_rules! float_extra {
    ($t:ty, $name:ident, $pareto_set:expr) => {
        pub mod $name {
            use super::*;

            fn pareto(size: usize, shape: f64) -> Vec<$t> {
                let p = Pareto::new(0.1, shape).unwrap();
                (0..size).into_par_iter().map(|_| p.sample(&mut rand::rng()) as $t).collect()
            }

            // Heavy-tailed sizes, prices, populations.
            pub fn pareto_075(size: usize) -> Vec<$t> {
                pareto(size, 0.75)
            }

            pub fn pareto_100(size: usize) -> Vec<$t> {
                pareto(size, 1.0)
            }

            pub fn pareto_200(size: usize) -> Vec<$t> {
                pareto(size, 2.0)
            }

            pub fn pareto_150(size: usize) -> Vec<$t> {
                pareto(size, 1.5)
            }

            // Unix times in seconds with a fraction, increasing with jitter.
            pub fn timestamps(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|i| (1.7e9 + i as f64 * 0.01 + rand::rng().random_range(0.0..0.05)) as $t)
                    .collect()
            }

            // Values at both ends: half near -10^6, half near +10^6.
            pub fn bimodal(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|i| {
                        let v = 1e6 + rand::rng().random_range(0.0..1000.0);
                        (if i % 2 == 0 { -v } else { v }) as $t
                    })
                    .collect()
            }

            // Signed magnitudes spread over 20 decades (10^-10 to 10^10).
            pub fn magnitudes(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = rand::rng();
                        let v = 10f64.powf(rng.random_range(-10.0..10.0));
                        (if rng.random::<bool>() { -v } else { v }) as $t
                    })
                    .collect()
            }

            // Sensor readings: 20 +- 5 with a 0.01 resolution.
            pub fn sensor(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| (rand::rng().random_range(1500..2500) as f64 / 100.0) as $t)
                    .collect()
            }

            // 90 % signed zeros (-0.0 and +0.0), 10 % random.
            pub fn zeros(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = rand::rng();
                        match rng.random_range(0..20) {
                            0 | 1 => rng.random_range(-1e3..1e3) as $t,
                            k if k % 2 == 0 => -0.0,
                            _ => 0.0,
                        }
                    })
                    .collect()
            }

            pub fn generators() -> Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)> {
                let mut v: Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)> = Vec::new();
                // The full Pareto set where the type's own list lacks it.
                if $pareto_set {
                    v.push((&pareto_075, "-- Pareto 0.75:"));
                    v.push((&pareto_100, "-- Pareto 1.00:"));
                    v.push((&pareto_200, "-- Pareto 2.00:"));
                }
                v.extend::<Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)>>(vec![
                    (&pareto_150, "-- Pareto 1.50:"),
                    (&timestamps, "-- Timestamps :"),
                    (&bimodal, "-- Bimodal    :"),
                    (&magnitudes, "-- Magnitudes :"),
                    (&sensor, "-- Sensor     :"),
                    (&zeros, "-- Zeros +-   :"),
                ]);
                v
            }
        }
    };
}

float_extra!(f32, extra_f32, false);
float_extra!(f64, extra_f64, true);
