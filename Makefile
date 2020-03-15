test: ## run test
	cargo test --release

check: ## check code
	cargo check

clean: ## clean build files
	cargo clean

lint: ## lint code
	@rustup component add clippy
	@cargo clippy -- -D warnings -A clippy::many-single-char-names -A clippy::match_wild_err_arm -A clippy::too-many-arguments -A clippy::redundant_closure -A clippy::comparison-chain

fmt: ## format code
	cargo fmt

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
.PHONY: check clean fmt help lint test
