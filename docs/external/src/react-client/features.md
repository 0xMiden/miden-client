---
title: Features
sidebar_position: 2
---

# Features

The Miden React SDK provides a complete toolkit for building React applications on the Miden rollup.

### Query hooks

Read blockchain state with auto-refreshing hooks that update after every sync: `useAccounts`, `useAccount`, `useNotes`, `useNoteStream`, `useSyncState`, `useTransactionHistory`, and `useAssetMetadata`.

### Mutation hooks

Build and submit transactions through hooks that manage the full execute → prove → submit lifecycle with built-in loading stages and error handling: `useSend`, `useMultiSend`, `useConsume`, `useMint`, `useSwap`, `useCreateWallet`, `useCreateFaucet`, and `useTransaction`.

### Automatic state management

A Zustand store manages all client state — accounts, notes, sync progress, asset metadata — and re-renders components efficiently when data changes. Background auto-sync keeps state fresh (configurable interval, default 15 seconds).

### Transaction staging

Every mutation hook exposes a `stage` property that tracks progress through `"idle"` → `"executing"` → `"proving"` → `"submitting"` → `"complete"`, enabling fine-grained UI feedback during long-running operations.

### External signer integration

First-class support for third-party wallet providers (Para, Turnkey, MidenFi) through a `SignerContext` that plugs into the provider tree. The SDK handles keystore delegation, account binding, and IndexedDB isolation automatically.

### WASM concurrency safety

The SDK serializes all WASM operations through an `AsyncLock` and `runExclusive()` pattern, preventing "recursive use of an object detected" errors that occur when multiple React components access the WASM bridge concurrently.

### Browser-native

Like the [TypeScript SDK](../web-client/index.md), all computation (proving, syncing, compilation) runs in a Web Worker off the main thread, keeping the React render loop responsive.
