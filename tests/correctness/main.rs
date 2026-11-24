//! Correctness of the public API, run by `cargo test` / `make test`: every
//! key type on every distribution of the generators (shared with the
//! benchmarks: `src/generators`), at the thresholds of the decision chains
//! and the benchmark sizes, against the standard library sort.
//!
//! `QUICK=1`: sizes up to 100k only.

#[path = "../../src/generators/mod.rs"]
#[allow(dead_code)]
mod generators;

mod api;
mod floats;
mod keys;
mod records;

/// `QUICK` set: sizes up to 100k only.
fn quick_max() -> usize {
    if std::env::var("QUICK").is_ok() {
        100_000
    } else {
        usize::MAX
    }
}

/// The sizes of the benchmarks.
const BENCH_SIZES: [usize; 10] =
    [1_000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000, 3_000_000, 10_000_000];

/// The fixed thresholds of the chains, each minus one, itself, plus one
/// (sizes up to 70 are all tested): small sort (`SMALL` 128 / 150 / 200,
/// `FEW_MIN` / `DETECT_MIN` 129), `FEW_MAX` 256, 350, `MANY_DISTINCT` 500,
/// `HEAVY_MIN` 512 / 2048, 700, `DROPS_MAX` 1024, the 16-bit probes on 18
/// keys up to 1099 (`TINY_MAX` 1100), `PROBE_MIN` 2048, the 16-bit short
/// sort up to 5504,
/// `VERGE_MIN` / `DENSE_MIN` / `TAIL_PATH_MIN` 4096, `WIDE_MIN` 8192,
/// `FEW_BIG_MIN` 16384, `IN_CACHE_MIN` / `UNSTABLE_PDQ_MAX` 32768,
/// `UNSTABLE_PDQ_MIN` / `RANGE_MAX` / SIMD merges 65536 (u16 counters of the
/// byte count up to 65535), then powers of two up to 4M. The thresholds
/// derived from the cache: `test_v2_cache_thresholds`.
const THRESHOLD_SIZES: [usize; 60] = [
    127, 128, 129, 130, 149, 150, 151, 199, 200, 201, 255, 256, 257, 350, 351, 499, 500, 501, 511,
    512, 513, 700, 701, 1_023, 1_024, 1_025, 1_099, 1_100, 1_101, 2_047, 2_048, 2_049, 4_095,
    4_096, 4_097, 5_000, 5_503, 5_504, 5_505, 8_191, 8_192, 8_193, 16_383, 16_384, 16_385, 32_767,
    32_768, 32_769, 65_535, 65_536, 65_537, 131_072, 262_144, 524_288, 1_048_575, 1_048_576,
    1_048_577, 2_097_152, 4_194_304, 8_388_608,
];
