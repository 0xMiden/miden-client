# @miden-sdk/vite-plugin

Vite plugin for Miden dApps. Automates WASM deduplication, cross-origin isolation headers, and gRPC-web proxy configuration.

## Installation

```bash
npm install @miden-sdk/vite-plugin --save-dev
# or
yarn add @miden-sdk/vite-plugin --dev
```

## Usage

```typescript
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";

export default defineConfig({
  plugins: [
    midenVitePlugin(), // zero-config: all defaults
    react(),
  ],
});
```

### With Options

```typescript
midenVitePlugin({
  rpcProxyTarget: "https://rpc.testnet.miden.io", // default
  rpcProxyPath: "/rpc.Api",                        // default
  crossOriginIsolation: true,                      // default
  wasmPackages: ["@miden-sdk/miden-sdk"],           // default
});
```

## What It Does

| Config | Purpose |
|--------|---------|
| `resolve.alias` | Force single copy of WASM module (avoids class identity issues) |
| `resolve.dedupe` | Vite deduplication hint |
| `resolve.preserveSymlinks` | Monorepo/symlink support |
| `optimizeDeps.exclude` | Don't pre-bundle WASM packages |
| `server.headers` (COOP/COEP) | SharedArrayBuffer for WASM workers |
| `server.proxy` | gRPC-web CORS bypass in dev |
| `build.target: "esnext"` | Top-level await for WASM |
| `worker.format: "es"` | ES module workers for WASM |

## Options

### `wasmPackages`
- **Type:** `string[]`
- **Default:** `["@miden-sdk/miden-sdk"]`
- Packages to deduplicate and exclude from pre-bundling.

### `crossOriginIsolation`
- **Type:** `boolean`
- **Default:** `true`
- Adds `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers to the dev server. Required for `SharedArrayBuffer` (used by WASM workers).

### `rpcProxyTarget`
- **Type:** `string | false`
- **Default:** `"https://rpc.testnet.miden.io"`
- gRPC-web proxy target URL for the dev server. Set to `false` to disable.

### `rpcProxyPath`
- **Type:** `string`
- **Default:** `"/rpc.Api"`
- Path prefix for gRPC-web proxy requests.

## Requirements

- Vite 5.x or 6.x

## License

MIT
