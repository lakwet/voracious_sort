# How to benchmark a sort without fooling yourself

*November 2026, voracious_radix_sort 2.1.0. Written with Claude (Anthropic's AI model), from the project's development journal.*

This post is not about results. It is about the measuring itself: every
way our benchmarks lied to us while we rebuilt the sorts of voracious, how
we found out, and the tool or rule each problem left behind. Results come
in separate posts, once the measuring can be trusted.

The short version: a sort is a short, branchy, memory-bound piece of code,
and almost everything around it moves its time by more than the change you
want to measure. The order in which you run the variants, where the
compiler puts a loop, what the allocator kept from the previous run,
whether the build targets your CPU, even whether the binary you run is the
one you just compiled.

---

## 1. Decide what you are measuring

### Real life, not the bench

A library sort is called once in a while by a program that does other
things in between. It is rarely called four thousand times in a row on
arrays of the same size. So we measure what such a call costs:

- **No buffer kept between calls.** Every sort allocates what it needs, at
  every call, and pays for it, page faults included. Keeping a buffer
  around would make our numbers better and our users' programs no faster.
- **Cold calls.** Before each timed call, the array is freshly copied and
  the caches are not prepared for the sort (details in section 5).
- **The build users get.** Release mode, without `-C target-cpu=native`
  (section 6).

### Many shapes, many sizes, and every cell on its own

A sort that wins on uniform random keys can lose badly on sorted arrays,
runs, saw teeth, few distinct values, or one heavy value. We measure 31
distributions for 16-bit keys (more for other types), at sizes from a few
dozen keys to tens of millions.

The result of a benchmark is then a grid of cells (distribution × size).
**We read it cell by cell.** A geometric mean over the distributions hides
exactly what matters. In one early summary the mean said "1.7 to 3.1 times
faster" while the median was 1.03 to 2.3: a few huge wins carried the mean
over real losses. Our summaries now count, per size, the cells won, equal
and lost against the best other sort of each cell, and list the lost cells
with their ratio.

### Absolute time matters too

A ratio of 2 on an array that is already sorted can mean 3 ns against
1.5 ns per call. Below a few nanoseconds per sort, the copy of the input
that we subtract (section 3) is itself noisier than the difference. Our
bench viewer therefore has an absolute floor (2 ns per sort by default)
under which a difference is shown as a tie, on top of the ±3 % noise band.

---

## 2. Know your machines

### Several machines, on purpose

We measure on four machines:

| | CPU | Released | OS | L2 | Last cache |
|---|---|---|---|---|---|
| Ryzen | AMD Ryzen 9 3950X (Zen 2) | 2019 | Ubuntu 26.04 | 512 KiB per core | 16 MiB per CCX |
| "petit intel" | Intel i5-4690K (Haswell) | 2014 | Ubuntu 18.04 | 256 KiB | 6 MiB |
| "gros intel" | Intel Core Ultra 9 185H (hybrid: performance, efficiency and low-power cores) | 2023 | Windows 11 | 2 MiB | 24 MiB |
| Surface | Snapdragon X Elite (ARM) | 2024 | Windows 11 | 12 MiB per cluster | none |

The "gros intel" is a work laptop under Windows. It cannot run benchmarks
during the day, so it gets the long runs at night and on weekends, launched
by hand. It is also the machine that taught us to pin threads (below), and
the one that showed that some decisions follow the CPU family rather than
the cache sizes: beyond the largest cache, the two Intel machines prefer
one engine where the AMD and ARM machines prefer another, whatever their
caches.

The point is not to have more numbers. Many decisions of a radix sort
depend on the cache sizes (when to count the whole domain, how wide the
counters are, when the buffer leaves the cache). Our decision trace
(section 7) shows that 340 of 7 816 cells take a different path on Haswell
than on the Ryzen, and 1 859 on the Surface. A rule tuned on one machine is
a rule for that machine. When the machines disagree, we choose per
architecture at compile time, and never let one machine decide for all.

### One measurement at a time per machine

Two benchmarks on the same machine share the memory bus even when they run
on different cores. Once we ran small experiments on the Ryzen while a long
benchmark was running on another core cluster, and perturbed its large
cases. The rule since then: one computation at a time per machine,
compiling included. Different machines can work in parallel.

The laptop adds a second reason. Compiling the test crate of the library
with the benchmark rivals takes about 7.5 GB for one `rustc`. Twice, a
compile next to something else ran the 16 GB Surface out of memory. On the
Surface we compile alone, then run.

### Quiet machines

- Linux machines run with the `performance` governor, inside `screen`, so
  that a dropped SSH session does not kill a ten-hour run.
- Hybrid CPUs (performance and efficiency cores) need pinning. Our CPU
  bench pins its thread to the fastest class of cores on Windows. On the
  Snapdragon, a slow core made every sort 12-15 % slower, all by the same
  factor; Windows kept an unpinned busy thread on the fast cores anyway.
  On the "gros intel", the efficiency cores are another microarchitecture
  with a shared L2, and the gap could be uneven between sorts: its runs are
  pinned to a performance core, and a short pinned run checked that the
  first, unpinned one had stayed on performance cores.
- Native toolchain only. An x86 toolchain emulated on an ARM laptop slows
  everything, and not evenly: a memory-bound radix sort suffers less than a
  comparison sort.

---

## 3. Compare two versions in one process

### Never compare runs from different times

The machine is not a constant: temperature, other processes, the state of
the allocator. Two measurements taken an hour apart differ by more than
most of our changes. The old and the new code always run in the same
process, on the same arrays, interleaved.

### The order of the variants

With a fixed order, the first column of our bench ran about 7 % faster at
10 million keys on the Ryzen, whatever code it held. We checked by swapping
the old and new versions: the advantage followed the position. An earlier
conclusion ("a code placement effect") turned out to be this bias. The order
of the variants is now drawn at random at every repetition.

### What we time

- The input is copied before each call, and the time of that copy is
  measured separately and subtracted.
- Each measurement sorts many arrays (12 million keys in total per
  measurement), repeated 11 times, and the median is kept. A disturbance
  (an interrupt, another process, a cache emptied by someone else) can
  only make a repetition slower, never faster than the code allows. The
  disturbed repetitions are all on the slow side: the median ignores them
  as long as they are fewer than half, where a mean would be pulled up by
  them.
- Every output is checked against the standard library's sort, at least at
  the first repetition. A wrong result is written in the output file.
- If a sort panics (crashes) on one array, the bench records that cell as
  failed for that sort and goes on with the other cells and the other sorts.
  A full bench takes hours: one crash should not end it and leave a file
  half empty the next morning, with the other sorts' measurements lost
  too.

### Noise: measure it, do not guess it

An A/A run (the same sort twice, as two variants) tells how much two
identical columns differ. On our machines one cell can move by up to 25 %
at 10 000 keys while the geometric mean stays within 0.3 %. A single cell
that moves less than the A/A noise is not a result. Our working threshold
is 3 % per cell, and 5 % over six code placements (next section).

### Where each version is compiled

The simplest way to try a new version of a function is to write it in the
test file and compare it with the old one, which lives in the library. Our
first A/Bs did that, and found large differences that had nothing to do
with our changes:

- the same logic, compiled once in the test file and once in the library,
  differed by 10 to 40 %;
- two strictly identical copies of a function, both in the test file,
  differed by up to 60 % on the Ryzen.

The compiler puts each copy at a different address and optimises it a
little differently in its context (inlining, loop alignment), and a hot
loop can become much slower or faster just because of where it lands (the
next section is about this).

So the two versions being compared must be compiled in the same
conditions, both in the library. We keep two instances of the 16-bit chain
there: the original, which is the code we ship, and a copy
(`ab_copy.rs`), compiled only with a development feature, which we edit
for the experiment. For the compiler, both have the same status. And since
placement still plays a part, the comparison is also repeated over six
placements.

---

## 4. The code placement lottery

### Up to 28 % without changing a line

On the Haswell machine, one distribution at 1 million keys ran at 0.59 ns
per key in our measuring example and 0.76 in the official bench. Same
sorting code, same generator, same decisions. `perf` showed the same
instructions and the same cache misses, but 20 % more cycles, and 30 times
more `dsb2mite_switches.penalty_cycles`: the hot counting loop (57 bytes)
fell across two or three 32-byte windows depending on its address, and in
the bad case the micro-op cache of Haswell stopped serving it.

*Code placement* is where the compiler and the linker put each function
and each loop in memory. The CPU does not fetch instructions one by one: it
fetches them in aligned blocks (16, 32 or 64 bytes), keeps them in small
caches and predicts branches by their address. The same loop can fit in
one block or straddle two, share its predictor entries with another branch
or not, depending on where it lands. Its speed then changes, though not one
instruction did. And the address of a loop moves whenever anything before
it in the binary changes size: another function edited, a function added
or removed, another compiler version, other compiler options.

Forcing `-C llvm-args=-align-loops=32` gave the good time back, on that
build. On another build it was the 32-byte alignment that was slow. There
is no alignment that is always right.

The Ryzen does it too: the same source compiled with loops aligned on 16,
then 32 bytes, moved 26 of 93 cells by more than 5 %. This is known in the
literature: Mytkowicz et al., *Producing Wrong Data Without Doing Anything
Obviously Wrong!* (ASPLOS 2009); Curtsinger and Berger, *STABILIZER*
(ASPLOS 2013), for whom a binary is one sample among the possible layouts;
Intel's JCC erratum for the 32-byte boundaries.

### Six placements per experiment

`make abench-align` builds the same A/B six times: loops aligned on 16, 32
or 64 bytes, with one or sixteen code generation units. A change counts
only if it wins in all six. We calibrated it with a copy that is identical
to the library: over two placements, five cells "won" by 3-5 % and twenty
disagreed by up to ±12 %; over six placements, a single false win of 5 %.
Hence the threshold above.

For the official bench, `make bench-type-placed` runs the whole bench over
the six placements and `tools/bench_placements.py` reports a range per
cell and counts only what holds in every placement.

### Switches inside the code make it worse

To try variants quickly we had put run-time switches in the 16-bit chain:
47 of them, each read at every call. They made the hot functions heavier
(a register reloaded from the stack inside a loop) and the timing much more
sensitive to placement. Removing them brought the unstable cells of the
Haswell machine (a cell is one distribution at one size; unstable: its time
moves by more than 5 % from one placement to another, with the same code)
from 39 to 7 out of 155.

So the shipped code has no switch. A variant lives in the copy of the
chain, which the measuring example calls by another name. Two lessons from
this week:

- Keep a switch outside the hot loop, and remember that a copy with
  switches still pays them. Our copy read six to ten atomics per call; at
  100 keys that is enough to make the copy look slower than the library for
  no other reason. At very small sizes, compare against a third sort that
  neither version touches.
- Write the final version without switches and measure that, before
  deciding.

---

## 5. Warm caches, cold caches, and what the allocator remembers

### A predictor that learns the input

A micro-benchmark that sorts the same array again and again lets the branch
predictor learn the sequence of branches. One of our merges measured
1.07 ns per element at 10 000 elements this way; it was 4 ns on fresh
inputs. Always rotate between many different arrays.

### The generator's core

Our generators fill large arrays in parallel, on other cores. The first
write from the measuring thread then has to fetch each cache line from
another core, and the first pass of a sort looks slow up to about a million
keys. The A/B benches were safe (they sort a copy made by the measuring
thread); our first per-step profiles were not.

### Cold calls

Sorting thousands of arrays of the same size in a row keeps the sort's
code, its tables and its branch history hot. A program that sorts once in a
while does not get that. In cold mode, each array is generated at the last
moment, then for each sort in turn: a fresh copy, a sweep over 24 MiB with
random branches (caches and predictor dirtied), the copy read once (as if
just filled), one timed call, the clock's own cost removed.

The cold cost per call was about zero for the standard library's in-place
sort, and 0.5 to 2.5 µs for the radix sorts at 500 to 10 000 keys (their
buffer, and the code and branches of their preparation). That changes the
ranking of the smallest sizes. The official bench (`make bench-type`) is now
cold. Beyond a million keys the fresh copy itself (tens of megabytes, page
faults) costs more than the sort, so there the working arrays are reused;
they are out of cache anyway.

### What the allocator keeps

On the Ryzen, one rival sort took 37 ms on a 10-million-key case in our CPU
bench, and 55 ms when called alone. Not the timing: it made 9 766 page
faults alone (its fresh buffer of 40 MB), and **zero** in the bench. After
the earlier sizes, glibc's heap kept enough already-touched free memory to
serve the buffer without asking the system. Every sort that allocates a
large fresh buffer was favoured, by an amount that depended on what ran
before it.

- On Linux, the bench now calls `malloc_trim(0)` before each measurement:
  each sort pays its page faults, as in an isolated call. About 1.3 µs per
  4 KiB page on the Ryzen, so a fresh buffer of n/2 64-bit keys costs about
  1.3 ns per key at 10 million keys.
- On Windows, large allocations go through `VirtualAlloc` and are paid at
  every call: the bench already measured isolated calls.

The trim did not make anyone "win more". It made the bench honest across
operating systems.

### Give every rival what it would take itself

One rival, a merge sort, lets the caller pass a `Vec` to use as its work
buffer, and uses all of that `Vec`'s free capacity. Called on its own, it
allocates a buffer of n/2 elements. The testbed of our merges passed it our
own working `Vec`, whose capacity was n: it ran with a buffer twice its
usual size. For a merge sort, a larger buffer changes the algorithm (fewer
passes, simpler merges), so we were not measuring that sort as its users
call it. The effect was small, a few percent in its favour, but the
principle matters.

A buffer given by the bench is also a buffer whose allocation is not timed:
in an isolated call, its page faults alone cost about 1.3 ns per key at 10
million keys (see above). So the official bench gives no buffer to anyone:
every sort, ours included, is called through its plain entry point
(`glidesort::sort(a)`, `blazing_radix::radix_sort(a)`, ...) and allocates
what it needs inside the timed call. A fair bench calls every sort the way
its users call it.

---

## 6. Native or generic

`-C target-cpu=native` compiles for the exact CPU of the machine. Users of
a crate almost never do that. We found out the hard way that it matters:

- On the Snapdragon, the native build (LLVM's `oryon-1` model) made our
  two-ended merge 3.5 to 4 times slower. Same instruction set with
  `+v8.7a` alone: no slowdown. It was the scheduling model, not the
  instructions. Many "regressions" against the old version in an early full
  bench came from that.
- A remote script had slipped `native` into every build for a day of
  16-bit work. Some gains on large arrays measured that way disappeared in
  the generic build on Haswell.

The rule: work and decide on the generic build. `make bench-16` runs the
generic build, then the native one, as a second opinion.

---

## 7. A fast loop for every change

A slow measurement changes what you dare to try. At first every change of
a rule was judged on a full bench, about 24 hours, at night; if it showed a
cell moved by mistake, the fix waited for the next night: one or two tries
a day. And the test binary of the library (the whole library and every test
and benchmark of every type) took 6 to 10 minutes to compile at each
change. Four tools cut that loop down.

### Measure in seconds

- **A measuring example** (`examples/ab_u16.rs`, feature `ab`) holds only
  the 16-bit sorts and their rivals. It compiles in 20 to 40 seconds and
  takes the sorts to compare by name, so one run measures several
  variants. A round went from about 12 minutes to 2 or 3.
- **A profiling mode** in the same example (`PROF=...`) sorts one
  distribution at one size for a few seconds, warm, so that `perf` sees
  only that. It is not a benchmark: it answers "where is the time" and
  "which of these two is faster" in seconds.

### Know what moved, without a clock

Most changes to a sort chain do not change an engine. They change which
engine or detector a given input gets. That can be checked without timing
anything.

- **Decision trace** (feature `v2_trace`, `make trace`): every decision
  point notes a word (`presorted`, `few`, `range`, `lsd`, ...). The trace
  runs every type, size and distribution of a fixed corpus (8 546 cells
  today) and compares the words with a reference file per CPU
  (`traces/`). A refactor must move 0 cells; a new rule moves exactly the
  cells it is meant to, and they are listed in `traces/moved.txt`.
- **Seeded generators.** Without a seed per cell, about 2 % of the cells
  changed path from one run to the next: some inputs sit on a decision
  boundary. That is worth knowing about the rule, and it would have made
  the trace useless.

### Time only what moved

The **sentinel bench** (`make sentinel`) answers one question, "does the
new rule make faster or slower what it changes?", in minutes.

- The new code is written behind one switch,
  `if ab::new_rule() { new } else { old }`, that can be turned on at run
  time.
- The trace, run with the switch on, lists the cells whose path changes.
  Every other cell takes exactly the same path as before: the rule cannot
  change its time, so there is no need to measure it.
- Only the moved cells are timed, switch off and on, in the same process,
  interleaved: say 40 cells instead of thousands.
- About a hundred control cells, which the rule does not touch, are timed
  the same way. They should give the same time off and on; the difference
  seen there is the noise of the machine that day (about ±2 % on the Ryzen,
  up to ±14 % on the Surface). A moved cell counts as won or lost only
  beyond that noise.

The first sentinel ran in 4 minutes on the Surface.

### Read summaries, not raw files

A benchmark file has thousands of lines. Small scripts count won / equal /
lost per size, list the lost cells, or print the grid of one sort against
another. We read those.

### The whole loop

For one change, in order:

| Step | Tool | Time |
|---|---|---|
| Iterate on the idea | measuring example, profiling mode | 2-3 min per round |
| Check it over six code placements | `make abench-align` (section 4) | 10-20 min |
| See which cells it moves | `make trace` (the machines in parallel) | 12-22 min |
| Time the moved cells against the noise | `make sentinel` | 4-20 min |
| Run the full tests | `make test` | 1-10 min, plus the build |

About three quarters of an hour for a checked change, instead of a day.
The full bench now runs once per release.

---

## 8. Reading the results

- **Per cell, against the best other sort of that cell.** Not against our
  previous version only, not against one chosen rival, and not as a mean.
- **Won / equal / lost with an explicit threshold** (3 % per cell, the
  absolute floor of 2 ns per sort).
- **Lost cells listed**, with their size and ratio, and kept in the
  published results, small ones included. A loss of 9 ns on an array of 100
  keys is a loss; saying so in absolute terms is more useful than hiding
  it.
- **Ratios and times.** The bench files give the time per key of each sort,
  not only ratios, so anyone can check the absolute cost.

The bench viewer (`bench_viewer/`, a static page) loads a bench CSV and
shows time against size per sort, a grid of ratios, bars per
distribution, and the summary tables, with the noise band and the absolute
floor applied.

---

## 9. The tools, and why each exists

| Tool | What it does | Why it exists |
|---|---|---|
| `make bench-type T=u16` | Official bench of one type: cold, 12 M keys per measurement, median of 11, generic build, every rival that takes the type | The reference numbers, measured the way a user calls the sort |
| `make bench-16` | u16 and i16, generic then native | Both builds, so the native second opinion is always at hand |
| `make bench-type-placed` + `tools/bench_placements.py` | The official bench over six code placements, a range per cell | Tell a real difference from the placement lottery |
| `make abench-align` + `tools/ab_align.py` | An A/B over six placements | Keep a change only if it wins in all of them |
| `examples/ab_u16.rs`, `ab_i16.rs` (feature `ab`) | Fast measuring binary: variants by name, cold mode, CSV for the viewer, profiling mode, decision trace per cell | 20-40 s per round instead of 6-10 min |
| `src/v2/devour/ab_copy.rs` (feature `ab`) | A second instance of the 16-bit chain, the one we edit | A/B without switches in the shipped code |
| `make trace`, `trace-write` | Decision trace against the reference of this CPU | Know what a change moves, without timing |
| `make sentinel` | Rule off / on on the moved cells and control cells | Minutes instead of a night |
| `bench_viewer/` | Static page reading bench CSV files | Look at a whole bench at once, with the noise rules applied |
| `malloc_trim` before each measurement (Linux) | Every sort pays its own page faults | Isolated calls on every OS |
| Thread pinning (Windows) | The fastest class of cores | Hybrid CPUs |

---

## What we would tell ourselves at the start

1. Decide what a "call" is in real life, and measure that.
2. Run old and new in the same process, interleaved, in random order, with
   the output checked.
3. Measure the noise (A/A) before believing a difference.
4. Expect the code placement to move hot loops by tens of percent; measure
   over several placements before keeping a change.
5. Build what users build.
6. Make a round of measurement cheap, or you will stop doing it.
7. Read every cell, and publish the losses.
