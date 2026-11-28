# Sorting 16-bit keys: the benchmark results

*Last updated: 28 November 2026. voracious_radix_sort 2.1.2. Written with Claude (Anthropic's AI model), from the project's development journal.*

[Engineering voracious radix sort for 16-bit keys](engineering_16_bits.md)
explains how the `u16` / `i16` sort of 2.1 was built. This post gives what
it is worth: the official benchmark of 2.1.2 against the standard library
and the best other Rust crates, on four machines, and an honest list of
where it still loses.

---

## 1. What was measured

**The machines.**

| Name here | CPU | Caches (L1d / L2 / L3) | OS |
|---|---|---|---|
| Ryzen | AMD Ryzen 9 3950X (Zen 2, 2019) | 32 KiB / 512 KiB / 16 MiB | Linux |
| Haswell | Intel Core i5-4690K (2014) | 32 KiB / 256 KiB / 6 MiB | Linux |
| Core Ultra | Intel Core Ultra 9 185H (laptop, 2023) | 32 KiB / 2 MiB / 24 MiB | Windows |
| Snapdragon | Qualcomm Snapdragon X Elite X1E80100 (laptop, ARM, 2024) | 96 KiB / 12 MiB / - | Windows |

A fifth machine, an Apple M3 Pro (ARM, macOS), ran the generic build of
2.1.1, before the 2.1.2 fixes; its results are given apart (section 2).

**The sorts.** voracious 2.1.2 (`voracious_sort`), and for each type the
standard library and the fastest crates on that type in an earlier full
benchmark:

| Type | Rivals |
|---|---|
| `u16` | `sort_unstable` (std), voracious 1.2.2, blazing-radix 0.1.0, turbosort 0.2.2 |
| `i16` | `sort_unstable` (std), voracious 1.2.2, glidesort 0.1.2, blazing-radix 0.1.0, radsort 0.1.1 |

**The inputs.** 31 distributions per type (uniform, few values, sorted and
nearly sorted, saw teeth, normal laws, clusters, sparse data, audio samples,
the worst cases of our own detectors...; see
[The data we sort](generators.md)), at 15 sizes from 500 to 30 million
keys: 465 cells per type and machine.

**The method** (see [How to benchmark a sort](how_to_bench.md)): cold calls
(each array made new, the caches and the branch predictor dirtied before the
timed call), every sort on the same arrays, the median of 11 repetitions.
Every buffer of 128 KiB or more gets new pages from the system, as in a
program that sorts once: no sort finds its buffer of the previous call
still mapped. Two builds: generic (`x86-64` baseline, or ARMv8 with NEON),
the one a crate user gets by default; and native (`-C target-cpu=native`:
AVX2 and BMI2 on the three x86 machines, the Snapdragon's own instruction
set on ARM).

**How a cell is counted.** A sort wins a cell when it is faster by more than
3 % and by more than 2 ns per sort; within 3 %, or within 2 ns, the cell is a
tie (that is the noise of a run, and the bench viewer's rule). We count
won / tied / lost cells, size by size. We do not average over distributions:
an average hides exactly the cases a user may care about.

---

## 2. The headline

Against the **best** rival of each cell (whichever is fastest there), cells
won / tied / lost out of 465:

| Machine | `u16` generic | `i16` generic | `u16` native | `i16` native |
|---|---|---|---|---|
| Ryzen | 392 / 50 / **23** | 362 / 65 / **38** | 386 / 49 / **30** | 357 / 57 / **51** |
| Haswell | 338 / 46 / **81** | 327 / 65 / **73** | 331 / 48 / **86** | 333 / 68 / **64** |
| Core Ultra | 335 / 38 / **92** | 323 / 58 / **84** | 308 / 42 / **115** | 307 / 65 / **93** |
| Snapdragon | 367 / 49 / **49** | 359 / 42 / **64** | 367 / 57 / **41** | 355 / 51 / **59** |
| M3 Pro (2.1.1) | 352 / 68 / **45** | 340 / 63 / **62** | - | - |

voracious is the fastest sort on 66 % to 84 % of the cells, depending on the
machine and the build, and loses 5 % to 25 % of them.

Against each rival alone, cells lost out of 465 (`u16` / `i16`):

| Rival | Build | Ryzen | Haswell | Core Ultra | Snapdragon |
|---|---|---|---|---|---|
| `sort_unstable` | generic | 3 / 3 | 4 / 4 | 4 / 4 | 2 / 4 |
| | native | 1 / 3 | 4 / 4 | 4 / 4 | 2 / 3 |
| voracious 1.2.2 | generic | 5 / 4 | 9 / 5 | 7 / 7 | 2 / 4 |
| | native | 3 / 4 | 7 / 6 | 5 / 8 | 2 / 3 |
| blazing-radix | generic | 12 / 35 | 62 / 67 | 59 / 76 | 46 / 59 |
| | native | 17 / 48 | 64 / 56 | 95 / 85 | 37 / 56 |
| turbosort (`u16` only) | generic | 10 | 19 | 44 | 9 |
| | native | 12 | 23 | 42 | 9 |
| glidesort (`i16` only) | generic | 1 | 1 | 1 | 1 |
| | native | 2 | 0 | 2 | 1 |
| radsort (`i16` only) | generic | 0 | 7 | 8 | 0 |
| | native | 0 | 8 | 9 | 0 |

blazing-radix is the one rival that matters: it is a recent radix sort with
the same kind of engineering (several of its ideas are in voracious 2.1, with
credit in the code). Every other rival loses almost every cell.

---

## 3. Size by size

Cells lost against blazing-radix by size band (62 cells from 500 to 1 000
keys, 217 from 5 000 to 200 000, 186 from 500 000 to 30 million), small /
middle / large:

| Machine | `u16` generic | `u16` native | `i16` generic | `i16` native |
|---|---|---|---|---|
| Ryzen | 2 / 7 / 3 | 5 / 9 / 3 | 14 / 17 / 4 | 26 / 20 / 2 |
| Haswell | 3 / 13 / 46 | 5 / 15 / 44 | 6 / 27 / 34 | 5 / 27 / 24 |
| Core Ultra | 16 / 21 / 22 | 20 / 44 / 31 | 24 / 31 / 21 | 16 / 41 / 28 |
| Snapdragon | 18 / 14 / 14 | 6 / 15 / 16 | 16 / 23 / 20 | 7 / 26 / 23 |

Each machine has its own weak band: the large arrays on the Haswell (its
6 MiB L3 is small against arrays of tens of MB), the small arrays on the
Snapdragon (generic build) and on the Ryzen in `i16`, the middle sizes on
the Core Ultra (native build above all, see section 6).

What a plain case looks like, uniform `u16` keys, ns per key:

| Size | Machine | voracious 2.1.2 generic | blazing-radix generic | voracious 2.1.2 native | blazing-radix native | voracious 1.2.2 | `sort_unstable` |
|---|---|---|---|---|---|---|---|
| 500 | Ryzen | 2.11 | 2.11 | 2.15 | 2.11 | 9.26 | 9.28 |
| | Haswell | 2.63 | 2.86 | 2.62 | 2.88 | 9.62 | 9.64 |
| | Core Ultra | 1.64 | 1.57 | 1.77 | 1.88 | 6.56 | 6.53 |
| | Snapdragon | 1.42 | 1.38 | 1.34 | 1.35 | 5.37 | 5.31 |
| 10 000 | Ryzen | 1.83 | 1.90 | 1.83 | 1.90 | 2.70 | 13.98 |
| | Haswell | 2.29 | 2.37 | 2.30 | 2.35 | 4.61 | 12.85 |
| | Core Ultra | 1.46 | 1.45 | 1.59 | 1.60 | 2.39 | 8.60 |
| | Snapdragon | 1.16 | 1.17 | 1.16 | 1.17 | 1.76 | 7.00 |
| 1 000 000 | Ryzen | 0.63 | 0.86 | 0.61 | 0.85 | 1.69 | 19.32 |
| | Haswell | 1.55 | 1.85 | 1.50 | 1.84 | 2.65 | 16.24 |
| | Core Ultra | 0.84 | 0.93 | 0.91 | 0.98 | 1.69 | 11.54 |
| | Snapdragon | 0.49 | 0.62 | 0.49 | 0.62 | 1.38 | 8.23 |
| 30 000 000 | Ryzen | 0.63 | 0.88 | 0.62 | 0.87 | 0.87 | 16.14 |
| | Haswell | 1.41 | 1.94 | 1.41 | 1.86 | 2.03 | 12.44 |
| | Core Ultra | 0.78 | 0.93 | 0.76 | 0.85 | 0.95 | 8.70 |
| | Snapdragon | 0.48 | 0.56 | 0.47 | 0.56 | 0.58 | 6.03 |

(voracious 1.2.2 and `sort_unstable`: generic build.)

On random keys the two radix sorts are close below 10 000 keys; from a
million keys voracious takes 10 % to 28 % less time. A comparison sort is 4 to 30
times slower at every size: on 16-bit keys a radix sort is the right tool.

---

## 4. Where voracious wins

Over the 7 440 cells of the benchmark (4 machines, 2 builds, 2 types, 465
cells each), voracious is the fastest on **5 547 (75 %)**, ties with the
best rival on 850 (11 %), and loses 1 043 (14 %).

By distribution, over the 240 cells each one has (15 sizes, 4 machines, 2
builds, `u16` and `i16`; `Depth` is `u16` only and `Audio` `i16` only, 120
cells each). The margin is the best rival's time divided by voracious's, on
the cells won: the median of those cells, then the widest.

| Distribution | Won / tied / lost | Margin: median | widest |
|---|---|---|---|
| **Few values and heavy duplicates** | | | |
| `Binary` (0 and 1) | 240 / 0 / 0 | 2.9x | 7.3x |
| `Few 4` (4 values) | 240 / 0 / 0 | 3.8x | 7.3x |
| `RootDup` (i mod sqrt(n)) | 223 / 7 / 10 | 1.4x | 9.6x |
| `Few 16` (16 values) | 212 / 12 / 16 | 2.3x | 8.3x |
| `Sparse 90%` (90 % zeros) | 212 / 2 / 26 | 2.2x | 3.4x |
| `FakeZipf` (half one value, a quarter another...) | 178 / 17 / 45 | 1.8x | 3.9x |
| **Narrow ranges and outliers** | | | |
| `Late outl.` (a narrow range, one far value late) | 234 / 6 / 0 | 2.0x | 7.7x |
| `Aligned` (low byte zero) | 229 / 2 / 9 | 1.45x | 2.4x |
| `Normal 8` (normal law, standard deviation 2^8) | 216 / 12 / 12 | 1.85x | 9.2x |
| `Range 16` (16 consecutive values) | 204 / 19 / 17 | 1.17x | 1.7x |
| `Sensor 3/4` (12 bits used) | 201 / 22 / 17 | 1.45x | 8.0x |
| **Nearly sorted** | | | |
| `Near sorted` (1 % of the positions swapped) | 226 / 4 / 10 | 1.6x | 5.4x |
| `Sort rnd tl` (sorted, a random tail) | 205 / 14 / 21 | 1.4x | 4.9x |
| `Sorted desc` | 187 / 22 / 31 | 1.2x | 2.0x |
| **Random-looking keys** | | | |
| `Gaussian` (audio samples, pixels) | 177 / 44 / 19 | 1.35x | 5.7x |
| `Zipf` | 173 / 39 / 28 | 1.35x | 3.6x |
| `Spread 4k` (4 096 random values) | 173 / 44 / 23 | 1.2x | 3.8x |
| `Unif` | 147 / 69 / 24 | 1.2x | 2.8x |

What lies behind the widest margins:

- **The chain counts instead of sorting.** On few values, a narrow range or
  one dominant value, the keys are counted and written back value by value:
  no scatter, no buffer. `Sparse 90%` at 10 000 keys takes 0.54 to 1.21 ns
  per key against 2.59 to 3.84 for blazing-radix; `FakeZipf` at 10 000 keys,
  0.60 to 1.29 against 2.67 to 4.04; `Bimodal` (values at both ends of the
  domain) at 500 keys, 0.53 to 1.13 against 1.44 to 3.57.
- **The order is checked before sorting.** Nearly sorted inputs are repaired
  or merged rather than sorted again (`Near sorted`: a median margin of
  1.6x).
- **Large arrays.** From a million uniform keys, voracious takes 10 % to 28 %
  less time than blazing-radix (section 3). From 131 072 keys the 16-bit
  sort counts the whole domain in a table of 256 KiB, kept on the stack
  since 2.1.2: on the heap that table got new pages, and their faults, at
  each call.
- **Against voracious 1.2.2**, 2.1.2 loses 2 to 9 cells per type, machine
  and build out of 465. The rewrite pays almost everywhere, most on sorted
  and constant inputs (`Equal` at a million keys: 0.01 to 0.12 ns per key
  against about 2) and on small arrays (3.5 times faster at 500 keys, the
  median over the distributions).

---

## 5. Where it is a tie

850 cells (11 %) are within 3 % or 2 ns of the best rival, almost always
blazing-radix. They gather in two places:

- **Sizes up to 50 000 keys**: 80 to 104 ties out of the 496 cells of each
  size, against 13 to 58 from 80 000 keys on. There both radix sorts make
  the same number of passes over data that sit in the L1 or L2 cache; what
  is left to win is a few nanoseconds of preparation per call.
- **Random-looking keys**: `Unif` (69 ties), `Heavy 4` (63), `Equal` (59),
  `Runs 8` (55), `Normal 13` (51), `Clusters` (49), `Spread 256` (45),
  `Spread 4k` (44), `Gaussian` (44). For random keys the work of a radix
  sort is fixed by the passes; for `Equal` both sorts read the array once
  and stop.

A tie at 1 000 keys is a difference of less than 60 ns in all: in practice,
the same time.

---

## 6. Where voracious still loses

**Small arrays dominated by one value, against `sort_unstable`.** `Sparse
90%` and `FakeZipf` at 500 and 1 000 keys: the standard library's sort (or
voracious 1.2.2) is 1.3 to 2.4 times faster on the Intel machines
(Haswell, `Sparse 90%` `i16` at 500 keys: 4.24 ns per key against 2.04).
Its partition settles all the keys equal to its pivot in one pass. On the
Ryzen the gap is 1.1 to 1.4x; on the Snapdragon `Sparse 90%` is lost only at
500 keys (1.1x), `FakeZipf` by 1.4 to 1.65x.

**`Bimodal` from 5 000 keys on the Intel machines.** Half the keys near 0,
half near the top: blazing-radix is 5 % to 30 % faster in the generic build
(Core Ultra, 10 000 `i16` keys: 0.38 against 0.35 ns per key), up to 1.7
times in the native one, and about 1.6 times at 200 000 keys on the
Haswell. `Bimodal` is the distribution voracious loses most (122 cells of
240), by a small margin on the Snapdragon (3 % to 18 %), hardly at all on the
Ryzen.

**Ordered shapes in large arrays on the Haswell.** Saw teeth and 256 spread
values from 500 000 keys: blazing-radix is 15 % to 60 % faster (`Desc Saw`
`u16`, 5 million keys: 1.09 against 0.68 ns per key).

**`Pipe organ`, `i16` above all.** From 500 to 100 000 keys, up to 20 %
slower than blazing-radix on the Ryzen and the Haswell, up to 37 % on the
Core Ultra (Ryzen, 10 000 keys: 1.61 against 1.36 ns per key); in `u16` the
gap is 3 % to 18 %. With the small arrays, this is why the `i16` count is worse
than the `u16` one on the Ryzen (38 cells lost against 23).

**Sorted and constant inputs.** `Equal` (95 cells lost), `Sorted` (79) and
saw teeth appear often in the lost lists, with large ratios in the worst
cells. These cells take 0.01 to 0.13 ns per key for every radix sort: one
read of the array, a few nanoseconds in all at 500 keys. A ratio there
measures the fixed cost of a call, and the noise of the clock, more than the
sort; we count them all the same.

**The native build.** On the x86 machines the native build loses more
cells than the generic one (Ryzen `u16`: 30 against 23, `i16`: 51 against
38), except the Haswell in `i16` (64 against 73): compiled for AVX2,
blazing-radix gains more than voracious does. On the Snapdragon the native
build loses fewer (`u16`: 41 against 49, `i16`: 59 against 64).

The Core Ultra keeps a gap of its own. Built for this CPU (`meteorlake`),
the compiler turned our counting loops into vector gathers, slow there, and
the native build was up to 2 times slower than the generic one on the
counting paths; 2.1.2 reads the counters so that no gather is made, and
`Range 16`, `Small` are now faster native than generic (10 000 `i16` keys:
0.34 against 0.44 and 0.43 ns per key). Some paths remain slower native
(`Bimodal` at 10 000 `i16` keys: 0.50 against 0.38), and the native build
still loses 115 `u16` cells against 92 generic.

---

## 7. Check it yourself

The result files are in
[`resultsbenchs/benchmark_2_1_2/`](../resultsbenchs/benchmark_2_1_2/), one
folder per CPU, one file per type and build:
`bench-<cpu>-<type>-<generic|native>-<YYYYMMDDHHMM>.csv` (and the `.txt`
report), for example
`bench-amd-ryzen-9-3950x-16-core-u16-generic-202611280426.csv`. Open them in
the bench viewer ([`bench_viewer/index.html`](../bench_viewer/index.html),
no install: drop a `.csv` on the page) to see every cell.

To run the same benchmark on your machine: `make bench-16` (about an hour
and a half; see the README). Results from other CPUs are welcome: send us
the files it writes.
