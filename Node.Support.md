# Plan: Enable Miden Web Client in Node.js

## Context

The web-client (`@miden-sdk/miden-sdk`) currently only works in browsers. We need it to work in a Node.js 20+ backend server context, with persistent storage, as a conditional export from the existing package.

The web-client is a WASM binary (compiled from `rust-client` + `idxdb-store`) wrapped with a JS layer. The Rust code is **environment-agnostic** — all browser dependencies come from two places:
1. **The JS storage layer** (Dexie.js/IndexedDB) — 58 JS functions called by WASM via wasm-bindgen
2. **The JS wrapper layer** (workers, WASM loading, sync locks) — already has non-browser fallbacks

The WASM binary itself uses `tonic-web-wasm-client` for RPC (which calls `fetch`, `Request`, `Response` via `web-sys`) and `getrandom` with `wasm_js` backend. Both work in Node.js 20+ (validated by PoC).

**Branch**: `wiktor-node`, based off `main` (v0.13.0, compatible with devnet).

## Approach: Dual-Build with SQLite Node Store

**Same WASM binary**, two JS layers:
- **Browser build** (existing `dist/`) — Dexie/IndexedDB storage, Web Workers
- **Node.js build** (new `dist-node/`) — SQLite storage via `better-sqlite3`, no Workers

The key insight: the WASM binary doesn't care what implements the 58 JS storage functions — it just calls them and expects Promises back. We swap the implementation at build time using Rollup aliases.

## Step 1: Validate feasibility (PoC) — DONE

**Result: All checks pass.** Full integration test runs against devnet (create accounts, mint, send, verify balances).

**Files created:**
- `crates/web-client/test/node/setup.mjs` — Reusable Node.js environment polyfills
- `crates/web-client/test/node/poc.mjs` — Quick feasibility check (13/13 pass)
- `crates/web-client/test/node/integration.mjs` — Full lifecycle test (18/18 pass)

**Required Node.js polyfills (documented in `setup.mjs`):**

| Polyfill | Why | Implementation |
|----------|-----|----------------|
| `fake-indexeddb/auto` | Dexie.js requires IndexedDB globals | `import "fake-indexeddb/auto"` |
| `undici` Agent with `allowH2: true` | gRPC-Web server requires HTTP/2; Node.js `fetch` defaults to HTTP/1.1 (browsers negotiate HTTP/2 via ALPN automatically) | `setGlobalDispatcher(new Agent({ allowH2: true }))` |
| `file://` fetch interception | wasm-bindgen loads `.wasm` via `fetch(new URL("...wasm", import.meta.url))`; Node.js `fetch` doesn't support `file://` URLs | Intercept fetch, read from disk with `fs.readFileSync`, return `Response` |
| `globalThis.self = globalThis` | Used by Dexie and some wasm-bindgen glue code | One-liner |

**Resolved risks from PoC:**
- WASM atomics: No `--experimental-wasm-threads` flag needed on Node.js 20
- `tonic-web-wasm-client`: Works perfectly once fetch uses HTTP/2
- `web-sys` bindings: All required globals (`fetch`, `Request`, `Response`, `Headers`, `crypto.getRandomValues`) are natively available
- Worker fallback: Correctly detects `typeof Worker === "undefined"` and runs on main thread

## Step 2: Create Node.js SQLite storage adapter

Create `crates/web-client/js/node-store/` with SQLite-backed implementations of all 58 JS functions that `idxdb-store` declares via wasm-bindgen.

**Files to create** (mirroring `crates/idxdb-store/src/js/`):

| New file | Mirrors | Functions |
|----------|---------|-----------|
| `js/node-store/schema.js` | `idxdb-store/src/js/schema.js` | DB init, connection registry |
| `js/node-store/accounts.js` | `idxdb-store/src/js/accounts.js` | 26 account functions |
| `js/node-store/notes.js` | `idxdb-store/src/js/notes.js` | 11 note functions |
| `js/node-store/chainData.js` | `idxdb-store/src/js/chainData.js` | 9 chain data functions |
| `js/node-store/transactions.js` | `idxdb-store/src/js/transactions.js` | 3 transaction functions |
| `js/node-store/sync.js` | `idxdb-store/src/js/sync.js` | 6 sync functions |
| `js/node-store/settings.js` | `idxdb-store/src/js/settings.js` | 4 settings functions |
| `js/node-store/export.js` | `idxdb-store/src/js/export.js` | 1 export function |
| `js/node-store/import.js` | `idxdb-store/src/js/import.js` | 1 import function |

**SQL schema**: Adapted from `crates/sqlite-store/src/store.sql` (18 tables, same structure). The existing `sqlite-store` SQL queries serve as reference for each function's implementation.

**Storage library**: `better-sqlite3` — synchronous SQLite for Node.js. Functions return `Promise.resolve(result)` to match the async interface expected by wasm-bindgen. Synchronous SQLite is fine because individual queries are fast (~microseconds) and won't block the event loop significantly.

**Database location**: Configurable via a new parameter in the Node.js entry point (defaults to `./miden-store.sqlite`).

## Step 3: Create Node.js build pipeline

Separate Rollup config `rollup.config.node.js`:
- **Reuse the .wasm** from the browser build (identical binary)
- Different JS entry point (`js/node-entry.js`)
- Rollup alias plugin redirects `idxdb-store` JS snippet imports → `node-store/` equivalents
- Output format: ESM to `dist-node/`
- External: `better-sqlite3`, `undici`, `node:*` builtins

**WASM loading**: The existing wasm-bindgen pattern (`fetch(new URL("...wasm", import.meta.url))`) works in Node.js with the `file://` fetch interception from `setup.mjs`. The Node.js entry point will include this setup automatically, so consumers don't need to do it themselves.

