# The data we sort: the input generators

*November 2026, voracious_radix_sort 2.1.1. Written with Claude (Anthropic's AI model), from the project's development journal.*

A sort is only as well tested and as well measured as the arrays it is
given. Uniform random keys are the classic benchmark, and the least
interesting input for voracious: most of its code is there for everything
else. This post explains which arrays the tests and the benchmarks use,
why each family is there, and how they are generated.

All of them live in [`src/generators/`](../src/generators/). The same
generators feed the correctness tests, the decision trace and every
benchmark, so a shape that is measured is also a shape that is tested.

---

## 1. Why so many

voracious is not one algorithm. For each key type it is a chain of
decisions (is the array already sorted, does it hold few distinct values,
a narrow range, one heavy value, sorted runs) followed by one of several
engines, chosen by the size of the array and the size of the caches. Each
input family is there for one of four reasons:

- **To reach a path.** A detector for "few distinct values" is only
  exercised by few distinct values. Every detector and engine has at least
  one family that sends the chain to it.
- **To fool a path.** A detector costs time when it fails. The worst case
  is data that look like its case for most of the array and are not: a
  narrow range with one outlier at the end, sixteen values and a
  seventeenth late, a sorted array with a random tail. These families
  measure what a detector costs when it gives up halfway, and test that it
  gives up correctly.
- **To look like real data.** Identifiers, timestamps, aligned addresses,
  sensor values, audio samples, text. A sort tuned only on synthetic
  shapes can be fast on paper and slow on a log file.
- **To compare with others.** The distributions of other sort benchmarks
  (IPS4o by Axtmann et al., the sort-research-rs suite of glidesort and
  ipnsort), so that our numbers can be read next to theirs.

---

## 2. How many, per type

Each type has its own list, plus a list shared by a family of types:

| Types | Own list | Shared list | Total |
|---|---|---|---|
| `u8`, `i8` | 7 | 16 (narrow) | 23 |
| `u16`, `i16` | 15 | 16 (narrow) | 31 |
| `u32` | 26 | 22 (integer) | 48 |
| `i32` | 20 | 22 (integer) | 42 |
| `u64` | 29 | 22 (integer) | 51 |
| `i64` | 28 | 22 (integer) | 50 |
| `u128`, `i128` | 2 | 22 (128-bit) | 24 |
| `f32` | 34 | 6 (float) | 40 |
| `f64` | 39 | 9 (float) | 48 |
| `char` | 5 | 15 (text) | 20 |
| `bool` | 13 | | 13 |

The shared lists were added so that every type of a family is measured on
the same situations; the own lists are older and differ more from type to
type. Records (structs sorted by a key) take the distributions of their key
type, plus a few of their own (section 6).

---

## 3. The families

### Random over a range

- **Uniform** over the whole domain (`Unif`), the classic benchmark.
- **Uniform over a narrower range**: `Unif 10^9` (0 to 10^9 on 32-bit and
  wider keys, about 30 bits used), `Small1` / `Small2` / `Small3` (8, 16
  and 24 bits on `u32`), `Small 4` to `Small 7` on `u64` (32 to 56 bits), ranges around 0
  for the signed types. A narrow range is the case where a radix sort can
  skip its empty high digits.
- **Range 16** (narrow types): 16 consecutive values starting at a random
  one.
- **Sensor 3/4**: 3/4 of the bits used (12 bits of a 16-bit key).

### Sorted, or almost

- **Sorted** and **reversed** (`Asc`, `Desc`, `Sorted`, `Sorted desc`),
  with and without duplicates.
- **Almost sorted**: a sorted array with log2(n) random swaps (`Almost
  Asc`, `Almost Desc`), or 1 % of the positions swapped (`Near sorted`);
  on 64-bit keys, sorted arrays whose first or last keys (1 per mille, 1 %
  or 10 % of them) are random.
- **One key out of place**: sorted, with the last key moved to the front
  (`Front`) or to the middle (`Middle`) of the order.
- **Sorted runs**: 8 or 64 independently sorted random sources put end to
  end (`Runs 8`, `Runs 64`), like logs from several machines.
- **Saw teeth** (`Asc Saw`, `Desc Saw`): about n / log2(n) values
  repeated as ascending or descending ramps; **pipe organ**: ascending
  half, then descending half.
- **Killers** (`Asc Killer`, `Desc Killer`, `u64`): the same saw teeth
  with a few more teeth (about 1.1·log2(n) instead of 0.9·log2(n)).

### Few distinct values

- **All equal** (`Equal`), **two values** (`Binary`, 0 and 1), **16
  small values** (`Alt16`: the values 0 to 15, at random), **4** or **16**
  random values (`Few 4`, `Few 16`); on `u16`, 2, 16, 256 and 4096 random
  values spread over the domain (`Spread 256`, `Spread 4k`).
- **Sqrt**: sqrt(n) values, each repeated sqrt(n) times, in blocks.
- **Zipf**: frequencies in 1 / rank (the most common value twice as
  frequent as the second, three times as the third...), over min(n, domain)
  random values.
- **FakeZipf**: n / 2 copies of one value, n / 4 of another, and so on,
  shuffled: about log2(n) distinct values with geometric frequencies.
- **Heavy hitters**: 60 % of the keys among 4 values, the rest random
  (`Heavy 4`); **sparse data**: 90 % zeros, 10 % random (`Sparse 90%`).
- **Composite keys**: the high half among 64 values (a tenant, a
  category), the low half random.

### Shapes of real data

- **Identifiers**: a shuffled 0..n (`Int 0..n`); **counters, ages,
  ratings**: 0 to 100 (`Int 0..100`).
- **Timestamps**: a large common prefix (an epoch: seconds since 1970 for
  32-bit keys, nanoseconds for 64-bit keys), increasing with jitter, as
  events logged slightly out of order.
- **Aligned addresses**: random multiples of 2^16 (`Aligned 2^16`; on 8-
  and 16-bit keys, the low half of the bits zero); **common prefix**: a
  fixed random prefix and 20 random low bits (`Prefix 20b`).
- **Clusters**: keys within a small distance of a few random centres; the
  top bits name the cluster.
- **Bimodal**: half near the bottom of the domain, half near the top;
  **Straddle**: a narrow range across the middle bit of the key.
- **Normal laws**: |N(0, sd)| for standard deviations from 2^8 to 2^63
  (`Normale 10`, `Normale 30`, ...), and on 8- and 16-bit keys a normal
  around the middle of the domain (`Gaussian`, audio samples or pixels).
- **Depth map** (`u16`): a smooth walk between 500 and 8000 millimetres
  along 640-pixel rows, with patches of invalid pixels (0).

### Worst cases of the detectors

- **Late outlier**: a narrow range, and one wide value at 95 % of the
  array (`Int late out`, `Late outl.`).
- **Late extra value**: 16 distinct values, and a 17th at 95 %
  (`Few16 late`).
- **Random tail**: sorted for 90 %, then random (`Asc rnd tail`, `Sort rnd
  tl`).

Each one makes a detector work through most of the array before it has to
give up and hand over to another path.

### Other benchmarks

- **Binary** keys (0 and 1), from sort-research-rs.
- From IPS4o: **Exponential** (a power-of-two interval chosen uniformly,
  then a value inside it), **RootDup** (i mod sqrt(n)), **TwoDup**
  ((i^2 + n/2) mod n), **EightDup** ((i^8 + n/2) mod n).

---

## 4. Floats

Floats take the integer shapes that make sense for them (uniform, sorted,
saw teeth, normal laws, Zipf, few values, sparse, runs, clusters, the
detector worst cases) and add their own:

- **Integers stored as floats**: 0 to 100, a shuffled 0..n; a
  **power-of-two grid** (multiples of 0.25 in [-32, 32)) and a **decimal
  grid** (prices in cents, not exactly representable in binary).
- **Heavy tails**: Pareto laws of shape 0.75, 1, 1.5 and 2 (sizes, prices,
  populations).
- **Timestamps** in seconds with a fraction; **values at both ends**
  (half near -10^6, half near +10^6); **magnitudes** spread over 20
  decades (10^-10 to 10^10, both signs); **sensor readings** (20 ± 5 at a
  0.01 resolution).
- **Signed zeros**: 90 % of -0.0 and +0.0, 10 % random.

The correctness tests also use the special values: NaNs of both signs,
infinities, signed zeros. The expected order is that of `total_cmp`,
compared bit for bit (see [Testing for a correct sort](testing_for_a_correct_sort.md)).

---

## 5. Text, flags and 128-bit keys

- **`char`**: uniform over the code points, all equal, small charsets, and
  real text: English-like ASCII (letters weighted by frequency, spaces and
  punctuation), French-like text (about 4 % accented letters), Cyrillic,
  CJK (3000 ideographs with Zipf-like frequencies), emoji, a web page
  (85 % ASCII, 10 % accented Latin, 3 % CJK, 2 % emoji), the digits and
  separators of a CSV file, sorted text, ASCII with one emoji late.
- **`bool`**: random, alternating, all true, all false, 1 % or 99 % true,
  16 true at random positions, nearly sorted, a random tail, runs of 1 to
  1000 equal values, pipe organs.
- **`u128` / `i128`**: UUIDs (v4 random, v7 time-ordered, v7 sorted),
  nanosecond timestamps, values on 32 or 64 bits, fixed-point numbers,
  composite keys, IPv6 addresses, and the common shapes (clusters,
  bimodal, sorted, runs, few values, heavy hitters, sparse, Zipf).

---

## 6. Records

Structs sorted by a key are generated from the keys: the key generators of
the type, then each key wrapped in a record of 8 to 128 bytes, the key first
or last. The first word of the payload is the record's original index, so a
stable sort that swaps two equal keys is caught. A record holding a `String`
(not `Copy`) and a composite key from two fields are there too.

A few inputs exist only for records, because their paths differ: keys at the
limit of the few-keys counting and just above it, ascending keys in runs of
8 equal ones, descending keys that are not strictly descending (they must
not be reversed: equal keys would change order), teeth and pipe organs with
duplicates, sorted runs with many ties between them, one key holding 60 %
of the records.

---

## 7. How they are generated

- **Seeded per cell.** A cell is a type, a size and a distribution. In the
  decision trace and the sentinel bench, each cell draws from a seed
  computed from its name, in a one-thread pool: the same keys at every run,
  on every machine. Without that, about 2 % of the cells changed path from
  one run to the next, because some inputs sit on a decision boundary. The
  correctness tests and the full benchmarks use fresh random inputs, to see
  more of them.
- **In parallel, then copied.** The generators fill large arrays on all
  cores. The first write from another core then has to fetch each cache
  line, which would make the first pass of the sort look slow; the
  benchmarks sort a copy made by the measuring thread (see
  [How to benchmark a sort](how_to_bench.md)).
- **New arrays for each measure.** Each measurement sorts newly generated
  arrays, so that no sort learns a fixed sequence of branches.
- **Every size.** The same distribution changes nature with the size:
  Sqrt has 31 values at 1000 keys and 3162 at 10 million; on 16-bit keys,
  uniform data are mostly distinct at 1000 keys and mostly duplicates at a
  million. The tests and benchmarks run each distribution from a few
  hundred keys to tens of millions.

---

## 8. Limits

- **The lists are not the same for all types.** The shared lists bring the
  types of a family close, but the own lists are older: `u64` has
  disturbed sorted arrays and killers that `u32` lacks, `i32` has fewer
  entries than `u32`, and a few entries of the older lists are switched
  off in the code. Bringing them to the same set is ongoing work.
- **Some names are historical.** `Alt16` holds the values 0 to 15 at
  random, not alternating ones; `Normale 20` uses a standard deviation of
  10^6, close to 2^20.
- **Synthetic is not real.** These shapes are our best guess at real data.
  If your data have a shape that is not here, and voracious is slow on it,
  please open an issue: it will become a generator.

---

## 9. Next

The generators were written type by type, as each sort was reworked. The
next step is a single description of every family, generated for every
type, so that a new shape is added once for all of them.
