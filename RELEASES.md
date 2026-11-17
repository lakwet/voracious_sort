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
