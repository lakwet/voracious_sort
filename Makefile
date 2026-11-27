# `make` alone lists the targets.

FEATURES = --features "voracious_multithread"
# The benchmarks also build the rival sorts (feature `bench`, off for `make test`).
BENCH_FEATURES = --features "voracious_multithread bench"

# ---------------------------------------------------------------------------
# Tests

# Duration measured on the reference machine (Surface Pro 11, Snapdragon X
# Elite X1E80100, 12 cores, 16 GB, Windows 11).
test: ## correctness tests (public API in tests/, unit tests, doc examples); QUICK=1: up to 100k
	@echo "Correctness tests: about 30 min on a Surface Pro 11 (Snapdragon X Elite X1E80100, 12 cores, 16 GB),"
	@echo "plus about 10 min to compile the first time."
	@echo "Slower machines take longer; QUICK=1 make test: sizes up to 100k only."
	@cargo test --release $(FEATURES)

test-miri: ## memory checks under Miri (nightly): small arrays, small cache forced
	@rustup component add miri rust-src --toolchain nightly
	@cargo +nightly miri test --lib $(FEATURES) miri_

test-fuzzy: ## random arrays against the standard sort for FUZZ_SECONDS (60); FUZZ_SEED=<seed> replays a case
	@cargo test --release $(FEATURES) --test fuzzy -- --ignored --nocapture

# ---------------------------------------------------------------------------
# Code

build: ## build the library (release)
	@cargo build --release $(FEATURES)

check: ## compile everything (library, tests, benchmarks) without running it
	@cargo check --tests $(BENCH_FEATURES)

lint: ## clippy on the library, the tests and the trace
	@rustup component add clippy
	@cargo clippy --lib --tests --features "voracious_multithread v2_trace bench"

fmt: ## format the code (nightly rustfmt, as the CI checks it)
	@rustup component add rustfmt --toolchain nightly
	@cargo +nightly fmt --all

doc: ## build the documentation and open it
	@cargo doc --no-deps --open $(FEATURES)

clean: ## remove the build files
	@cargo clean

# ---------------------------------------------------------------------------
# Benchmarks (src/tests/bench/bench_cpu.rs): voracious 2.0 against 1.x, std
# and the best rivals of each type. Two files per run in
# resultsbenchs/<machine>/: a report to read (.txt, also printed) and the
# raw data (.csv, for tools/bench_report.py).

ifeq ($(OS),Windows_NT)
# Avoid the dlltool dependency of getrandom on the x86 windows-gnu
# toolchain (not on Windows on ARM's aarch64 msvc one: no rdrand there).
ifneq ($(findstring x86_64-pc-windows-gnu,$(shell rustc -vV)),)
BENCH_RUSTFLAGS ?= --cfg getrandom_backend="rdrand" -C target-feature=+rdrand
endif
endif
# NATIVE=1: every sort compiled for this CPU (-C target-cpu=native); by
# default the generic build a user of the crate gets.
ifeq ($(NATIVE),1)
BENCH_RUSTFLAGS += -C target-cpu=native
endif

BENCH_RUN = cargo test --release --lib $(BENCH_FEATURES) bench_cpu -- --ignored --nocapture --test-threads=1
BENCH_SIZES_ALL = 1000,10000,30000,50000,100000,200000,600000,1000000,3000000,10000000
BENCH_SIZES_RECORDS = 500,1000,5000,10000,20000,50000,80000,100000,200000,500000,1000000,2000000,5000000,10000000
# Key types: up to 30M as well (beyond the last cache level of every machine).
BENCH_SIZES_TYPE = $(BENCH_SIZES_RECORDS),30000000
# Large arrays: as far as the memory allows (an array, its copy and a buffer
# within half the RAM, BENCH_MEM_MB), one measurement each.
BENCH_SIZES_BIG = 30000000,100000000,300000000,1000000000,2000000000,5000000000,10000000000
BENCH_TYPES_BIG = u32,u64,i64,f32,f64,u128
# Large arrays of the by-type benchmarks: more arrays than 3 measurements give.
BENCH_MIN_SORTS_TYPE = 2000000:8,5000000:5,10000000:5

bench: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench: export BENCH_NAME = bench
bench: export BENCH_PARTS = int,float,wide
bench: export BENCH_SORTS = best
bench: export BENCH_SIZES = $(BENCH_SIZES_ALL)
bench: ## every key type, 1k to 10M keys (several hours)
	@$(BENCH_RUN)

