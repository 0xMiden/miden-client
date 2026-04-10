---
title: Library
sidebar_position: 0
---

# Library

The React SDK provides hooks organized into two categories: **query hooks** that read state, and **mutation hooks** that execute transactions. Both integrate with the Zustand store for automatic re-rendering.

## Query hooks

| Hook | Purpose | Reference |
|------|---------|-----------|
| `useAccounts()` | List all accounts, categorized into wallets and faucets | [Accounts](./accounts.md) |
| `useAccount(id)` | Get full account details, assets, and balance | [Accounts](./accounts.md) |
| `useNotes(filter?)` | List and filter notes with consumability info | [Notes](./notes.md) |
| `useNoteStream(options?)` | Temporal note tracking with built-in filtering | [Notes](./notes.md) |
| `useSyncState()` | Sync progress, block height, manual sync trigger | [Sync](./sync.md) |
| `useTransactionHistory(options?)` | Transaction history with status tracking | [Transactions](./transactions.md) |
| `useAssetMetadata(assetIds)` | Token metadata (symbol, decimals) by faucet ID | [Accounts](./accounts.md) |

## Mutation hooks

| Hook | Purpose | Reference |
|------|---------|-----------|
| `useCreateWallet()` | Create a new wallet account | [Accounts](./accounts.md) |
| `useCreateFaucet()` | Create a new faucet account | [Accounts](./accounts.md) |
| `useImportAccount()` | Import an account by file, ID, or seed | [Accounts](./accounts.md) |
| `useSend()` | Send tokens to another account | [Transactions](./transactions.md) |
| `useMultiSend()` | Send tokens to multiple recipients | [Transactions](./transactions.md) |
| `useInternalTransfer()` | Transfer between accounts in the same client | [Transactions](./transactions.md) |
| `useConsume()` | Consume specific notes | [Transactions](./transactions.md) |
| `useMint()` | Mint tokens from a faucet | [Transactions](./transactions.md) |
| `useSwap()` | Swap tokens between two faucets | [Transactions](./transactions.md) |
| `useTransaction()` | Execute an arbitrary transaction request | [Transactions](./transactions.md) |
| `useWaitForCommit()` | Wait for a transaction to be committed | [Sync](./sync.md) |
| `useWaitForNotes()` | Wait for notes to arrive for an account | [Sync](./sync.md) |

## Provider and context

| Hook/Component | Purpose | Reference |
|----------------|---------|-----------|
| `MidenProvider` | Root provider — client lifecycle, auto-sync, WASM access | [Provider](./provider.md) |
| `useMiden()` | Access client, sync function, `runExclusive()` | [Provider](./provider.md) |
| `useSigner()` | Access external signer connection state | [Signers](./signers.md) |
| `useMultiSigner()` | List, connect, and switch between multiple signers | [Signers](./signers.md) |
| `MultiSignerProvider` | Registry for multiple signer providers | [Signers](./signers.md) |
| `SignerSlot` | Registers a signer into the multi-signer registry | [Signers](./signers.md) |
| `SignerContext` | Context for custom signer integration | [Signers](./signers.md) |

## Utilities

| Utility | Purpose | Reference |
|---------|---------|-----------|
| `formatAssetAmount()` / `parseAssetAmount()` | Amount formatting and parsing | [Utilities](./utilities.md) |
| `getNoteSummary()` / `formatNoteSummary()` | Note summary extraction and display | [Utilities](./utilities.md) |
| `readNoteAttachment()` / `createNoteAttachment()` | Note attachment encoding/decoding | [Utilities](./utilities.md) |
| `normalizeAccountId()` / `accountIdsEqual()` | Account ID normalization and comparison | [Utilities](./utilities.md) |
| `MidenError` / `wrapWasmError()` | Typed error handling | [Utilities](./utilities.md) |
