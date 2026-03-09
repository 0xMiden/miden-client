---
title: Design
sidebar_position: 4
---

# Design

This page explains how the React SDK is structured and how its pieces fit together at runtime.

## How the SDK reaches the browser

The React SDK is a pure TypeScript/React package that wraps the [WASM bridge](../index.md). It consumes `@miden-sdk/miden-sdk` as a peer dependency and adds React-specific patterns — hooks, context providers, and Zustand state management — on top.

```mermaid
flowchart LR
    A["Rust core<br/>(miden-client)"] -->|"compiled to WASM"| B["WASM bridge<br/>(@miden-sdk/wasm-bridge)"]
    B -->|"peer dependency"| C["React SDK<br/>(@miden-sdk/react)"]
    C --> D["Your React app"]
```

The React SDK contains no Rust or WASM code of its own. From your perspective, `npm install @miden-sdk/react @miden-sdk/miden-sdk` is all you need.

## Runtime architecture

At runtime, the SDK introduces three layers between your components and the WASM bridge:

```mermaid
flowchart TB
    subgraph App["Your React app"]
        Components["React components"]
    end

    subgraph ReactSDK["@miden-sdk/react"]
        Hooks["Hooks layer<br/>query + mutation hooks"]
        Store["Zustand store<br/>accounts · notes · sync<br/>asset metadata · loading states"]
        Provider["MidenProvider<br/>client lifecycle · auto-sync<br/>exclusive WASM access"]
    end

    subgraph Worker["Web Worker thread"]
        WASM["Rust client core<br/>(WASM bridge)"]
        WASM --> IDB["IndexedDB<br/>store + keystore"]
        WASM --> RPC["gRPC-web"]
    end

    Components --> Hooks
    Hooks --> Store
    Hooks --> Provider
    Provider -->|"runExclusive()<br/>postMessage"| WASM
    Store -.->|"re-renders on<br/>state change"| Components
    RPC --> Node["Miden node"]
    RPC --> NT["Note transport"]
```

### Provider

`MidenProvider` is the root of the SDK. It:

1. **Initializes the WASM client** — loads the WebAssembly binary, creates the Web Worker, and connects to the configured RPC endpoint
2. **Manages auto-sync** — polls the Miden node at a configurable interval (default: 15s), updating the Zustand store after each sync
3. **Serializes WASM access** — all operations go through `runExclusive()`, which uses an `AsyncLock` to prevent concurrent WASM calls
4. **Detects external signers** — if a `SignerContext` is present above the provider, it creates the client with an external keystore instead of the default local one

### Zustand store

The SDK uses [Zustand](https://zustand.docs.pmnd.rs/) for centralized state:

```mermaid
flowchart LR
    subgraph Hooks["Selector hooks"]
        direction TB
        H1["useAccounts()"]
        H2["useAccount(id)"]
        H3["useNotes()"]
        H4["useSyncState()"]
        H5["useNoteStream()"]
    end

    subgraph Store["MidenStore"]
        direction TB
        Accounts["Account cache"]
        Notes["Note cache"]
        Sync["Sync state"]
        Assets["Asset metadata"]
        Client["Client state"]
    end

    H1 --> Accounts
    H2 --> Accounts
    H2 --> Assets
    H3 --> Notes
    H4 --> Sync
    H5 --> Notes
```

| Store slice | Contents |
|-------------|----------|
| **Account cache** | `accountHeaders`, `accountDetails` |
| **Note cache** | `inputNotes`, `consumableNotes`, `noteFirstSeen` |
| **Sync state** | `syncHeight`, `isSyncing`, `lastSyncTime` |
| **Asset metadata** | `symbol`, `decimals` per faucet ID |
| **Client state** | `isReady`, `isInitializing`, `config` |

Components subscribe to specific slices of the store via selector hooks, so they only re-render when their data changes — not on every sync cycle.

### Hooks layer

Hooks follow two patterns:

**Query hooks** (`useAccounts`, `useAccount`, `useNotes`, `useNoteStream`, `useSyncState`, `useTransactionHistory`, `useAssetMetadata`) read from the Zustand store, auto-fetch on mount if the cache is empty, and re-fetch after each sync.

**Mutation hooks** (`useSend`, `useConsume`, `useMint`, `useSwap`, `useCreateWallet`, `useCreateFaucet`, `useTransaction`) execute transactions through the provider's `runExclusive()`, track progress through stages, and refresh the store on completion.

```mermaid
sequenceDiagram
    participant C as Component
    participant H as useSend()
    participant P as MidenProvider
    participant W as Web Worker (WASM)
    participant N as Miden node

    C->>H: send({ from, to, amount })
    H->>H: stage = "executing"
    H->>P: runExclusive(send)
    P->>W: postMessage
    W->>W: Build request + execute in VM
    H->>H: stage = "proving"
    W->>W: Generate ZK proof
    H->>H: stage = "submitting"
    W->>N: Submit proven transaction
    N-->>W: Transaction ID
    H->>H: stage = "complete"
    H->>P: trigger sync + store refresh
    P-->>C: re-render with updated state
```

## WASM concurrency safety

The WASM bridge is single-threaded — concurrent access from multiple React components causes "recursive use of an object detected" errors. The SDK prevents this with two mechanisms:

1. **`AsyncLock`** — `runExclusive()` queues all WASM operations so only one runs at a time
2. **`isBusyRef`** — mutation hooks use a ref guard to prevent double-invocation (e.g., a user clicking "Send" twice)

WASM object pointers (like `AccountId`) are only valid within the `runExclusive` callback scope. The SDK converts them to stable JavaScript values (strings, numbers) before storing in Zustand.

## External signer integration

The SDK supports third-party wallet providers through a layered context pattern. A single signer provider can sit directly above `MidenProvider`, or multiple signers can be registered via `MultiSignerProvider`:

```mermaid
flowchart TB
    Para["ParaSignerProvider"] --> Multi["MultiSignerProvider<br/>(optional)"]
    Turnkey["TurnkeySignerProvider"] --> Multi
    MidenFi["MidenFiSignerProvider"] --> Multi

    Multi -->|"forwards active SignerContext"| MP["MidenProvider"]
    MP --> App["Your components"]
    MP -.->|"creates client with external keystore"| WASM["Web Worker"]
```

When `MidenProvider` detects a `SignerContext` above it in the tree, it:

1. Uses `WebClient.createClientWithExternalKeystore()` instead of the default constructor
2. Routes signing operations to the signer's `signCb` callback
3. Isolates IndexedDB storage per signer (using the signer's `storeName`)
4. Binds the signer's account configuration to account creation

For multi-signer apps, `MultiSignerProvider` maintains a registry of all `SignerSlot` children and forwards the active signer's context to `MidenProvider`. Users switch signers at runtime via `useMultiSigner().connectSigner(name)`. Single-signer apps can skip `MultiSignerProvider` entirely — a signer provider directly above `MidenProvider` still works.

Built-in signer packages: `@miden-sdk/para`, `@miden-sdk/miden-turnkey-react`, `@miden-sdk/wallet-adapter-react`.

## Configuration

```tsx
<MidenProvider config={{
  rpcUrl: "testnet",           // "devnet" | "testnet" | "localhost" | custom URL
  autoSyncInterval: 15000,     // ms (0 to disable)
  prover: "testnet",           // "local" | "devnet" | "testnet" | custom URL
  noteTransportUrl: "...",     // For private note delivery
  seed: new Uint8Array(32),    // Deterministic RNG seed
}}>
```

The provider also accepts `loadingComponent` and `errorComponent` props for customizing the WASM initialization UI.
