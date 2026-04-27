# Miden Client Migration Guide

This document provides migration guidance for breaking changes introduced in new versions of the Miden Client.
Use this guide to migrate from 0.12.x to 0.13.0; it covers 0.13.0 breaking changes only.

## Version 0.13.0

0.13.0 aligns the SDK with protocol 0.13, refactors note and transaction APIs, and tightens web bindings and storage schemas. Expect updates in input note handling, storage slot naming, auth/key APIs, and web client data shapes.

## Table of Contents
- [Input notes API unified](#input-notes-api-unified)
- [Account components and storage slots now use named slots](#account-components-and-storage-slots-now-use-named-slots)
- [Authentication and key management updates](#authentication-and-key-management-updates)
- [NodeRpcClient account proof API changed](#noderpcclient-account-proof-api-changed)
- [FetchedNote and RPC note shapes refactored](#fetchednote-and-rpc-note-shapes-refactored)
- [Protocol 0.13 note metadata tags and attachments](#protocol-013-note-metadata-tags-and-attachments)
- [NoteScreener relevance replaced by NoteConsumptionStatus](#notescreener-relevance-replaced-by-noteconsumptionstatus)
- [WebClient IndexedDB naming for multiple instances](#webclient-indexeddb-naming-for-multiple-instances)
- [Block numbers are numeric in web APIs and IndexedDB](#block-numbers-are-numeric-in-web-apis-and-indexeddb)
- [NetworkId custom networks and toBech32Custom removal](#networkid-custom-networks-and-tobech32custom-removal)
- [Client RNG must be Send and Sync](#client-rng-must-be-send-and-sync)
- [CLI swap payback_note_type removed](#cli-swap-payback_note_type-removed)

### Input notes API unified
**PR:** #1624

#### Summary
Input notes are no longer split into authenticated and unauthenticated lists. Builders now accept full `Note` objects and the client determines authentication internally, and the WebClient consume request now accepts `Note[]` instead of note ID strings.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::NoteId;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, ClientError};

async fn build_request<AUTH: TransactionAuthenticator + Sync>(
    client: &Client<AUTH>,
    note_id: NoteId,
) -> Result<miden_client::transaction::TransactionRequest, ClientError> {
    let tx_request = TransactionRequestBuilder::new()
        .authenticated_input_notes(vec![(note_id, None)])
        .build()?;
    Ok(tx_request)
}
```

```rust
// After (0.13.0)
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::{Note, NoteId};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, ClientError};

async fn build_request<AUTH: TransactionAuthenticator + Sync>(
    client: &Client<AUTH>,
    note_id: NoteId,
) -> Result<miden_client::transaction::TransactionRequest, ClientError> {
    let record = client.get_input_note(note_id).await?.expect("note not found");
    let note: Note = record.try_into().expect("failed to convert note record");

    let tx_request = TransactionRequestBuilder::new()
        .input_notes(vec![(note, None)])
        .build()?;
    Ok(tx_request)
}
```

**TypeScript:**
```typescript
// Before (0.12.x)
import {
  NoteIdAndArgs,
  NoteIdAndArgsArray,
  TransactionRequestBuilder,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const consumeRequest = client.newConsumeTransactionRequest([noteId]);

const noteIdAndArgs = new NoteIdAndArgs(noteId, null);
const txRequest = new TransactionRequestBuilder()
  .withAuthenticatedInputNotes(new NoteIdAndArgsArray([noteIdAndArgs]))
  .build();
```

```typescript
// After (0.13.0)
import {
  NoteAndArgs,
  NoteAndArgsArray,
  TransactionRequestBuilder,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const record = await client.getInputNote(noteId);
if (!record) {
  throw new Error(`Note with ID ${noteId} not found`);
}
const note = record.toNote();

const consumeRequest = client.newConsumeTransactionRequest([note]);

const noteAndArgs = new NoteAndArgs(note, null);
const txRequest = new TransactionRequestBuilder()
  .withInputNotes(new NoteAndArgsArray([noteAndArgs]))
  .build();
```

#### Migration Steps
1. Replace `authenticated_input_notes` and `unauthenticated_input_notes` with `input_notes`.
2. Convert `NoteId` values into `Note` objects (Rust: `InputNoteRecord` -> `Note` via `try_into`; Web: `InputNoteRecord.toNote()`).
3. Update `newConsumeTransactionRequest` to pass `Note[]`, and replace `NoteIdAndArgs` with `NoteAndArgs`.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `method not found: authenticated_input_notes` | Deprecated builder methods removed | Use `input_notes` with `Note` values |
| `expected Note, found NoteId` | Passing IDs where full notes are required | Fetch the note record and convert to `Note` |
| `newConsumeTransactionRequest expects Note[]` | API now requires notes, not strings | Call `getInputNote(...).toNote()` first |

### Account components and storage slots now use named slots
**PRs:** #1626, #1627

#### Summary
Storage slots are now identified by name instead of index, and the web binding for `AccountComponent.compile` now requires a compiled `AccountComponentCode`. The filesystem keystore is also no longer generic over RNG.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::account::{StorageMap, StorageSlot};

fn storage_slots(storage_map: StorageMap) -> Vec<StorageSlot> {
    vec![StorageSlot::Map(storage_map)]
}
```

```rust
// After (0.13.0)
use miden_client::account::{StorageMap, StorageSlot, StorageSlotName};

fn storage_slots(storage_map: StorageMap) -> Vec<StorageSlot> {
    let slot_name = StorageSlotName::new("miden::example::map")
        .expect("slot name must be valid");
    vec![StorageSlot::with_map(slot_name, storage_map)]
}
```

```rust
// Before (0.12.x)
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;

let builder = ClientBuilder::<FilesystemKeyStore<_>>::new();
```

```rust
// After (0.13.0)
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;

let builder = ClientBuilder::<FilesystemKeyStore>::new();
```

**TypeScript:**
```typescript
// Before (0.12.x)
import {
  AccountComponent,
  StorageMap,
  StorageSlot,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const builder = client.createCodeBuilder();
const storageMap = new StorageMap();
const slot = StorageSlot.map(storageMap);

const component = AccountComponent.compile(accountCode, builder, [slot]);
const value = account.storage().getMapItem(1, key);
```

```typescript
// After (0.13.0)
import {
  AccountComponent,
  StorageMap,
  StorageSlot,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const builder = client.createCodeBuilder();
const storageMap = new StorageMap();
const slotName = "miden::example::map";
const slot = StorageSlot.map(slotName, storageMap);

const componentCode = builder.compileAccountComponentCode(accountCode);
const component = AccountComponent.compile(componentCode, [slot]);

const value = account.storage().getMapItem(slotName, key);
const slotNames = account.storage().getSlotNames();
```

#### Migration Steps
1. Replace index-based slots with named slots (`StorageSlotName` in Rust, string names in Web).
2. Update account component compilation to use `CodeBuilder.compileAccountComponentCode` and pass the resulting `AccountComponentCode`.
3. If you used `FilesystemKeyStore<_>` generics, drop the RNG parameter.
4. Update any storage accessors (`getItem`, `getMapItem`, `getMapEntries`) to pass slot names.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `expected StorageSlotName` | Creating slots without names | Use `StorageSlotName::new("namespace::slot")` |
| `AccountComponent.compile takes 2 arguments` | Old binding passed `CodeBuilder` directly | Compile to `AccountComponentCode` first |
| `type annotations needed for FilesystemKeyStore` | Removed RNG generic | Use `FilesystemKeyStore` without type params |

### Authentication and key management updates
**PRs:** #1546, #1578, #1592, #1608

#### Summary
WebClient auth APIs now take the `AuthScheme` enum instead of numeric IDs, `SecretKey` has been removed in favor of `AuthSecretKey`, and `addAccountSecretKeyToWebStore` now requires an account ID. In Rust, `build_wallet_id` no longer accepts a raw scheme ID and instead infers the scheme from `PublicKey`. Scheme-specific public key methods on `AuthSecretKey` were removed; use `getPublicKeyAsWord` instead.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::account::{build_wallet_id, AccountStorageMode};
use miden_client::auth::{PublicKey, RPO_FALCON_SCHEME_ID};

fn build_id(
    seed: [u8; 32],
    public_key: &PublicKey,
) -> Result<miden_client::account::AccountId, miden_client::ClientError> {
    build_wallet_id(
        seed,
        public_key,
        AccountStorageMode::Public,
        true,
        RPO_FALCON_SCHEME_ID,
    )
}
```

```rust
// After (0.13.0)
use miden_client::account::{build_wallet_id, AccountStorageMode};
use miden_client::auth::PublicKey;

fn build_id(
    seed: [u8; 32],
    public_key: &PublicKey,
) -> Result<miden_client::account::AccountId, miden_client::ClientError> {
    build_wallet_id(seed, public_key, AccountStorageMode::Public, true)
}
```

**TypeScript:**
```typescript
// Before (0.12.x)
import {
  AccountComponent,
  AccountStorageMode,
  SecretKey,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const wallet = await client.newWallet(AccountStorageMode.public(), true, 0, seed);

const secretKey = SecretKey.rpoFalconWithRNG(seed);
const commitment = secretKey.getRpoFalcon512PublicKeyAsWord();
const authComponent = AccountComponent.createAuthComponentFromCommitment(commitment, 0);

await client.addAccountSecretKeyToWebStore(secretKey);
```

```typescript
// After (0.13.0)
import {
  AccountComponent,
  AccountStorageMode,
  AuthScheme,
  AuthSecretKey,
  WebClient,
} from "@miden-sdk/miden-sdk";

const client = await WebClient.createClient();
const wallet = await client.newWallet(
  AccountStorageMode.public(),
  true,
  AuthScheme.AuthRpoFalcon512,
  seed
);

const secretKey = AuthSecretKey.rpoFalconWithRNG(seed);
const commitment = secretKey.getPublicKeyAsWord();
const authComponent = AccountComponent.createAuthComponentFromCommitment(
  commitment,
  AuthScheme.AuthRpoFalcon512
);
const fromSecret = AccountComponent.createAuthComponentFromSecretKey(secretKey);

await client.addAccountSecretKeyToWebStore(wallet.id(), secretKey);
const commitments = await client.getPublicKeyCommitmentsOfAccount(wallet.id());
```

#### Migration Steps
1. In Rust, drop the `auth_scheme_id` argument from `build_wallet_id`; the scheme is inferred from `PublicKey`.
2. In the WebClient, replace numeric auth scheme IDs with `AuthScheme` enum values.
3. Replace `SecretKey` with `AuthSecretKey` and update calls to `createAuthComponentFromSecretKey`.
4. Replace `getRpoFalcon512PublicKeyAsWord` and `getEcdsaK256KeccakPublicKeyAsWord` with `getPublicKeyAsWord`.
5. Pass an account ID to `addAccountSecretKeyToWebStore` and use `getPublicKeyCommitmentsOfAccount` when you need associated commitments.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `this function takes 4 arguments but 5 were supplied` | `build_wallet_id` signature changed | Remove the `auth_scheme_id` argument |
| `SecretKey is not defined` | Model removed | Use `AuthSecretKey` |
| `Argument of type number is not assignable to AuthScheme` | Numeric scheme IDs removed | Use `AuthScheme.AuthRpoFalcon512` or `AuthScheme.AuthEcdsaK256Keccak` |
| `createAuthComponent is not a function` | Method removed | Use `createAuthComponentFromSecretKey` |

### NodeRpcClient account proof API changed
**PR:** #1616

#### Summary
The batch `get_account_proofs` API is replaced with a single-account call that requires `AccountStateAt`, and the known code parameter is now optional per account.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use std::collections::{BTreeMap, BTreeSet};

use miden_client::account::{AccountCode, AccountId};
use miden_client::block::BlockNumber;
use miden_client::rpc::domain::account::AccountProof;
use miden_client::rpc::NodeRpcClient;
use miden_client::transaction::ForeignAccount;

async fn fetch_proofs(
    rpc: &dyn NodeRpcClient,
    accounts: BTreeSet<ForeignAccount>,
    known_codes: BTreeMap<AccountId, AccountCode>,
) -> Result<(BlockNumber, Vec<AccountProof>), miden_client::rpc::RpcError> {
    rpc.get_account_proofs(&accounts, known_codes).await
}
```

```rust
// After (0.13.0)
use miden_client::account::AccountCode;
use miden_client::block::BlockNumber;
use miden_client::rpc::domain::account::AccountProof;
use miden_client::rpc::{AccountStateAt, NodeRpcClient};
use miden_client::transaction::ForeignAccount;

async fn fetch_proof(
    rpc: &dyn NodeRpcClient,
    account: ForeignAccount,
    known_code: Option<AccountCode>,
) -> Result<(BlockNumber, AccountProof), miden_client::rpc::RpcError> {
    rpc.get_account(account, AccountStateAt::ChainTip, known_code).await
}
```

#### Migration Steps
1. Replace `get_account_proofs` with `get_account` and call it per `ForeignAccount`.
2. Pass the desired state via `AccountStateAt::ChainTip` or `AccountStateAt::Block`.
3. Update implementations of `NodeRpcClient` to match the new signature and return type.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `method not found: get_account_proofs` | Old trait method removed | Use `get_account` and loop |
| `missing argument: account_state` | New `AccountStateAt` required | Pass `AccountStateAt::ChainTip` or `AccountStateAt::Block` |
| `expected AccountProof, found Vec<AccountProof>` | Return type changed | Handle single proof per call |

### FetchedNote and RPC note shapes refactored
**PRs:** #1536, #1606

#### Summary
Fetched notes now carry a `NoteHeader` for private notes and always expose the inclusion proof in the WebClient. Web `FetchedNote` exposes `header`, `note`, and `inclusionProof`, with `asInputNote()` for public notes.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::rpc::domain::note::FetchedNote;

fn handle_note(note: FetchedNote) {
    match note {
        FetchedNote::Private(note_id, metadata, proof) => {
            let _ = (note_id, metadata, proof);
        }
        FetchedNote::Public(note, proof) => {
            let _ = (note, proof);
        }
    }
}
```

```rust
// After (0.13.0)
use miden_client::rpc::domain::note::FetchedNote;

fn handle_note(note: FetchedNote) {
    match note {
        FetchedNote::Private(header, proof) => {
            let note_id = header.id();
            let metadata = header.metadata();
            let _ = (note_id, metadata, proof);
        }
        FetchedNote::Public(note, proof) => {
            let _ = (note, proof);
        }
    }
}
```

**TypeScript:**
```typescript
// Before (0.12.x)
const fetched = (await rpcClient.getNotesById([noteId]))[0];
if (fetched.inputNote) {
  const scriptRoot = fetched.inputNote.note().script().root();
}
```

```typescript
// After (0.13.0)
const fetched = (await rpcClient.getNotesById([noteId]))[0];
const proof = fetched.inclusionProof;
const note = fetched.note;
if (note) {
  const scriptRoot = note.script().root();
}
const inputNote = fetched.asInputNote();
```

#### Migration Steps
1. Update pattern matches for `FetchedNote::Private` to use `NoteHeader`.
2. In the WebClient, replace `inputNote` access with `note` plus `inclusionProof`, or call `asInputNote()`.
3. Use `header` for shared access to `noteId` and `metadata`.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `pattern has 3 fields, but the corresponding tuple variant has 2 fields` | `FetchedNote::Private` shape changed | Use `FetchedNote::Private(header, proof)` |
| `Property 'inputNote' does not exist on type 'FetchedNote'` | Web shape updated | Use `note`, `inclusionProof`, or `asInputNote()` |

### Protocol 0.13 note metadata tags and attachments
**PR:** #1685

#### Summary
Note metadata and tagging APIs were simplified. Account-target tags now use `withAccountTarget`/`with_account_target`, `NoteExecutionMode` was removed, `NoteMetadata` no longer accepts execution hints in the constructor, and `NoteAttachment` now uses `NoteAttachmentScheme` with new accessors.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::note::{
    NoteExecutionHint,
    NoteMetadata,
    NoteTag,
    NoteType,
};
use miden_client::Felt;

let tag = NoteTag::from_account_id(target_account_id);
let metadata = NoteMetadata::new(
    sender_account_id,
    NoteType::Private,
    tag,
    NoteExecutionHint::none(),
    Felt::default(),
)
.expect("valid metadata");
```

```rust
// After (0.13.0)
use miden_client::note::{
    NoteAttachment,
    NoteAttachmentScheme,
    NoteMetadata,
    NoteTag,
    NoteType,
};

let tag = NoteTag::with_account_target(target_account_id);
let metadata = NoteMetadata::new(sender_account_id, NoteType::Private, tag);

let scheme = NoteAttachmentScheme::new(42);
let attachment = NoteAttachment::new_word(scheme, word);
let metadata_with_attachment = metadata.with_attachment(attachment);
```

**TypeScript:**
```typescript
// Before (0.12.x)
import {
  NoteAttachment,
  NoteExecutionHint,
  NoteExecutionMode,
  NoteMetadata,
  NoteTag,
  NoteType,
} from "@miden-sdk/miden-sdk";

const tag = NoteTag.fromAccountId(targetAccountId, NoteExecutionMode.newLocal());
const metadata = new NoteMetadata(
  senderAccountId,
  NoteType.Private,
  tag,
  NoteExecutionHint.none()
);
const attachment = NoteAttachment.newWord(42, word);
```

```typescript
// After (0.13.0)
import {
  NoteAttachment,
  NoteAttachmentScheme,
  NoteExecutionHint,
  NoteMetadata,
  NoteTag,
  NoteType,
} from "@miden-sdk/miden-sdk";

const tag = NoteTag.withAccountTarget(targetAccountId);
const metadata = new NoteMetadata(senderAccountId, NoteType.Private, tag);

const scheme = new NoteAttachmentScheme(42);
const attachment = NoteAttachment.newWord(scheme, word);
const metadataWithAttachment = metadata.withAttachment(attachment);

// Optional: target a network account via attachment.
const networkAttachment = NoteAttachment.newNetworkAccountTarget(
  targetAccountId,
  NoteExecutionHint.none()
);
```

#### Migration Steps
1. Replace `NoteTag.fromAccountId` with `NoteTag.withAccountTarget`/`withCustomAccountTarget` (Rust: `NoteTag::with_account_target`/`NoteTag::with_custom_account_target`).
2. Drop `NoteExecutionMode` usages; attach execution context via `NoteAttachment` if needed.
3. Update `NoteMetadata` construction to `new NoteMetadata(sender, type, tag)` and add attachments with `withAttachment` (Rust: `NoteMetadata::new(...)` plus `with_attachment`).
4. Wrap attachment scheme values in `NoteAttachmentScheme` and use the new accessors to read payloads (Web: `asWord()`/`asArray()`, Rust: match on `NoteAttachment::content()`).

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `NoteExecutionMode is not defined` | Class removed | Remove it and use attachments if needed |
| `NoteTag.fromAccountId is not a function` | API renamed | Use `NoteTag.withAccountTarget` |
| `Argument of type number is not assignable to NoteAttachmentScheme` | Scheme wrapper added | Construct `new NoteAttachmentScheme(value)` |

### NoteScreener relevance replaced by NoteConsumptionStatus
**PR:** #1630

#### Summary
`NoteRelevance` was removed; `NoteScreener` now reports `NoteConsumptionStatus` values, and the WebClient exposes consumption status objects rather than a single `consumableAfterBlock` field.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::note::{NoteRelevance, NoteScreener};

let relevances = note_screener.check_relevance(&note).await?;
for (_, relevance) in relevances {
    if relevance == NoteRelevance::Now {
        // ...
    }
}
```

```rust
// After (0.13.0)
use miden_client::note::{NoteConsumptionStatus, NoteScreener};

let relevances = note_screener.check_relevance(&note).await?;
for (_, status) in relevances {
    match status {
        NoteConsumptionStatus::Consumable
        | NoteConsumptionStatus::ConsumableWithAuthorization => {
            // ...
        }
        NoteConsumptionStatus::ConsumableAfter(_) => {
            // ...
        }
        _ => {}
    }
}
```

**TypeScript:**
```typescript
// Before (0.12.x)
const records = await client.getConsumableNotes(accountId);
const after = records[0].noteConsumability()[0].consumableAfterBlock();
```

```typescript
// After (0.13.0)
const records = await client.getConsumableNotes(accountId);
const status = records[0].noteConsumability()[0].consumptionStatus();
const after = status.consumableAfterBlock();
```

#### Migration Steps
1. Replace `NoteRelevance` with `NoteConsumptionStatus` in Rust logic and pattern matching.
2. Update WebClient consumption checks to call `consumptionStatus()` and then `consumableAfterBlock()`.
3. Remove any reliance on `NoteRelevance::Now` / `After` variants.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `use of undeclared type NoteRelevance` | Type removed | Use `NoteConsumptionStatus` |
| `consumableAfterBlock is not a function` | API moved under `consumptionStatus()` | Call `consumptionStatus().consumableAfterBlock()` |

### WebClient IndexedDB naming for multiple instances
**PR:** #1645

#### Summary
The WebClient store is now named, so multiple clients can coexist in the same browser. `WebClient.createClient` and `createClientWithExternalKeystore` accept an optional store name before callback arguments.

#### Affected Code

**TypeScript:**
```typescript
// Before (0.12.x)
const client = await WebClient.createClient(rpcUrl, noteTransportUrl, seed);

const clientWithKeystore = await WebClient.createClientWithExternalKeystore(
  rpcUrl,
  noteTransportUrl,
  seed,
  getKeyCb,
  insertKeyCb,
  signCb
);
```

```typescript
// After (0.13.0)
const client = await WebClient.createClient(
  rpcUrl,
  noteTransportUrl,
  seed,
  "app-db"
);

const clientWithKeystore = await WebClient.createClientWithExternalKeystore(
  rpcUrl,
  noteTransportUrl,
  seed,
  "app-db",
  getKeyCb,
  insertKeyCb,
  signCb
);
```

#### Migration Steps
1. Add a store name to `createClient` when you need multiple instances in one origin.
2. Shift external keystore callback arguments one position to the right and pass `storeName` first.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `Expected 7 arguments, but got 6` | Missing `storeName` in `createClientWithExternalKeystore` | Insert the store name before callbacks |

### Block numbers are numeric in web APIs and IndexedDB
**PRs:** #1528, #1684

#### Summary
WebClient transaction interfaces and IndexedDB storage now use numeric block numbers instead of strings.

#### Affected Code

**TypeScript:**
```typescript
// Before (0.12.x)
const summary = await client.syncState();
const blockNum = parseInt(summary.blockNum(), 10);
```

```typescript
// After (0.13.0)
const summary = await client.syncState();
const blockNum = summary.blockNum();
```

#### Migration Steps
1. Remove `parseInt` or `Number(...)` wrappers around `blockNum()` results.
2. If you integrate with `idxdb-store` helpers directly, pass numbers instead of strings.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `Argument of type 'string' is not assignable to parameter of type 'number'` | Block numbers now numeric | Pass `number` values directly |

### NetworkId custom networks and toBech32Custom removal
**PR:** #1612

#### Summary
`NetworkId` is now a class with static constructors and supports custom prefixes. `toBech32Custom` was removed; use `NetworkId.custom(...)` with `toBech32` instead.

#### Affected Code

**TypeScript:**
```typescript
// Before (0.12.x)
const bech32 = accountId.toBech32Custom("cstm", AccountInterface.BasicWallet);
const network = NetworkId.Testnet;
```

```typescript
// After (0.13.0)
const network = NetworkId.custom("cstm");
const bech32 = accountId.toBech32(network, AccountInterface.BasicWallet);
const testnet = NetworkId.testnet();
```

#### Migration Steps
1. Replace enum-style `NetworkId.Mainnet/Testnet/Devnet` with `NetworkId.mainnet()/testnet()/devnet()`.
2. Replace `toBech32Custom(prefix, ...)` with `toBech32(NetworkId.custom(prefix), ...)`.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `Property 'Mainnet' does not exist on type 'typeof NetworkId'` | Enum replaced by class | Use `NetworkId.mainnet()` and friends |
| `toBech32Custom is not a function` | Method removed | Use `NetworkId.custom(...)` + `toBech32` |

### Client RNG must be Send and Sync
**PR:** #1677

#### Summary
The client RNG must now be `Send + Sync` via the `ClientFeltRng` marker and `ClientRngBox` alias so `Client` can be `Send + Sync`.

#### Affected Code

**Rust:**
```rust
// Before (0.12.x)
use miden_client::builder::ClientBuilder;
use miden_client::crypto::{FeltRng, RpoRandomCoin};

let rng: Box<dyn FeltRng> = Box::new(RpoRandomCoin::new([0u8; 32]));
let builder = ClientBuilder::new().rng(rng);
```

```rust
// After (0.13.0)
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::ClientRngBox;

let rng: ClientRngBox = Box::new(RpoRandomCoin::new([0u8; 32]));
let builder = ClientBuilder::new().rng(rng);
```

#### Migration Steps
1. Ensure your RNG implements `Send + Sync`.
2. Wrap the RNG in `ClientRngBox` and pass it to `ClientBuilder::rng`.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `the trait bound ...: Send + Sync is not satisfied` | RNG type not thread-safe | Use a `Send + Sync` RNG or wrap it safely |

### CLI swap payback_note_type removed
**PR:** #1700

#### Summary
The CLI swap command no longer accepts a `payback_note_type` argument; the payback note type is now fixed.

#### Affected Code

**CLI:**
```bash
# Before (0.12.x)
miden-client swap \
  --offered-asset 10::0x... \
  --requested-asset 5::0x... \
  --note-type public \
  --payback-note-type public
```

```bash
# After (0.13.0)
miden-client swap \
  --offered-asset 10::0x... \
  --requested-asset 5::0x... \
  --note-type public
```

#### Migration Steps
1. Remove `--payback-note-type` from swap command invocations or scripts.

#### Common Errors
| Error Message | Cause | Solution |
|---------------|-------|----------|
| `unexpected argument '--payback-note-type'` | Flag removed | Drop the flag from the command |

## Need Help?

If you hit issues migrating, reach out here:
- [Discord Community](https://discord.gg/miden)
- [GitHub Issues](https://github.com/0xMiden/miden-client/issues)
