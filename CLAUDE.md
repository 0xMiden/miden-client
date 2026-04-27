# Claude Code Guidelines for miden-client

## Code Comments

**Never reference the previous state of the code in comments.** No "previous version did X", "used to do Y", "changed from Z", "the old pattern was", etc. Comments must describe the CURRENT code's rationale — the invariant, constraint, or subtle correctness reason that isn't obvious from the code itself. If context about a change matters, put it in the PR description or commit message, not in the source. The source gets read long after the diff stops being relevant, and stale "previously" comments become actively misleading.

## Important: Always Format Code

**After making any code changes, always run `make format` before committing.** This ensures consistent formatting across Rust, TypeScript, and other files.

## Repository Overview

Miden Client is the official client library for interacting with the Miden rollup. It provides APIs for managing accounts, notes, and transactions on the Miden network.

**Key crates:**
- `rust-client` - Core client library (no_std compatible)
- `web-client` - WASM/JavaScript SDK wrapper for browsers
- `sqlite-store` - SQLite persistence backend
- `idxdb-store` - IndexedDB persistence for browsers
- `miden-cli` - Command-line interface

## Build System

The project uses **Cargo** with a **Makefile** wrapper. Always prefer Make targets over raw cargo commands.

### Essential Commands

```bash
# Before submitting PRs - runs all lints
make lint

# Run unit tests
make test

# Run integration tests (requires test node)
make start-node-background
make integration-test
make stop-node

# Build everything
make build

# Build WASM targets
make build-wasm

# Install development tools (nextest, taplo, etc.)
make install-tools
```

### Web Client Development

```bash
cd crates/web-client
yarn install
yarn build

# For idxdb-store TypeScript changes
cd crates/idxdb-store/src
yarn build   # Compiles TS to JS - BOTH must be committed
```

**Important:** The `idxdb-store` JavaScript files are generated from TypeScript but BOTH are committed. The CI compiles TS and compares against committed JS. Always run `yarn build` after modifying `.ts` files.

## Testing

- **Unit tests:** `make test` (uses cargo-nextest)
- **Doc tests:** `make test-docs`
- **Integration tests:** `make integration-test` (requires running test node)
- **Web client tests:** `make integration-test-web-client` (uses Playwright)
- **React SDK tests:** `cd packages/react-sdk && yarn test`

## Code Style

### Rust
- Edition 2024, MSRV 1.90
- Use section headers for major code sections:
  ```rust
  // SECTION NAME
  // ================================================================================================
  ```
- Rustdoc with 100-char line limit
- All public APIs must be documented
- Clippy warnings are errors in CI

### TypeScript/JavaScript
- Prettier for formatting
- ESLint for linting
- Run `make format` to format all code

### React SDK (`packages/react-sdk`)
- **TypeScript strict mode is enabled** (`strict: true` in tsconfig.json). All code must pass `tsc --noEmit` with no errors.
- **After any react-sdk changes, always verify locally before committing:**
  ```bash
  cd packages/react-sdk
  yarn typecheck    # tsc --noEmit - catches type errors
  yarn lint         # eslint
  yarn test         # vitest unit tests
  yarn build        # tsup build including DTS generation (catches type errors that tsc --noEmit also catches)
  ```
- **CI runs 3 react-sdk checks** (all must pass): "React SDK lint and typecheck", "Build React SDK", and "React SDK integration tests". The build step generates `.d.ts` type declarations and will fail on any TypeScript error.
- When modifying Zustand store types (e.g., in `MidenStore.ts`), ensure all consumers (hooks, context providers) are compatible with the updated types. Pay special attention to nullable types (`T | null`) — if state can be `null`, setter functions must accept `null` too.

## Architecture

### Storage Trait Pattern
The `Store` trait abstracts persistence. Implementations:
- `SqliteStore` - Native applications
- `WebStore` (IndexedDB) - Browser/WASM

### Client Structure
```rust
pub struct Client<AUTH> {
    // AUTH implements TransactionAuthenticator
    // Uses Arc-wrapped Store and RpcClient
}
```

### Key Directories
```
crates/
├── rust-client/src/
│   ├── account/      # Account management
│   ├── note/         # Note handling
│   ├── transaction/  # TX building/execution
│   ├── rpc/          # Node communication
│   ├── store/        # Persistence trait
│   └── sync/         # State synchronization
├── web-client/src/   # WASM bindings
├── idxdb-store/src/
│   ├── ts/           # TypeScript source
│   └── js/           # Generated JavaScript (committed)
└── sqlite-store/src/ # SQLite implementation
```

## Git Workflow

- **Main branch:** `next`
- **Commit format:** `type(scope): description`
  - Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`
  - Example: `fix(web-client): handle IndexedDB race condition`
- **Always update CHANGELOG.md** for user-visible changes
- **Rebase before PR** (avoid merge commits)

## Common Patterns

### Error Handling
```rust
// Use ClientError with proper context
return Err(ClientError::StoreError(err));

// ErrorHint provides user-friendly help messages
```

### Async Operations
```rust
// Most store operations are async
let header = self.store.get_block_header_by_num(block_num).await?;
```

### IndexedDB Transactions (idxdb-store)
When multiple IndexedDB operations must be atomic, wrap them in a Dexie transaction:
```typescript
await db.transaction("rw", [table1, table2], async () => {
    // All operations here are atomic
});
```

## CI Checks

PRs must pass:
1. `make lint` - Format, clippy, typos, TOML
2. `make test` - Unit tests
3. `make test-docs` - Documentation tests
4. Integration tests (run automatically in CI)
5. WASM build check
6. React SDK lint, typecheck, build, and tests (when `packages/react-sdk` is modified)

## Feature Flags

- `std` - Standard library support
- `tonic` - gRPC client
- `testing` - Test utilities and mocks
- `web-tonic` - WASM gRPC support

## Debugging

### WASM Debug Build
```bash
make build-web-client-debug
# or
MIDEN_WEB_DEV=true yarn build
```

### Test Node
```bash
make start-node-background  # Start
make stop-node              # Stop
```

## Dependencies

Key external dependencies:
- `miden-objects`, `miden-tx`, `miden-lib` - Miden protocol crates
- `tokio` - Async runtime
- `rusqlite` - SQLite bindings
- `wasm-bindgen` - WASM/JS interop
- `dexie` (JS) - IndexedDB wrapper

## Versioning

- All workspace crates share version (currently 0.12.x)
- NPM package: `@miden-sdk/miden-sdk`
- Protocol version must match node version
