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

## Creating a Custom Contract Account

Custom contracts are regular accounts whose code is provided by the caller. Use `AccountType.ImmutableContract` (code cannot be updated after deployment) or `AccountType.MutableContract` (code can be updated).

### Compiling the contract component

Use `client.compile.component()` to compile Miden Assembly (MASM) code into an `AccountComponent`. See the [Compiler resource guide](./compile.md) for full details.

```typescript
import { MidenClient, AccountType, AuthSecretKey, StorageSlot } from "@miden-sdk/miden-sdk";

const counterCode = `
    use miden::protocol::active_account
    use miden::protocol::native_account
    use miden::core::word
    use miden::core::sys

    const COUNTER_SLOT = word("miden::tutorials::counter")

    pub proc get_count
        push.COUNTER_SLOT[0..2] exec.active_account::get_item
        exec.sys::truncate_stack
    end

    pub proc increment_count
        push.COUNTER_SLOT[0..2] exec.active_account::get_item
        add.1
        push.COUNTER_SLOT[0..2] exec.native_account::set_item
        exec.sys::truncate_stack
    end
`;

try {
    const client = await MidenClient.create();

    // Compile the MASM component (fresh CodeBuilder per call)
    const component = await client.compile.component({
        code: counterCode,
        slots: [StorageSlot.emptyValue("miden::tutorials::counter")]
    });

    // Generate a seed for deterministic account ID derivation
    const seed = crypto.getRandomValues(new Uint8Array(32));

    // auth must be kept by the caller for signing
    const auth = AuthSecretKey.rpoFalconWithRNG(seed);

    // Create the contract — storage defaults to "public" for contracts
    const contract = await client.accounts.create({
        type: AccountType.ImmutableContract,
        seed,
        auth,
        components: [component]
    });

    console.log("Contract ID:", contract.id().toString());
    console.log("Is public:", contract.isPublic());   // true
} catch (error) {
    console.error("Failed to create contract:", error.message);
}
```

### Storage mode

Unlike wallets (which default to `"private"`), contracts default to `"public"` so other accounts can read their state for foreign procedure invocation (FPI). Pass `storage: "private"` to override.

### Auth key

`auth` must be a concrete `AuthSecretKey` object (not a string scheme). The caller must retain it — the client uses it for signing during `transactions.execute()`.

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
