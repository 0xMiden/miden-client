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
FEATURES_CLIENT=--features "testing, std"
WARNINGS=RUSTDOCFLAGS="-D warnings"

PROVER_DIR="crates/testing/prover"
WEB_CLIENT_DIR=crates/web-client
RUST_CLIENT_DIR=crates/rust-client

# --- Linting -------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Run Clippy with configs. We need two separate commands because the `testing-remote-prover` cannot be built along with the rest of the workspace. This is because they use different versions of the `miden-tx` crate which aren't compatible with each other.
	cargo clippy --workspace --exclude miden-client-web --exclude testing-remote-prover --all-targets -- -D warnings
	cargo clippy --package testing-remote-prover --all-targets -- -D warnings

.PHONY: clippy-wasm
clippy-wasm: rust-client-ts-build ## Run Clippy for the miden-client-web package
	cargo clippy --package miden-client-web --target wasm32-unknown-unknown --all-targets $(FEATURES_WEB_CLIENT) -- -D warnings

.PHONY: fix
fix: ## Run Fix with configs. We need two separate commands because the `testing-remote-prover` cannot be built along with the rest of the workspace. This is because they use different versions of the `miden-tx` crate which aren't compatible with each other.
	cargo +nightly fix --workspace --exclude miden-client-web --exclude testing-remote-prover --allow-staged --allow-dirty --all-targets
	cargo +nightly fix --package testing-remote-prover --all-targets --allow-staged --allow-dirty

.PHONY: fix-wasm
fix-wasm: ## Run Fix for the miden-client-web package
	cargo +nightly fix --package miden-client-web --target wasm32-unknown-unknown --allow-staged --allow-dirty --all-targets $(FEATURES_WEB_CLIENT)

.PHONY: format
format: ## Run format using nightly toolchain
	cargo +nightly fmt --all && yarn prettier . --write && yarn eslint . --fix

.PHONY: format-check
format-check: ## Run format using nightly toolchain but only in check mode
	cargo +nightly fmt --all --check && yarn prettier . --check && yarn eslint .

.PHONY: lint
lint: format fix toml clippy fix-wasm clippy-wasm typos-check ## Run all linting tasks at once (clippy, fixing, formatting, typos)

.PHONY: toml
toml: ## Runs Format for all TOML files
	taplo fmt

.PHONY: toml-check
toml-check: ## Runs Format for all TOML files but only in check mode
	taplo fmt --check --verbose

.PHONY: typos-check
typos-check: ## Run typos to check for spelling mistakes
	@typos --config ./.typos.toml

# --- Documentation -------------------------------------------------------------------------------

.PHONY: doc
doc: ## Generate & check rust documentation. You'll need `jq` in order for this to run.
	@cd crates/rust-client && \
	FEATURES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "miden-client") | .features | keys[] | select(. != "web-tonic" and . != "idxdb")' | tr '\n' ',') && \
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features "$$FEATURES" --keep-going --release

.PHONY: book
book: ## Builds the book & serves documentation site
	mdbook serve --open docs

.PHONY: typedoc
typedoc: rust-client-ts-build ## Generate web client package documentation.
	@cd crates/web-client && \
	npm run build-dev && \
	yarn typedoc

# --- Testing -------------------------------------------------------------------------------------

.PHONY: test
test: ## Run tests
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude testing-remote-prover --release --lib $(FEATURES_CLIENT)

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
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude testing-remote-prover --release --test=integration

.PHONY: integration-test-web-client
integration-test-web-client: ## Run integration tests for the web client
	$(CODEGEN) cd ./crates/web-client && npm run test:clean

.PHONY: integration-test-remote-prover-web-client
integration-test-remote-prover-web-client: ## Run integration tests for the web client with remote prover
	$(CODEGEN) cd ./crates/web-client && npm run test:remote_prover

.PHONY: integration-test-full
integration-test-full: ## Run the integration test binary with ignored tests included
	$(CODEGEN) cargo nextest run --workspace --exclude miden-client-web --exclude testing-remote-prover --release --test=integration
	cargo nextest run --workspace --exclude miden-client-web --exclude testing-remote-prover --release --test=integration --run-ignored ignored-only -- import_genesis_accounts_can_be_used_for_transactions

.PHONY: start-prover
start-prover: ## Start the remote prover
	cd $(PROVER_DIR) && RUST_LOG=info cargo run --release --locked

.PHONY: start-prover-background
start-prover-background: ## Start the remote prover in background
	cd $(PROVER_DIR) && ../../../scripts/start-binary-bg.sh testing-remote-prover

.PHONY: stop-prover
stop-prover: ## Stop prover process
	-pkill -f "testing-remote-prover"
	sleep 1

# --- Installing ----------------------------------------------------------------------------------

install: ## Install the CLI binary
	cargo install --path bin/miden-cli --locked

install-tests: ## Install the tests binary
	cargo install --path bin/integration-tests --locked

# --- Building ------------------------------------------------------------------------------------

build: ## Build the CLI binary and client library in release mode
	CODEGEN=1 cargo build --workspace --exclude miden-client-web --exclude testing-remote-prover --release
	cargo build --package testing-remote-prover --release

build-wasm: rust-client-ts-build ## Build the client library for wasm32
	CODEGEN=1 cargo build --package miden-client-web --target wasm32-unknown-unknown $(FEATURES_WEB_CLIENT)

.PHONY: rust-client-ts-build
rust-client-ts-build:
	cd $(RUST_CLIENT_DIR)/src/store/web_store && yarn && yarn tsc --build --force

# --- Check ---------------------------------------------------------------------------------------

.PHONY: check
check: ## Build the CLI binary and client library in release mode
	cargo check --workspace --exclude miden-client-web --exclude testing-remote-prover --release

.PHONY: check-wasm
check-wasm: ## Build the client library for wasm32
	cargo check --package miden-client-web --target wasm32-unknown-unknown $(FEATURES_WEB_CLIENT)

## --- Setup --------------------------------------------------------------------------------------

.PHONY: check-tools
check-tools: ## Checks if development tools are installed
	@echo "Checking development tools..."
	@command -v mdbook        >/dev/null 2>&1 && echo "[OK] mdbook is installed"        || echo "[MISSING] mdbook       (make install-tools)"
	@command -v typos         >/dev/null 2>&1 && echo "[OK] typos is installed"         || echo "[MISSING] typos        (make install-tools)"
	@command -v cargo nextest >/dev/null 2>&1 && echo "[OK] cargo-nextest is installed" || echo "[MISSING] cargo-nextest(make install-tools)"
	@command -v taplo         >/dev/null 2>&1 && echo "[OK] taplo is installed"         || echo "[MISSING] taplo        (make install-tools)"
	@command -v yarn          >/dev/null 2>&1 && echo "[OK] yarn is installed"          || echo "[MISSING] yarn         (make install-tools)"

.PHONY: install-tools
install-tools: ## Installs Rust + Node tools required by the Makefile
	@echo "Installing development tools..."
	# Rust-related
	cargo install mdbook --locked
	cargo install typos-cli --locked
	cargo install cargo-nextest --locked
	cargo install taplo-cli --locked
	# Web-related
	command -v yarn >/dev/null 2>&1 || npm install -g yarn
	yarn --cwd $(WEB_CLIENT_DIR) --silent  # installs prettier, eslint, typedoc, etc.
	yarn --cwd $(RUST_CLIENT_DIR)/src/store/web_store --silent
	yarn --silent
	yarn
	@echo "Development tools installation complete!"
