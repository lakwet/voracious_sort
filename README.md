# Voracious sort

![CI](https://github.com/lakwet/voracious_sort/workflows/CI/badge.svg)
![docs.rs](https://docs.rs/voracious_radix_sort/badge.svg)

Radix sorts for Rust: every primitive type (`bool`, `char`, `u8`, `u16`,
`u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`,
`f32`, `f64`) and any struct sorted by a key.

## Version 2.0: a word before you upgrade

Version 2.0 is a complete rewrite of the single-thread sort, and a
**breaking change** (see [RELEASES.md](RELEASES.md)). It is still young: if
you prefer something that has been used for years, version **1.2.2** remains
a good choice and stays available on crates.io. Both are yours to pick.

The multithread sort (`voracious_mt_sort`, peeka sort) has **not been
reworked at all yet**: in 2.0 it is the code of 1.x, unchanged.

**What comes next.** The performance will be worked on one key type at a
time: each type has its own code path, so improving one cannot slow down
another. Each improvement will come out in a small release (2.0.1, 2.0.2,
...), listed in [RELEASES.md](RELEASES.md).

No benchmark results are published for 2.0 for now (the folder
`resultsbenchs/` holds those of 1.x). You can run the benchmarks on your own
machine (see below): they show how long each sort takes there.

> **How version 2.0 was made.** Version 2.0 was written with the help of
> Claude Opus 5.5 (Anthropic's AI model), used as a pair programmer:
> designing, coding, testing and benchmarking with me. Where other sorts had
> good ideas (glidesort, blazing-radix, the Rust standard sorts, research
> papers), we did not copy their code: we rewrote those algorithms our own
> way, for what this crate needed, and each idea is credited in the code.

## Usage

```toml
[dependencies]
voracious_radix_sort = "2.0"
# The multithread sort:
# voracious_radix_sort = { version = "2.0", features = ["voracious_multithread"] }
```

```rust
use voracious_radix_sort::RadixSort;

let mut v = vec![5u32, 3, 12, 0, 40];
v.voracious_sort();

// Structs: sorted by a key, stably (equal keys keep their order).
#[derive(Clone, Copy)]
struct Event { time: i64, id: u32 }
let mut events = vec![Event { time: 7, id: 1 }, Event { time: -2, id: 2 }];
events.voracious_sort_by_key(|e| e.time);
// Or unstably (equal keys in any order, sometimes faster):
events.voracious_sort_unstable_by_key(|e| e.time);
```

The `RadixSort` trait works on slices and on `Vec`:

| method | sorts |
|---|---|
| `voracious_sort()` | `bool`, `char`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32`, `f64` |
| `voracious_stable_sort()` | the same (for these types a sort is always stable) |
| `voracious_mt_sort(threads)` | the same, multithread (feature `voracious_multithread`) |
| `voracious_sort_by_key(key)` | any element, by a key of one of the types above, or of a type implementing `SortKey` |
| `voracious_sort_unstable_by_key(key)` | the same, equal keys in any order |

Two free functions do the same as `voracious_sort` for one family of types:
`devour_sort` (`bool`, `char`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`) and `ravenous_sort` (`f32`,
`f64`).

Floats are sorted in the order of [`f64::total_cmp`]: negative NaNs first,
then the negative values, -0.0, +0.0, the positive values, positive NaNs.

A key of your own (a date, a composite key...) only needs to be encoded into
an unsigned integer of the same order: implement `SortKey` (see the
[documentation](https://docs.rs/voracious_radix_sort/)).

### Memory

Some paths of the sort use a buffer as large as the array or half of it
(allocated by each call and freed at its end). Small arrays and many large
ones are sorted in place, but which path is taken depends on the data, so
**plan for a buffer as large as the array** to avoid a bad surprise on
memory. A strictly in-place sort (no buffer at all) will be added later.

## How it works

Each key type has its own chain of decisions: very small arrays go to
sorting networks and merges; already sorted or reversed arrays are found in
one scan; then the array is read for its shape (nearly sorted, long sorted
runs, few distinct values, narrow ranges, heavy duplicates, clusters...) and
the cheapest method is chosen: a counting sort, an LSD or MSD radix sort, a
merge of the runs found, or a fallback to the standard sort where it is
better. The thresholds follow the cache sizes of the machine, read at run
time.

### Credits

Voracious stands on the work of many people. Every idea below was
rewritten in our own code (no code was copied) and is credited again where
it is used in the source.

Algorithms of voracious itself (1.x, reworked in 2.0). Many parts of the
1.x code were heavily modified to be integrated into 2.0 (the engines, the
run detection, the merges, the codecs of the keys):

- **DLSD**, diverting LSD radix sort: a simpler variant of
  [DFR sort](https://github.com/ramou/dfr), with a different diversion and a
  variable radix.
- **Rollercoaster sort**: an MSD radix sort that switches to an LSD radix
  sort on its buckets.
- **Peeka sort** (multithread, unchanged in 2.0): an improvement of
  [Regions sort](https://people.csail.mit.edu/jshun/RegionsSort.pdf)
  (Obeya et al., MIT, 2019).

Algorithms and ideas from others:

- **Verge sort** pre-processing ([Morwenn](https://github.com/Morwenn/vergesort)):
  long sorted runs are kept and merged instead of sorted again.
- **American flag sort** (McIlroy, Bostic & McIlroy, 1993) and the swap
  loop of **Ska sort** (Malte Skarupke, 2016): the in-place MSD passes.
- **Powersort** (Munro & Wild, 2018): the order in which the k-way merge
  merges the runs.
- **TimSort** (Tim Peters): galloping in skewed merges.
- **glidesort** (Orson Peters, 2023): gap merges split at the crossover
  point; advancing the left run as `l - t + 1` in branchless merge steps.
- **blazing-radix** (Max Tikhomirov, 2026, MIT license): in the fused LSD,
  all histograms in one read on several lanes, `u32` counters, 11-bit
  digits when they save a pass, digits equal in every key skipped, four keys
  read before being written, the scratch buffer left uninitialized, the
  sample of 16 spread keys; the float detectors (constant arrays,
  integer-valued floats, power-of-two grids, few distinct values); for
  `u8` / `i8`, each key placed at the start of its cell through a small
  stack buffer.
- **turbosort**: sorted, reversed and constant arrays found in one scan.
- **IPS4o** (Axtmann, Witt, Ferizovic & Sanders, 2017): handling heavy
  duplicate keys apart, as its equality buckets do; the Exponential,
  RootDup, TwoDup and EightDup distributions of the benchmarks.
- **Vectorized bitonic merges** of sorted blocks: AA-sort (Inoue et al.,
  2007), Chhugani et al. (VLDB 2008), and for NEON arXiv 2409.03970.
- **Batcher's odd-even merge networks** (Batcher, 1968), pruned: the sorting
  networks of very small arrays.
- **Boyer-Moore majority vote**: finding a dominant value on a sample.
- **The Rust standard library sorts** (ipnsort and driftsort, Lukas Bergdoll
  and Orson Peters): the fallback where they are faster, and the
  `random_binary` distribution of their benchmark suite
  ([sort-research-rs](https://github.com/Voultapher/sort-research-rs)).
- **Fast radix sort** ([AwardOfSky](https://github.com/AwardOfSky/Fast-Radix-Sort),
  after Thiel et al., SEA 2015) and **Ska sort**: kept from 1.x as
  references.

The other crates in the benchmarks (glidesort, blazing-radix, rdst,
radsort, turbosort) are compared as they are published, untouched.

## Tests

```sh
make test         # the correctness tests (what the CI runs)
make test-miri    # memory checks under Miri
make test-fuzzy   # random arrays, for a while
```

- **`make test`** (or `cargo test --release --features voracious_multithread`):
  every key type on many distributions of the keys (uniform, sorted, reversed,
  sawtooth, runs, few values, Zipf, clusters, timestamps, NaNs...), at every
  size where the sort changes its method and at the benchmark sizes, compared
  with the standard library sort. The duration on a reference machine is
  printed first; `QUICK=1 make test` stops at 100 000 elements.
- **`make test-miri`**: runs the sorts under
  [Miri](https://github.com/rust-lang/miri), an interpreter of Rust that checks
  every memory access of the unsafe code: reads out of bounds, reads of
  uninitialized memory, invalid aliasing, leaks. It is very slow, so it uses
  small arrays, with the cache sizes forced small so that the methods meant
  for large arrays run too. Needs the nightly toolchain.
- **`make test-fuzzy`**: builds random arrays by combining random choices
  (type, size, range of values, duplicates, sorted or reversed parts, special
  float values), sorts them and compares with the standard sort, for
  `FUZZ_SECONDS` seconds (60 by default). A failure prints the seed of its
  case; `FUZZ_SEED=<seed>` replays it.

## Benchmarks

```sh
make bench                  # every key type, 1 000 to 10 000 000 elements
make bench-type T=u64       # one key type (u8 ... i128, f32, f64, char), 500 to 30 000 000
make bench-records          # structs sorted by key (T=u64: keys of one type)
make bench-big              # 30 million elements and more, as far as the RAM allows
make bench-big-type T=u64
NATIVE=1 make bench         # any of them, compiled for this CPU (see below)
```

The rival sorts are optional dependencies behind the `bench` feature, which
the `make bench...` targets turn on: `make test` and the CI do not build them.
By hand: `cargo test --release --lib --features "voracious_multithread bench"
bench_cpu -- --ignored --nocapture --test-threads=1`.

### How they are built

- **The inputs**: for each key type, dozens of distributions of the keys
  (about fifty for the integers, `src/generators/`): uniform over the whole
  range or a narrower one, sorted, reversed, nearly sorted, sawtooth, pipe
  organ, sorted runs, few distinct values, all equal, Zipf, normal laws,
  clusters, timestamps, aligned addresses, common prefixes, sparse data, heavy
  duplicates, composite keys, and the distributions of other benchmarks
  (binary keys; the exponential, RootDup, TwoDup and EightDup of IPS4o), plus
  the worst cases of our own detectors (a late outlier, a late extra value, a
  random tail). Floats add NaNs, infinities and signed zeros; records add
  sizes from 8 to 128 bytes.
- **The sizes**: 1 000 to 10 000 000 elements (500 to 10 000 000 by type),
  and from 30 million up to what the memory holds for the large benchmarks
  (the array, its copy and a buffer within half the RAM; a size that does
  not fit is written as skipped).
- **The sorts**: voracious 2.0, voracious 1.x, the standard library sort
  (stable or unstable, as the case) and the best other crates of each type:
  the two fastest on that type, and a third one where it beats voracious on
  cases the two others do not (chosen on an earlier full benchmark over three
  machines).
- **The measure**, to be close to real use: every sort sorts the same arrays,
  made new for each measurement; each sort is called once per array, as a
  program would, and pays for its own memory (no buffer kept from one call
  to the next; freed memory is given back to the system between sorts, so
  that page faults are paid as in an isolated call); small arrays are sorted
  several per measurement so that it lasts long enough; the order of the
  sorts changes at each measurement; one thread, pinned to a fast core;
  every result is checked to be sorted.
- **The time of one array**: a measurement times each sort on a batch of
  new arrays of the same size, many for small sizes (4 000 arrays of 1 000
  keys, for example) and one for the largest. The time of one array in that
  measurement is the time of the whole batch divided by the number of
  arrays. This is repeated (3 measurements, more for the large sizes, each
  on new arrays) and the median of these times is kept: it is the duration
  shown in the report; the time per element is that duration divided by the
  size. One clock around the whole batch, rather than one per array, keeps
  the cost and the resolution of the clock out of the result for small
  arrays, which take a few microseconds each; these arrays are sorted one
  after the other, as in a program that sorts often.

**Real life first.** By default the benchmarks compile every sort the way
a user of the crate gets it: a release build with no CPU-specific flags
(cargo never adds `-C target-cpu=native` by itself). That is what most
programs run, so that is what we measure, for voracious and for every other
sort alike. `NATIVE=1` compiles all of them for the CPU of the machine
instead, to see how far each sort goes when it may use every instruction of
that CPU; the report lists the target features used.

### What you get

Each run writes two files in `resultsbenchs/<machine>/`:

- a **report** to read (`.txt`, also printed while it runs): the machine
  (CPU, caches, memory, OS, compiler, commit), how the report reads, then
  for every type and size a table with one line per distribution and, for
  each sort, the duration of one sort, the time per element and the ratio
  to voracious 2.0 (above 1: voracious is faster); the title of each table
  gives the array size, the arrays per measurement and the number of
  measurements; summary lines give the geometric mean and median of each
  column and how many distributions voracious wins;
- the **raw data** (`.csv`), for the tools.

### Tooling

- `python tools/bench_report.py <files or folders>`: summarises any number of
  raw files, from several machines: where voracious loses, by how much,
  against which sort, the geometric mean and median of the ratios per sort,
  size, type and distribution. Options pick the reference column (`--ref`),
  the sorts compared (`--vs`, for an A/B of two variants), types, sizes,
  distributions, a loss threshold, a CSV of the losses (`--help`).
- `python tools/gen_networks.py`: generates the sorting networks of the small
  arrays (`src/v2/engines/small/networks.rs`).

## How we change the code

Each key type has its own chain of decisions (`src/v2/types/`), so a change
for one type cannot move another. A change goes through three steps, with
the tools below, before it is kept.

1. **The decision trace** (`make trace`, `make trace-ab`, `make trace-write`;
   feature `v2_trace`): for every cell (type, size, distribution, with the
   same seeded inputs at each run), the path the sort takes: presorted check,
   detectors, Verge, engines. No timing. Each machine family has a reference
   trace in `traces/` (named after the architecture and the cache sizes the
   rules read); `make trace` lists the cells whose path differs from it. A
   rule under test is written behind a switch (`if new_rule() { new } else {
   old }`, `src/v2/dev/ab.rs`); `make trace-ab` traces every cell with it off
   then on and writes the cells it moves to `traces/moved.txt`. So a change
   shows exactly which cells it touches, and only those need timing.
2. **The sentinel** (`make sentinel`): times the rule off against on, in one
   process, on the moved cells plus a fixed set of control cells (one per
   family of types, size band and shape). The controls give the noise of the
   machine: the change is judged against that noise, in minutes instead of a
   full benchmark.
3. **The tests** (`make test`, and `make test-miri` / `make test-fuzzy` for
   changes to unsafe code), then the switch goes, the winner stays, and the
   reference trace is rewritten (`make trace-write`) when cells moved.

## Contributing

Pull requests are welcome. The CI checks that the code is formatted
(`cargo +nightly fmt`) and that the correctness tests pass on Linux, Windows
and macOS.

**You can also help without writing code: run a benchmark on your machine.**
One key type is enough, for example `make bench-type T=u64` (or `T=u32`,
`T=f64`...), and send the two files it writes in `resultsbenchs/<machine>/`
(`.txt` and `.csv`) to the email address in `Cargo.toml`. Each file records
the characteristics of the machine (CPU, cores, caches, memory, operating
system, compiler, commit), so results from many machines can be compared:
that is how the thresholds of each type get tuned beyond the few machines
we own.

Before a benchmark, put the CPU in its performance mode, so that the clock
does not change during the run: on Linux the `performance` governor (for
example `sudo cpupower frequency-set -g performance`); on Windows the power
mode "Best performance", plugged in; on macOS plugged in, low power mode
off. Close what keeps the machine busy. The report records the power
setting it ran under.

### Windows with the GNU toolchain

The test dependencies (`rand` → `getrandom`) need `dlltool` on the
`x86_64-pc-windows-gnu` toolchain (`error calling dlltool`). Either install
the MinGW binutils, use the MSVC toolchain, or make `getrandom` use `RDRAND`:
`RUSTFLAGS='--cfg getrandom_backend="rdrand" -C target-feature=+rdrand'`.
This only concerns building the tests, not crates that depend on this one.

## The name

A colleague proposed Voracious sort: _Variation Of Radixsort Axelle Cleverly
Imagined Over Uncountable Sessions_. Or someone very voracious of radish.

## References

- [What is radixsort](https://axelle.me/2020/11/08/what-is-radixsort/)
- [Voracious sort](https://axelle.me/2020/11/21/voracious-sort/)
- [DLSD sort](https://axelle.me/2022/04/19/diverting-lsd-sort/)
- [Rollercoaster sort](https://axelle.me/2022/04/19/rollercoaster-sort-the-best-of-the-two-worlds/)
- [Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
- [Verge sort](https://github.com/Morwenn/vergesort)
- [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort):
  [Theoretically-Efficient and Practical Parallel In-Place Radix Sorting](https://people.csail.mit.edu/jshun/RegionsSort.pdf)
- [DFR sort](https://github.com/ramou/dfr)
- [Fast radix sort](https://github.com/AwardOfSky/Fast-Radix-Sort):
  [Relaxing the Counting Requirement for Least Significant Digit Radix Sorts](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf)
- [Powersort](https://www.wild-inter.net/publications/munro-wild-2018) (Munro, Wild)
- [glidesort](https://github.com/orlp/glidesort), [PDQ sort](https://github.com/orlp/pdqsort)

## License

MIT, see the [LICENSE](LICENSE) file.
