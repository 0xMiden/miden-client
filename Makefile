.DEFAULT_GOAL := help

.PHONY: help
help: ## Show description of all commands
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# --- Variables -----------------------------------------------------------------------------------

# Enable file generation in the `src` directory.
# This is used in the build script of the client to generate the node RPC-related code, from the
# protobuf files.
CODEGEN=CODEGEN=1

FEATURES_CLIENT=--features "std"
WARNINGS=RUSTDOCFLAGS="-D warnings"

PROVER_DIR="crates/testing/prover"
WEB_CLIENT_DIR=crates/web-client
RUST_CLIENT_DIR=crates/rust-client

EXCLUDE_WASM_PACKAGES=--exclude miden-client-web --exclude miden-idxdb-store
NOTE_TRANSPORT_ENDPOINT=http://127.0.0.1:57292

# --- Linting -------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Run Clippy with configs. We need two separate commands because the `testing-remote-prover` cannot be built along with the rest of the workspace. This is because they use different versions of the `miden-tx` crate which aren't compatible with each other.
	cargo clippy --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --all-targets -- -D warnings
	cargo clippy --package testing-remote-prover --all-targets -- -D warnings

.PHONY: clippy-wasm
clippy-wasm: rust-client-ts-build ## Run Clippy for the wasm packages (web client and idxdb store)
	cargo clippy --package miden-client-web --target wasm32-unknown-unknown --all-targets -- -D warnings
	cargo clippy --package miden-idxdb-store --target wasm32-unknown-unknown --all-targets -- -D warnings

.PHONY: fix
fix: ## Run Fix with configs, building tests with proper features to avoid type split.
	cargo +nightly fix --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --features "testing std" --all-targets --allow-staged --allow-dirty
	cargo +nightly fix --package testing-remote-prover --all-targets --allow-staged --allow-dirty

.PHONY: fix-wasm
fix-wasm: ## Run Fix for the wasm packages (web client and idxdb store)
	cargo +nightly fix --package miden-client-web --target wasm32-unknown-unknown --allow-staged --allow-dirty --all-targets
	cargo +nightly fix --package miden-idxdb-store --target wasm32-unknown-unknown --allow-staged --allow-dirty --all-targets

.PHONY: format
format: ## Run format using nightly toolchain
	cargo +nightly fmt --all && yarn prettier . --write && yarn eslint . --fix

.PHONY: format-check
format-check: ## Run format using nightly toolchain but only in check mode
	cargo +nightly fmt --all --check && yarn prettier . --check && yarn eslint .

.PHONY: lint
lint: format fix toml clippy fix-wasm clippy-wasm typos-check rust-client-ts-lint ## Run all linting tasks at once (clippy, fixing, formatting, typos)

.PHONY: toml
toml: ## Runs Format for all TOML files
	taplo fmt

.PHONY: toml-check
toml-check: ## Runs Format for all TOML files but only in check mode
	taplo fmt --check --verbose

.PHONY: typos-check
typos-check: ## Run typos to check for spelling mistakes
	@typos --config ./.typos.toml

.PHONY: rust-client-ts-lint
rust-client-ts-lint:
	cd crates/idxdb-store/src && yarn && yarn lint

.PHONY: react-sdk-lint
react-sdk-lint: ## Run lint for the React SDK
	cd packages/react-sdk && yarn && yarn lint

# --- Documentation -------------------------------------------------------------------------------

.PHONY: doc
doc: ## Generate & check rust documentation. Ensure you have the nightly toolchain installed.
	@cd crates/rust-client && \
	RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo +nightly doc --lib --no-deps --all-features --keep-going --release

doc-open: ## Generate & open rust documentation in browser. Ensure you have the nightly toolchain installed.
	@cd crates/rust-client && \
	RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo +nightly doc --lib --no-deps --all-features --keep-going --release --open

.PHONY: serve-docs
serve-docs: ## Serves the docs
	cd docs/external && npm run start:dev

.PHONY: typedoc
typedoc: rust-client-ts-build ## Generate web client package documentation.
	@cd crates/web-client && \
	npm run build-dev && \
	yarn typedoc

# --- Testing -------------------------------------------------------------------------------------

.PHONY: test
test: ## Run tests
	cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release --lib $(FEATURES_CLIENT)

.PHONY: test-docs
test-docs: ## Run documentation tests
	cargo test --doc $(FEATURES_CLIENT)

.PHONY: test-react-sdk
test-react-sdk: ## Run React SDK unit tests
	cd packages/react-sdk && yarn && yarn test:unit

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

.PHONY: start-note-transport-background
start-note-transport-background: ## Start the note transport service in background
	./scripts/start-note-transport-bg.sh

.PHONY: stop-note-transport
stop-note-transport: ## Stop the note transport service
	./scripts/stop-note-transport.sh

.PHONY: start-note-transport
start-note-transport:
	./scripts/start-note-transport.sh

.PHONY: integration-test
integration-test: ## Run integration tests
	cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release --test=integration

.PHONY: integration-test-web-client
SHARD_PARAMETER ?= ""
integration-test-web-client: ## Run integration tests for the web client (with a chromium browser)
	cd ./crates/web-client && yarn run test:clean -- --project=chromium $(SHARD_PARAMETER)

.PHONY: integration-test-web-client-webkit
integration-test-web-client-webkit: ## Run integration tests for the web client (with webkit)
	cd ./crates/web-client && yarn run test:clean -- --project=webkit

.PHONY: integration-test-remote-prover-web-client
integration-test-remote-prover-web-client: ## Run integration tests for the web client with remote prover
	cd ./crates/web-client && yarn run test:remote_prover -- --project=chromium

