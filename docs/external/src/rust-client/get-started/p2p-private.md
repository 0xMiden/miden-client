---
title: Private peer-to-peer transfer
sidebar_position: 4
---

In this tutorial, you'll transfer assets between two accounts using private notes and the Rust library API.

:::info Prerequisite
Complete the [Create account](./create-account-use-faucet.md) tutorial first. You should have a funded account (Account A) and a working client.
:::

## Create a second account

Create a private account (Account B) within the same client:

```rust
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::AuthSecretKey;
use miden_client::crypto::SecretKey;
use miden_objects::account::auth::AuthRpoFalcon512;
use miden_objects::account::component::BasicWallet;

let key_pair = SecretKey::with_rng(client.rng());
let init_seed: [u8; 32] = rand::random();

let account_b = AccountBuilder::new(init_seed)
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Private)
    .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
    .with_component(BasicWallet)
    .build()?;

let keystore = client.keystore();
keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair), account_b.id()).await?;
client.add_account(&account_b, false).await?;

println!("Account B created: {:?}", account_b.id());
```

## Send assets with a private note

Build and submit a pay-to-id transaction from Account A to Account B using a private note:

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
    account_a_id,   // sender
    account_b.id(), // target
);

let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
    payment,
    None,
    NoteType::Private, // Private note — details not publicly visible
    client.rng(),
)?;

// Execute and submit
let tx_result = client.new_transaction(account_a_id, tx_request).await?;
client.submit_transaction(tx_result).await?;

println!("Private P2ID note sent!");
```

## Consume the private note

Since both accounts are in the same client, Account B can discover the note by syncing:

```rust
// Sync to update note state
client.sync_state().await?;

// Get consumable notes for Account B
let consumable = client.get_consumable_notes(Some(account_b.id())).await?;
let note_ids: Vec<_> = consumable.iter().map(|n| n.note.id()).collect();

if !note_ids.is_empty() {
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(account_b.id(), note_ids)?;

    let tx_result = client.new_transaction(account_b.id(), tx_request).await?;
    client.submit_transaction(tx_result).await?;

    println!("Notes consumed by Account B!");
}
```

## Verify

Sync and check both accounts:

```rust
client.sync_state().await?;

let (acct_a, _) = client.get_account(account_a_id).await?;
let (acct_b, _) = client.get_account(account_b.id()).await?;

println!("Account A vault: {:?}", acct_a.vault());
println!("Account B vault: {:?}", acct_b.vault());
```

## Using the note transport network

When accounts belong to different users (separate clients), use the note transport network to exchange private notes:

```rust
// Sender: after creating the note, get the output note ID
let output_notes = client.get_output_notes().await?;
let note_id = output_notes.last().unwrap().id();

// Send the note via the transport network (requires recipient's bech32 address)
client.send_note_via_transport(note_id, recipient_address).await?;

// Recipient: fetch notes from the transport network
client.fetch_notes_from_transport().await?;

// Then sync and consume as normal
client.sync_state().await?;
```

:::note
The client fetches notes for tracked note tags. By default, tags are derived from the recipient's account ID. For increased privacy, use random tags and track them explicitly.
:::

## Congratulations!

You've completed all the getting started tutorials. You can now:

- Create accounts (private and public)
- Execute transactions with ZK proofs
- Transfer assets via public and private notes
- Use the note transport network for multi-user scenarios

For more advanced patterns, see the [Library reference](../library/index.md) and [Examples](../examples.md).
