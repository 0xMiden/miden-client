---
title: Provider
sidebar_position: 1
---

# Provider

## MidenProvider

The root provider that initializes the WASM client and provides context to all hooks.

```tsx
import { MidenProvider } from "@miden-sdk/react";

<MidenProvider
  config={{
    rpcUrl: "testnet",
    autoSyncInterval: 15000,
    prover: "testnet",
  }}
  loadingComponent={<div>Loading...</div>}
  errorComponent={(error) => <div>Error: {error.message}</div>}
>
  <App />
</MidenProvider>
```

### Props

| Prop | Type | Description |
|------|------|-------------|
| `config` | `MidenConfig` | Client configuration (see below) |
| `loadingComponent` | `ReactNode` | Shown during WASM initialization |
| `errorComponent` | `ReactNode \| (error: Error) => ReactNode` | Shown if initialization fails |
| `children` | `ReactNode` | Your application |

### MidenConfig

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `rpcUrl` | `string` | — | `"devnet"` \| `"testnet"` \| `"localhost"` \| custom URL |
| `autoSyncInterval` | `number` | `15000` | Sync interval in ms. Set `0` to disable. |
| `prover` | `string` | `"local"` | `"local"` \| `"devnet"` \| `"testnet"` \| custom URL |
| `proverTimeoutMs` | `number` | — | Timeout for remote prover requests |
| `noteTransportUrl` | `string` | — | Note transport network endpoint |
| `seed` | `Uint8Array` | — | 32-byte seed for deterministic RNG |

## useMiden()

Access the client instance and core utilities.

```tsx
import { useMiden } from "@miden-sdk/react";

function MyComponent() {
  const {
    client,          // WebClient instance (null until ready)
    isReady,         // Client fully initialized
    isInitializing,  // Initialization in progress
    error,           // Initialization error
    sync,            // Manual sync function
    runExclusive,    // Execute WASM operations exclusively
  } = useMiden();

  // Use runExclusive for direct WASM access
  const doSomething = async () => {
    await runExclusive(async () => {
      const account = await client.getAccount(accountId);
      // WASM pointers only valid inside this callback
    });
  };
}
```

### `runExclusive(fn)`

Queues a function to run with exclusive WASM access. Prevents concurrent WASM calls that would cause errors. All mutation hooks use this internally.

```tsx
const { runExclusive } = useMiden();

// Safe — operations are serialized
await runExclusive(async () => {
  await client.someWasmOperation();
});
```

:::warning
WASM object pointers (like `AccountId`, `NoteId`) are only valid within the `runExclusive` callback. Convert them to strings or numbers before returning.
:::
