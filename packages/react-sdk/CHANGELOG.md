# Changelog

## 0.13.1 (TBD)

* Added unified signer interface (`SignerContext`, `useSigner`) for external keystore providers (Para, Turnkey, MidenFi) with `MidenProvider` integration and comprehensive test coverage ([#1732](https://github.com/0xMiden/miden-client/pull/1732)).

### Features

* [FEATURE][web] `SignerContext` now supports optional `getKeyCb` and `insertKeyCb` callbacks for full external keystore integration. `MidenProvider` passes these through to `WebClient.createClientWithExternalKeystore()`. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))
* [FEATURE][web] Added `useExportStore` and `useImportStore` hooks for encrypted wallet backup and restore via IndexedDB dump/import. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))
* [FEATURE][web] `useTransaction` now accepts a `privateNoteRecipient` option. When set, it uses the 4-step transaction pipeline (execute → prove → submit → apply) and delivers private output notes to the recipient via `sendPrivateNote()`. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))
* [FEATURE][web] Added `useImportNote` and `useExportNote` hooks for note import from bytes (QR codes, dApp requests) and export to bytes. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))
* [FEATURE][web] `ProverConfig` now supports fallback configuration with `primary`/`fallback` targets, `disableFallback` predicate, and `onFallback` callback. Transaction hooks automatically retry with the fallback prover on failure. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))
* [FEATURE][web] Added `useSyncControl` hook to pause and resume auto-sync intervals. ([#1861](https://github.com/0xMiden/miden-client/pull/1861))

## 0.13.0

* Initial release of `@miden-sdk/react` hooks library with a provider, hooks, and an example app for the web client ([#1711](https://github.com/0xMiden/miden-client/pull/1711)).
