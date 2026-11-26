# Testing for a correct sort

*November 2026, voracious_radix_sort 2.1.0. Written with Claude (Anthropic's AI model), from the project's development journal.*

A sort has one job: give back the same elements, in order. A fast sort that
is wrong once in a million calls is worse than a slow one. This post
explains how voracious is tested so that you can trust it with your data,
and why the tests look the way they do.

The difficulty is specific to a library like this one. voracious is not one
algorithm. For each key type it is a chain of decisions: is the array
already sorted, does it hold few distinct values, a narrow range, one heavy
value, sorted runs; then one of several engines (sorting networks, merges,
LSD and MSD radix sorts, counting sorts), chosen by the size of the array
and by the size of the CPU's caches. A test that sorts a few random arrays
goes through one or two of these paths. Most of the code would never run.

So the tests are built around one question: **does every path run, on every
type, and give exactly the standard library's answer?**

---

## 1. The reference: the standard library

Every check compares voracious with the standard library's sort, which is
well tested and simple to trust:

- **Integers and `char`**: the output must be equal, element for element,
  to `sort_unstable()`.
- **Floats**: the output must be equal, **bit for bit**, to the order of
  `total_cmp`. That order is strict about what a comparison with `<` leaves
  vague: negative NaNs first, then -infinity, the negative numbers, -0.0
  before +0.0, the positive numbers, +infinity, positive NaNs last. Comparing
  bits catches a -0.0 and a +0.0 swapped, which a numerical comparison would
  miss.
- **Structs sorted by a key, stable sort**: the output must equal the
  standard stable `sort_by` on the key. The records carry their original
  index, so any reordering of two records with equal keys fails the test.
- **Structs sorted by a key, unstable sort**: the keys in order, and the
  same multiset of records (both sides are put in one canonical order before
  the comparison).

Comparing with the full expected output, not only checking that the result
is in order, matters: a sort that loses one element and duplicates another
returns an array "in order".

---

## 2. Every shape of data

The test data come from the same generators as the benchmarks: dozens of
distributions per type (31 for 16-bit keys, more for 64-bit ones). Among
them:

- uniform random keys over the whole domain, or over a few bits;
- sorted, reversed, nearly sorted, sorted with a random tail;
- saw teeth (ascending and descending), pipe organ, runs of every length;
- all keys equal, two values, few values (4, 16), Zipf-like frequencies,
  one value holding 90 % of the keys;
- narrow ranges, ranges that wrap around the end of the domain (values near
  0 and near the maximum), values aligned on a power of two (only the high
  byte varies), clusters, bimodal and normal distributions;
- extreme values: `MIN`, `MAX`, 0, and for floats the special values above.

These shapes are not decoration: each one exists because it sends the
chain to a different detector or engine. A detector that fires on "few
values" has to be tested on few values, and also on data that almost look
like few values and are not (a few values plus random outliers), because
that is where a detector gives up halfway.

---

## 3. Every size that matters

The choice of path depends on the size of the array, so the sizes are
chosen from the code, not at random:

- **every size up to 70**: the sorting networks and the small merges;
- **every fixed threshold of the chains, minus one, itself, plus one**:
  127-129, 149-151, 199-201, 255-257, 499-501, 511-513, 1023-1025,
  1099-1101 (the probes of the 16-bit short sort), 2047-2049, 4095-4097,
  5503-5505 (the 16-bit short sort and its stack buffer), 8191-8193, up to
  65535-65537 (where 16-bit counters would overflow) and powers of two up to
  8 million;
- **the sizes of the benchmarks**, from 1 000 to 10 million keys (the
  benchmark per type starts at 500, which is among the thresholds above;
  its largest size, 30 million, is left out of the default tests to keep
  them around half an hour).

An off-by-one at a threshold is the classic bug of a hybrid sort: the code
for "up to N keys" receives N + 1, or a buffer sized for N gets N + 1
writes. Testing each threshold on both sides catches it.

### Thresholds that depend on the machine

Some thresholds are not constants. voracious reads the size of the CPU's
caches at run time and derives from it when an array is "beyond the cache"
(different engines, different counter widths, different merge strategies).
On a test machine with a large cache, those paths would only run on arrays
of hundreds of millions of keys.

So the tests **force the cache size**. A test-only override sets the cache
to a few kilobytes, which brings the "beyond the cache" paths down to a few
thousand keys. One test (`test_v2_cache_thresholds`) sorts every key type
around the thresholds derived from the forced and the real cache, on the
shapes that reach those engines. Other tests force a small L2 to reach the
code that only runs on CPUs with a small cache, whatever CPU runs the
tests.

---

## 4. Unit tests: each piece on its own

Above, the public functions are tested as a user calls them. Inside, each
piece of the sort is tested alone, next to its code: about twenty test
modules in the library (engines, detectors, merges, run detection, the
near-sorted repair, the 128-bit keys, the struct sorts).

They exist because a chain can hide a broken piece. If a detector gives up
too early, the engine behind it still sorts the array correctly, and the
public tests pass. A unit test calls the detector or the engine directly,
on every shape and size it is meant for, and checks it alone:

- each **engine** (networks, small merges, LSD, MSD, counting, DLSD,
  rollercoaster) on random, sorted, reversed, few-valued and narrow inputs,
  in both key orders where signed keys reuse the unsigned code;
- each **detector** on the inputs it must catch, and on the near misses it
  must reject without writing anything;
- the **merges**, including the SIMD block merges, on runs of every length
  ratio, short and long, equal keys across the runs;
- the **struct sorts**: every record-size class and every key type, stable
  and unstable (a matrix of all combinations).

---

## 5. The public API, as you call it

`tests/correctness` calls the library only through its public API, the
way a user does:

- every method of the `RadixSort` trait, on slices and on `Vec`;
- the free functions (`devour_sort`, `ravenous_sort`,
  `famished_sort_by_key`, `famished_sort_unstable_by_key`);
- every primitive type: `bool`, `char`, `u8` to `u128`, `i8` to `i128`,
  `usize`, `isize`, `f32`, `f64`;
- edge sizes: empty, one element, two;
- structs sorted by a key of every type;
- a key type defined by the user (a date compared as year, month, day),
  through the `SortKey` trait.

This is what `cargo test` runs. It takes about 30 minutes on a recent
laptop (`QUICK=1 make test` stops at 100 000 keys), and it runs on every
pull request on Linux x86-64, Windows x86-64 and macOS ARM, plus a build
with the minimum supported Rust version.

---

## 6. Fuzzy tests: what we did not think of

Chosen shapes and sizes test what we thought of. *Fuzzing* is the opposite
approach: instead of writing the cases, a program generates inputs at
random, in huge numbers, and checks each result. It finds the combinations
nobody would write by hand.

Our fuzzy test (`make test-fuzzy`) builds each array by mixing random
choices: the type, the size (mostly small, where the thresholds are,
sometimes up to two million), the number of significant bits, a pool of few
values or not, then sorted runs, reversed parts, a few swaps and constant
blocks pasted over random places. Each array is sorted by the public API
and compared with the standard sort, again and again, for as long as asked
(`FUZZ_SECONDS`, one minute by default; before a release we let it run
for several minutes on each of our machines). It is not part of
`cargo test`, so that the default tests keep a fixed length.

Each case is built from one seed. When a case fails, the test prints its
seed, and `FUZZ_SEED=<seed>` replays that exact case alone, so a failure is
never "something went wrong once".

Random tests have paid. One of them, carried over from the first version of
the library, found a bug in the narrow-range detector of 64-bit keys: an
array containing both `i64::MIN` and `i64::MAX`, whose sample looked
neither wide nor narrow, made a 64-bit width overflow to 0, and the array
came back unsorted. The bug had been there since that detector was written.
With the fix came a fixed test, not a random one, that always builds
exactly that kind of array. It failed on the old code (proof that it really
catches the bug) and passes on the fixed one. It stays in the suite: if a
later change ever brings the bug back, it fails at once, without waiting
for a lucky random draw.

---

## 7. Memory safety: Miri

Radix sorts move data through buffers with raw pointers and unchecked
indexing, for speed. Rust cannot check those parts at compile time; they
are marked `unsafe`, and each block states why it is correct.

Miri, an interpreter for Rust's intermediate code, runs the program and
checks every memory access: reads of uninitialised memory, writes out of
bounds, invalid pointers, aliasing violations. It is far too slow for large
arrays, so `make test-miri` runs small arrays with the cache forced small,
which makes even the engines meant for huge arrays (rollercoaster, LSD
beyond the cache, DLSD, the merges) run on a few thousand keys, on random,
narrow, few-valued, sorted, reversed, saw and constant inputs.

---

## 8. Every benchmark checks its output

The benchmarks are not tests, but they sort millions of arrays of every
shape on several machines. Every benchmark checks the output of every sort
against the standard library (at least at the first repetition), and a
wrong result is reported in the output file. Our fast measuring program has
its own check mode that sorts every size from 33 to 1 099 keys, then one
size every 997 keys up to 70 000, on nine shapes, before it measures
anything.

---

## 9. Knowing which path ran

A test that passes says the output was right. It does not say which code
produced it. To know that every path is exercised, the library has a
decision trace (a development feature): each decision point notes a word
(`presorted`, `few`, `range`, `lsd`, ...). A trace run sorts every type,
size and distribution of a fixed corpus and records the path of each of
its 8 546 cells, per CPU. It serves two purposes:

- coverage: a path that never appears in the trace is a path no test
  reaches;
- safety of changes: a refactor must change no path at all, and a new rule
  must move exactly the cells it is meant to move. Anything else is a
  surprise to explain before going further.

---

## 10. On every machine

The same tests run on the three machines we develop on, not only on the CI:
an AMD Zen 2 (x86-64), an Intel Haswell (x86-64, small caches) and a
Snapdragon X Elite (ARM). Different cache sizes send the same array down
different paths (hundreds of the trace's cells differ between these
machines), and some code is specific to an architecture. Before each
release, the full tests pass on all three.

---

## What this means for you

- Every type and every public function is checked against the standard
  library, on dozens of shapes, at every threshold of the code and at sizes
  up to millions of keys.
- The paths that only run on large arrays or small caches are forced to run
  in the tests.
- Fuzzy tests, with a replayable seed, look for what we did not think of.
- Miri checks the memory accesses of the `unsafe` code.
- Every release passes all of this on x86-64 and ARM.

No test suite proves a program correct. If you ever find an array that
voracious sorts differently from the standard library, please open an issue
with the array (or the code that builds it): it will become a test.
