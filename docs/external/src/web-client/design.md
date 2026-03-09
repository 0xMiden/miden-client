---
title: Design
sidebar_position: 4
---

# Design

This page explains how the TypeScript SDK is structured and how its pieces fit together at runtime.

## How the SDK reaches the browser

The TypeScript SDK is a pure TypeScript package that wraps the [WASM bridge](../index.md) — a separate layer that compiles the [Rust client core](../rust-client/design.md) to WebAssembly and bundles it with browser infrastructure (Web Workers, IndexedDB, JS glue). The SDK itself contains no Rust or WASM code; it consumes the bridge as an npm dependency and provides the idiomatic TypeScript API you interact with.


```mermaid
flowchart LR
    A["Rust core<br/>(miden-client)"] -->|"compiled to WASM"| B["WASM bridge<br/>(@miden-sdk/wasm-bridge)"]
    B -->|"npm dependency"| C["TypeScript SDK<br/>(@miden-sdk/ts-sdk)"]
    C --> D["Your application"]
```


From your perspective, `npm install` is all you need — the WASM binary, Web Worker script, and all bindings are bundled transitively.

## Runtime architecture

At runtime, the SDK splits work across two threads:


```mermaid
flowchart TB
    subgraph Main["Main thread (your app)"]
        App["Application code"] --> API["MidenClient API<br/>(thin proxy)"]
    end

    subgraph Worker["Web Worker thread"]
        Core["Rust client core<br/>(WASM)"]
        Core --> Store["IndexedDB<br/>store"]
        Core --> RPC["gRPC-web<br/>RPC client"]
        Core --> KS["IndexedDB<br/>keystore"]
    end

    API <-->|"postMessage"| Core
    RPC <-->|"gRPC-web"| Node["Miden node"]
    RPC <-->|"gRPC-web"| NT["Note transport<br/>network"]
```


**Why a Web Worker?** Transaction proving and MASM compilation are CPU-intensive. Running them on the main thread would freeze the UI. The Web Worker keeps your application responsive while heavy operations run in the background.

Every `MidenClient` instance owns one Web Worker. The TypeScript API you interact with is a thin proxy that serializes calls to the worker via `postMessage` and returns the results as promises.

## Client API surface

The `MidenClient` exposes a resource-based API — each domain area is grouped under a namespace:


```mermaid
flowchart TB
    Client["MidenClient"]
    Client --> Accounts["client.accounts"]
    Client --> Transactions["client.transactions"]
    Client --> Notes["client.notes"]
    Client --> Compile["client.compile"]
    Client --> Tags["client.tags"]
    Client --> Sync["client.sync()"]

    Accounts ~~~ A1["create · get · list<br/>getDetails · getBalance<br/>import · export<br/>addAddress · removeAddress"]
    Transactions ~~~ T1["mint · send · consume<br/>consumeAll · swap · execute<br/>submit · preview<br/>list · waitFor"]
    Notes ~~~ N1["get · list · listSent<br/>listAvailable<br/>import · export<br/>sendPrivate · fetchPrivate"]
    Compile ~~~ C1["component · txScript"]
    Tags ~~~ TG1["add · remove · list"]
```


| Namespace | Responsibility |
|-----------|---------------|
| **`client.accounts`** | Account lifecycle: create wallets/faucets/contracts, retrieve state, check balances, import/export, manage addresses |
| **`client.transactions`** | Transaction lifecycle: build, execute, prove, submit, and track transactions. Supports mint, send, consume, swap, custom MASM scripts, and FPI |
| **`client.notes`** | Note management: list/filter input and output notes, import/export, send and fetch private notes via the note transport network |
| **`client.compile`** | MASM compilation: compile account components and transaction scripts in the browser, with library linking support |
| **`client.tags`** | Tag management: add/remove note tags that control which notes the client discovers during sync |
| **`client.sync()`** | State synchronization: pull the latest state from the Miden node and update local data |

Top-level methods like `client.exportStore()`, `client.importStore()`, and `client.terminate()` handle store backup/restore and worker lifecycle.

## Transaction lifecycle

When you call a high-level method like `client.transactions.send()`, the SDK handles the full lifecycle automatically:


```mermaid
sequenceDiagram
    participant App as Your app
    participant API as MidenClient
    participant Worker as Web Worker (WASM)
    participant Node as Miden node

    App->>API: transactions.send({ ... })
    API->>Worker: postMessage(send request)
    Worker->>Worker: 1. Build TransactionRequest
    Worker->>Worker: 2. Execute in Miden VM
    Worker->>Worker: 3. Generate ZK proof
    Worker->>Node: 4. Submit proven transaction
    Node-->>Worker: Transaction ID
    Worker->>Worker: 5. Store transaction + output notes
    Worker-->>API: TransactionId
    API-->>App: TransactionId
```


Steps 2–3 are the most expensive. For complex transactions, you can offload proving to a remote prover by passing `proverUrl` at client creation or a `prover` per-transaction.

## Persistence

The SDK uses [IndexedDB](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API) for all browser-side persistence — no server or filesystem needed.

**Store** — persists the client's view of the blockchain:
- Account state (history, vault assets, code)
- Transactions and their scripts
- Input and output notes
- Note tags
- Block headers and chain data required for transaction execution

Because Miden supports off-chain execution and proving, the client only stores the blockchain history intervals relevant to its tracked accounts — not the entire chain.

**Keystore** — persists private keys for tracked accounts, stored separately from the main store. Keys are used to sign transactions during execution.

Both databases are scoped to the browser origin, so different domains cannot access each other's data.

## Network communication

The SDK communicates with two external services:

| Service | Protocol | Purpose |
|---------|----------|---------|
| **Miden node** | gRPC-web | Sync state, submit transactions, fetch public account data |
| **Note transport network** | gRPC-web | Exchange private notes between users |

Standard gRPC uses HTTP/2 features that browsers don't expose directly. The SDK uses [gRPC-web](https://github.com/grpc/grpc-web), which works over HTTP/1.1 and HTTP/2, making it compatible with browser environments. A gRPC-web proxy (like Envoy) may be required in front of the Miden node depending on the deployment.

## Resource management

Each `MidenClient` holds a Web Worker thread and IndexedDB connections. In long-running applications (e.g., a wallet that switches between networks), it's important to release these resources:

```typescript
// Explicit cleanup
client.terminate();

// Or automatic cleanup via TC39 Explicit Resource Management
{
  using client = await MidenClient.create();
  // client.terminate() called automatically at end of scope
}
```

After `terminate()`, all method calls throw `Error("Client terminated")`.
