test: ## run test
	@cargo test --release --features "voracious_multithread"

check: ## check code
	@cargo check --features "voracious_multithread"

clean: ## clean build files
	@cargo clean

lint: ## lint code
	@rustup component add clippy
	@cargo clippy --lib --tests --features "voracious_multithread v2_trace"

doc: ## build doc and open it in the browser
	@cargo doc --open

fmt: ## format code
	@rustup component add rustfmt
	@rustup component add rustfmt --toolchain nightly
	@cargo +nightly fmt

build-dev: ## build in dev mode
	@cargo build --features "voracious_multithread"

build-dev-single: ## build in dev mode without multithread sort
	@cargo build

build-release: ## build in release mode
	@cargo build --release --features "voracious_multithread"

build-release-single: ## build in release mode without multithread sort
	@cargo build --release

# Usage: make bench speed_test_u64 (any test name or filter, e.g. speed_dedicated)
# The speed modules must be enabled in src/tests/bench/mod.rs (see README).
BENCH_ARGS := $(filter-out bench,$(MAKECMDGOALS))
ifeq ($(OS),Windows_NT)
# Avoid the dlltool dependency of getrandom on the windows-gnu toolchain.
BENCH_RUSTFLAGS ?= --cfg getrandom_backend="rdrand" -C target-feature=+rdrand
endif

bench: export RUSTFLAGS = $(BENCH_RUSTFLAGS)
bench: ## run a speed test with its output: make bench <test_name>
	@cargo test --release --features "voracious_multithread" $(BENCH_ARGS) -- --nocapture --test-threads=1

# Turn the extra goals given to "make bench" into no-ops.
ifeq (bench,$(firstword $(MAKECMDGOALS)))
$(BENCH_ARGS):
	@:
endif

# CPU benchmark of the v2 sorts against the other sorts: writes one file,
# bench-cpu-<host>-<date>.txt, at the repository root (about a night):
# make benchcpu
# Compiled for the CPU of the machine (every sort alike).
BENCH_CPU_RUSTFLAGS = $(BENCH_RUSTFLAGS) -C target-cpu=native

benchcpu: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchcpu: ## CPU benchmark (one output file bench-cpu-*.txt, about a night)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchcpu-quick: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchcpu-quick: export BENCH_MAX = 1000000
benchcpu-quick: export BENCH_REPS = 1
benchcpu-quick: ## short CPU benchmark (up to 1M keys, one repetition): checks the setup
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchtune: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchtune: export BENCH_PARTS = tune
benchtune: export BENCH_MAX = 10000000
benchtune: ## CPU tuning: each cache-derived decision alone, 10k to 10M (a few hours)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchfix: export RUSTFLAGS =
benchfix: export BENCH_PARTS = int,float
benchfix: export BENCH_TYPES = u32,i32,u64,i64,f32,f64
benchfix: export BENCH_SIZES = 10000,100000,1000000,10000000
benchfix: export BENCH_SORTS = std,v1,glidesort,blazing
benchfix: export BENCH_MIN_ARRAYS = 1
benchfix: ## short check of the merge fixes: keys against std, v1, glidesort, blazing, generic build (~15 min)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchfix-native: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchfix-native: export BENCH_PARTS = int,float
benchfix-native: export BENCH_TYPES = u32,i32,u64,i64,f32,f64
benchfix-native: export BENCH_SIZES = 10000,100000,1000000,10000000
benchfix-native: export BENCH_SORTS = std,v1,glidesort,blazing
benchfix-native: export BENCH_MIN_ARRAYS = 1
benchfix-native: ## the same with target-cpu=native (~15 min)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchbilan: export RUSTFLAGS =
benchbilan: export BENCH_PARTS = int,float,wide,records
benchbilan: export BENCH_TYPES = u8,i8,u16,i16,u32,i32,u64,i64,u128,i128,f32,f64,u32 8B,u64 16B
benchbilan: export BENCH_SIZES = 1000,10000,50000,100000,1000000,5000000,10000000
benchbilan: export BENCH_SORTS = std,std unstable,std cached,v1,glidesort,blazing,radsort,rdst,turbosort
benchbilan: export BENCH_MIN_ARRAYS = 1
benchbilan: ## every type against every sort, 1k to 10M, generic build: the list of losses (1-3 h)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

# Benchmarks by type (src/tests/bench): v2, v1, std (stable or unstable as
# the case) and the best rivals of the type: two, a third when it beats
# the v2 on its own cases (bench_cpu: best_rivals),
# 500 to 10M (14 sizes), generic build, one output file bench-cpu-*.txt.
#   make bench-type T=u32      one key type (u8 ... i128, f32, f64, char)
#   make bench-keys            every key type
#   make bench-records         every record; T=u64: the records of u64 keys
BENCH_BY_TYPE_SIZES = 500,1000,5000,10000,20000,50000,80000,100000,200000,500000,1000000,2000000,5000000,10000000
BENCH_BY_TYPE_SORTS = best

