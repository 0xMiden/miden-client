# @miden-sdk/miden-sdk

## Overview

The `@miden-sdk/miden-sdk` is a comprehensive software development toolkit (SDK) for interacting with the Miden blockchain and virtual machine from within a web application. It provides developers with everything needed to:

- Interact with the Miden chain (e.g. syncing accounts, submitting transactions)
- Create and manage Miden transactions
- Run the Miden VM to execute programs
- Generate zero-knowledge proofs using the Miden Prover (with support for delegated proving)
- Integrate Miden capabilities seamlessly into browser-based environments

Whether you're building a wallet, dApp, or other blockchain-integrated application, this SDK provides the core functionality to bridge your frontend with Miden's powerful ZK architecture.

> **Note:** This README provides a high-level overview of the web client SDK.
> For more detailed documentation, API references, and usage examples, see the documentation [here](../../docs/src/web-client) (TBD).

### SDK Structure and Build Process

This SDK is published as an NPM package, built from the `web-client` crate. The `web-client` crate is a Rust crate targeting WebAssembly (WASM), and it uses `wasm-bindgen` to generate JavaScript bindings. It depends on the lower-level `rust-client` crate, which implements the core functionality for interacting with the Miden chain.

Both a `Cargo.toml` and a `package.json` are present in the `web-client` directory to support Rust compilation and NPM packaging respectively.

The build process is powered by a custom `rollup.config.js` file, which orchestrates three main steps:

1. **WASM Module Build**: Compiles the `web-client` Rust crate into a WASM module using `@wasm-tool/rollup-plugin-rust`, enabling WebAssembly features such as atomics and bulk memory operations.

2. **Worker Build**: Bundles a dedicated web worker file that enables off-main-thread execution for computationally intensive functions.

3. **Main Entry Point Build**: Bundles the top-level JavaScript module (`index.js`) which serves as the main API surface for consumers of the SDK. This module also imports `wasm.js`, which
   provides a function to load the wasm module in an async way. Since there's a [known issue](https://github.com/wasm-tool/rollup-plugin-rust?tab=readme-ov-file#usage-with-vite)
   with vite, there's a check to avoid loading the wasm module when SSR is enabled.

This setup allows the SDK to be seamlessly consumed in JavaScript environments, particularly in web applications.

## Installation

### Stable Version

A non-stable version of the SDK is also maintained, which tracks the `next` branch of the Miden client repository (essentially the development branch). To install the pre-release version, run:

```javascript
npm i @miden-sdk/miden-sdk
```

Or using Yarn:

```javascript
yarn add @miden-sdk/miden-sdk
```

### Pre-release ("next") Version

A non-stable version is also maintained. To install the pre-release version, run:

```javascript
npm i @miden-sdk/miden-sdk@next
```

Or with Yarn:

```javascript
yarn add @miden-sdk/miden-sdk@next
```

