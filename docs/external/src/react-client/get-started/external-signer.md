---
title: External signer
sidebar_position: 4
---

# External signer

The React SDK supports third-party wallet providers through a `SignerContext`. When a signer is present, the SDK delegates key management and transaction signing to the external provider instead of the local keystore.

## Built-in signer packages

| Package | Provider | Auth method |
|---------|----------|-------------|
| `@miden-sdk/para` | Para | EVM wallets |
| `@miden-sdk/miden-turnkey-react` | Turnkey | Passkey authentication |
| `@miden-sdk/wallet-adapter-react` | MidenFi | Native wallet adapter |

## Using a built-in signer

Wrap `MidenProvider` with the signer provider:

```tsx
import { MidenProvider } from "@miden-sdk/react";
import { ParaSignerProvider } from "@miden-sdk/para";

function App() {
  return (
    <ParaSignerProvider apiKey="your-api-key" environment="PRODUCTION">
      <MidenProvider config={{ rpcUrl: "testnet" }}>
        <MyApp />
      </MidenProvider>
    </ParaSignerProvider>
  );
}
```

Inside your components, use the `useSigner` hook to access connection state:

```tsx
import { useSigner } from "@miden-sdk/react";

function ConnectButton() {
  const { isConnected, connect, disconnect, name } = useSigner();

  if (isConnected) {
    return <button onClick={disconnect}>Disconnect {name}</button>;
  }

  return <button onClick={connect}>Connect Wallet</button>;
}
```

All other hooks (`useAccounts`, `useSend`, etc.) work the same way — the SDK routes signing through the signer automatically.

## Custom signer

To integrate a wallet provider that doesn't have a built-in package, provide a `SignerContext` directly:

```tsx
import { MidenProvider, SignerContext } from "@miden-sdk/react";

function App() {
  const signerValue = {
    name: "MyWallet",
    storeName: `mywallet_${userAddress}`,
    isConnected: true,
    accountConfig: {
      publicKeyCommitment: userPublicKeyCommitment,
      storageMode: "private",
      accountType: "RegularAccountUpdatableCode",
    },
    signCb: async (pubKey, signingInputs) => {
      // Route to your signing backend
      return signature;
    },
    connect: async () => { /* auth flow */ },
    disconnect: async () => { /* logout */ },
  };

  return (
    <SignerContext.Provider value={signerValue}>
      <MidenProvider config={{ rpcUrl: "testnet" }}>
        <MyApp />
      </MidenProvider>
    </SignerContext.Provider>
  );
}
```

Key properties:

| Property | Description |
|----------|-------------|
| `signCb` | Async function that receives a public key and signing inputs, returns a signature |
| `accountConfig` | Account creation parameters (storage mode, type, optional custom components) |
| `storeName` | IndexedDB isolation key — use a unique value per user/wallet to prevent data collisions |

## Next steps

- [Library reference](../library/index.md) — detailed hooks and API documentation
