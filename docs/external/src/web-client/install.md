---
title: Installation
sidebar_position: 1
---

# Installation

## Prerequisites

- [Node.js](https://nodejs.org/) v18 or later
- A WASM-capable browser (Chrome, Firefox, Safari, Edge)

## Install the SDK

```bash
npm install @miden-sdk/miden-sdk
# or
yarn add @miden-sdk/miden-sdk
```

## Verify the installation

Create a quick test to confirm everything works:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

const client = await MidenClient.create();
console.log("Miden SDK initialized successfully");
client.terminate();
```

If the client initializes without errors, you're ready to go.

:::tip
Use `MidenClient.createTestnet()` to connect to the Miden testnet with sensible defaults (auto-sync enabled, testnet RPC endpoint pre-configured).
:::
