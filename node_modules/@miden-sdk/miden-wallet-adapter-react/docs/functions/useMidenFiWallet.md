[**@miden-sdk/miden-wallet-adapter-react**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-react](../README.md) / useMidenFiWallet

# Function: useMidenFiWallet()

> **useMidenFiWallet**(): [`WalletContextState`](../interfaces/WalletContextState.md)

Hook for MidenFi wallet operations beyond the unified useSigner interface.
Use this to access wallet-specific methods like requestTransaction, requestAssets, etc.

## Returns

[`WalletContextState`](../interfaces/WalletContextState.md)

## Example

```tsx
const { connected, connect, disconnect, wallets, select } = useMidenFiWallet();

// Connect
await connect();

// Request a transaction
const txId = await requestTransaction({ ... });
```