bench-type: export RUSTFLAGS = $(BENCH_RUSTFLAGS) $(PLACE_FLAGS)
bench-type: export BENCH_NAME = bench-type-$(T)$(if $(filter 1,$(NATIVE)),-native)$(PLACE_NAME)
bench-type: export BENCH_PARTS = int,float,wide
bench-type: export BENCH_TYPES = $(T)
bench-type: export BENCH_SORTS = best
bench-type: export BENCH_SIZES = $(BENCH_SIZES_TYPE)
bench-type: export BENCH_MIN_ARRAYS = 1
bench-type: export BENCH_MIN_SORTS = $(BENCH_MIN_SORTS_TYPE)
bench-type: export BENCH_COLD = 1
bench-type: export FRESH = 1
bench-type: export BENCH_REPS = 11
bench-type: ## one key type, 500 to 30M keys, cold, fresh pages (median of 11): make bench-type T=u32 (u8 ... i128, f32, f64, char)
	@test -n "$(T)" || (echo "usage: make bench-type T=u32" && exit 1)
	@$(BENCH_RUN)

# The bench of one type over several code placements of the same source
# (PLACEMENTS="align:units ...", as abench-align): one build and one file
# per placement (bench-type-<T>[-native]-place-<align>-<units>-...); the
# placement alone moves some cells by 5-35 %, tools/bench_placements.py
# gives each cell's range and counts only what holds in every placement.
bench-type-placed: ## bench-type over PLACEMENTS (6 builds by default): make bench-type-placed T=u16 [NATIVE=1]
	@test -n "$(T)" || (echo "usage: make bench-type-placed T=u16" && exit 1)
	@for p in $(PLACEMENTS); do a=$${p%%:*}; u=$${p##*:}; 	  CARGO_TARGET_DIR=target/place-$$a-$$u CARGO_PROFILE_RELEASE_CODEGEN_UNITS=$$u 	  $(MAKE) --no-print-directory bench-type T=$(T) NATIVE=$(NATIVE) 	    PLACE_FLAGS="-C llvm-args=-align-loops=$$a" PLACE_NAME="-place-$$a-$$u" || exit 1; 	done

# Simulated u16 bench (examples/ab_u16.rs, cold, median of 11): a few minutes
# instead of the test binary's build; v2 against std, turbosort and blazing,
# written to bench_work/<host>/abench-cold-u16-<host>-<UTC date>.csv for
# the bench viewer. Not the official bench (published numbers: bench-type).
ABENCH_SIZES = 500,1000,5000,10000,20000,50000,80000,100000,200000,500000,1000000,2000000,5000000,10000000,30000000
ABENCH_HOST := $(shell hostname)
ifeq ($(OS),Windows_NT)
ABENCH_CPU := $(shell powershell -NoProfile -Command "(Get-CimInstance Win32_Processor).Name")
else ifeq ($(shell uname),Darwin)
ABENCH_CPU := $(shell sysctl -n machdep.cpu.brand_string)
else
ABENCH_CPU := $(shell grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | sed 's/^ //')
endif

abench-u16: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
abench-u16: ## simulated u16 bench, cold, a few minutes (for the bench viewer)
	@mkdir -p bench_work/$(ABENCH_HOST)
	COLD=1 TOTAL=12000000 REPS=11 CPU="$(ABENCH_CPU) (ab_u16 example, cold)" 	  CSV=bench_work/$(ABENCH_HOST)/abench-cold-u16-$(ABENCH_HOST)-$$(date -u +%Y-%m-%d-%H%M).csv 	  SORTS="v2,std,turbosort" DOM_SIZES=$(ABENCH_SIZES) 	  cargo run --release --example ab_u16 --features "bench ab"

# The 16-bit family (u16 and i16): the full bench of each type (bench-type,
# cold), generic build then native build (-C target-cpu=native), each in its
# own files (bench-type-u16-..., bench-type-u16-native-...); two builds of
# the test binary (several minutes each).
bench-16: ## u16 and i16, generic then native: 4 bench-type runs (long; one file each)
	@$(MAKE) --no-print-directory bench-type T=u16 NATIVE=0
	@$(MAKE) --no-print-directory bench-type T=i16 NATIVE=0
	@$(MAKE) --no-print-directory bench-type T=u16 NATIVE=1
	@$(MAKE) --no-print-directory bench-type T=i16 NATIVE=1

# A borrowed machine for one hour at most: u16 and i16 up to 10M keys (no
# 30M), median of 7, one build (generic, or native with the -native target);
# ~30-40 min with the build on an Apple M3 Pro.
BENCH16_SHORT = BENCH_SIZES=$(BENCH_SIZES_RECORDS) BENCH_REPS=7 PLACE_NAME=-short
bench-16-short: ## u16 and i16 up to 10M, median of 7, generic build (< 1 h; borrowed machines)
	@$(MAKE) --no-print-directory bench-type T=u16 NATIVE=0 $(BENCH16_SHORT)
	@$(MAKE) --no-print-directory bench-type T=i16 NATIVE=0 $(BENCH16_SHORT)
	@echo "done: send the bench-type-u16-short-* and bench-type-i16-short-* files (.csv and .txt)"

bench-16-short-native: ## bench-16-short with -C target-cpu=native (< 1 h; borrowed machines)
	@$(MAKE) --no-print-directory bench-type T=u16 NATIVE=1 $(BENCH16_SHORT)
	@$(MAKE) --no-print-directory bench-type T=i16 NATIVE=1 $(BENCH16_SHORT)
	@echo "done: send the bench-type-u16-native-short-* and bench-type-i16-native-short-* files (.csv and .txt)"

# Simulated i16 bench (examples/ab_i16.rs, cold, median of 11): the generic
# chain against std, turbosort and blazing, written to
# bench_work/<host>/abench-cold-i16-<host>-<UTC date>.csv for the viewer.
abench-i16: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
abench-i16: ## simulated i16 bench, cold, ~15 min (for the bench viewer)
	@mkdir -p bench_work/$(ABENCH_HOST)
	CHECK=1 COLD=1 TOTAL=12000000 REPS=11 CPU="$(ABENCH_CPU) (ab_i16 example, cold)" 	  CSV=bench_work/$(ABENCH_HOST)/abench-cold-i16-$(ABENCH_HOST)-$$(date -u +%Y-%m-%d-%H%M).csv 	  SORTS=gen,std,turbo DOM_SIZES=$(ABENCH_SIZES) cargo run --release --example ab_i16 --features "bench ab"

# A/B of our own variants over several code placements of the same source
# (loops aligned on 16, 32 or 64 bytes, 1 or 16 codegen units): the
# placement alone moves some cells by 5-35 % (Haswell's uop cache, Zen 2),
# so a variant counts only where it wins in every placement
# (tools/ab_align.py). EX=ab_u16 or ab_i16; SORTS (`v2,copy`: the library
# against the A/B copy of src/v2/devour/ab_copy.rs) and DOM_SIZES as for the
# example; PLACEMENTS="align:units ...". Cold, median of 11 each.
EX ?= ab_u16
PLACEMENTS ?= 16:16 32:16 64:16 16:1 32:1 64:1
abench-align: ## A/B over 6 code placements (EX, SORTS=v2,copy, DOM_SIZES): wins must hold in all
	@mkdir -p bench_work/$(ABENCH_HOST)
	@for p in $(PLACEMENTS); do a=$${p%%:*}; u=$${p##*:}; 	  CARGO_TARGET_DIR=target/place-$$a-$$u CARGO_PROFILE_RELEASE_CODEGEN_UNITS=$$u 	  RUSTFLAGS="$(BENCH_RUSTFLAGS) -C llvm-args=-align-loops=$$a" 	    cargo build -q --release --example $(EX) --features "bench ab" || exit 1; 	  COLD=1 TOTAL=12000000 REPS=11 ./target/place-$$a-$$u/release/examples/$(EX) 	    > bench_work/$(ABENCH_HOST)/abench-place-$$a-$$u.txt || exit 1; 	done
	python3 tools/ab_align.py $(foreach p,$(PLACEMENTS),bench_work/$(ABENCH_HOST)/abench-place-$(subst :,-,$(p)).txt)

gather-check: export RUSTFLAGS = $(BENCH_RUSTFLAGS) -C target-cpu=native
gather-check: ## native A/B of the 16-bit copy without vector gathers (Core Ultra): two files in bench_work/<host>/
	@mkdir -p bench_work/$(ABENCH_HOST)
	CARGO_TARGET_DIR=target/native 	  cargo build -q --release --example ab_u16 --example ab_i16 --features "bench ab"
	CHECK=1 SORTS=v2,copy DOM_SIZES=1000,10000,100000,1000000 TOTAL=4000000 	  ./target/native/release/examples/ab_u16 > bench_work/$(ABENCH_HOST)/gather-native-u16.txt
	CHECK=1 SORTS=gen,copy DOM_SIZES=1000,10000,100000,1000000 TOTAL=4000000 	  ./target/native/release/examples/ab_i16 > bench_work/$(ABENCH_HOST)/gather-native-i16.txt

ABENCH32_SIZES ?= 500,1000,5000,10000,20000,50000,100000,200000,500000,1000000
abench-32: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
abench-32: ## cold A/B (fresh pages for every buffer) of the 32-bit chain copy against 2.1, std, blazing, turbosort, glidesort (u32, i32; viewer CSV in bench_work/<host>/; NATIVE=1)
	@mkdir -p bench_work/$(ABENCH_HOST)
	cargo build -q --release --example ab_u32 --features "bench ab"
	@d=$$(date -u +%Y-%m-%d-%H%M); for t in u32 i32; do 	  FRESH=1 COLD=1 TOTAL=12000000 REPS=7 T=$$t CPU="$(ABENCH_CPU) (ab_u32 example, cold, fresh pages$(if $(filter 1,$(NATIVE)), native))" 	  CSV=bench_work/$(ABENCH_HOST)/abench-cold-$$t$(if $(filter 1,$(NATIVE)),-native)-$(ABENCH_HOST)-$$d.csv 	  DOM_SIZES=$(ABENCH32_SIZES) SORTS=copy:512,v2,std,blz,turbo,glide 	  ./target/release/examples/ab_u32 > bench_work/$(ABENCH_HOST)/abench-cold-$$t.txt || exit 1; done

gros-intel: ## the Core Ultra's run (~1h45): abench-32 generic, then native; files in bench_work/<host>/
	@$(MAKE) --no-print-directory abench-32
	@$(MAKE) --no-print-directory abench-32 NATIVE=1
	@echo "done: send bench_work/$(ABENCH_HOST)/abench-cold-*.csv"

bench-records: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench-records: export BENCH_NAME = bench-records$(if $(T),-$(T))
bench-records: export BENCH_PARTS = records
bench-records: export BENCH_TYPES = $(T)
bench-records: export BENCH_SORTS = best
bench-records: export BENCH_SIZES = $(BENCH_SIZES_RECORDS)
bench-records: export BENCH_MIN_ARRAYS = 1
bench-records: export BENCH_MIN_SORTS = $(BENCH_MIN_SORTS_TYPE)
bench-records: ## structs sorted by key, 500 to 10M (T=u64: keys of one type)
	@$(BENCH_RUN)

bench-big: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench-big: export BENCH_NAME = bench-big
bench-big: export BENCH_PARTS = int,float,wide
bench-big: export BENCH_TYPES = $(BENCH_TYPES_BIG)
bench-big: export BENCH_SORTS = best
bench-big: export BENCH_SIZES = $(BENCH_SIZES_BIG)
bench-big: export BENCH_MAX = 10000000000
bench-big: export BENCH_MIN_ARRAYS = 1
bench-big: export BENCH_REPS = 1
bench-big: ## main key types, 30M keys up to what the RAM holds (hours)
	@$(BENCH_RUN)

bench-big-type: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench-big-type: export BENCH_NAME = bench-big-type-$(T)
bench-big-type: export BENCH_PARTS = int,float,wide
bench-big-type: export BENCH_TYPES = $(T)
bench-big-type: export BENCH_SORTS = best
bench-big-type: export BENCH_SIZES = $(BENCH_SIZES_BIG)
bench-big-type: export BENCH_MAX = 10000000000
bench-big-type: export BENCH_MIN_ARRAYS = 1
bench-big-type: export BENCH_REPS = 1
bench-big-type: ## one key type, 30M keys up to what the RAM holds: make bench-big-type T=u64
	@test -n "$(T)" || (echo "usage: make bench-big-type T=u64" && exit 1)
	@$(BENCH_RUN)

# ---------------------------------------------------------------------------
# Development: a change goes trace-ab (which cells it moves) -> sentinel
# (time those cells and controls) -> tests -> trace-write (see README).

TRACE_RUN = cargo test --release --lib --features "voracious_multithread v2_trace" tests::tools::trace -- --ignored --nocapture --test-threads=1

trace: ## decision trace against the reference of this CPU (no timing; TRACE_MAX=10M: faster)
	@$(TRACE_RUN)

trace-ab: export TRACE_AB = 1
trace-ab: ## cells moved by the rule under test (src/v2/dev/ab.rs), listed in traces/moved.txt
	@$(TRACE_RUN)

trace-write: export TRACE_WRITE = 1
trace-write: ## write the reference trace of this CPU (new machine, or after cells moved)
	@$(TRACE_RUN)
	@git status --short --untracked-files=all traces

sentinel: export RUSTFLAGS = $(BENCH_RUSTFLAGS) -C target-cpu=native
sentinel: ## rule under test off / on, on the moved cells and controls (minutes)
	@cargo test --release --lib $(FEATURES) tests::tools::sentinel -- --ignored --nocapture --test-threads=1

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: test test-miri test-fuzzy build check lint fmt doc clean bench bench-type bench-records bench-big bench-big-type trace trace-ab trace-write sentinel help
