# Version **2.1.2** (November 28<sup>th</sup> 2026)

A new sort for `u32` and `i32`, and two fixes for `u16` and `i16`.

- `u32` and `i32` have a new decision chain, built as the 16-bit one was
  (engines alone, then each question measured in front of the winner):
  - up to 512 keys on x86 (80 to 256 on ARM), comparison networks in SIMD
    registers (AVX2, detected at run time; NEON), the range of the keys
    checked in the registers and a narrow one counted instead;
  - up to 600 keys, a short sort with few questions: one pass gives the
    descents and the bounds of the keys (sorted, reversed, narrow ranges
    counted on the stack), 8 probes catch arrays of at most 4 values or one
    dominant value;
  - up to 8192 keys, a light chain: only the cheap questions (range, few
    values, a dominant value, dense range), then the engine;
  - the diverting LSD keeps its buffer on the stack below 4096 keys, and
    sends to the plain LSD the keys whose order shows only from afar
    (timestamps with jitter) and composite keys (a few groups of uniform
    values) from 65 536 keys;
  - the counting tables of 128 to 256 KiB are on the stack (new pages on
    the heap fault at each call; see the README, Memory).
  On our machines the new sort wins more inputs than 2.1 at every size,
  most of all below 20 000 keys; the benchmark of `u32` / `i32` comes with
  a later release.

- From 131 072 keys the 16-bit sort counts the whole domain in a table of
  256 KiB. That table was allocated at each call, and an allocation of that
  size gets new pages from the system: a page fault for every 4 KiB of it,
  a large part of the time from 200 000 to 1 million keys. The table is now
  on the stack (the sort needs 256 KiB of stack for these arrays). In a
  program that sorts once, 200 000 to 1 million keys are now up to 2 times
  faster on our machines (the median input 1.5 times at 200 000 keys,
  1.15 times at 1 million).
- Built with `-C target-cpu=native` (or for `alderlake`, `meteorlake` and
  other CPUs with AVX-VNNI or AVX-512), the compiler turned the counting
  loops of the 16-bit sort into vector gathers, which are slow on these
  CPUs: narrow ranges, values at both ends of the domain and aligned values
  were up to 2 times slower than in the generic build. The counters are now
  read in a way the compiler cannot gather on these CPUs; on a Core Ultra 9
  185H the affected inputs are up to 2 times faster. The generic build and
  other CPUs run the same code as before.

# Version **2.1.1** (November 25<sup>th</sup> 2026)

A lighter build, nothing else changes.

- The code of the 1.x sorts, which only the multithread sort still uses, is
  now compiled only with the feature `voracious_multithread`. Without it the
  crate builds about 45 % faster (release build of the library: 44 s to
  25 s on our machine) and its compiled library is a third smaller. Programs
  that use the crate run the same code as with 2.1.0.

# Version **2.1.0** (November 24<sup>th</sup> 2026)

The 16-bit integers (`u16`, `i16`) reworked, and the code split into three
sorts. No breaking change: the 2.0 API is unchanged.

### What changes for users

- New free functions `famished_sort_by_key` (stable) and
  `famished_sort_unstable_by_key`: structs sorted by a key, the same sorts as
  the trait methods `voracious_sort_by_key` and
  `voracious_sort_unstable_by_key`.
- `u16` and `i16` are faster, from a few hundred keys to tens of millions
  (benchmark results to follow in `resultsbenchs/`).

### Inside

- Three sorts, one folder each: `devour` (integers, `bool`, `char`),
  `ravenous` (floats), `famished` (structs by key).
- One chain of decisions for each size of integer, signed and unsigned
  together (the sign bit flipped wherever the order matters): `u16` and
  `i16` share one code, so a fix for one is a fix for the other. The other
  sizes follow in the next releases.