bench-type: export RUSTFLAGS =
bench-type: export BENCH_PARTS = int,float,wide
bench-type: export BENCH_TYPES = $(T)
bench-type: export BENCH_SIZES = $(BENCH_BY_TYPE_SIZES)
bench-type: export BENCH_SORTS = $(BENCH_BY_TYPE_SORTS)
bench-type: export BENCH_MIN_ARRAYS = 1
bench-type: ## one key type against v1 and the rivals: make bench-type T=u32
	@test -n "$(T)" || (echo "usage: make bench-type T=u32" && exit 1)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

bench-keys: export RUSTFLAGS =
bench-keys: export BENCH_PARTS = int,float,wide
bench-keys: export BENCH_SIZES = $(BENCH_BY_TYPE_SIZES)
bench-keys: export BENCH_SORTS = $(BENCH_BY_TYPE_SORTS)
bench-keys: export BENCH_MIN_ARRAYS = 1
bench-keys: ## every key type against v1 and the rivals
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

bench-records: export RUSTFLAGS =
bench-records: export BENCH_PARTS = records
bench-records: export BENCH_TYPES = $(T)
bench-records: export BENCH_SIZES = $(BENCH_BY_TYPE_SIZES)
bench-records: export BENCH_SORTS = $(BENCH_BY_TYPE_SORTS)
bench-records: export BENCH_MIN_ARRAYS = 1
bench-records: ## every record against v1 and the rivals (T=u64: records of u64 keys only)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchbig: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchbig: export BENCH_PARTS = big
benchbig: export BENCH_MAX = 1000000000
benchbig: export BENCH_REPS ?= 1
benchbig: ## very large arrays, 30M to 1G keys: rollercoaster against LSD (a few hours, memory: 3 arrays)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchlsd32: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
benchlsd32: export BENCH_PARTS = big
benchlsd32: export BENCH_MAX = 1000000000
benchlsd32: export BENCH_TYPES = u32,f32
benchlsd32: export BENCH_BIG_SIZES = 1000000,3000000,10000000,30000000,100000000
benchlsd32: export BENCH_REPS = 3
benchlsd32: ## u32 / f32, 1M to 100M: current rule against LSD and rollercoaster forced (~15 min)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchbigrivals: export RUSTFLAGS =
benchbigrivals: export BENCH_PARTS = int,float
benchbigrivals: export BENCH_TYPES = u32,f32
benchbigrivals: export BENCH_SIZES = 30000000,100000000
benchbigrivals: export BENCH_SORTS = v1,glidesort,blazing,radsort
benchbigrivals: export BENCH_MIN_ARRAYS = 1
benchbigrivals: ## u32 / f32 at 30M and 100M against v1, glidesort, blazing, radsort (~2 h)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

benchnuit: benchlsd32 benchbigrivals ## the two above in a row (~2-3 h)

# Method (see src/tests/tools/trace.rs, src/tests/tools/sentinel.rs): a change goes
# trace-ab (which cells move) -> sentinel (time those cells and controls).
trace: ## decision trace against the reference of this CPU (no timing; ~10 min, TRACE_MAX=10M ~1 min)
	@cargo test --release --lib --features "voracious_multithread v2_trace" tests::tools::trace -- --ignored --nocapture --test-threads=1

trace-ab: export TRACE_AB = 1
trace-ab: ## cells moved by the rule under test (v2::dev::ab), list in traces/moved.txt
	@cargo test --release --lib --features "voracious_multithread v2_trace" tests::tools::trace -- --ignored --nocapture --test-threads=1

trace-write: export TRACE_WRITE = 1
trace-write: ## write the reference trace of this CPU (after a commit that moves cells)
	@cargo test --release --lib --features "voracious_multithread v2_trace" tests::tools::trace -- --ignored --nocapture --test-threads=1

trace-ref: export TRACE_WRITE = 1
trace-ref: ## new machine: update dev, write its reference trace, show the file to send (~20-30 min, no timing)
	git checkout dev
	git pull
	@cargo test --release --lib --features "voracious_multithread v2_trace" tests::tools::trace -- --ignored --nocapture --test-threads=1
	@echo "File to send:"
	@git status --short --untracked-files=all traces

sentinel: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
sentinel: ## rule under test off / on on the moved cells + controls (minutes)
	@cargo test --release --lib --features "voracious_multithread" tests::tools::sentinel -- --ignored --nocapture --test-threads=1

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: test bench bench-type bench-keys bench-records benchcpu benchcpu-quick benchfix benchfix-native benchbilan benchlsd32 benchbigrivals benchnuit trace trace-ab trace-write trace-ref sentinel check clean lint doc fmt build-dev build-dev-single build-release build-release-single help
