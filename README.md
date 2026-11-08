# Voracious sort

![Rust](https://github.com/lakwet/voracious_sort/workflows/Rust/badge.svg)
![Rust](https://docs.rs/voracious_radix_sort/badge.svg)

Dear visitor, welcome.

## Introduction

This project's purpose is to have a Rust implementation of a **state
of the art radix sort** in a single thread and in a multi thread versions.

This crate should be easy to use and the sort should be able to sort almost
"everything". Radix sort is criticized because people think it can only sort
unsigned integers. This project proves this wrong, **Voracious sort can sort all
Rust primitive types** (except String, Tuple and Array for now) **and custom struct**.
**It is way faster than Rust standard sort and Rust unstable sort** on most of
the types and data distribution.

Because of Rust Orphan Rule, we chose not to support tuple sorting. You can use
Struct instead.

You will find here:
- Version
- License
- A word about the sort’s name
- The **documentation** on how to use this sort,
- Radix sort: basic notions,
- **Benchmarks**,
- For developers and researchers
- **References** we used for this project,
- Future work,

## Version

Last version tested/used:
- Rustc: 1.46.0 stable
- Rustfmt: 1.4.18 stable
- Cargo: 1.46.0 stable
- Clippy: 0.0.212

## License

This project is under the **MIT** license.

See the license file.

## A word about the sort's name

When I asked what name to give to this sort, a colleague of mine proposed
Voracious sort (_Variation Of Radixsort Axelle Cleverly Imagined Over Uncountable
Sessions_). The name was fun so I took it. It is true that I spent uncountable
sessions to code this project.

You can also think about it like someone very voracious of radish...

## Disclaimer about memory consumption

Implemented sorts in the voracious trait may be out of place, it means it can
use memory up to 2x the array size. If you want to be sure to use in place
algorithm, please use sorting functions instead of the trait.

## Disclaimer about array size

A radix sort is meant to be used on very big arrays. I make this crate to also sort
smaller arrays but Voracious will fallback on native rust sorts. I let the user read
the benchmark results to know if they need the Voracious sort.

## Documentation: How to use it ?

Since it is alreay explained in the crate documentation, we just provide the link:
- [Voracious sort Rust Doc](https://docs.rs/voracious_radix_sort/)

And we assume that you know how to code in Rust...

Other implementation examples:
- [struct](https://github.com/lakwet/voracious_sort/blob/master/src/types/custom.rs)

## Radix sort: basic notions

Radix sort is a **non-comparative** sort. It requires a key and **uses the binary
representation** to sort the input instead of a comparative function. Amongst
radix sorts there are two sub groups:

- LSD: **L**east **S**ignificant **D**igit; (or LSB: **L**east **S**ignificant **B**it)
- MSD: **M**ost **S**ignificant **D**igit; (or MSB: **M**ost **S**ignificant **B**it)

Their differences are based on how the algorithm iterates through the binary
representation.

Sorts can be **stable** or **unstable**; in general, LSD are stable whereas MSD
tend to be unstable. Please note that the term stable can be ambiguous in the
computer science field, we are refering to this one [wikipedia page on sorting algorithm
stability](https://en.wikipedia.org/wiki/Sorting_algorithm#Stability).

Sorts can be **in place** or **out of place**. A sort is said to be **in place**
if it does not require more memory than the initial memory used by the input. A
sort is said **out of place** if it does require more memory (usually, in order
of `O(n)`). LSD sorts are out of place, and MSD sorts can be either one.

**Diversion** is something we do because radix sorts perform badly on small
inputs. In order to avoid this loss of performance, a radix sort fallbacks on
another sort when the input becomes too small. Insertion sort is very good for
input's length less or equal to 32 elements. Diversion can occur at the
beginning of the algorithm, we check the input's length and we choose the sort
accordingly, or we trigger it at the end of recursive calls when the remaining
input is small enough.

The **radix** itself, is the number of bits the sorting algorithm take into
account per pass. The radix of choice is often 8, however, it may vary depending
on your use case.

In a radix sort, you use an **histogram** which has a size of `2.pow(radix)`.
Given a **level**<sup>[1]</sup> and a radix, we can compute the histogram. This
histogram gives all the bucket sizes and thus, the algorithm knows where to move
each elements. For a detail example, I let the reader read the Wikipedia page on
[radix sort](https://en.wikipedia.org/wiki/Radix_sort).

<sup>[1]</sup>: If a radix of 8 is choosen, level 1 is the first 8 bits, level
2 is the next 8 following bits, and so on until there is no more bit to handle.

## Benchmarks

First, please, read: [PROFILING.md](https://github.com/lakwet/voracious_sort/blob/master/PROFILING.md).

Benchmarks are available here:
[results folder](https://github.com/lakwet/voracious_sort/tree/master/results)

For each sort, 3 columns:
- 1st column: time un micro second
- 2nd column: standard deviation (if more than 1 iteration) in nano second
- 3rd column: time per item in nano second

## For developers and researchers

- American flag sort is a MSD radix sort. It is an implementation of the very well
known algorithm [https://en.wikipedia.org/wiki/American_flag_sort](https://en.wikipedia.org/wiki/American_flag_sort).

- Voracious sort is a MSD radix sort. It is an improvement of the
[Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
and it uses the [Verge sort pre-processing heuristic](https://github.com/Morwenn/vergesort). Depending on the type and the input size, another sort might be choosen (LSD sort, Counting sort, etc...).

- DLSD (Diverting LSD radix sort) is a simpler version of the
[DFR sort](https://github.com/ramou/dfr) with a different diversion and
a variable radix (see [article](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf)).

- Thiel sort is a LSD radix sort. It is an implementation of [Fast radix sort](https://github.com/AwardOfSky/Fast-Radix-Sort): [Relaxing the Counting Requirement for Least Significant Digit Radix Sorts](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf) (Implementation in C/C++ is not from the article's author)

- Rollercoaster sort is a hybrid radix sort. It starts as an MSD radix sort, and then switches as
a LSD radix sort. It is a mix between the Voracious sort and the DLSD sort. This sort
has a heuristic for signed integers. It performs very well on float (32 or 64 bits) and
signed integer. It is my contribution to the science.

- Peeka sort is a multithread MSD radix sort. It is an improvement of the MIT's
researchers Regions sort algorithm: [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort): [Theoretically-Efficient and Practical Parallel In-Place Radix Sorting](https://people.csail.mit.edu/jshun/RegionsSort.pdf). It is also my contribution to the science.

- All sorts fallback on the [PDQ sort](https://github.com/stjepang/pdqsort) (Rust
Unstable sort) for very small inputs or on Rust (stable) sort for stable sorts.

## Running the tests

The test suite relies on the multithread feature (`rayon`) to generate its
inputs:

```sh
make test
# or
cargo test --release --features "voracious_multithread"
```

### Windows with the GNU toolchain (`x86_64-pc-windows-gnu`)

The test dependencies (`rand` → `getrandom`) link against Windows system DLLs
through `raw-dylib`. On the GNU toolchain, rustc delegates this to `dlltool`,
which in turn needs an assembler (`as`). Without one, the build fails with:

```
error: error calling dlltool 'dlltool.exe': program not found
```

or

```
error: dlltool could not create import library with ... dlltool.exe: CreateProcess
```

This only affects building the tests: `getrandom` is a dev-dependency, so
crates depending on `voracious_radix_sort` are not concerned.

Fix it with one of the following:

- **Install MinGW binutils** and make sure `dlltool` and `as` are in your
  `PATH`. With [MSYS2](https://www.msys2.org/):
  ```sh
  pacman -S mingw-w64-x86_64-binutils
  ```
  then add `C:\msys64\mingw64\bin` to your `PATH`.
- **Use the MSVC toolchain** (requires the Visual Studio Build Tools):
  ```sh
  rustup toolchain install stable-x86_64-pc-windows-msvc
  cargo +stable-x86_64-pc-windows-msvc test --release --features "voracious_multithread"
  ```
- **Workaround without installing anything**: make `getrandom` use the
  `RDRAND` CPU instruction instead of the Windows API (x86_64 only, tests
  only):
  ```sh
  RUSTFLAGS='--cfg getrandom_backend="rdrand" -C target-feature=+rdrand' \
    cargo test --release --features "voracious_multithread"
  ```

## Running the benchmarks

The speed tests are regular `#[test]` functions hidden in the test suite. They
are disabled by default because they are slow and print a colored table meant
for humans.

1. **Enable the modules** in `src/tests/mod.rs` by uncommenting these lines
   (`test_helpers` is required by both speed modules):
   ```rust
   mod test_helpers;
   #[rustfmt::skip] mod speed_sort;
   #[rustfmt::skip] mod speed_sort_ded;
   ```

2. **Pick what to compare** in `src/tests/speed_sort.rs`, function
   `helper_sort`:
   - `sorts_name`: the column titles, uncomment the sorts you want.
   - the `helper_sort_aux(...)` calls: uncomment the matching sorts, **in the
     same order** as `sorts_name` (the titles are only labels).
   - `runs` (number of iterations), `thread_n` (for multithread sorts),
     `with_check` (checks the result against Rust's sort, slower) and `sizes`.
   - at the bottom of the file, uncomment the `#[test]` functions for the types
     you want (`speed_test_u64`, `speed_test_f32`, ...).
   - the [`blazing-radix`](https://crates.io/crates/blazing-radix) crate
     (dev-dependency only) is passed as the last argument of `helper_sort` for
     the types it supports (`i8`..`i64`, `u8`..`u64`, `f32`), `None` otherwise.
     Add a `"Blazing"` entry in `sorts_name` where its column appears.
   - for floats (`f32`, `f64`), replace the `T: Ord` bound of `helper_sort` with
     the commented `T: PartialOrd` one.

   `src/tests/speed_sort_ded.rs` benchmarks the dedicated sorts (`lsd_f32`,
   `lsd_u32`, `cs_u16`) against `blazing-radix`.

3. **Run** in release mode, with `--nocapture` to see the output and a single
   test thread so the benchmarks don't disturb each other:
   ```sh
   # shortcut (any test name or filter, adds the getrandom workaround on Windows)
   make bench speed_test_u64
   make bench speed_dedicated

   # all enabled speed tests
   cargo test --release --features "voracious_multithread" speed_ -- --nocapture --test-threads=1
   # a single one
   cargo test --release --features "voracious_multithread" speed_test_u64 -- --nocapture --test-threads=1
   # dedicated sorts
   cargo test --release --features "voracious_multithread" speed_dedicated -- --nocapture --test-threads=1
   ```

For each sort, the output shows the mean time in µs (green), the standard
deviation in ns (red) and the time per item in ns (yellow). Large `sizes` need a
lot of memory (the input plus a copy when `with_check` is enabled).

## References

- [What is radixsort](https://axelle.me/2020/11/08/what-is-radixsort/)
- [Voracious sort](https://axelle.me/2020/11/21/voracious-sort/)
- [DLSD sort](https://axelle.me/2022/04/19/diverting-lsd-sort/)
- [Rollercoaster sort](https://axelle.me/2022/04/19/rollercoaster-sort-the-best-of-the-two-worlds/)

- [Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
- [Verge sort pre-processing heuristic](https://github.com/Morwenn/vergesort)
- [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort): [Theoretically-Efficient and Practical Parallel In-Place Radix Sorting](https://people.csail.mit.edu/jshun/RegionsSort.pdf)
- [DFR sort](https://github.com/ramou/dfr)
- [Fast radix sort](https://github.com/AwardOfSky/Fast-Radix-Sort): [Relaxing the Counting Requirement for Least Significant Digit Radix Sorts](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf) (Sort's implementation and article are not from the same person)
- [PDQ sort](https://github.com/stjepang/pdqsort)

## Future work

- Finish profiling.
- Improve k-way-merge algorithm (add multithread).
- Add a sort for String (Spreadsort and/or Burstsort).
- Find a way to multithread the verge sort pre-processing heuristic.
- Add stable multithread sort.
- Improve multithread sort for signed integer.
- Add a stable/unstable sort by key (to avoid to move/copy elements during the sorting).
- More improvement !
