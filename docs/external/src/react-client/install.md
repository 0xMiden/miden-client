---
title: Installation
sidebar_position: 1
---

# Installation

## Prerequisites

- [Node.js](https://nodejs.org/) v18 or later
- React 18+ (`react` and `react-dom`)
- A WASM-capable browser (Chrome, Firefox, Safari, Edge)

## Install the SDK

```bash
npm install @miden-sdk/react-sdk @miden-sdk/wasm-bridge
# or
yarn add @miden-sdk/react-sdk @miden-sdk/wasm-bridge
```

`@miden-sdk/wasm-bridge` is a peer dependency that provides the WASM bridge and core bindings.

## Verify the installation

Wrap your app in `MidenProvider` and check that initialization succeeds:

```tsx
import { MidenProvider, useMiden } from "@miden-sdk/react-sdk";

function Status() {
  const { isReady, isInitializing, error } = useMiden();

  if (isInitializing) return <div>Initializing WASM...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (isReady) return <div>Miden SDK ready</div>;
  return null;
}

function App() {
  return (
    <MidenProvider config={{ rpcUrl: "testnet" }}>
      <Status />
    </MidenProvider>
  );
}
```

If you see "Miden SDK ready", you're good to go.