> **Note:** The `next` version of the SDK must be used in conjunction with a locally running Miden node built from the `next` branch of the `miden-node` repository. This is necessary because the public testnet runs the stable `main` branch, which may not be compatible with the latest development features in `next`. Instructions to run a local node can be found [here](https://github.com/0xMiden/miden-node/tree/next) on the `next` branch of the `miden-node` repository. Additionally, if you plan to leverage delegated proving in your application, you may need to run a local prover (see [Remote prover instructions](https://github.com/0xMiden/miden-node/tree/next/bin/remote-prover)).

## Building and Testing the Web Client

If you're interested in contributing to the web client and need to build it locally, you can do so via:

```
yarn install
yarn build
```

This will:

- Install all JavaScript dependencies,
- Compile the Rust code to WebAssembly,
- Generate the JavaScript bindings via wasm-bindgen,
- And bundle the SDK into the dist/ directory using Rollup.

To run integration tests after building, use:

```
yarn test
```

This runs a suite of integration tests to verify the SDK’s functionality in a web context.

### Building the npm package

Follow the steps below to produce the contents that get published to npm (`dist/` plus the license file). All commands are executed from `crates/web-client`.

1. **Install prerequisites**
   - Install the Rust toolchain version specified in `rust-toolchain.toml`.
   - Install Node.js ≥18 and Yarn.
2. **Install dependencies**
   ```bash
   yarn install
   ```
   This installs both the JavaScript tooling and the `@wasm-tool/rollup-plugin-rust` dependency that compiles the Rust crate.
3. **Build the package**
   ```bash
   yarn build
   ```
   The `build` script (see `package.json`) performs the following:
   - Removes the previous `dist/` directory (`rimraf dist`).
   - Runs `npm run build-rust-client-js`, which builds the `idxdb-store` TypeScript helper that the SDK imports.
   - Invokes Rollup with `RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""` so the Rust `getrandom` crate targets browser entropy and so that atomics/bulk-memory WebAssembly features are enabled.
   - Copies the generated TypeScript declarations from `js/types` into `dist/`.
   - Executes `node clean.js` to strip paths from the generated `.js` files, leaving only the artifacts needed on npm.
4. **Inspect the artifacts**
   - `dist/index.js` is the ESM entry point referenced by `"main"`/`"browser"`/`"exports"`.
   - `dist/index.d.ts` and the rest of the `.d.ts` files provide the TypeScript surface.
   Use `npm pack` if you want to preview the exact tarball that would be published.

> Tip: during development you can set `MIDEN_WEB_DEV=true` before running `yarn build` (or run `npm run build-dev`) to skip the clean step and keep extra debugging metadata in the bundled output. This debugging metadata also includes debug symbols for the generated wasm binary

### Checking the generated TypeScript bindings

The script at `crates/web-client/scripts/check-bindgen-types.js` verifies that every type exported by the generated wasm bindings (`dist/crates/miden_client_web.d.ts`) is re-exported from the public declarations (`js/types/index.d.ts`). Run it after a build with:

```
yarn check:wasm-types
```

`WebClient` is intentionally excluded because the wrapper defines its own implementation. If the check reports missing exports, update `js/types/index.d.ts` so consumers get the full generated surface.

## Usage

The following are just a few simple examples to get started. For more details, see the [API Reference](../../docs/typedoc/web-client/README.md).

### Quick Start

```typescript
import { MidenClient, AuthScheme } from "@miden-sdk/miden-sdk";

// 1. Create client (defaults to testnet)
const client = await MidenClient.create();

// 2. Create a wallet and a token (faucet account)
const wallet = await client.accounts.create();
const dagToken = await client.accounts.create({
  type: "faucet", symbol: "DAG", decimals: 8, maxSupply: 10_000_000n
});

// 3. Mint tokens to the wallet (mint + sync + consume in one call)
await client.transactions.mintAndConsume({ faucet: dagToken, to: wallet, amount: 1000n });

// 4. Send tokens to another address
await client.transactions.send({
  account: wallet,
  to: "0xBOB",
  token: dagToken,
  amount: 100n
});

// 5. Check balance
const balance = await client.accounts.getBalance(wallet, dagToken);
console.log(`Balance: ${balance}`); // 900n

// 6. Cleanup
client.terminate();
```

### Create a New Wallet

```typescript
import { MidenClient, AuthScheme } from "@miden-sdk/miden-sdk";

const client = await MidenClient.create();

// Default wallet (private storage, mutable, Falcon auth)
const wallet = await client.accounts.create();

// Wallet with options
const wallet2 = await client.accounts.create({
  storage: "public",
  mutable: false,
  auth: AuthScheme.ECDSA,
  seed: "deterministic"
});

console.log(wallet.id().toString()); // account id as hex
console.log(wallet.isPublic()); // false
console.log(wallet.isPrivate()); // true
console.log(wallet.isFaucet()); // false
```

### Create a Faucet

```typescript
const faucet = await client.accounts.create({
  type: "faucet",
  symbol: "DAG",
  decimals: 8,
  maxSupply: 10_000_000n
});

console.log(faucet.id().toString());
console.log(faucet.isFaucet()); // true
```

### Send Tokens

```typescript
const txId = await client.transactions.send({
  account: wallet,
  to: "0xBOB",
  token: dagToken,
  amount: 100n
});
```

### Consume Notes

```typescript
// Sync state to discover new notes
await client.sync();

// Consume all available notes for an account
const result = await client.transactions.consumeAll({ account: wallet });
console.log(`Consumed ${result.consumed} notes, ${result.remaining} remaining`);
```

### Check Balance

```typescript
const balance = await client.accounts.getBalance(wallet, dagToken);
console.log(`Balance: ${balance}`);
```

### Cleanup

When you're finished using a MidenClient instance, call `terminate()` to release its Web Worker:

```typescript
client.terminate();

// Or use explicit resource management:
{
  using client = await MidenClient.create();
  // ... use client ...
} // client.terminate() called automatically
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
