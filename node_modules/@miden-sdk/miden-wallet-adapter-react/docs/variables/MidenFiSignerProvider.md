[**@miden-sdk/miden-wallet-adapter-react**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-react](../README.md) / MidenFiSignerProvider

# Variable: MidenFiSignerProvider

> `const` **MidenFiSignerProvider**: `FC`\<[`MidenFiSignerProviderProps`](../interfaces/MidenFiSignerProviderProps.md)\>

MidenFiSignerProvider bridges the MidenFi wallet with MidenProvider.

This is a unified provider that handles both wallet connection and signer context.

## Examples

```tsx
// Simplest usage - uses MidenWalletAdapter by default
<MidenFiSignerProvider>
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</MidenFiSignerProvider>

// With custom options
<MidenFiSignerProvider
  appName="My DApp"
  network={WalletAdapterNetwork.Testnet}
  autoConnect={true}
>
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</MidenFiSignerProvider>

// With custom wallets
<MidenFiSignerProvider wallets={[new CustomWalletAdapter()]}>
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</MidenFiSignerProvider>
```

For wallet operations, use the useMidenFiWallet hook:

```tsx
const { connected, connect, disconnect, select, wallets } = useMidenFiWallet();

// If multiple wallets, select one first
select(wallets[0].adapter.name);

// Then connect
await connect();
```
