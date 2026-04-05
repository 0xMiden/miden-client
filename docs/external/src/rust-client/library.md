---
title: Library
sidebar_position: 5
---

To use the Miden client library in a Rust project, include it as a dependency.

In your project's `Cargo.toml`, add:

```toml
miden-client = { version = "0.11" }
```

## Client instantiation

The recommended way to create a client is using the `ClientBuilder`. For standard networks, use the pre-configured constructors:

```rust
use std::sync::Arc;
use miden_client::builder::ClientBuilder;
use miden_client_sqlite_store::SqliteStore;

// Create store
let sqlite_store = SqliteStore::new("path/to/store".try_into()?).await?;
let store = Arc::new(sqlite_store);

// Build client for testnet (pre-configured RPC, prover, and note transport)
let client = ClientBuilder::for_testnet()
    .store(store)
    .filesystem_keystore("path/to/keys")?
    .build()
    .await?;
```

Other network constructors are available:
- `ClientBuilder::for_testnet()` - Pre-configured for Miden testnet
- `ClientBuilder::for_devnet()` - Pre-configured for Miden devnet
- `ClientBuilder::for_localhost()` - Pre-configured for local development

For custom configurations, use `ClientBuilder::new()` and configure each component:

```rust
use std::sync::Arc;
use miden_client::builder::ClientBuilder;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client_sqlite_store::SqliteStore;

// Create store
let sqlite_store = SqliteStore::new("path/to/store".try_into()?).await?;
let store = Arc::new(sqlite_store);

// Setup the gRPC endpoint
let endpoint = Endpoint::new("https".into(), "localhost".into(), Some(57291));

let client = ClientBuilder::new()
    .grpc_client(&endpoint, None)
    .store(store)
    .filesystem_keystore("path/to/keys")?
    // Optional: custom prover via .prover(Arc::new(prover))
    // Optional: note transport via .note_transport(Arc::new(nt_client))
    // Optional: debug mode via .in_debug_mode(DebugMode::Enabled)
    .build()
    .await?;
```

## Create local account

With the Miden client, you can create and track any number of public and local accounts. For local accounts, the state is tracked locally, and the rollup only keeps commitments to the data, which in turn guarantees privacy.

The `AccountBuilder` can be used to create a new account with the specified parameters and components. The following code creates a new local account:

```rust
let key_pair = SecretKey::with_rng(client.rng());

let new_account = AccountBuilder::new(init_seed) // Seed should be random for each account
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Private)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), new_account.id()).await?;
client.add_account(&new_account, false).await?;
```
Once an account is created, it is kept locally and its state is automatically tracked by the client.

To create an public account, you can specify `AccountStorageMode::Public` like so:

```Rust
let key_pair = SecretKey::with_rng(client.rng());
let anchor_block = client.get_latest_epoch_block().await.unwrap();

let new_account = AccountBuilder::new(init_seed) // Seed should be random for each account
    .anchor((&anchor_block).try_into().unwrap())
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Public)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), new_account.id()).await?;
client.add_account(&new_account, false).await?;
```

The account's state is also tracked locally, but during sync the client updates the account state by querying the node for the most recent account data.

## Execute transaction

In order to execute a transaction, you first need to define which type of transaction is to be executed. This may be done with the `TransactionRequest` which represents a general definition of a transaction. Some standardized constructors are available for common transaction types.

Here is an example for a `pay-to-id` transaction type:

```rust
// Define asset
let faucet_id = AccountId::from_hex(faucet_id)?;
let fungible_asset = FungibleAsset::new(faucet_id, *amount)?.into();

let sender_account_id = AccountId::from_hex(bob_account_id)?;
let target_account_id = AccountId::from_hex(alice_account_id)?;
let payment_description = PaymentNoteDescription::new(
    vec![fungible_asset.into()],
    sender_account_id,
    target_account_id,
);

let transaction_request = TransactionRequestBuilder::new().build_pay_to_id(
    payment_description,
    None,
    NoteType::Private,
    client.rng(),
)?;

// Execute transaction. No information is tracked after this.
let transaction_execution_result = client.new_transaction(sender_account_id, transaction_request.clone()).await?;

// Prove and submit the transaction, which is stored alongside created notes (if any)
client.submit_transaction(transaction_execution_result).await?
```

You can decide whether you want the note details to be public or private through the `note_type` parameter.
You may also customize the transaction request with the other `TransactionRequestBuilder` methods. This allows you to run custom code, with custom note arguments and additional output/input notes as well.
