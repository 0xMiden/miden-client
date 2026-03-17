---
title: New Accounts
sidebar_position: 2
---

# Creating Accounts

This guide demonstrates how to create different types of accounts using the Miden Client Rust library.

## Creating a private wallet

Private accounts store state locally — the rollup only keeps commitments, which guarantees privacy.

```rust
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::AuthSecretKey;
use miden_client::crypto::SecretKey;
use miden_objects::account::auth::AuthRpoFalcon512;
use miden_objects::account::component::BasicWallet;

let key_pair = SecretKey::with_rng(client.rng());
let init_seed: [u8; 32] = rand::random();

let new_account = AccountBuilder::new(init_seed)
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Private)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;

// Store the key and register with the client
let keystore = client.keystore();
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), new_account.id()).await?;
client.add_account(&new_account, false).await?;

println!("Account created: {:?}", new_account.id());
```

Once created, the account is tracked locally and its state is automatically updated during sync.

## Creating a public wallet

Public accounts store their state on-chain. Other accounts can read their state directly (useful for foreign procedure invocation).

```rust
let key_pair = SecretKey::with_rng(client.rng());
let anchor_block = client.get_latest_epoch_block().await?;
let init_seed: [u8; 32] = rand::random();

let new_account = AccountBuilder::new(init_seed)
    .anchor((&anchor_block).try_into()?)
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Public)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;

let keystore = client.keystore();
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), new_account.id()).await?;
client.add_account(&new_account, false).await?;
```

:::note
Public accounts require an anchor block from the current epoch. The client fetches this with `get_latest_epoch_block()`. During sync, the client updates public account state by querying the node.
:::

## Creating a mutable wallet

Use `RegularAccountUpdatableCode` to create an account whose code can be updated after deployment:

```rust
let new_account = AccountBuilder::new(init_seed)
    .account_type(AccountType::RegularAccountUpdatableCode)
    .storage_mode(AccountStorageMode::Private)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;
```

## Creating a faucet

Faucets are special accounts that can mint fungible or non-fungible tokens:

```rust
let new_faucet = AccountBuilder::new(init_seed)
    .account_type(AccountType::FungibleFaucet)
    .storage_mode(AccountStorageMode::Public)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    // Faucet-specific components would be added here
    .build()?;
```

## Account types

| Type | Description |
|------|-------------|
| `RegularAccountImmutableCode` | Standard wallet, code cannot be updated |
| `RegularAccountUpdatableCode` | Standard wallet, code can be updated after deployment |
| `FungibleFaucet` | Can mint fungible tokens |
| `NonFungibleFaucet` | Can mint non-fungible tokens |

## Storage modes

| Mode | Description |
|------|-------------|
| `AccountStorageMode::Private` | State tracked locally, only commitments on-chain (default) |
| `AccountStorageMode::Public` | Full state stored on-chain, readable by other accounts |
