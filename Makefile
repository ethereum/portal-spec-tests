.DEFAULT_GOAL := help

##@ Help
.PHONY: help
help: # Display this help.
	@awk 'BEGIN {FS = ":.*#"; printf "Usage:\n  make \033[34m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?#/ { printf "  \033[34m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) }' $(MAKEFILE_LIST)

##@ Others
.PHONY: clean
clean: # Run `cargo clean`.
	cargo clean

.PHONY: lint
lint: # Run `clippy` and `rustfmt`.
	cargo +nightly fmt --all
	cargo clippy --all --all-targets --all-features --no-deps -- --deny warnings