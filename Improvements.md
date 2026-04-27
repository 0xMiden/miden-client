# Web Client Production Readiness — Gap Analysis

## Consumer API Surface

The public API is the `MidenClient` class (`client.js`) with resource-based sub-objects:

```ts
const client = await MidenClient.create({ rpcUrl: "testnet", autoSync: true });
client.accounts.create() / .get() / .list() / .import() / .export() / .getBalance()
client.transactions.send() / .mint() / .consume() / .swap() / .execute() / .preview() / .waitFor()
client.notes.list() / .get() / .import() / .export() / .fetchPrivate() / .sendPrivate()
client.tags.add() / .remove() / .list()
client.settings.get() / .set() / .remove() / .listKeys()
client.compile.component() / .txScript()
client.keystore.insert() / .get() / .remove() / .getCommitments()
client.sync() / .getSyncHeight() / .terminate()
```

All analysis below is against this surface, not the internal `WebClient`/WASM layer.

---

## Critical Gaps

### 1. Resilience & Connectivity

**Retry:** The Rust `GrpcClient` already retries every RPC call (`call_with_retry()` in `tonic_client/mod.rs`) with 4 attempts on `ResourceExhausted`/`Unavailable`, honoring `retry-after` headers. This works on WASM via `gloo_timers`. A JS-layer retry wrapper would be mostly redundant and dangerous for compound operations like `transactions.send()` which orchestrates execute -> prove -> submit -> apply internally (`#submitOrSubmitWithProver`). Re-executing after a submit-success/apply-failure would re-run the whole transaction.

**What's actually missing:**
- **Retry config not exposed to consumers.** `max_retries=4` and `retry_interval_ms=100` are hardcoded in Rust. `ClientOptions` has no way to tune them.
- **Retry state invisible.** Consumers never know retries are happening. When all 4 attempts fail, they get a single error with no indication of how long the SDK tried.
- **Compound operation recovery.** `#submitOrSubmitWithProver` does execute -> prove -> submit -> apply as one atomic sequence. If submit succeeds but apply fails, there's no recovery path — the consumer gets an error despite the transaction being on-chain. `waitFor()` handles sync failures gracefully (catch and continue polling), but the submit path doesn't.
- **No connection health exposure.** The node has a `Status` RPC endpoint (`get_status_unversioned()`), but nothing on `MidenClient` exposes connection state. Consumers can't show "disconnected" UI or skip operations they know will fail.
- **Single RPC endpoint.** `ClientOptions.rpcUrl` accepts one URL. No failover list.

### 2. Security Hardening

- **IndexedDB keys unencrypted at rest** when using the default (non-external) keystore. The `ClientOptions.keystore` callback pattern is good architecture for HSM/passkey integration, but consumers who skip it get plaintext keys in IndexedDB.
- **No security documentation.** The `keystore` option exists but the types don't guide consumers toward using it. No docs on when/why to use an external keystore, RPC endpoint trust model, or XSS considerations.
- **Seed passed to worker via postMessage.** `hashSeed()` runs SHA-256 before passing to WASM, but the hashed seed still crosses the worker boundary in a transferable buffer.

### 3. Observability

- **No event hooks on MidenClient.** There's no `onEvent`/`onError`/`onSync` callback in `ClientOptions` or on the client instance. Consumers can't plug in Sentry/OTel without monkey-patching.
- **Rust tracing goes to console only.** `setupLogging()` configures `tracing-wasm` which routes to `console.*`. No way to intercept or redirect these events.
- **No timing data.** `transactions.send()` can take 30+ seconds (execute + prove + submit). No way for consumers to know which phase is slow. `waitFor()` has `onProgress` (which reports "pending"/"submitted"/"committed"), but the submit pipeline has no equivalent.
- **Console logs not production-safe.** ~15 `console.log/error` calls in the JS layer leak internal method names and serialization details.

---

## High-Value Feature Gaps

### 4. Transaction Lifecycle Visibility

`transactions.send/mint/consume/swap` all call `#submitOrSubmitWithProver` which does 4 internal steps with no intermediate signals:
1. `executeTransaction` (compile + execute locally)
2. `proveTransaction` (generate ZK proof — the slow step)
3. `submitProvenTransaction` (send to node)
4. `applyTransaction` (update local state)

Consumers only see: pending... then either success or error. The `waitFor()` method has `onProgress` for the confirmation polling phase, but the submission phase (which is much slower due to proving) has nothing.

**What would help:** An `onProgress` callback on `TransactionOptions` (which `send/mint/consume/swap/execute` all accept) that reports stages: `"executing" | "proving" | "submitting" | "applying"`.

### 5. Missing High-Level Features (not just WASM bindings)

