---
title: Note Transport
sidebar_position: 10
---

# Note Transport

The note transport layer allows you to send and receive private notes between users without exposing note details on-chain. This is useful when the sender and recipient don't share a local client.

## Prerequisites

Note transport must be configured when building the client:

```rust
let mut client = ClientBuilder::for_testnet()
    .store(store)
    .filesystem_keystore("path/to/keys")?
    .build()
    .await?;

// Check if transport is enabled
if client.is_note_transport_enabled() {
    println!("Note transport is available");
}
```

The pre-configured constructors (`for_testnet()`, `for_devnet()`) enable note transport automatically. For custom configurations, configure it via `ClientBuilder::new().note_transport(...)`.

## Sending private notes

After creating a private note via a transaction, send it to the recipient through the transport network:

```rust
// The sender needs the recipient's bech32 address
let recipient_address: Address = "mtst1recipient...".parse()?;

// Get the note from a recent transaction's output
let output_notes = client.get_output_notes(NoteFilter::All).await?;
let note = output_notes.last().unwrap();

// Send via the transport network
client.send_private_note(note.clone().try_into()?, &recipient_address).await?;
```

The note is end-to-end encrypted using the recipient's address details. Only the recipient can decrypt and consume it.

## Fetching private notes

The recipient fetches notes addressed to their tracked accounts:

```rust
// Fetch with pagination (recommended for ongoing use)
client.fetch_private_notes().await?;

// Or fetch all notes at once (useful for initial setup)
client.fetch_all_private_notes().await?;

// View the fetched notes
let notes = client.get_input_notes(NoteFilter::All).await?;
for note in &notes {
    println!("Note: {:?}, Status: {:?}", note.id(), note.status());
}
```

### Pagination

`fetch_private_notes()` uses internal cursor-based pagination to avoid downloading duplicate notes across calls. Use this for periodic fetching.

`fetch_all_private_notes()` fetches everything without pagination. Reserve this for special cases like initial setup or recovery.

## Complete example

```rust
// === Sender side ===

// 1. Create a private note via transaction
let payment = PaymentNoteDescription::new(vec![asset], sender_id, target_id);
let tx_request = TransactionRequestBuilder::new()
    .build_pay_to_id(payment, None, NoteType::Private, client.rng())?;
let tx_result = client.new_transaction(sender_id, tx_request).await?;
client.submit_transaction(tx_result).await?;

// 2. Send via transport
let output_notes = client.get_output_notes(NoteFilter::All).await?;
let note = output_notes.last().unwrap();
client.send_private_note(note.clone().try_into()?, &recipient_address).await?;

// === Recipient side ===

// 3. Fetch private notes
client.fetch_private_notes().await?;

// 4. Consume the received note
let consumable = client.get_consumable_notes(Some(account_id)).await?;
let note_ids: Vec<_> = consumable.iter().map(|n| n.note.id()).collect();
let tx_request = TransactionRequestBuilder::new()
    .build_consume_notes(account_id, note_ids)?;
let tx_result = client.new_transaction(account_id, tx_request).await?;
client.submit_transaction(tx_result).await?;
```

:::note
The client only fetches notes for tracked note tags. By default, tags are derived from account IDs. For increased privacy, use random tags and track them with `client.add_note_tag(tag)`. See [Tags](./tags.md).
:::
