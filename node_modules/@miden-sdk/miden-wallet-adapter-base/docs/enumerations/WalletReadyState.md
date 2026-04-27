[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / WalletReadyState

# Enumeration: WalletReadyState

A wallet's readiness describes a series of states that the wallet can be in,
depending on what kind of wallet it is. An installable wallet (eg. a browser
extension like Phantom) might be `Installed` if we've found the Phantom API
in the global scope, or `NotDetected` otherwise. A loadable, zero-install
runtime (eg. Torus Wallet) might simply signal that it's `Loadable`. Use this
metadata to personalize the wallet list for each user (eg. to show their
installed wallets first).

## Enumeration Members

### Installed

> **Installed**: `"Installed"`

User-installable wallets can typically be detected by scanning for an API
that they've injected into the global context. If such an API is present,
we consider the wallet to have been installed.

***

### Loadable

> **Loadable**: `"Loadable"`

Loadable wallets are always available to you. Since you can load them at
any time, it's meaningless to say that they have been detected.

***

### NotDetected

> **NotDetected**: `"NotDetected"`

***

### Unsupported

> **Unsupported**: `"Unsupported"`

If a wallet is not supported on a given platform (eg. server-rendering, or
mobile) then it will stay in the `Unsupported` state.
