# Changelog

## 0.13.1 (TBD)

* Added unified signer interface (`SignerContext`, `useSigner`) for external keystore providers (Para, Turnkey, MidenFi) with `MidenProvider` integration and comprehensive test coverage ([#1732](https://github.com/0xMiden/miden-client/pull/1732)).
* [FIX][web] Fixed `useSend` and `useMultiSend` hooks accessing WASM pointers after `applyTransaction` invalidated them, causing use-after-free errors ([#1810](https://github.com/0xMiden/miden-client/pull/1810)).

## 0.13.0

* Initial release of `@miden-sdk/react` hooks library with a provider, hooks, and an example app for the web client ([#1711](https://github.com/0xMiden/miden-client/pull/1711)).