Checking against the Rust client's capabilities vs what `MidenClient` exposes:
- **Block header queries** — `get_block_header_by_number()` exists in Rust but isn't on `MidenClient`. Useful for explorers/dashboards.
- **Nullifier queries** — `check_nullifiers()` and `sync_nullifiers()` exist in Rust. No way to check if a note has been consumed without doing a full sync.
- **Non-fungible assets** — `accounts.create()` supports `NonFungibleFaucet` type, but there's no NFT minting path in `transactions.mint()` or asset construction helpers.
- **Raw RPC access** — no `client.rpc.*` escape hatch for advanced consumers who need direct node queries.
- **Store export/import on MidenClient** — `exportStore()` and `importStore()` exist as standalone functions, not on the client instance. The React SDK has `useExportStore`/`useImportStore` hooks that call these, but the JS SDK makes you import them separately.

### 6. IndexedDB Store Limitations

- **No storage quota awareness.** IndexedDB has browser-enforced quotas (~50MB-1GB). Writes can fail silently or throw when full. No pre-check or warning.
- **No auto-pruning.** `accounts.pruneAccountHistory()` exists (via `#inner.pruneAccountHistory`) but isn't exposed on `AccountsResource`. Historical data grows unbounded.
- **Multi-tab coordination.** Web Locks API prevents concurrent syncs, but no mechanism for multi-tab state invalidation (tab A syncs, tab B has stale data).

### 7. Developer Experience

- **TypeDoc generated but unpublished.** Exists at `docs/typedoc/web-client/` but isn't hosted.
- **No troubleshooting guide for WASM errors.** See `Improvements.Errors.md` for the full error pipeline analysis — worker serialization drops `.help` properties, panics produce `"unreachable"`.
- **`ClientOptions` missing useful knobs:** no retry config, no log level, no event hooks, no prune/quota settings. The `logLevel` is accepted on `WasmWebClient.createClient` but not forwarded through `MidenClient.create()`.
- **`package.json` missing `engines` field** in all three packages.

---

## Nice-to-Have / Next-Level

| Idea | Impact | Effort | Notes |
|------|--------|--------|-------|
| **`onProgress` for transaction pipeline** | High | Low | Add to `TransactionOptions`, report executing/proving/submitting/applying stages from `#submitOrSubmitWithProver` |
| **Event emitter on MidenClient** | High | Medium | `onSync`, `onTransaction`, `onError` hooks in `ClientOptions`. Natural integration point for Sentry/OTel |
| **Expose retry config** | High | Low | Add `retry: { maxAttempts, intervalMs }` to `ClientOptions`, pass through to Rust `GrpcClient` |
| **Connection health on client** | High | Medium | `client.checkHealth()` calling `get_status_unversioned()`, optional polling with events |
| **Compound operation recovery** | High | Medium | If submit succeeds but apply fails in `#submitOrSubmitWithProver`, retry only the apply step |
| **`logLevel` on `MidenClient.create()`** | Medium | Low | Currently only on internal `WasmWebClient`, not forwarded through `ClientOptions` |
| **Expose `pruneAccountHistory`** | Medium | Low | Add to `AccountsResource` — the WASM binding exists |
| **Store export/import on MidenClient** | Medium | Low | `client.exportStore()` / `client.importStore()` instead of standalone functions |
| **Multi-RPC failover** | Medium | Medium | Accept array of URLs in `ClientOptions.rpcUrl` |
| **Bundle size CI budget** | Medium | Low | Fail builds if WASM or JS bundle exceeds threshold |
| **Storage quota pre-check** | Medium | Medium | Check `navigator.storage.estimate()` before heavy writes |
| **Remote prover pool** | Medium | Medium | Accept multiple prover URLs, load-balance |
| **IndexedDB encryption layer** | Medium | High | Optional at-rest encryption for default keystore |
| **Offline queue** | Low | High | Buffer mutations when disconnected — hard to do safely with ZK transactions |

---

## What's Already Strong

- **Resource-based API** with clean separation (`client.accounts`, `client.transactions`, `client.notes`) and full TypeScript types
- **Flexible account creation** — wallets, faucets, and custom contracts with compiled MASM components via `client.compile`
- **Transaction orchestration** — `send/mint/consume/swap/execute` + `preview` for dry-runs + `waitFor` with progress callback
- **External keystore/signer** architecture via `ClientOptions.keystore` callbacks
- **Network presets** — `MidenClient.createTestnet()` / `.createDevnet()` with sensible defaults (prover, transport, autoSync)
- **Worker offloading** — all heavy WASM ops (execute, prove, sync) run off main thread
- **Sync coalescing** — concurrent `client.sync()` calls share one result via Web Locks
- **`waitFor()` resilience** — sync failures during confirmation polling are caught and retried (line 348-351 of transactions.js)
- **Mock client** — `MidenClient.createMock()` for testing without a node
- **Vite plugin** handles WASM dedup, COOP/COEP, gRPC proxy with zero-config defaults
