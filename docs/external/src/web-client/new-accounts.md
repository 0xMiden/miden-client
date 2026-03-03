---
title: New-accounts
sidebar_position: 2
---

# Creating Accounts with the Miden SDK

This guide demonstrates how to create and work with different types of accounts using the Miden SDK.

## Creating a Regular Wallet Account

```typescript
import { MidenClient, AccountType, AuthScheme } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Default wallet (private storage, mutable, Falcon auth)
    const wallet = await client.accounts.create();

    // Wallet with custom options
    const wallet2 = await client.accounts.create({
        storage: "public",                          // "private" or "public"
        type: AccountType.ImmutableWallet,            // AccountType.MutableWallet (default) or AccountType.ImmutableWallet
        auth: AuthScheme.ECDSA,                     // AuthScheme.Falcon (default) or AuthScheme.ECDSA
        seed: "my-seed"                             // Optional deterministic seed (auto-hashed)
    });

    // Access account properties
    console.log(wallet.id().toString());      // Unique identifier (hex)
    console.log(wallet.nonce().toString());   // Current nonce (starts at 0)
    console.log(wallet.isPublic());           // false
    console.log(wallet.isPrivate());          // true
    console.log(wallet.isFaucet());           // false
    console.log(wallet.isRegularAccount());   // true
} catch (error) {
    console.error("Failed to create wallet:", error.message);
}
```

## Creating a Faucet Account

```typescript
import { MidenClient, AccountType, AuthScheme } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Create faucet — only required fields
    const faucet = await client.accounts.create({
        type: AccountType.FungibleFaucet,
        symbol: "TEST",
        decimals: 8,
        maxSupply: 10_000_000n   // Accepts number or bigint
    });

    // With custom options
    const faucet2 = await client.accounts.create({
        type: AccountType.FungibleFaucet,
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
        storage: "public",
        auth: AuthScheme.Falcon
    });

    console.log(faucet.id().toString());
    console.log(faucet.isFaucet());          // true
    console.log(faucet.isRegularAccount());  // false
} catch (error) {
    console.error("Failed to create faucet:", error.message);
}
```
