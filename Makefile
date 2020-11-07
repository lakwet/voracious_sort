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

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: test check clean lint doc fmt build-dev build-dev-single build-release build-release-single help
