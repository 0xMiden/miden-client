---
title: Accounts
sidebar_position: 1
---

# Retrieving Accounts with the Miden SDK

This guide demonstrates how to retrieve and work with existing accounts using the Miden SDK.

## Retrieving a Single Account

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Get account by hex string, bech32 string, or AccountId object
    const account = await client.accounts.get("0x1234...");

    if (!account) {
        console.log("Account not found");
        return;
    }

    console.log(account.id().toString());
    console.log(account.nonce().toString());
    console.log(account.isPublic());
    console.log(account.isFaucet());
} catch (error) {
    console.error("Failed to retrieve account:", error.message);
}
```

## Get or Import an Account

`getOrImport()` returns the locally stored account if it exists, or imports it from the network if it does not. This is a convenience wrapper around `get()` + `import()` and is the recommended way to load a public account you may or may not already have locally:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // If the account is already local, returns it immediately.
    // If not, fetches its public state from the network and stores it.
    const account = await client.accounts.getOrImport("mtst1arjemrxne8lj5qz4mg9c8mtyxg954483");
    console.log("Account nonce:", account.nonce().toString());
} catch (error) {
    console.error("Failed:", error.message);
}
```

This is particularly useful when working with contracts deployed by other parties — you do not need to know whether the account has been imported before.

## Listing All Accounts

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();
    const accounts = await client.accounts.list();

    for (const header of accounts) {
        console.log(header.id().toString());
        console.log(header.nonce().toString());
    }
} catch (error) {
    console.error("Failed to retrieve accounts:", error.message);
}
```

## Getting Account Details

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Returns { account, vault, storage, code, keys } in a single call
    const details = await client.accounts.getDetails("0x1234...");

    console.log(details.account.id().toString());
    console.log(details.vault);
    console.log(details.storage);
    console.log(details.code);
    console.log(details.keys);
} catch (error) {
    console.error("Failed to get account details:", error.message);
}
```

## Checking Account Balance

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Quick balance check (wraps accountReader)
    const balance = await client.accounts.getBalance("0xACCOUNT...", "0xFAUCET...");
    console.log(`Balance: ${balance}`);
} catch (error) {
    console.error("Failed to get balance:", error.message);
}
```

## Address Management

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    await client.accounts.addAddress("0xACCOUNT...", "mtst1address...");
    await client.accounts.removeAddress("0xACCOUNT...", "mtst1address...");
} catch (error) {
    console.error("Failed to manage address:", error.message);
}
```

> **Note:** `get()` returns `null` when an account is not found. All other methods (`getDetails()`, `getBalance()`, `export()`) throw `"Account not found: 0x..."` if the account doesn't exist.
