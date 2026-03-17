---
title: Peer-to-peer transfer
sidebar_position: 3
---

In this tutorial, you'll transfer assets between two accounts using public notes and the Rust library API.

:::info Prerequisite
Complete the [Create account](./create-account-use-faucet.md) tutorial first. You should have a funded account (Account A) and a working client.
:::

## Create a second account

Create a public account (Account C) so its state can be retrieved from the network:

```rust
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::AuthSecretKey;
use miden_client::crypto::SecretKey;
use miden_objects::account::auth::AuthRpoFalcon512;
use miden_objects::account::component::BasicWallet;

let key_pair = SecretKey::with_rng(client.rng());
let anchor_block = client.get_latest_epoch_block().await?;
let init_seed: [u8; 32] = rand::random();

let account_c = AccountBuilder::new(init_seed)
    .anchor((&anchor_block).try_into()?)
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Public)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;

let keystore = client.keystore();
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), account_c.id()).await?;
client.add_account(&account_c, false).await?;

println!("Account C created: {:?}", account_c.id());
```

## Send assets with a public note

Build and submit a pay-to-id transaction from Account A to Account C using a public note:

```rust
use miden_client::transaction::{TransactionRequestBuilder, PaymentNoteDescription};
use miden_objects::note::NoteType;
use miden_objects::asset::FungibleAsset;
use miden_objects::account::AccountId;

// Define the asset to send
let faucet_id = AccountId::from_hex("0x<your-faucet-id>")?;
let asset = FungibleAsset::new(faucet_id, 50)?.into();

// Build the payment request
let payment = PaymentNoteDescription::new(
    vec![asset],
    account_a_id,  // sender
    account_c.id(), // target
);

let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
    payment,
    None,
    NoteType::Public, // Public note — recipient can discover it by syncing
    client.rng(),
)?;

// Execute and submit
let tx_result = client.new_transaction(account_a_id, tx_request).await?;
client.submit_transaction(tx_result).await?;

println!("Public P2ID note sent!");
```

## Receive and consume the note

Since the note is public, Account C can discover it by syncing with the network:

```rust
// Sync to discover the public note
client.sync_state().await?;

// Get consumable notes for Account C
let consumable = client.get_consumable_notes(Some(account_c.id())).await?;
let note_ids: Vec<_> = consumable.iter().map(|n| n.note.id()).collect();

if !note_ids.is_empty() {
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(account_c.id(), note_ids)?;

    let tx_result = client.new_transaction(account_c.id(), tx_request).await?;
    client.submit_transaction(tx_result).await?;

    println!("Notes consumed by Account C!");
}
```

## Verify

Sync and check Account C's balance:

```rust
client.sync_state().await?;

let (account, _) = client.get_account(account_c.id()).await?;
println!("Account C vault: {:?}", account.vault());
```

Account C should now hold the transferred tokens.
