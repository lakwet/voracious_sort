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
# Avoid the dlltool dependency of getrandom on the windows-gnu toolchain.
BENCH_RUSTFLAGS ?= --cfg getrandom_backend="rdrand" -C target-feature=+rdrand
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

bench-type: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench-type: export BENCH_NAME = bench-type-$(T)
bench-type: export BENCH_PARTS = int,float,wide
bench-type: export BENCH_TYPES = $(T)
bench-type: export BENCH_SORTS = best
bench-type: export BENCH_SIZES = $(BENCH_SIZES_TYPE)
bench-type: export BENCH_MIN_ARRAYS = 1
bench-type: export BENCH_MIN_SORTS = $(BENCH_MIN_SORTS_TYPE)
bench-type: ## one key type, 500 to 30M keys: make bench-type T=u32 (u8 ... i128, f32, f64, char)
	@test -n "$(T)" || (echo "usage: make bench-type T=u32" && exit 1)
	@$(BENCH_RUN)

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
	COLD=1 TOTAL=12000000 REPS=11 CPU="$(ABENCH_CPU) (ab_u16 example, cold)" 	  CSV=bench_work/$(ABENCH_HOST)/abench-cold-u16-$(ABENCH_HOST)-$$(date -u +%Y-%m-%d-%H%M).csv 	  SKIP_MASKS="0:v2,0:std,0:turbosort" DOM_SIZES=$(ABENCH_SIZES) 	  cargo run --release --example ab_u16 --features "bench ab"

# The 16-bit family (u16 and i16), one after the other: the full bench of
# each type (bench-type, the test binary: its build takes several minutes),
# each type in its own files (bench-type-u16-..., bench-type-i16-...).
bench-16: ## u16 then i16: make bench-type for both (long; one file per type)
	@$(MAKE) --no-print-directory bench-type T=u16
	@$(MAKE) --no-print-directory bench-type T=i16

# Simulated i16 bench (examples/ab_i16.rs, cold, median of 11): the generic
# chain against std, turbosort and blazing, written to
# bench_work/<host>/abench-cold-i16-<host>-<UTC date>.csv for the viewer.
abench-i16: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
abench-i16: ## simulated i16 bench, cold, ~15 min (for the bench viewer)
	@mkdir -p bench_work/$(ABENCH_HOST)
	CHECK=1 COLD=1 TOTAL=12000000 REPS=11 CPU="$(ABENCH_CPU) (ab_i16 example, cold)" 	  CSV=bench_work/$(ABENCH_HOST)/abench-cold-i16-$(ABENCH_HOST)-$$(date -u +%Y-%m-%d-%H%M).csv 	  SORTS=gen,std,turbo DOM_SIZES=$(ABENCH_SIZES) cargo run --release --example ab_i16 --features "bench ab"

# The per-architecture choices of the u16 sort flipped one at a time (does a
# choice follow the instruction set or the cache size?): window before the
# domain count from 262 144 keys, 16-bit LSD counters, high pass one key at a
# time, order scan by blocks of 16, direct Pipe organ merge. Cold; the table
# goes to bench_work/<host>/abench-arch-<host>-<UTC date>.txt.
ABENCH_ARCH_SIZES = 1000,5000,20000,100000,200000,500000,1000000,2000000,5000000
abench-arch: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
abench-arch: ## u16 per-architecture choices flipped one at a time, cold (~15 min)
	@mkdir -p bench_work/$(ABENCH_HOST)
	COLD=1 TOTAL=12000000 REPS=11 	  SKIP_MASKS="0:new,576460752303423488:win,281474976710656:c16,4194304:hiseq,1073741824:blk16,524288:pipe" 	  DOM_SIZES=$(ABENCH_ARCH_SIZES) cargo run --release --example ab_u16 --features "bench ab" 	  | tee bench_work/$(ABENCH_HOST)/abench-arch-$(ABENCH_HOST)-$$(date -u +%Y-%m-%d-%H%M).txt

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