.PHONY: integration-test-full
integration-test-full: ## Run the integration test binary with ignored tests included (requires note transport service)
	TEST_MIDEN_NOTE_TRANSPORT_ENDPOINT=$(NOTE_TRANSPORT_ENDPOINT) TEST_WITH_NOTE_TRANSPORT=1 cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release --test=integration
	cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release --test=integration --run-ignored ignored-only -- import_genesis_accounts_can_be_used_for_transactions

.PHONY: test-dev
test-dev: ## Run tests with debug assertions enabled via test-dev profile
	cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --cargo-profile test-dev --lib $(FEATURES_CLIENT)

.PHONY: integration-test-dev
integration-test-dev: ## Run integration tests with debug assertions enabled via test-dev profile
	cargo nextest run --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --cargo-profile test-dev --test=integration

.PHONY: integration-test-binary
integration-test-binary: ## Run the integration tests using the standalone binary (requires note transport service)
	TEST_MIDEN_NOTE_TRANSPORT_ENDPOINT=$(NOTE_TRANSPORT_ENDPOINT) TEST_WITH_NOTE_TRANSPORT=1 cargo run --package miden-client-integration-tests --release --locked

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

build: ## Build the CLI binary, client library and tests binary in release mode
	CODEGEN=1 cargo build --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release
	cargo build --package testing-remote-prover --release --locked
	cargo build --package miden-client-integration-tests --release --locked

build-wasm: rust-client-ts-build ## Build the wasm packages (web client and idxdb store)
	CODEGEN=1 cargo build --package miden-client-web --target wasm32-unknown-unknown --locked
	cargo build --package miden-idxdb-store --target wasm32-unknown-unknown --locked

.PHONY: rust-client-ts-build
rust-client-ts-build:
	cd crates/idxdb-store/src && yarn && yarn build

.PHONY: build-react-sdk
build-react-sdk: ## Build the React SDK package
	cd crates/web-client && yarn && yarn build
	cd packages/react-sdk && yarn && yarn build

# --- Check ---------------------------------------------------------------------------------------

.PHONY: check
check: ## Build the CLI binary and client library in release mode
	cargo check --workspace $(EXCLUDE_WASM_PACKAGES) --exclude testing-remote-prover --release

.PHONY: check-wasm
check-wasm: ## Check the wasm packages (web client and idxdb store)
	cargo check --package miden-client-web --target wasm32-unknown-unknown
	cargo check --package miden-idxdb-store --target wasm32-unknown-unknown

## --- Setup --------------------------------------------------------------------------------------

.PHONY: check-tools
check-tools: ## Checks if development tools are installed
	@echo "Checking development tools..."
	@command -v mdbook        >/dev/null 2>&1 && echo "[OK] mdbook is installed"        || echo "[MISSING] mdbook       (make install-tools)"
	@command -v typos         >/dev/null 2>&1 && echo "[OK] typos is installed"         || echo "[MISSING] typos        (make install-tools)"
	@command -v cargo nextest >/dev/null 2>&1 && echo "[OK] cargo-nextest is installed" || echo "[MISSING] cargo-nextest(make install-tools)"
	@command -v taplo         >/dev/null 2>&1 && echo "[OK] taplo is installed"         || echo "[MISSING] taplo        (make install-tools)"
	@command -v yarn          >/dev/null 2>&1 && echo "[OK] yarn is installed"          || echo "[MISSING] yarn         (make install-tools)"
	@command -v wasm-opt      >/dev/null 2>&1 && echo "[OK] wasm-opt is installed"      || echo "[MISSING] wasm-opt     (brew install binaryen / apt-get install binaryen)"

.PHONY: install-tools
install-tools: ## Installs Rust + Node tools required by the Makefile
	@echo "Installing development tools..."
	@rustup show active-toolchain >/dev/null 2>&1 || (echo "Rust toolchain not detected. Install rustup + toolchain first." && exit 1)
	@echo "Ensuring wasm32-unknown-unknown target is installed..."
	@rustup target add wasm32-unknown-unknown >/dev/null
	@RUST_TC=$$(rustup show active-toolchain | awk '{print $$1}'); \
		echo "Ensuring required Rust components are installed for $$RUST_TC..."; \
		rustup component add --toolchain $$RUST_TC clippy rust-src rustfmt >/dev/null
	# Rust-related
	cargo install mdbook --locked
	cargo install typos-cli --locked
	cargo install cargo-nextest --locked
	cargo install taplo-cli --locked
	# Binaryen (wasm-opt) – needed by web-client build
	@command -v wasm-opt >/dev/null 2>&1 && echo "wasm-opt already installed" || { \
		echo "Installing binaryen (wasm-opt)..."; \
		if [ "$$(uname)" = "Darwin" ]; then \
			brew install binaryen; \
		else \
			sudo apt-get update && sudo apt-get install -y binaryen; \
		fi; \
	}
	# Web-related
	command -v yarn >/dev/null 2>&1 || npm install -g yarn
	yarn --cwd $(WEB_CLIENT_DIR) --silent  # installs prettier, eslint, typedoc, etc.
	yarn --cwd crates/idxdb-store/src --silent
	yarn install --prefix docs/external --no-progress
	yarn --silent
	yarn
	@echo "Development tools installation complete!"

## --- Debug --------------------------------------------------------------------------------------
.PHONY: build-web-client-debug
build-web-client-debug: # build the web-client with debug symbols for the WASM-generated rust code
	cd crates/web-client && yarn build-dev

.PHONY: link-web-client-dep
link-web-client-dep: # links the local web-client for debugging JS applications.
	cd crates/web-client && yarn link