## Step 4: Create Node.js entry point

New file: `crates/web-client/js/node-entry.js`

This file:
1. Applies the Node.js polyfills from `setup.mjs` (IndexedDB, HTTP/2 fetch, file:// fetch, globalThis.self)
2. Imports and re-exports everything from `index.js` (which exports `WebClient` + all WASM types)
3. The `WebClient.createClient(rpcUrl)` API stays the same — no API changes needed

```javascript
// Node.js polyfills (applied before any WASM imports)
import "fake-indexeddb/auto";
import { Agent, setGlobalDispatcher } from "undici";
setGlobalDispatcher(new Agent({ allowH2: true }));
// ... file:// fetch patch, globalThis.self

// Re-export everything from the main entry
export * from "./index.js";
```

**Note**: On `main` branch, the public API is `WebClient.createClient(rpcUrl, noteTransportUrl, seed, network)` — there is no `MidenClient` class (that exists only on the `wiktor-api-rework` branch).

## Step 5: Update package.json

```json
{
  "exports": {
    ".": {
      "node": "./dist-node/index.js",
      "default": "./dist/index.js"
    }
  },
  "dependencies": {
    "dexie": "^4.0.1"
  },
  "optionalDependencies": {
    "better-sqlite3": "^11.0.0",
    "undici": "^7.21.0"
  }
}
```

Using `optionalDependencies` for `better-sqlite3` and `undici` so browser consumers don't need to install them. `fake-indexeddb` is bundled into `dist-node/` (or listed as optional dep).

## Step 6: Add Makefile targets

```makefile
build-wasm-node:     # Build Node.js variant (rollup with node config, reuses browser .wasm)
test-node-client:    # Run Node.js integration tests against devnet/localhost
```

## Step 7: Integration tests — DONE

Already created at `crates/web-client/test/node/`:
- `integration.mjs` — Full lifecycle test (18 steps, all passing against devnet):
  1. Load SDK, create client, sync
  2. Create wallet + faucet
  3. Mint 1000 tokens (faucet → wallet), prove, submit, wait for commit
  4. Consume minted note (wallet receives tokens)
  5. Verify wallet balance = 1000
  6. Create wallet2, send 100 tokens (wallet → wallet2)
  7. Consume sent note (wallet2 receives tokens)
  8. Verify final balances: wallet=900, wallet2=100
  9. List all accounts (3 total)

These tests currently use `fake-indexeddb` (in-memory). Once Step 2 is complete, they'll be updated to use the SQLite store to verify persistence across client restarts.

## Key Files Reference

### Files to modify
- `crates/web-client/package.json` — conditional exports, dependencies
- `Makefile` — new build/test targets

### Files to create
- `crates/web-client/js/node-entry.js` — Node.js bootstrap with polyfills
- `crates/web-client/js/node-store/*.js` — 9 SQLite storage files (58 functions total)
- `crates/web-client/rollup.config.node.js` — Node.js build config

### Already created (Step 1)
- `crates/web-client/test/node/setup.mjs` — Reusable polyfill setup
- `crates/web-client/test/node/poc.mjs` — Quick feasibility PoC
- `crates/web-client/test/node/integration.mjs` — Full integration test
- `crates/web-client/test/node/package.json` — Test dependencies

### Reference files (read-only, for SQL query patterns)
- `crates/sqlite-store/src/store.sql` — SQL schema (18 tables)
- `crates/sqlite-store/src/account/accounts.rs` — Account SQL queries
- `crates/sqlite-store/src/note/mod.rs` — Note SQL queries
- `crates/sqlite-store/src/transaction.rs` — Transaction SQL queries
- `crates/sqlite-store/src/chain_data.rs` — Block header SQL queries
- `crates/sqlite-store/src/sync.rs` — Sync state SQL queries
- `crates/idxdb-store/src/js/*.js` — Existing Dexie implementations (function signatures)
- `crates/idxdb-store/src/*/js_bindings.rs` — Rust wasm-bindgen extern declarations (exact signatures)

### No changes needed
- **No Rust code changes** — WASM binary is identical for both targets
- `crates/idxdb-store/` — untouched, still used for browser build
- `crates/rust-client/` — untouched
- `crates/web-client/js/syncLock.js` — fallback already handles Node.js (in-process mutex)
- `crates/web-client/js/index.js` — Worker fallback already works (`typeof Worker === "undefined"`)

## Risks

| Risk | Severity | Status |
|------|----------|--------|
| `tonic-web-wasm-client` web-sys bindings incompatible with Node.js | ~~High~~ | **Resolved** — works with HTTP/2 fetch |
| WASM atomics require `--experimental-wasm-threads` flag | ~~Medium~~ | **Resolved** — not needed on Node.js 20 |
| Rollup can't alias wasm-bindgen snippet imports for idxdb-store | **Medium** | Alternative: build idxdb-store with swapped JS files, or use a Rollup transform plugin |
| `better-sqlite3` requires native compilation (node-gyp) | **Low** | Well-maintained with prebuilt binaries. Alternative: `sql.js` (pure WASM) |
| Data format differences between Dexie (JSON-ish) and SQLite (BLOB) | **Medium** | Both store trait implementations serialize to bytes. Match the existing format from idxdb-store JS |

## Verification

1. **PoC validates**: WASM loads in Node.js, RPC calls work, crypto works — **DONE**
2. **Integration tests**: Full client lifecycle against devnet — **DONE** (18/18 pass)
3. **Unit tests**: Each of the 58 SQLite storage functions tested against same inputs/outputs as Dexie
4. **Persistence tests**: Data survives client restart (SQLite on disk)
5. **CI job**: `test-node-client` runs against test node in CI pipeline
6. **Build verification**: `build-wasm-node` produces working `dist-node/` output
