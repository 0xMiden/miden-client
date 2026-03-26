# Changelog

## 0.13.3 (TBD)

* [FEATURE][web] Added `customComponents` field to `SignerAccountConfig`, allowing signer providers to attach arbitrary `AccountComponent` instances (e.g. compiled `.masp` packages) to accounts during `initializeSignerAccount`. Components are appended after the default basic wallet component.

## 0.13.2 (2026-02-10)

* [FIX][web] Fixed concurrent WASM access during initialization by performing initial sync before `setClient`, preventing race conditions between init sync and auto-sync ([#1755](https://github.com/0xMiden/miden-client/pull/1755)).

## 0.13.1 (2026-02-09)

* Added unified signer interface (`SignerContext`, `useSigner`) for external keystore providers (Para, Turnkey, MidenFi) with `MidenProvider` integration and comprehensive test coverage ([#1732](https://github.com/0xMiden/miden-client/pull/1732)).
* [FIX][web] Fixed `useSend` and `useMultiSend` hooks accessing WASM pointers after `applyTransaction` invalidated them, causing use-after-free errors ([#1810](https://github.com/0xMiden/miden-client/pull/1810)).

## 0.13.0

* Initial release of `@miden-sdk/react` hooks library with a provider, hooks, and an example app for the web client ([#1711](https://github.com/0xMiden/miden-client/pull/1711)).
