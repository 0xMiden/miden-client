# Changelog

## 0.13.4 (TBD)

### Features
* [FEATURE][web] All user-facing amount fields (`amount`, `maxSupply`, `offeredAmount`, `requestedAmount`) now accept `number` in addition to `bigint`, removing the need for `n` suffixes or `BigInt()` wrappers. Values are coerced to `bigint` internally before passing to WASM. Output types (balances, note assets) remain `bigint`.
* [FEATURE][web] `formatAssetAmount()` now accepts `number | bigint` for convenience.
* [FEATURE][web] Added `MultiSignerProvider` and `SignerSlot` components for dapps that support multiple external signers (Para, Turnkey, MidenFi). Users can switch between signers at runtime via `useMultiSigner()`. ([#1872](https://github.com/0xMiden/miden-client/pull/1872))
* [FEATURE][web] `midenVitePlugin()` now handles esbuild externalization of `@miden-sdk/react`, React deduplication, and `esnext` esbuild target automatically — consuming apps no longer need manual config for these. ([#1872](https://github.com/0xMiden/miden-client/pull/1872))
* [FEATURE][web] `midenVitePlugin()` `crossOriginIsolation` option now defaults to `false`, avoiding breakage of OAuth popup flows (e.g. Para) that rely on `window.opener`. ([#1872](https://github.com/0xMiden/miden-client/pull/1872))

### Fixes
* [FIX][web] Fixed signer disconnect destroying WebClient and wiping cached state. The client now stays alive for reads on disconnect, hot-swaps `signCb` on same-identity reconnect (no WASM reinit), and only creates a new client when a different identity connects. All mutation hooks block with a clear error while disconnected ([#1842](https://github.com/0xMiden/miden-client/pull/1842)).

## 0.13.3 (2026-02-25)

### Features
* [FEATURE][web] Added `customComponents` field to `SignerAccountConfig`, allowing signer providers to attach arbitrary `AccountComponent` instances (e.g. compiled `.masp` packages) to accounts during `initializeSignerAccount`. Components are appended after the default basic wallet component.
* [FEATURE][web] Added `MidenError` class and `wrapWasmError()` utility that intercepts cryptic WASM errors and replaces them with actionable messages including fix suggestions ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `readNoteAttachment()` and `createNoteAttachment()` utilities for encoding/decoding arbitrary `bigint[]` payloads on notes, with automatic Word vs Array detection and 4-element boundary padding ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `normalizeAccountId()` and `accountIdsEqual()` utilities for format-agnostic account ID comparison across hex and bech32 ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `bytesToBigInt()`, `bigIntToBytes()`, and `concatBytes()` utilities for cryptographic data conversions ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `migrateStorage()`, `clearMidenStorage()`, and `createMidenStorage()` utilities for IndexedDB version migration and namespaced localStorage persistence ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `noteFirstSeen` temporal tracking to `MidenStore` with smart diffing so only new note IDs receive timestamps ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `attachment` option, `skipSync` option (auto-sync before send), concurrency guard (`SEND_BUSY`), and `sendAll` flag to `useSend` ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `attachment` support (per-recipient overrides), auto-sync, and concurrency guard to `useMultiSend` ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `skipSync` option and concurrency guard to `useTransaction` ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `sender` and `excludeIds` filter options to `useNotes` ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `useNoteStream()` hook for temporal note tracking with unified `StreamedNote` type, built-in filtering (sender, status, since, excludeIds, amountFilter), `markHandled`/`markAllHandled`, and `snapshot()` ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `useSessionAccount()` hook for session wallet lifecycle management (create, fund, consume) with step tracking, localStorage persistence, and configurable polling ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `waitForWalletDetection()` utility for wallet extension detection with configurable timeout, event-based polling, and TOCTOU race condition handling ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FEATURE][web] Added `@miden-sdk/vite-plugin` package for zero-config Miden dApp Vite setup: WASM deduplication, COOP/COEP cross-origin isolation headers, gRPC-web proxy, esnext build target, and ES module workers ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).

### Fixes
* [FIX][web] Fixed React StrictMode double-initialization in `MidenProvider` by adding `initializingRef` guard to prevent concurrent WASM client init ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FIX][web] Standardized all hooks to privacy-first defaults (`NoteType.Private`) and ensured all mutation paths go through `runExclusive` for WASM concurrency safety ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).
* [FIX][web] Removed incorrect `AccountId` single-use reference from `WASM_POINTER_CONSUMED` error message — `AccountId` APIs take references and are not typically consumed ([#1818](https://github.com/0xMiden/miden-client/pull/1818)).

## 0.13.2 (2026-02-10)

* [FIX][web] Fixed concurrent WASM access during initialization by performing initial sync before `setClient`, preventing race conditions between init sync and auto-sync ([#1755](https://github.com/0xMiden/miden-client/pull/1755)).

## 0.13.1 (2026-02-09)

* Added unified signer interface (`SignerContext`, `useSigner`) for external keystore providers (Para, Turnkey, MidenFi) with `MidenProvider` integration and comprehensive test coverage ([#1732](https://github.com/0xMiden/miden-client/pull/1732)).
* [FIX][web] Fixed `useSend` and `useMultiSend` hooks accessing WASM pointers after `applyTransaction` invalidated them, causing use-after-free errors ([#1810](https://github.com/0xMiden/miden-client/pull/1810)).

## 0.13.0

* Initial release of `@miden-sdk/react` hooks library with a provider, hooks, and an example app for the web client ([#1711](https://github.com/0xMiden/miden-client/pull/1711)).
