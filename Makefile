test: ## run test
	@cargo test --release

check: ## check code
	@cargo check

clean: ## clean build files
	@cargo clean

lint: ## lint code
	@rustup component add clippy
	@cargo clippy -- -A clippy::comparison_chain -A clippy::unused_unit

fmt: ## format code
	@rustup component add rustfmt
	@rustup component add rustfmt --toolchain nightly
	@cargo +nightly fmt

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: check clean fmt help lint test
