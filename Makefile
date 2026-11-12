test: ## run test
	@cargo test --release --features "voracious_multithread"

check: ## check code
	@cargo check --features "voracious_multithread"

clean: ## clean build files
	@cargo clean

lint: ## lint code
	@rustup component add clippy
	@cargo clippy -- -A clippy::comparison_chain -A clippy::unused_unit

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
# The speed modules must be enabled in src/tests/mod.rs (see README).
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
# bench-cpu-<host>-<date>.txt, at the repository root (several hours).
# Compiled for the CPU of the machine (every sort alike).
BENCH_CPU_RUSTFLAGS = $(BENCH_RUSTFLAGS) -C target-cpu=native

bench-cpu: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
bench-cpu: ## CPU benchmark (one output file bench-cpu-*.txt, several hours)
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

bench-cpu-quick: export RUSTFLAGS = $(BENCH_CPU_RUSTFLAGS)
bench-cpu-quick: export BENCH_MAX = 1000000
bench-cpu-quick: export BENCH_REPS = 1
bench-cpu-quick: ## short CPU benchmark (up to 1M keys, one repetition): checks the setup
	@cargo test --release --lib --features "voracious_multithread" bench_cpu -- --ignored --nocapture --test-threads=1

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: test bench bench-cpu bench-cpu-quick check clean lint doc fmt build-dev build-dev-single build-release build-release-single help