- 16 bits, by size: up to 5504 keys a short sort (one order scan, cheap
  questions on 18 keys or on a sample, then a two-pass LSD on a stack
  buffer); up to 65 536 keys a light chain (the same questions, then the
  near-sorted repair, runs and Verge); beyond, the full chain. The
  questions that mostly fail (few values, one heavy value, random keys)
  are answered without a branch per key, so they cost almost nothing when
  they fail;
- 16 bits: a short preparation below a few thousand keys (dominant value,
  narrow window of 256 values, few distinct values estimated on a sample);
  the counting of the whole domain reading four keys per load; LSD counters
  of 16 or 32 bits and the choice between a 256-value window and the domain
  count following the size of the L2 cache, read at run time; faster run
  merges (pipe organ, mostly descending sawtooth arrays).
- The sorts of structs by key: one code for records up to 8 bytes and one
  for larger records (before: one file per key type and record size).
- Benchmarks: `make bench-type` now measures cold (each array sorted once,
  on a fresh copy, the arrays together larger than the caches; median of 11
  measurements); `make bench-16` runs `u16` and `i16`, generic build then
  native build.

# Version **2.0.0** (November 21<sup>st</sup> 2026)

**Breaking change.** The single-thread sort is rewritten; the public API is
smaller and simpler. Version 1.2.2 stays available for those who prefer it.

### What changes for users

- `RadixSort` is now one trait without type parameters, implemented for
  slices (and `Vec` through `Deref`):
  - `voracious_sort()` and `voracious_stable_sort()` sort `bool`, `char`,
    `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`,
    `i128`, `isize`, `f32` and `f64`;
  - `voracious_mt_sort(thread_n)` (feature `voracious_multithread`): the
    multithread sort of 1.x (peeka sort), **not reworked yet**;
  - `voracious_sort_by_key(key)` (stable) and
    `voracious_sort_unstable_by_key(key)` sort any element by a key.
- **`Radixable` and `RadixKey` are removed.** Structs no longer implement a
  trait to be sorted: call `voracious_sort_by_key(|x| x.key)` instead. A key
  type of your own implements `SortKey` (its encoding into an unsigned
  integer of the same order).
- **The 1.x sorting functions are no longer public**: `american_flag_sort`,
  `counting_sort`, `dlsd_radixsort`, `lsd_radixsort`, `lsd_stable_radixsort`,
  `msd_radixsort`, `msd_stable_radixsort`, `peeka_sort`,
  `rollercoaster_sort`, `ska_sort`, `awardofthesky_sort`, `voracious_sort`,
  `boolean_sort`, `insertion_sort`, `cs_u16`, `lsd_f32`, `lsd_u32`, and the
  `Dispatcher` trait. Use the trait methods.
