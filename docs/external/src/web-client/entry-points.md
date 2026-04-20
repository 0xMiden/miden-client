---
title: Entry points (eager vs lazy)
sidebar_position: 0.5
---

# Entry Points: Eager (default) and `/lazy`

The SDK ships two entry points with an **identical public API**. They differ only in **when** the WebAssembly module is initialized.

| Import path                 | WASM initializes at                                                  | When to use                                                        |
| --------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `@miden-sdk/miden-sdk`      | Module evaluation (top-level `await`)                                | Plain browser apps, Vite, CRA, esbuild, Webpack client bundles     |
| `@miden-sdk/miden-sdk/lazy` | First `MidenClient.ready()` / first `await`-ing SDK method           | Next.js / SSR, Capacitor WKWebView, framework adapters (React SDK) |

Picking the wrong entry for your environment can hang module evaluation, so this choice matters. Once an entry is picked, day-to-day code is the same on both.

## How each entry works

### Eager (default)

The default entry (`dist/eager.js`) does this at the very top of its module body:

```js
await getWasmOrThrow();
```

That top-level `await` resolves before any `import` statement referencing the module completes. As a result, once you've imported anything from `@miden-sdk/miden-sdk`, every wasm-bindgen constructor is safe to call **synchronously** on the next line — no `await MidenClient.ready()`, no `isReady` gate.

```typescript
import { AccountId, Felt, TransactionProver } from "@miden-sdk/miden-sdk";

// All three lines run synchronously, immediately after the import resolves.
const id = AccountId.fromHex("0x…");
const felt = new Felt(42n);
const prover = TransactionProver.newLocalProver();
```

### Lazy (`/lazy`)

The lazy entry (`dist/index.js`) does **not** run any top-level `await`. WebAssembly is loaded on demand — the first call to `MidenClient.ready()`, or the first async SDK method (`createTestnet`, `accounts.get`, `transactions.send`, etc.) triggers initialization.

Because nothing has awaited at import time, calling a bare wasm-bindgen constructor straight after the import will throw (`wasm.accountid_fromHex` is undefined). You must await readiness first:

```typescript
import { MidenClient, AccountId, Felt } from "@miden-sdk/miden-sdk/lazy";

await MidenClient.ready(); // initializes WASM if not already started
const id = AccountId.fromHex("0x…"); // now safe
const felt = new Felt(42n);
```

Async SDK methods gate on readiness internally, so if you only use those you don't have to call `ready()` explicitly:

```typescript
const client = await MidenClient.createTestnet(); // awaits WASM init internally
await client.sync();
const accounts = await client.accounts.list();
```

## Why `/lazy` exists

Two environments fail under top-level `await`:

- **Next.js / SSR** — TLA blocks server-side module evaluation. The server never finishes rendering the page, and the request times out.
- **Capacitor WKWebView hosts (Miden Wallet iOS / Android)** — the `capacitor://localhost` custom-scheme handler interacts poorly with TLA in the main WebView. The same TLA resolves in under 100&nbsp;ms in a dApp WebView (vanilla HTTPS), but hangs indefinitely in the Capacitor host. Verified empirically against the wallet's iOS E2E suite on devnet: TLA → 1 fail + 6 skipped; `/lazy` → 7/7 pass.

If your environment is neither of those, prefer the default (eager) entry — it eliminates the `await ready()` ceremony.

## `MidenClient.ready()` is idempotent

`ready()` is a thin alias for the internal WASM loader, which memoizes both the in-flight promise and the resolved module:

- **Concurrent callers** share the same in-flight promise (no duplicate work, no race).
- **Post-init callers** resolve immediately from the cached module.

This means it's safe to call from multiple places without coordination — a framework adapter (`MidenProvider`), a tutorial helper, and application code can all call `MidenClient.ready()` independently and they all observe the same single initialization.

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk/lazy";

// Concurrent — both awaits resolve from the same underlying promise.
await Promise.all([MidenClient.ready(), MidenClient.ready()]);

// Later calls resolve synchronously from cache.
await MidenClient.ready(); // no-op
```

On the eager entry, `ready()` also works — it just resolves immediately because WASM was already initialized by the module's top-level `await`.

## Example: Next.js (App Router)

```tsx
// app/page.tsx
"use client";

import { useEffect, useState } from "react";
import { MidenClient } from "@miden-sdk/miden-sdk/lazy";

export default function Page() {
  const [height, setHeight] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      // Optional — createTestnet awaits WASM internally. Call ready()
      // explicitly if you plan to construct wasm-bindgen types before
      // the first async client method.
      await MidenClient.ready();

      const client = await MidenClient.createTestnet();
      const h = await client.getSyncHeight();
      if (!cancelled) setHeight(h);
      client.terminate();
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return <div>Sync height: {height ?? "…"}</div>;
}
```

Importing from the default `@miden-sdk/miden-sdk` in a Next.js server component or API route will hang the server — always use `/lazy` in Next.js.

## Example: Capacitor iOS / Android

Inside a Capacitor shell, the main WebView is served under `capacitor://localhost`. Always import from `/lazy` here; the default entry will lock up the WebView on first import.

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk/lazy";

await MidenClient.ready();
const client = await MidenClient.createTestnet();
```

Note that dApp WebViews opened inside the Capacitor shell use vanilla HTTPS and are not subject to the TLA hang — a dApp bundle can safely use the default eager entry. Only the Capacitor host bundle itself must use `/lazy`.

## Framework adapters

`@miden-sdk/react` ships both variants (`@miden-sdk/react` and `@miden-sdk/react/lazy`), built from a single source tree. Internally both import `@miden-sdk/miden-sdk/lazy` because the React provider manages WASM readiness via its own `isReady` state. See the [React SDK README](https://github.com/0xMiden/miden-client/blob/main/packages/react-sdk/README.md) for details.

## Quick reference

```text
Plain browser app (Vite, CRA, Webpack):
    import { ... } from "@miden-sdk/miden-sdk";

Next.js (any rendering mode):
    import { ... } from "@miden-sdk/miden-sdk/lazy";
    + await MidenClient.ready() before sync wasm-bindgen calls

Capacitor iOS / Android host shell:
    import { ... } from "@miden-sdk/miden-sdk/lazy";
    + await MidenClient.ready() before sync wasm-bindgen calls

React app (default):
    import { MidenProvider, ... } from "@miden-sdk/react";

React app (Next.js / Capacitor host):
    import { MidenProvider, ... } from "@miden-sdk/react/lazy";
```
