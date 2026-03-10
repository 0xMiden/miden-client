---
title: Create account
sidebar_position: 2
---

In this tutorial, you'll create a Miden Client, generate a new account, request tokens from the public faucet, and consume the resulting note — all using the Rust library API.

## Set up the project

1. Create a new Rust project:

   ```sh
   cargo new miden-getting-started
   cd miden-getting-started
   ```

2. Add dependencies to `Cargo.toml`:

   ```toml
   [dependencies]
   miden-client = { version = "0.14", features = ["tonic"] }
   miden-client-sqlite-store = { version = "0.14" }
   tokio = { version = "1", features = ["full"] }
   rand = "0.8"
   ```

## Create the client

Instantiate a client connected to the Miden testnet:

```rust
use std::sync::Arc;
use miden_client::builder::ClientBuilder;
use miden_client_sqlite_store::SqliteStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a SQLite store for local state persistence
    let sqlite_store = SqliteStore::new("store.sqlite3".try_into()?).await?;
    let store = Arc::new(sqlite_store);

    // Build a client pre-configured for testnet
    let mut client = ClientBuilder::for_testnet()
        .store(store)
        .filesystem_keystore("keystore")?
        .build()
        .await?;

    Ok(())
}
```

## Create a new account

Use `AccountBuilder` to create a private wallet account:

```rust
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::AuthSecretKey;
use miden_client::crypto::SecretKey;
use miden_objects::account::auth::AuthRpoFalcon512;
use miden_objects::account::component::BasicWallet;

// Generate a key pair for the account
let key_pair = SecretKey::with_rng(client.rng());

// Generate a random seed for the account
let init_seed: [u8; 32] = rand::random();

// Build the account
let new_account = AccountBuilder::new(init_seed)
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Private)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;

// Store the key and register the account with the client
let keystore = client.keystore();
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), new_account.id()).await?;
client.add_account(&new_account, false).await?;

println!("Account created: {:?}", new_account.id());
```

Save the printed account ID — you'll need it for the faucet step.

## Request tokens from the public faucet

1. Navigate to the [Miden faucet website](https://faucet.testnet.miden.io/).
2. Paste your account ID into the **Request test tokens** field.
3. Choose an amount and click **Send Private Note**.
4. Download the `note.mno` file.

:::tip
You can also click **Send Public Note**. If you do, the note details will be public and you can skip the import step — just sync the client directly.
:::

## Import and sync

If you received a private note file, import it:

```rust
use std::path::Path;

// Import the note from the downloaded file
client.import_note_from_file(Path::new("path/to/note.mno")).await?;
```

Sync the client to confirm the note exists on-chain:

```rust
let sync_summary = client.sync_state().await?;
println!("Synced to block {}", sync_summary.block_num);
```

After syncing, the note's status changes from `Expected` to `Committed`, confirming it is the result of a valid on-chain transaction.

## Consume the note

Consume all available notes for your account to receive the faucet tokens:

```rust
use miden_client::transaction::TransactionRequestBuilder;

// Get consumable notes for the account
let consumable_notes = client.get_consumable_notes(Some(new_account.id())).await?;
let note_ids: Vec<_> = consumable_notes.iter().map(|n| n.note.id()).collect();

if !note_ids.is_empty() {
    // Build a consume-notes transaction
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(new_account.id(), note_ids)?;

    // Execute and submit the transaction (this generates a ZK proof)
    let tx_result = client.new_transaction(new_account.id(), tx_request).await?;
    client.submit_transaction(tx_result).await?;

    println!("Notes consumed — ZK proof generated and submitted!");
}
```

## Verify the balance

Sync again to confirm the transaction, then check your account balance:

```rust
client.sync_state().await?;

let (account, _) = client.get_account(new_account.id()).await?;
println!("Account vault: {:?}", account.vault());
```

You should see the faucet tokens in your account's vault.

## Congratulations!

You've created a Miden Client, generated an account, and executed your first client-side zero-knowledge proof using the Rust library. Next, try transferring assets between accounts:

- [Public peer-to-peer transfer](./p2p-public.md)
- [Private peer-to-peer transfer](./p2p-private.md)

## Debugging tips

- All local state is stored in the SQLite database (the path you passed to `SqliteStore::new`). Delete the file for a fresh start.
- Enable debug mode when building the client: `.in_debug_mode(DebugMode::Enabled)` on `ClientBuilder`.