- New free functions: `devour_sort` (`bool`, `char`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`) and
  `ravenous_sort` (`f32`, `f64`), the same sorts as `voracious_sort`.
- Floats are sorted in [`f64::total_cmp`] order (NaNs of both signs, -0.0
  before +0.0), bit for bit.
- Minimum Rust version: 1.88.

### Migration

```rust
// 1.x
impl Radixable<u64> for Event { type Key = u64; fn key(&self) -> u64 { self.time } }
events.voracious_sort();
// 2.0
events.voracious_sort_by_key(|e| e.time);
```

### Inside

- One chain of decisions per key type: sorting networks and merges for
  small arrays, detection of sorted, reversed and nearly sorted arrays, of
  narrow ranges, few values, heavy duplicates, plateaus and clusters, Verge
  runs merged by a k-way merge (powersort order, half-size buffer), LSD /
  DLSD / MSD radix sorts and the rollercoaster, thresholds derived from the
  cache sizes read at run time.
- A new sort for records by key (key and index packed when they fit).
- New tests: the public API on every type and many distributions at every
  threshold, Miri (`make test-miri`), fuzzy tests (`make test-fuzzy`); CI on
  Linux, Windows and macOS.
- New benchmarks (`make bench`, `make bench-type`, `make bench-records`,
  `make bench-big`): duration, time per element and ratio to v2 of every
  sort; no results published with this release.

# Version **1.2.2** (November 2026)

### Bug fix:

- Sorting `f32` / `f64` arrays (or structs with a float key) that contain NaN
panicked, from about ten elements on (the comparison parts of the sorts, such
as the small-array fallbacks and the run detection, cannot compare a NaN).
NaNs are now set aside before sorting: negative NaNs first, positive NaNs
last (each group in the order of its bits), the other values sorted between
them. This applies to the trait methods (`voracious_sort`,
`voracious_stable_sort`, `voracious_mt_sort`) and to the public sorts
(`american_flag_sort`, `counting_sort`, `dlsd_radixsort`, `lsd_radixsort`,
`lsd_stable_radixsort`, `msd_radixsort`, `msd_stable_radixsort`,
`peeka_sort`, `rollercoaster_sort`, `ska_sort`, `thiel_radixsort`,
`voracious_sort`).

### Performance:

- Float sorts read the array once more before sorting, to find NaNs; an
already sorted float array is now returned at once. Integer types are not
affected. **The performance of this release has not been measured**: small
regressions are possible on float arrays. Version 2.0 will replace these
sorts.

# Version **1.2.1** (November 15<sup>th</sup> 2026)

### Bug fix:

- `voracious_sort` and `voracious_stable_sort` could panic on `f64` arrays
(index out of bounds in `copy_by_histogram`), depending on the data, for
example timestamps of 20 000 to 200 000 elements. The dedicated `f64`
histogram did not count the lowest digit in its 4-level loop, so the LSD radix
sort used wrong bucket heads when it sorted exactly four levels.
- `lsd_radixsort` (and the sorts that call it with these levels) could panic
on `u128` keys spanning 113 to 120 bits: the generic 15-level histogram did not
count the highest digit of the last 1 to 3 elements. The trait methods were not
affected.
- `voracious_stable_sort` called on a `Vec` ran the unstable sort. It now runs
the stable sort, as on a slice (thanks to Yikai-Liao, PR #24).
- Stable sorts were not stable on descending runs: the preprocessing reversed
them with their groups of equal keys. Each group now keeps its original order
(thanks to Yikai-Liao, PR #24).
- The crate did not build on 32-bit targets (for example
`wasm32-unknown-unknown`): missing feature guards and imports (thanks to
Ferdinand Schober, PR #21).

### Dependency:

- Use Rayon 1.12.0 instead of 1.7.0.
- Tests and generators: use rand 0.10.3 instead of 0.7.3 and rand_distr 0.6.0
instead of 0.3.0.

### Misc:

- Readme: how to run the tests, and how to build them on Windows with the GNU
toolchain.
- Tests: the "almost descending" generators of u32 and u64 gave almost
ascending arrays; new tests of stability, of the histograms of every type at
every level, and of the u128 LSD at every key width.

# Version **1.2.0** (March 18<sup>th</sup> 2023)

### Dependency:

- Use Rayon 1.7.0 instead of 1.5.3.

### Bug fix:

- Peeka sort did not scale correctly because of constants. Now it uses dynanmic
values for blocks size for the parallele algorithm. It now scale for arrays whose
size is bigger than 1_000_000_000.
- Dispatcher trait was bugged for the stable sort. If using stable sort and custom
structs, the stable sort called the unstable rust sort as fallback. It now calls
the stable rust sort for the fallback.

### Misc:

- Add disclaimer about memory consumption in the Readme.
- Add disclaimer about array size in the Readme.
- Add human readable benchmark results.

# Version **1.1.1** (August 20<sup>th</sup> 2022)

### Dependency:

- Use Rayon 1.5.3 instead of 1.5.0.

### Misc:

- Update Readme.
- Add Github workflows.

# Version **1.1.0** (November 7<sup>th</sup> 2020)

### Features:

Now you can choose only the single thread version, without having `rayon` as
dependency. Or the full version, including the multithread version, and with
`rayon` as a dependency so the compilation time is longer.

You just  have to add (or not) the features flag "`voracious_multithread`". See
the doc.

Since the compilation time was longer for the multithread version, and not
everyone need it, it is now possible to skip it.

Moreover it means the data you sort do not need anymore to by `Send + Sync` if
you use the single thread version.

### Improvement:

- Update the fallback constant in the Peeka sort. It is a bit faster now.
Instead of "fallbacking" when the chunk is smaller than 20_000 elements, it now
fallbacks when the chunk is smaller than 128_000 elements.

### Bugs fixes:

- Fix the bug with the vergesort pre-processing heuristic. This improves
performances for few distributions.

### Other:

- Bump rayon version to 1.5.0.
- Fix typo in doc.
- Update doc.
- Add more benchmark results.
- Add more distributions.

# Version **1.0.0** (September 9<sup>th</sup> 2020)

### New single thread sort:

- Rollercoaster sort (MSD radix sort).
- LSD stable sort (LSD radix sort).

### New multi thread sort:

- Peeka sort (MSD radix sort). An improvement of the MIT's researchers Regions sort.

### New dedicated single thread sorts:

- LSD u32 sort (LSD radix sort for u32).
- Counting sort for u16.

### New dependency:

- Rayon 1.4.0

### Improvements:

- DSLD sort fallback.
- Use Rollercoaster sort for `f32`, `f64`, `i32` and `i64`, which significantly improve performance.

### Bugs fixes:

- Fix the case where a vector (or slice) has only zeros.
- Fix "left shift" for MSD radix sort (American flag sort, MSD sort, MSD stable sort, Ska sort, Voracious sort).
- Fix stable sort in trait.

### Other:

- Add support for **isize** and **usize** types.
- Add multithreading to generate random vectors for tests.
- Use Rayon multi thread sort to check if an array is well sorted in tests.
- Add Pareto distribution for f32 and f64 random vectors for tests.
- Add more pattern for vectors for tests.
- Add tests for dedicated sorts.
- Add tests for new sorts.
- Add missing tests for helpers functions.
- Replace a lot of unsafe code by using `chunks_exact` method.
- Remove useless trait constraints.
- Update documentation and [README.md](https://github.com/lakwet/voracious_sort/blob/master/README.md).
- Add [RELEASES.md](https://github.com/lakwet/voracious_sort/blob/master/RELEASES.md).
- Add [PROFILING.md](https://github.com/lakwet/voracious_sort/blob/master/PROFILING.md).
- Replace obsolete benchmark by new benchmark.

### Profiling:

- Start the profiling for `bool`, `char`, `f32`, `f64`, `u8` on a Ryzen 9 3950x.
- See more in [PROFILING.md](https://github.com/lakwet/voracious_sort/blob/master/PROFILING.md).

# Version **0.1.0** (March 16<sup>th</sup> 2020)

Initial release.

### Traits:

- Dispatcher trait: Which sort is used and how for a given type.
- RadixKey trait: Usefull methods for each type.
- Radixable trait: Main trait, where all the logic is to make sorts generic.
- RadixSort trait: Add sort methods for vector and slice.

### Generic single thread sorts:

- American flag sort (MSD radix sort).
- Boolean sort (other).
- Insertion sort (comparative sort).
- Counting sort (radix sort)
- DLSD sort - Diverting LSD sort - (LSD radix sort).
- LSD sort (radix sort).
- MSD sort (MSD radix sort).
- MSD stable sort (MSD radix sort).
- Ska sort (MSD radix sort).
- Thiel sort (LSD radix sort).
- Voracious sort (MSD radix sort).

### Dedicated single thread sorts: (it works only on one type)

- LSD f32 sort (LSD radix sort for f32).

### Benchmarks:

- First benchmark (in result folder).
