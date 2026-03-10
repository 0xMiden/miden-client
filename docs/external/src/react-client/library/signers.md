---
title: Signers
sidebar_position: 6
---

# Signers

The SDK supports external wallet providers through a `SignerContext` that plugs into the provider tree above `MidenProvider`.

## useSigner()

Access the current signer's connection state.

```tsx
import { useSigner } from "@miden-sdk/react-sdk";

function WalletButton() {
  const { isConnected, connect, disconnect, name } = useSigner();

  if (isConnected) {
    return <button onClick={disconnect}>Disconnect {name}</button>;
  }

  return <button onClick={connect}>Connect Wallet</button>;
}
```

### Return value

| Property | Type | Description |
|----------|------|-------------|
| `isConnected` | `boolean` | Whether the signer is connected |
| `connect` | `() => Promise<void>` | Trigger the signer's auth flow |
| `disconnect` | `() => Promise<void>` | Disconnect the signer |
| `name` | `string` | Display name (e.g., "Para", "Turnkey") |

## SignerContext

The context interface that signer providers must implement.

```tsx
interface SignerContextValue {
  signCb: (pubKey: Uint8Array, signingInputs: Uint8Array) => Promise<Uint8Array>;
  accountConfig: SignerAccountConfig;
  storeName: string;
  name: string;
  isConnected: boolean;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
}
```

| Property | Description |
|----------|-------------|
| `signCb` | Signing function — receives public key and signing inputs, returns signature bytes |
| `accountConfig` | Account creation params: `publicKeyCommitment`, `storageMode`, `accountType`, optional `customComponents` |
| `storeName` | IndexedDB isolation key — must be unique per user/wallet to prevent data collisions |
| `name` | Display name shown in UI |

## Built-in signer packages

| Package | Provider | Usage |
|---------|----------|-------|
| `@miden-sdk/use-miden-para-react` | Para (EVM wallets) | `<ParaSignerProvider apiKey="..." environment="PRODUCTION">` |
| `@miden-sdk/miden-turnkey-react` | Turnkey (passkeys) | `<TurnkeySignerProvider ...>` |
| `@miden-sdk/miden-wallet-adapter-react` | MidenFi (native) | `<MidenFiSignerProvider ...>` |

## How it works

When `MidenProvider` detects a `SignerContext` above it:

1. Creates the client with `WebClient.createClientWithExternalKeystore()` instead of the default constructor
2. Routes all signing operations to the signer's `signCb`
3. Isolates IndexedDB storage using the signer's `storeName`
4. Binds account creation to the signer's `accountConfig`

All hooks (`useCreateWallet`, `useSend`, etc.) work identically regardless of whether a local or external keystore is used.

## Multi-signer support

For dapps that support multiple wallet providers, wrap them with `MultiSignerProvider` and place a `SignerSlot` inside each one. Users can then switch between signers at runtime.

### Setup

```tsx
import {
  MidenProvider,
  MultiSignerProvider,
  SignerSlot,
} from "@miden-sdk/react-sdk";
import { ParaSignerProvider } from "@miden-sdk/use-miden-para-react";
import { TurnkeySignerProvider } from "@miden-sdk/miden-turnkey-react";
import { MidenFiSignerProvider } from "@miden-sdk/miden-wallet-adapter-react";

function App() {
  return (
    <MultiSignerProvider>
      <ParaSignerProvider apiKey="your-key" environment="BETA">
        <SignerSlot />
      </ParaSignerProvider>
      <TurnkeySignerProvider>
        <SignerSlot />
      </TurnkeySignerProvider>
      <MidenFiSignerProvider network="testnet">
        <SignerSlot />
      </MidenFiSignerProvider>
      <MidenProvider config={{ rpcUrl: "testnet", prover: "testnet" }}>
        <YourApp />
      </MidenProvider>
    </MultiSignerProvider>
  );
}
```

`MultiSignerProvider` is optional — single-signer apps continue to work with a signer provider directly above `MidenProvider`.

### useMultiSigner()

List registered signers and switch between them.

```tsx
import { useMultiSigner } from "@miden-sdk/react-sdk";

function SignerSelector() {
  const multiSigner = useMultiSigner();

  return (
    <div>
      {multiSigner?.signers.map((s) => (
        <button key={s.name} onClick={() => multiSigner.connectSigner(s.name)}>
          {s.name}
        </button>
      ))}
      <button onClick={() => multiSigner?.disconnectSigner()}>
        Use Local Keystore
      </button>
    </div>
  );
}
```

### Return value

`useMultiSigner()` returns `null` when used outside a `MultiSignerProvider`.

| Property | Type | Description |
|----------|------|-------------|
| `signers` | `SignerContextValue[]` | All registered signer providers |
| `activeSigner` | `SignerContextValue \| null` | The currently connected signer, or `null` for local keystore mode |
| `connectSigner` | `(name: string) => Promise<void>` | Switch to a signer by name — disconnects the previous signer and calls `connect()` on the new one |
| `disconnectSigner` | `() => Promise<void>` | Disconnect the active signer and revert to local keystore mode |

### SignerSlot

A render-less component that registers its nearest ancestor's `SignerContext` into the `MultiSignerProvider` registry. Place one inside each signer provider:

```tsx
<ParaSignerProvider apiKey="...">
  <SignerSlot />
</ParaSignerProvider>
```

### How it works

1. Each `SignerSlot` registers its parent signer into a shared registry
2. `MultiSignerProvider` forwards the active signer's `SignerContext` down to `MidenProvider`
3. When `connectSigner(name)` is called, the previous signer is disconnected and the new one is connected
4. When `disconnectSigner()` is called, `MidenProvider` sees `null` and reverts to local keystore mode
5. `MidenProvider` reinitializes the client with the appropriate keystore whenever the active signer changes
