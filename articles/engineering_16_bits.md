# Engineering voracious radix sort for 16-bit keys

*November 2026, voracious_radix_sort 2.1.0. Written with Claude (Anthropic's AI model), from the project's development journal.*

Sixteen-bit keys look like the easy case for a radix sort. Two byte passes
are enough, and from a few tens of thousands of keys a single counting pass
over the 65 536 possible values is enough. Yet in voracious 2.0.0, the
16-bit sort was slower than it should be below a few thousand keys, and on
several distributions beyond that.

This post explains how the u16 / i16 sort was rebuilt for 2.1.0: what each
size range needs, which ideas paid and which did not. It is about the
techniques; the benchmark results and their analysis will have their own
post, and the way we measure has one:
[*How to benchmark a sort without fooling yourself*](how_to_bench.md).

Most of the gains did not come from one clever trick. They came from a
stricter method (measure each engine alone, add the detectors one at a
time, keep only what wins on every machine) and from one principle that
grew stronger along the way: **a question that usually fails must cost
almost nothing when it fails.**

---

## 1. The setting

### Why 16-bit keys

Sixteen bits is a common key width in real data: sensor readings, audio
samples, image channels, counters, short identifiers, offsets. Arrays of a
few hundred keys are often sorted again and again, one after another. Arrays
of millions of keys come from logs and measurements. Both ends matter.

### The bench

- **31 distributions**, from uniform keys to near-sorted arrays, runs, saw
  teeth, few distinct values, Zipf-like and bimodal data, sparse arrays,
  values clustered around one byte, and so on.
- **Sizes** from 500 to 30 million keys, plus targeted runs from 8 to 2 000
  keys.
- **Isolated calls.** No buffer is kept between two sorts, as in a real
  program that sorts once in a while.
- **Generic build.** We compile in release mode without
  `-C target-cpu=native`, because that is what a user of the crate gets.
- **Machines:**
  - AMD Ryzen 9 3950X (Zen 2, 512 KiB of L2 per core);
  - Intel i5-4690K (Haswell, 256 KiB of L2, 6 MiB of L3);
  - Snapdragon X Elite (ARM).
- **Rivals:** blazing-radix (the one to beat on 16 bits), the standard
  library's `sort_unstable`, voracious 1.x and turbosort.

Several ideas below come from reading blazing-radix. They are credited
where they appear and reimplemented from scratch: we reuse ideas, never
code.

---

## 2. The method

### Engines first, then one detector at a time

A voracious sort is a chain. Cheap detectors run first: already sorted,
reversed, constant, few distinct values, narrow range, heavy hitters, and so
on. An engine sorts whatever is left. The first attempts at the 16-bit chain
patched filters in front of the detectors, one A/B test after another. Each
patch fixed one loss and created another.

We switched to a three-step method:

1. **Engines alone.** Find the fastest engine for each size range with no
   detector at all.
2. **One detector at a time** in front of that engine, measured on every
   distribution, to see what each detector wins and what it costs when it
   fails.
3. **Assembly** per size range, from these tables.

### How we measured

Old and new code ran in the same process, interleaved in random order, on
cold calls, in the generic build (no `-C target-cpu=native`), over 31
distributions, and every change was checked over six code placements
before being kept. Why each of these rules exists, and what went wrong
before them, is the subject of
[*How to benchmark a sort without fooling yourself*](how_to_bench.md).

---

## 3. Big arrays: counting the whole domain

From 131 072 keys on, the 16-bit sort counts every key in a table of 65 536
counters, then writes the values back in order. Two radix passes would read
and write the array twice. Counting reads it once and writes it once.

### The counters

The table of 32-bit counters takes 256 KiB. On Haswell, with 256 KiB of L2,
that table does not stay in the L2 next to the data. There, the counters are
16-bit (a 128 KiB table). A counter that wraps to zero records its value,
which happens once per 65 536 equal keys, behind a branch that is almost
never taken. Haswell counts 15 to 38 % faster this way. On Zen 2 (512 KiB of
L2), 16-bit counters are 10 to 22 % slower on values that stay in cache, so
the rule depends on the size of the L2.

### Writing the values

The obvious loop writes `count[v]` copies of each value `v`. Its branch
mispredicts whenever the counts are small and irregular. Two changes:

- **Fixed 32-key stores** (idea from blazing-radix's `counting16`). Each
  touched bucket is written with full 32-key blocks. The extra copies past
  the end of a bucket are overwritten by the next bucket, so there is no
  branch on the count. Uniform 1M: 15 % faster; Sensor 300k: 22 to 25 %.
- **Flat writing.** When empty and full buckets alternate at random (Zipf,
  Heavy 4), even the test "is this bucket empty?" mispredicts. A mispredicted
  test costs about 18 cycles, a wasted 32-key block about 4. So when the
  transitions between empty and full buckets are frequent enough (measured
  on 256 pairs of neighbouring counters), every bucket writes one block, empty
  or not. Zipf and Heavy 4: 14 to 46 % faster.

### Beyond the caches

When the array is larger than twice the last cache level, its lines are no
longer in cache when the values are written back. Non-temporal stores write
them without reading them first: 6 to 15 % faster beyond that point on the
three machines.

### Runs of equal keys

Near-sorted arrays of millions of 16-bit keys hold long runs of equal keys
(about 150 per value at 10M). Every increment then waits for the previous one
on the same counter. A shortcut counts four equal keys with one `+= 4`.
It makes these arrays 42 to 61 % faster, but its branch costs up to 3 times
on data without runs. It is only enabled when at least half of 64 sampled
groups of four keys hold a single value.

### What the rest of the chain costs here

At this size, counting is so cheap that most of the chain costs more than it
saves. Repairing a near-sorted array, sorting a sorted head first, or
splitting the array into runs (Verge) were 1.4 to 2.5 times slower than just
counting the domain. Above 131 072 keys the 16-bit chain keeps only three
steps: the sorted / reversed / constant check, a comparison-based count of up
to four distinct values, and a count of the high byte alone when the low byte
is constant (multiples of 256). Everything else goes straight to the domain
count.

---

## 4. Five sorts by size

The 2.1.0 u16 / i16 sort is five sorts, chosen by the size of the array:

| Size | Sort |
|---|---|
| up to 32 keys | a sorting network |
| 33 to 64 | sorted blocks and a merge |
| 65 to 5 504 | the **short sort** (section 5) |
| 5 505 to 65 536 | the **light chain** (section 6) |
| beyond | the full chain, counting the whole domain from 131 072 keys (section 3) |

The short sort and the light chain ask the same questions as the full chain
(is it sorted, few values, a narrow range, one heavy value, random keys),
but each question is built to cost almost nothing when the answer is no.
Below 65 536 keys most arrays are answered by the engine, so every failed
question is pure overhead, paid on every call.

### The engines

Two LSD engines sort what the questions leave, two byte passes each:

- **The short LSD** (x86, and ARM up to 600 keys):
  - the buffer on the stack up to 5 504 keys (11 KiB), on the heap beyond;
  - both histograms in one read of the keys;
  - one counting table per byte, unless at least 3 of the first 16
    neighbouring keys share their high byte (smooth or repeated keys, where
    one table would make each increment wait on the previous one): then two
    tables, keys in turn;
  - from 4 096 keys, two tables and 8 keys sharing a byte added with one
    `+= 8` (idea from blazing-radix);
  - 4 keys read before their 4 stores, and 4 keys of one digit taking their
    slots with one update (ideas from blazing-radix);
  - a byte that is the same in every key is not scattered: the other byte's
    counts write the result directly.
- **The 16-bit-counter LSD** (ARM from 600 keys): the counters fit in 16
  bits below 65 536 keys, which halves the tables; on the Snapdragon this is
  2 to 7 % faster from 1 100 keys, and up to 26 % on Zipf, Normal 8, Depth
  or Clusters, where the short LSD's single table waits on its counters. On
  x86 the same 16-bit counters are 25 to 30 % slower (Zen 2), so the choice
  is made per architecture at compile time.

Tables returned from inside an `if` are copied (two `memcpy` of 1 KiB seen
in the ARM assembly): the engine counts in place, in tables declared once.

---

## 5. The short sort (65 to 5 504 keys)

### Where the time went

At these sizes the engine is a few hundred nanoseconds. Everything in front
of it competes with it. The full chain asked, in order: sorted or reversed
(two scans), then probes on a sample, then the near-sorted repair, run
detection and Verge, before the engine. Measured piece by piece at 100 keys
on the Snapdragon: 18 ns for each order scan, 44 ns for the probes, 254 ns
for the engine. And at 1 500 keys, the few-values estimate alone cost
0.45 ns per key on Heavy 4 (4 values for 60 % of the keys, the rest
random), where it always fails.

So the short sort keeps the questions and changes how they are asked.

### One order scan

The array is scanned once, by blocks of 16 pairs taken from two overlapping
reads (keys `i..i+16` against `i+1..i+17`), with no branch inside a block.
The scan returns the length of the ascending head:

- all of it: the array is sorted (or constant), done;
- else, if the first key is above the last, one descending scan; all of it:
  reversed, done (equal keys may open a descending array, so the test is on
  the ends, not on the first pair);
- else, the length of the sorted head is kept for later (Pipe organ).

A constant array is caught before, by one OR of differences, read only when
the first, middle and last keys agree.

On Zen 2 this scan made sorted inputs and saw teeth 20 to 35 % faster from
1 000 keys than the shared scan.

### Probes on 18 keys (below 1 100 keys)

The first 16 keys, the middle one and the last one. In the spirit of
blazing-radix's preparation, each probe gives up at once:

- at most 4 values: every key compared with them, nothing written unless
  every key is one of them;
- one byte varying: only that byte counted, while the other byte is checked
  in the same read (OR of differences);
- a range of at most 256 values, in order or rotated by half the domain
  (values near both ends): checked on every key, counted, written by blocks
  of 16 keys that may spill over the next values' slots (no branch per
  value; a `fill` per value cost about 10 ns each).

On ARM the probes of the full chain win on some shapes (two lanes of 16-bit
counters on a range of 16 values, the high byte alone, one dominant value),
so ARM keeps them, rewritten without a branch per key: the 18 keys' range,
rotated range and varying bits are folded at once; the counts of each key
among the 18 are computed only when the first five keys repeat one (random
keys: almost never). 44 to 21 ns at 100 keys.

### The sample's questions (1 100 keys and more)

A spread sample of 64 keys feeds the detectors. The expensive part of the
full chain was not the detectors that fire, but those that fail: a branchy
vote for a dominant value, a hash-table estimate of the distinct values,
each mispredicting on random keys.

Eight spread picks of the sample are counted over the whole sample, one
vectorised compare each. These eight counts answer three questions at
once:

- **random keys?** If no pick repeats, the random-sample test runs (and
  sends the array straight to the engine). If one pick repeats, the keys
  are not random and the test is skipped;
- **one dominant value?** A value holding half of the sample is among the 8
  picks but once in 256; its count decides, without the vote;
- **few values?** If three picks or more are seen only once, there are too
  many values, and the few-values estimate is skipped.

When the estimate does run, it is branch-free: three maps of 2 048 hashed
bits (values seen once, twice, three times) give the number of distinct
values seen, and those seen once (f1) and twice (f2), for the Chao1
estimate. One more rule came from Heavy 4: few-valued inputs show at most 5
keys seen once in 64, Heavy 4 shows 17 to 29. With Chao1 alone, some Heavy 4
arrays passed the estimate, counted the whole array and failed. Now more
than 8 keys seen once in 64 means "not few".

At 1 500 keys on Zen 2, Zipf, Clusters and Heavy 4 went 0.2 ns per key
faster; the detectors that fire (FakeZipf, Few 16, Sparse) kept their gains.

### The engine, and only the engine

What the questions leave goes to the LSD directly. The near-sorted repair,
the run detection and Verge are not tried: at these sizes they cost more on
the inputs where they fail than they win where they fire. One exception:
when the sorted head found by the order scan covers half the array (Pipe
organ), the tail is sorted alone and merged.

### Where it stops

The short sort was measured against the full chain up to 65 536 keys, to
find where to switch. On the Snapdragon it stayed ahead up to 65 536. On
Zen 2 it fell behind from about 16 000 keys, and the cells it lost there
were large: constant arrays (it had no constant check yet), near-sorted
arrays (no repair), sorted runs (the shared scan is faster on long sorted
runs on x86). Hence a fifth sort between the two.

---

## 6. The light chain (5 505 to 65 536 keys)

The light chain is the short sort's questions and engine with the full
chain's handling of ordered inputs:

- the constant check;
- the order scan: the overlapping-window scan on ARM, the shared scan on
  x86 (faster there on long sorted runs);
- the sample's questions of the short sort;
- then the whole path of the full chain: near-sorted repair, sorted head
  first, run detection, Verge, with the short sort's engine for the chunks.

Beyond 65 536 keys the full chain takes over. Its domain count (one key per
counter from 65 536 keys) wins Clusters by 1.7 to 2.2 times from 32 000
keys; below that, the light chain's cheaper questions win.

### Below 200 keys

The short sort still pays a fixed cost per call that the fastest rivals do
not: its order scan and probes take a few tens of nanoseconds, against a
sort of a few hundred. On random keys of 80 to 200 keys this is about 9 ns
per call. We did not hide it with a special path: in absolute time, any
other work around a sort of 100 keys costs more than that.

---

## 7. Signed keys (i16)

u16 and i16 are one code. The keys are handled as their 16 bits, and a
constant `F` is XORed onto them wherever their order matters: 0 for u16,
the sign bit for i16 (the order of i16 is that of its bits with the sign
bit flipped). Comparisons, the digits counted and the values written from a
digit all go through `bits ^ F`; equality tests and moves of whole keys need
nothing. A fix for one type is a fix for the other.

---

## 8. Engineering techniques, on 16-bit keys

The sections above follow the chain. This one follows the techniques: the
low-level ones a sort can use, whatever its algorithm, and where each one
stands on 16-bit keys. **Used** means it is in 2.1.0; **tried** means it
was measured and left out; **not yet** means it was never tried on 16 bits.

### Computation

| Technique | On 16-bit keys |
|---|---|
| Auto-vectorisation (loops written so the compiler turns them into SIMD) | **Used**: the constant check (one OR of differences), the order scan by blocks of 16 pairs, the comparison count of up to 4 values, the 8 picks counted over the sample |
| Explicit SIMD (intrinsics) | **Used** for the non-temporal stores only (AVX2 detected at run time on x86, `stnp` on ARM); the counting loops stay scalar |
| SIMD sorting networks | **Not yet** |
| Code for the CPU (`target-cpu=native`) | **Not used**: the crate is built generic, as users get it. Choices that differ by architecture are made at compile time (`cfg(target_arch)`): the probes and the small-array engine differ between x86 and ARM |

### Instruction-level parallelism

| Technique | On 16-bit keys |
|---|---|
| Several counting tables | **Used**: two tables, keys in turn, when neighbouring keys share their high byte; four tables in the probes from 1 024 keys and in the window of 256 values. A repeated value no longer waits on its own counter |
| Equal keys added at once | **Used**: four equal keys counted with one `+= 4` on long runs (big arrays); 8 keys sharing a byte added with one `+= 8` from 4 096 keys (short LSD) |
| Read before write | **Used**: the short LSD reads 4 keys before their 4 stores |
| Wide reads | **Used**: 4 keys read with one 64-bit load in the domain count (Zen 2 issues only two loads per cycle). **Tried**: 2 keys per 32-bit load (fast in one build, slow in the next) and the same wide read in the window of 256 values (slower on Haswell, where the tables sit in the L1) |
| No branch per key | **Used**: the order scan, the picks, the hashed bitmaps of the few-values estimate, the 32-key stores, the flat writing of buckets |
| Predictable branches | **Used**: the flat writing decides once per call, from a sample, instead of testing every bucket |
| Hand-written assembly for the increment (`inc [reg]`) | **Tried**: slower on both x86 machines |

### Caches

| Technique | On 16-bit keys |
|---|---|
| Thresholds derived from the cache sizes | **Used**, read at run time: 16- or 32-bit counters from the size of the L2, the window of 256 values while the array fits in the L2, non-temporal stores beyond twice the last level |
| Smaller tables | **Used**: 16-bit counters halve the domain table (128 KiB instead of 256) on small L2s, and the counting tables of the ARM LSD below 65 536 keys |
| No hidden copies of tables | **Used**: tables declared once and counted in place (a table returned from an `if` was copied, seen in the assembly) |
| Non-temporal stores | **Used** beyond twice the last cache level: the written lines are not read first |
| Software prefetch | **Not yet** |
| Tables aligned on cache lines | **Not yet** |

### Memory and allocation

| Technique | On 16-bit keys |
|---|---|
| Stack instead of heap | **Used**: the short LSD's buffer on the stack up to 5 504 keys (11 KiB), on the heap beyond. **Tried**: a smaller stack buffer for the shortest arrays, no gain |
| No useless initialisation | **Used**: the buffers are left uninitialised (`MaybeUninit`), each slot is written before it is read |
| No buffer at all | **Used** where it fits: the domain count and the probes rewrite the array from their counts. They allocate no buffer of the array's size (only the counting table), so they pay no page faults on it, which a fresh buffer of millions of keys does at every isolated call |
| Huge pages, TLB | **Not yet** |

### Compiler and generated code

| Technique | On 16-bit keys |
|---|---|
| Generics and const generics | **Used**: one code for u16 and i16 (the constant `F`, section 7); the number of tables, the byte shift and the window offset are constants of each instance |
| Bounds checks removed | **Used**: blocks read with `as_chunks`, indices masked to the table size |
| Control of inlining | **Used**: the counting kernels are inlined into their caller. **Tried**: the kernels kept apart (`#[inline(never)]`), no gain once the code was clean, slower on Zen 2 |
| Reading the assembly | **Used** to explain surprises: the copied tables, and why a rival's i16 code read 4 keys per load where its u16 code read one |
| Code placement | **Used** as a design rule: short, isolated hot loops and no switch between variants in the shipped code, because the alignment of a loop alone moved some cells by up to 28 % |
| Profile-guided optimisation, LTO | **Not yet** |

What is left (explicit SIMD for the counting, prefetching, aligned tables,
huge pages, PGO) is for the next rounds. The multithread techniques belong
to the multithread sort, which is not reworked yet.

---

## 9. What did not work

| Idea | Why it failed |
|---|---|
| Counting the product of the present bytes in the LSD | Wins only when that product is far below n; costs about 3 % on average to compute it at every call |
| Rejecting the `few` detector on samples full of single values | Would also reject Late outliers, where `few` is twice as fast |
| 16-bit counters in the small-array LSD | +2 to +12 % on Zen 2 for -1 to -4 % on Haswell |
| `sort_unstable` below 100 keys | No gain, 3 to 6 % slower at 70 keys |
| A second order probe in the middle of small arrays | No gain |
| Reusing the small-array range counter for 2k-131k keys | 1.2 to 1.4 times slower than the existing range counter |
| No heavy-hitter extraction below 131k keys | Sparse arrays 3 to 5 times slower |
| No "sorted head first" below 131k keys | Pipe organ 1.3 times slower |
| Choosing 16- or 32-key stores per value when writing counts | The branch mispredicts around 15 keys per value, so the gain vanishes |
| Sending random-looking short arrays straight to the engine, before the probes (several gates tried) | The gates cost more than they saved, and some sent Pipe organ or Aligned to the engine |
| The short sort's x86 probes on ARM | Range 16, RootDup and Bimodal 1.1 to 1.7 times slower than the chain's probes with 16-bit counters |
| The short sort from 33 keys instead of the merge | More cells lost at 48-64 keys on Haswell and the Snapdragon |
| An order scan in front of the merge at 33-64 keys | Same: sorted inputs faster, random ones slower, more cells lost |
| The few-values estimate as a 64 × 64 compare of the sample | Not vectorised well enough: slower than the hash table it replaced |
| A smaller stack buffer (1 024 keys) for the shortest arrays | No measurable gain (the stack probe of Windows was not the cost) |


---

## 10. Next

The other key types follow, one per release, with the same method and the
same principle: the 32-bit integers first. Each type's release is also the
time to recheck when `sort_unstable` is still worth calling on small
arrays, and whether the short sort's ways of asking (one order scan,
questions without a branch per key, the engine directly) carry over.
