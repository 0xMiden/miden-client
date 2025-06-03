.DEFAULT_GOAL := help

.PHONY: help
help: ## Show description of all commands
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# --- Variables -----------------------------------------------------------------------------------

# Enable file generation in the `src` directory.
# This is used in the build script of the client to generate the node RPC-related code, from the
# protobuf files.
CODEGEN=CODEGEN=1

FEATURES_WEB_CLIENT=--features "testing"
FEATURES_CLIENT=--features "testing, concurrent" --no-default-features
WARNINGS=RUSTDOCFLAGS="-D warnings"

PROVER_DIR="crates/testing/prover"

# --- Linting -------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Run Clippy with configs
	cargo clippy --workspace --exclude miden-client-web --exclude miden-prover-service --all-targets -- -D warnings

.PHONY: clippy-wasm
clippy-wasm: ## Run Clippy for the miden-client-web package
	cargo clippy --package miden-client-web --target wasm32-unknown-unknown --all-targets $(FEATURES_WEB_CLIENT) -- -D warnings

.PHONY: clippy-prover
clippy-prover: ## Run Clippy for the prover package
	cd $(PROVER_DIR) && cargo clippy --all-targets -- -D warnings

.PHONY: fix
fix: ## Run Fix with configs
	cargo +nightly fix --workspace --exclude miden-client-web --exclude miden-prover-service --allow-staged --allow-dirty --all-targets

.PHONY: fix-wasm
fix-wasm: ## Run Fix for the miden-client-web package
	cargo +nightly fix --package miden-client-web --target wasm32-unknown-unknown --allow-staged --allow-dirty --all-targets $(FEATURES_WEB_CLIENT)

.PHONY: fix-prover
fix-prover: ## Run Fix for the prover package
	cd $(PROVER_DIR) && cargo +nightly fix --all-targets --allow-staged --allow-dirty

.PHONY: format
format: ## Run format using nightly toolchain
	cargo +nightly fmt --all && yarn prettier . --write && yarn eslint . --fix

.PHONY: format-check
format-check: ## Run format using nightly toolchain but only in check mode
	cargo +nightly fmt --all --check && yarn prettier . --check && yarn eslint .

.PHONY: lint
lint: format fix clippy fix-wasm clippy-wasm fix-prover clippy-prover ## Run all linting tasks at once (clippy, fixing, formatting)

# --- Documentation --------------------------------------------------------------------------

.PHONY: doc
doc: ## Generate & check rust documentation. You'll need `jq` in order for this to run.
	@cd crates/rust-client && \
	FEATURES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "miden-client") | .features | keys[] | select(. != "web-tonic" and . != "idxdb")' | tr '\n' ',') && \
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features "$$FEATURES" --keep-going --release

.PHONY: book
book: ## Builds the book & serves documentation site
	mdbook serve --open docs

# --- Testing -------------------------------------------------------------------------------------

.PHONY: test
test: ## Run tests
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude miden-prover-service --release --lib $(FEATURES_CLIENT)

.PHONY: test-deps
test-deps: ## Install dependencies for tests
	$(CODEGEN) cargo install cargo-nextest

.PHONY: test-docs
test-docs: ## Run documentation tests
	$(CODEGEN) cargo test --doc $(FEATURES_CLIENT)

# --- Integration testing -------------------------------------------------------------------------

.PHONY: start-node
start-node: ## Start the testing node server
	RUST_LOG=info cargo run --release --package node-builder --locked

.PHONY: start-node-background
start-node-background: ## Start the testing node server in background
	./scripts/start-binary-bg.sh node-builder

.PHONY: stop-node
stop-node: ## Stop the testing node server
	-pkill -f "node-builder"
	sleep 1

.PHONY: integration-test
integration-test: ## Run integration tests
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude miden-prover-service --release --test=integration

.PHONY: integration-test-web-client
integration-test-web-client: ## Run integration tests for the web client
	$(CODEGEN) cd ./crates/web-client && npm run test:clean

.PHONY: integration-test-remote-prover-web-client
integration-test-remote-prover-web-client: ## Run integration tests for the web client with remote prover
	$(CODEGEN) cd ./crates/web-client && npm run test:remote_prover

.PHONY: integration-test-full
integration-test-full: ## Run the integration test binary with ignored tests included
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude miden-prover-service --release --test=integration
	cargo nextest run --workspace --exclude miden-client-web --exclude miden-prover-service --release --test=integration --run-ignored ignored-only -- test_import_genesis_accounts_can_be_used_for_transactions

.PHONY: start-prover
start-prover: ## Start the prover service
	cd $(PROVER_DIR) && RUST_LOG=info cargo run --release --locked

.PHONY: start-prover-background
start-prover-background: ## Start the prover service in background
	cd $(PROVER_DIR) && ../../../scripts/start-binary-bg.sh miden-prover-service

.PHONY: stop-prover
stop-prover: ## Stop prover process
	-pkill -f "miden-prover-service"
	sleep 1

# --- Installing ----------------------------------------------------------------------------------

install: ## Install the CLI binary
	cargo install --path bin/miden-cli --locked

# --- Building ------------------------------------------------------------------------------------

build: ## Build the CLI binary and client library in release mode
	CODEGEN=1 cargo build --workspace --exclude miden-client-web --exclude miden-prover-service --release

build-wasm: ## Build the client library for wasm32
	CODEGEN=1 cargo build --package miden-client-web --target wasm32-unknown-unknown $(FEATURES_WEB_CLIENT)

build-prover: ## Build the prover package
	cd $(PROVER_DIR) && cargo build --release

# --- Check ---------------------------------------------------------------------------------------

.PHONY: check
check: ## Build the CLI binary and client library in release mode
	cargo check --workspace --exclude miden-client-web --exclude miden-prover-service --release

.PHONY: check-wasm
check-wasm: ## Build the client library for wasm32
	cargo check --package miden-client-web --target wasm32-unknown-unknown $(FEATURES_WEB_CLIENT)
