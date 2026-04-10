---
title: New Transactions
sidebar_position: 6
---

# Creating Transactions

This guide demonstrates how to create and submit transactions using the Miden Client Rust library. Transactions follow a two-step flow: execute, then submit.

## Basic transaction flow

Every transaction follows this pattern:

```rust
// 1. Build a TransactionRequest
let tx_request = TransactionRequestBuilder::new()
    .build_pay_to_id(payment, NoteType::Private, &mut client.rng())?;

// 2. Execute, prove, and submit in one call
client.submit_new_transaction(sender_id, tx_request).await?;
```

For more control, use the staged flow:

```rust
let tx_result = client.execute_transaction(sender_id, tx_request).await?;
let proven_tx = client.prove_transaction(&tx_result).await?;
client.submit_proven_transaction(proven_tx, tx_result).await?;
```

After submission, the transaction is tracked locally. Sync to confirm it has been committed on-chain.

## Sending tokens (pay-to-id)

Transfer fungible assets from one account to another:

```rust
use miden_client::transaction::{TransactionRequestBuilder, PaymentNoteDescription};
use miden_objects::note::NoteType;
use miden_objects::asset::FungibleAsset;
use miden_objects::account::AccountId;

let faucet_id = AccountId::from_hex("0xFAUCET...")?;
let asset = FungibleAsset::new(faucet_id, 100)?.into();

let sender_id = AccountId::from_hex("0xSENDER...")?;
let target_id = AccountId::from_hex("0xTARGET...")?;

let payment = PaymentNoteDescription::new(
    vec![asset],
    sender_id,
    target_id,
);

let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
    payment,
    NoteType::Private, // or NoteType::Public
    &mut client.rng(),
)?;

client.submit_new_transaction(sender_id, tx_request).await?;
```

### Note types

| Type | Description |
|------|-------------|
| `NoteType::Private` | Note details are not publicly visible; recipient needs the note data to consume |
| `NoteType::Public` | Note details are stored on-chain; recipient can discover it by syncing |

### Recallable notes

Set a reclaim height on the payment description to allow the sender to reclaim the note if the recipient hasn't consumed it:

```rust
let payment = PaymentNoteDescription::new(vec![asset], sender_id, target_id)
    .with_reclaim_height(100.into()); // Sender can reclaim after block 100

let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
    payment,
    NoteType::Public,
    &mut client.rng(),
)?;
```

## Consuming notes

Consume notes to receive assets into an account:

```rust
// Get consumable notes for an account
let consumable = client.get_consumable_notes(Some(account_id)).await?;
let notes: Vec<_> = consumable.into_iter().map(|n| n.note).collect();

if !notes.is_empty() {
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(notes)?;

    client.submit_new_transaction(account_id, tx_request).await?;
}
```

## Minting tokens

Mint tokens from a faucet account:

```rust
let faucet_id = AccountId::from_hex("0xFAUCET...")?;
let target_id = AccountId::from_hex("0xTARGET...")?;
let asset = FungibleAsset::new(faucet_id, 1000)?.into();

let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
    asset,
    target_id,
    NoteType::Public,
    &mut client.rng(),
)?;

client.submit_new_transaction(faucet_id, tx_request).await?;
```

## Swap transactions

Create an atomic swap — offer one asset in exchange for another:

```rust
let offered_asset = FungibleAsset::new(faucet_a_id, 100)?.into();
let requested_asset = FungibleAsset::new(faucet_b_id, 200)?.into();

let swap_data = SwapNoteDescription::new(
    source_account_id,
    offered_asset,
    requested_asset,
);

let tx_request = TransactionRequestBuilder::new().build_swap(
    swap_data,
    NoteType::Public,
    NoteType::Private, // payback note type
    &mut client.rng(),
)?;

client.submit_new_transaction(source_account_id, tx_request).await?;
```

When another account consumes the swap note, it receives the offered asset and the requested asset is removed from its vault into a new note the original account can consume.

## Using a remote prover

Offload proof generation to a remote prover for better performance:

```rust
use miden_client::RemoteTransactionProver;

let remote_prover = Arc::new(RemoteTransactionProver::new("https://prover.example.com"));

// Build client with remote prover
let client = ClientBuilder::for_testnet()
    .store(store)
    .filesystem_keystore("path/to/keys")?
    .prover(remote_prover)
    .build()
    .await?;

// All transactions automatically use the remote prover
```

For fallback patterns (remote with local fallback), see [Examples](../examples.md).

## Custom transactions

Use `TransactionRequestBuilder` for full control over inputs, outputs, and scripts:

```rust
let tx_request = TransactionRequestBuilder::new()
    .custom_script(transaction_script)?
    .own_output_notes(output_notes)
    .expected_output_notes(expected_notes)
    .build()?;

client.submit_new_transaction(account_id, tx_request).await?;
```

:::note
Custom transactions require understanding of the Miden VM instruction set and note scripts.
:::
