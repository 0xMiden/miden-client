# Implementation Plan: Automate Note Script Registration for Public NTX Notes

## GitHub Issue

**Issue:** [#1457 - Automate the submission of note scripts for public ntx notes](https://github.com/0xMiden/miden-client/issues/1457)

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Background: How NTX Works](#background-how-ntx-works)
3. [Codebase Investigation Findings](#codebase-investigation-findings)
4. [Proposed Solutions](#proposed-solutions)
5. [Detailed Implementation Plan](#detailed-implementation-plan)
6. [Files to Modify](#files-to-modify)
7. [Testing Plan](#testing-plan)
8. [Open Questions](#open-questions)

---

## Problem Statement

### The Scenario

1. User creates a note (Note A) targeting a network (ntx) account
2. User's transaction succeeds - Note A is submitted to the network
3. Network account consumes Note A via the ntx-builder on the node
4. **During consumption, the network account's logic creates OUTPUT notes**
5. If those output note scripts aren't registered in the node's script registry, the ntx fails silently

### Why This Is Problematic

- The user sees their transaction succeed (Note A was created)
- The ntx failure happens later on the node side
- Logs are only available on the node, making debugging nearly impossible for users/developers
- There's no feedback mechanism to tell the user what went wrong

### Core Challenge

The client needs to:
1. Know what scripts the network transaction will use for its output notes
2. Ensure those scripts are registered on the node before the ntx executes
3. If not registered, provide a mechanism to register them

---

## Background: How NTX Works

### Network Transaction Builder (ntx-builder) Architecture

**Location:** `miden-node/crates/ntx-builder/`

The ntx-builder on the node:
- Maintains account actors (one per network account)
- Monitors mempool for notes targeting network accounts
- Executes transactions consuming those notes
- Uses an LRU script cache (~1000 entries) to avoid repeated RPC calls

**Key Configuration:**
```rust
pub struct NtxBuilderConfig {
    pub max_notes_per_tx: NonZeroUsize,     // Default: 20
    pub max_concurrent_txs: usize,          // Default: 4
    pub max_note_attempts: usize,           // Default: 30
    pub script_cache_size: NonZeroUsize,    // Default: 1000
}
```

**Execution Flow:**
1. Filter notes via `NoteConsumptionChecker::check_notes_consumability`
2. Execute transaction via `TransactionExecutor::execute_transaction`
3. Prove transaction (local or remote prover)
4. Submit to block producer

### How Output Notes Are Created

Output notes are **NOT pre-defined**. They are:
1. Created by the network account's code during execution
2. Built via `OutputNoteBuilder` in the transaction kernel
3. Determined by the input note's script and the account's procedures

The input note script defines what happens when consumed, which may include calling account procedures that create output notes.

### Script Registration Mechanism

Scripts are registered by creating **public notes** with those scripts. The node stores scripts from public notes in its `note_script` table. When a note is created, the node looks up its script by root hash - if not found, the transaction fails.

---

## Codebase Investigation Findings

### 1. TransactionRequest's `expected_future_notes` Field

**Location:** `crates/rust-client/src/transaction/request/mod.rs:78-82`

```rust
/// A map of details and tags of notes we expect to be created as part of future transactions
/// with their respective tags.
///
/// For example, after a swap note is consumed, a payback note is expected to be created.
expected_future_notes: BTreeMap<NoteId, (NoteDetails, NoteTag)>,
```

**Key Insight:** `NoteDetails` contains `NoteRecipient`, which contains `NoteScript`:

```rust
// NoteRecipient structure (from miden-protocol)
pub struct NoteRecipient {
    serial_num: Word,
    script: NoteScript,    // <-- THE SCRIPT IS HERE
    inputs: NoteInputs,
    digest: Word,
}
```

**Current Usage:** Swap transactions already use this for payback notes:

```rust
// In build_swap() - crates/rust-client/src/transaction/request/builder.rs:344-368
let (created_note, payback_note_details) = create_swap_note(
    swap_data.account_id(),
    swap_data.offered_asset(),
    swap_data.requested_asset(),
    note_type,
    NoteAttachment::default(),
    payback_note_type,
    NoteAttachment::default(),
    rng,
)?;

let payback_tag = NoteTag::with_account_target(swap_data.account_id());

self.expected_future_notes(vec![(payback_note_details, payback_tag)])
    .own_output_notes(vec![OutputNote::Full(created_note)])
    .build()
```

**Current Limitation:** Future note scripts are tracked for client-side state management but are **NOT validated** against the node's script registry. They're stored in `TransactionResult.future_notes` but never checked for registration.

### 2. Current Script Storage in Client

**Location:** `crates/rust-client/src/transaction/mod.rs:299-304`

```rust
// Upsert note scripts for later retrieval from the client's DataStore
let output_note_scripts: Vec<NoteScript> = transaction_request
    .expected_output_recipients()
    .map(|n| n.script().clone())
    .collect();
self.store.upsert_note_scripts(&output_note_scripts).await?;
```

This stores scripts from `expected_output_recipients` (notes the transaction creates directly), but NOT from `expected_future_notes` (notes that will be created when consuming the transaction's output notes).

### 3. RPC Capabilities for Script Checking

**Check if script exists:**

```rust
// crates/rust-client/src/rpc/mod.rs:345-349
/// Fetches the note script with the specified root.
///
/// Errors:
/// - [`RpcError::ExpectedDataMissing`] if the note with the specified root is not found.
async fn get_note_script_by_root(&self, root: Word) -> Result<NoteScript, RpcError>;
```

**Error handling for not found:**

```rust
// crates/rust-client/src/rpc/errors.rs:93-96
#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("resource not found")]
    NotFound,
    // ...
}
```

When a script is not found, the RPC returns:
```rust
RpcError::GrpcError {
    endpoint: NodeRpcClientEndpoint::GetNoteScriptByRoot,
    error_kind: GrpcError::NotFound,
    ..
}
```

### 4. Fetch Network Account for Simulation

**Location:** `crates/rust-client/src/rpc/mod.rs:179-183`

```rust
/// Fetches the current state of an account from the node using the `/GetAccountDetails` RPC
/// endpoint.
async fn get_account_details(&self, account_id: AccountId) -> Result<FetchedAccount, RpcError>;
```

Returns:
- `FetchedAccount::Public(account, summary)` - Full `Account` with code, vault, storage for public accounts
- `FetchedAccount::Private(..)` - Only commitment for private accounts

### 5. Existing Execution Infrastructure

The client already has all the building blocks for local simulation:

- **`TransactionExecutor`** (`miden-tx`) - Executes transactions locally
- **`NoteConsumptionChecker`** - Checks if notes can be consumed by an account
- **`ClientDataStore`** (`crates/rust-client/src/store/data_store.rs`) - Provides data for transaction execution
- **`ForeignAccount` support** - Can fetch and use account data from network

---

## Proposed Solutions

### Solution 1: Use `expected_future_notes` for Script Registration

**Concept:** Developers specify what notes the ntx will produce using the existing `expected_future_notes` field. The client extracts scripts and ensures they're registered.

**Workflow:**
1. Developer creates `TransactionRequest` with `expected_future_notes` containing the notes the ntx will produce
2. Client extracts scripts via `note_details.recipient().script().clone()`
3. Client checks if scripts are registered via `get_note_script_by_root()`
4. If not registered:
   - Option A: Return error with clear message listing unregistered scripts
   - Option B: Auto-register by creating public registration notes

**Pros:**
- Uses existing field (no new API needed)
- Already established pattern for swap payback notes
- Simple implementation
- Developer explicitly declares expectations

**Cons:**
- Requires developer to know what the ntx will produce
- Static approach - doesn't discover scripts dynamically

### Solution 2: Local NTX Dry-Run (Simulation)

**Concept:** Client fetches the network account's state and code, then simulates the note consumption locally to discover what output notes would be created.

**Workflow:**
1. User creates a note targeting a network account
2. Before submitting, client calls `dry_run_ntx(network_account_id, input_note)`
3. Client fetches network account via `get_account_details()`
4. Client executes note consumption locally using `TransactionExecutor`
5. Client extracts output notes (with scripts) from execution result
6. Client checks which scripts need registration
7. Returns `NtxDryRunResult` with output notes and unregistered scripts

**Pros:**
- Dynamic discovery of output note scripts
- No developer knowledge required about ntx internals
- Catches unexpected output notes

**Cons:**
- More complex implementation
- Cannot simulate private network accounts (only public accounts have accessible code)
- Potential storage/state incompleteness for accounts with large storage
- Additional RPC calls for account data

### Recommended Approach

Implement **both solutions** as complementary features:

1. **Solution 1** for the common case where developers know what the ntx produces
2. **Solution 2** as an optional diagnostic tool for debugging or when the developer doesn't know

---

## Detailed Implementation Plan

### Phase 1: Expected Future Notes Script Validation

#### 1.1 Add Error Type for Unregistered Scripts

**File:** `crates/rust-client/src/transaction/request/mod.rs`

Add to `TransactionRequestError` enum (around line 427):

```rust
#[error("note scripts required for ntx execution are not registered on the node: {scripts:?}")]
NtxScriptsNotRegistered {
    /// Script roots that need to be registered on the node.
    scripts: Vec<Word>,
},
```

#### 1.2 Add Script Extraction and Validation Method

**File:** `crates/rust-client/src/transaction/mod.rs`

Add a new method to the `Client` impl block (after `validate_request` around line 615):

```rust
/// Checks if note scripts from expected future notes are registered on the node.
///
/// This is useful when creating transactions that target network accounts (ntx).
/// Network transactions may create output notes whose scripts must be registered
/// on the node for the transaction to succeed.
///
/// # Returns
///
/// A vector of tuples containing (script_root, script) for scripts that are NOT
/// registered on the node. Returns an empty vector if all scripts are registered.
///
/// # Example
///
/// ```no_run
/// let unregistered = client.get_unregistered_ntx_scripts(&transaction_request).await?;
/// if !unregistered.is_empty() {
///     // Handle unregistered scripts - either error or register them
///     let script_roots: Vec<Word> = unregistered.iter().map(|(root, _)| *root).collect();
///     return Err(ClientError::TransactionRequestError(
///         TransactionRequestError::NtxScriptsNotRegistered { scripts: script_roots }
///     ));
/// }
/// ```
pub async fn get_unregistered_ntx_scripts(
    &self,
    transaction_request: &TransactionRequest,
) -> Result<Vec<(Word, NoteScript)>, ClientError> {
    let mut unregistered = Vec::new();

    for (note_details, _tag) in transaction_request.expected_future_notes() {
        let script = note_details.recipient().script();
        let script_root = script.root();

        match self.rpc_api.get_note_script_by_root(script_root).await {
            Ok(_) => continue, // Already registered
            Err(RpcError::GrpcError { error_kind: GrpcError::NotFound, .. }) => {
                unregistered.push((script_root, script.clone()));
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(unregistered)
}
```

**Required imports** (add to existing imports around line 73):
```rust
use crate::rpc::{GrpcError, RpcError};
```

#### 1.3 Add Registration Note Factory

**File:** `crates/rust-client/src/transaction/request/builder.rs`

Add a helper function after the `SwapTransactionData` impl block (around line 600):

```rust
// SCRIPT REGISTRATION HELPERS
// ================================================================================================

/// Creates a minimal public note to register a script on the node.
///
/// The note has no assets and targets the sender account so they can consume it
/// to clean up (though this is optional). The primary purpose is to ensure the
/// script is stored in the node's script registry.
///
/// # Arguments
///
/// * `script` - The note script to register
/// * `sender_account_id` - The account creating the registration note
/// * `rng` - Random number generator for generating the note's serial number
///
/// # Returns
///
/// A public note that, when included in a transaction, will cause the script
/// to be registered on the node.
pub fn create_script_registration_note(
    script: NoteScript,
    sender_account_id: AccountId,
    rng: &mut ClientRng,
) -> Result<Note, NoteError> {
    use miden_protocol::note::{NoteAssets, NoteInputs, NoteMetadata, NoteRecipient};

    let serial_num = rng.draw_word();
    let inputs = NoteInputs::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, script, inputs);

    // Target the sender so they can consume it to clean up (optional)
    let tag = NoteTag::with_account_target(sender_account_id);
    let metadata = NoteMetadata::new(
        sender_account_id,
        NoteType::Public,  // Must be public to register the script
        tag,
    )?;

    Note::new(NoteAssets::new(vec![])?, metadata, recipient)
}
```

**Required imports** (add to existing imports):
```rust
use miden_protocol::note::NoteScript;
```

#### 1.4 Add Builder Method for Script Registration

**File:** `crates/rust-client/src/transaction/request/builder.rs`

Add to `TransactionRequestBuilder` impl block (after `auth_arg` method, around line 257):

```rust
/// Adds output notes that will register the provided scripts on the node.
///
/// Creates minimal public notes with each script. When the transaction is
/// executed and submitted, these notes will cause the scripts to be stored
/// in the node's script registry.
///
/// This is useful when creating transactions that target network accounts (ntx)
/// where the ntx will create output notes using these scripts.
///
/// # Arguments
///
/// * `scripts` - The note scripts to register
/// * `sender_account_id` - The account creating the registration notes
/// * `rng` - Random number generator for generating serial numbers
///
/// # Example
///
/// ```no_run
/// // Check for unregistered scripts
/// let unregistered = client.get_unregistered_ntx_scripts(&request).await?;
///
/// if !unregistered.is_empty() {
///     let scripts: Vec<NoteScript> = unregistered.into_iter()
///         .map(|(_, script)| script)
///         .collect();
///
///     // Create a transaction that registers the scripts
///     let registration_request = TransactionRequestBuilder::new()
///         .with_script_registration_notes(scripts, sender_id, &mut rng)?
///         .build()?;
///
///     client.submit_new_transaction(sender_id, registration_request).await?;
/// }
/// ```
pub fn with_script_registration_notes(
    mut self,
    scripts: Vec<NoteScript>,
    sender_account_id: AccountId,
    rng: &mut ClientRng,
) -> Result<Self, TransactionRequestError> {
    for script in scripts {
        let registration_note = create_script_registration_note(
            script,
            sender_account_id,
            rng,
        )?;
        self.own_output_notes.push(OutputNote::Full(registration_note));
    }
    Ok(self)
}
```

---

### Phase 2: Local NTX Dry-Run (Optional Enhancement)

#### 2.1 Define Dry-Run Result Type

**File:** `crates/rust-client/src/transaction/mod.rs`

Add after the imports section (around line 150):

```rust
// NTX DRY-RUN RESULT
// ================================================================================================

/// Result of a network transaction dry-run simulation.
///
/// This is returned by [`Client::dry_run_ntx`] and contains information about
/// what would happen if a network account consumed a specific note.
#[derive(Debug, Clone)]
pub struct NtxDryRunResult {
    /// Output notes that would be created by the network transaction.
    pub output_notes: Vec<Note>,
    /// Scripts from output notes that are not registered on the node.
    /// Each tuple contains (script_root, script).
    pub unregistered_scripts: Vec<(Word, NoteScript)>,
    /// Whether the simulation succeeded.
    pub success: bool,
    /// Error message if simulation failed.
    pub error: Option<String>,
}
```

#### 2.2 Add Error Type for Private Account Simulation

**File:** `crates/rust-client/src/errors.rs`

Add to `ClientError` enum (around line 147):

```rust
#[error("cannot simulate network transaction for private account {0}")]
CannotSimulatePrivateAccount(AccountId),
```

#### 2.3 Implement Dry-Run Method

**File:** `crates/rust-client/src/transaction/mod.rs`

Add to the `Client` impl block (after `get_unregistered_ntx_scripts`):

```rust
/// Simulates what a network account would do when consuming the given note.
///
/// Fetches the network account's state and code, then executes the note
/// consumption locally to discover what output notes would be created.
///
/// # Arguments
///
/// * `network_account_id` - The ID of the network account that will consume the note
/// * `input_note` - The note that will be consumed
///
/// # Returns
///
/// An [`NtxDryRunResult`] containing:
/// - The output notes that would be created
/// - Any scripts that are not registered on the node
/// - Success/failure status and error message if applicable
///
/// # Limitations
///
/// - Cannot simulate private network accounts (their code is not accessible)
/// - Storage state may be incomplete for accounts with very large storage
/// - The simulation uses current network state which may change before actual execution
///
/// # Example
///
/// ```no_run
/// // Create a note targeting a network account
/// let swap_note = create_swap_note(...)?;
///
/// // Dry-run to see what the network account will produce
/// let result = client.dry_run_ntx(network_account_id, swap_note.clone()).await?;
///
/// if !result.success {
///     println!("Simulation failed: {:?}", result.error);
///     return Err(...);
/// }
///
/// if !result.unregistered_scripts.is_empty() {
///     // Register scripts before submitting the actual transaction
///     let scripts: Vec<NoteScript> = result.unregistered_scripts
///         .into_iter()
///         .map(|(_, script)| script)
///         .collect();
///     // ... register scripts ...
/// }
/// ```
pub async fn dry_run_ntx(
    &mut self,
    network_account_id: AccountId,
    input_note: Note,
) -> Result<NtxDryRunResult, ClientError> {
    use miden_protocol::transaction::InputNote as TxInputNote;

    // 1. Check if account is public (we can only simulate public accounts)
    let fetched_account = self.rpc_api.get_account_details(network_account_id).await?;

    let account = match fetched_account {
        FetchedAccount::Public(account, _) => *account,
        FetchedAccount::Private(..) => {
            return Ok(NtxDryRunResult {
                output_notes: vec![],
                unregistered_scripts: vec![],
                success: false,
                error: Some(format!(
                    "Cannot simulate network transaction for private account {}. \
                     Only public accounts can be simulated.",
                    network_account_id
                )),
            });
        }
    };

    // 2. Get current block for execution context
    let block_num = self.store.get_sync_height().await?;

    // 3. Create data store and load account code
    let data_store = ClientDataStore::new(self.store.clone());
    data_store.mast_store().load_account_code(account.code());

    // 4. Build input notes
    let input_notes = InputNotes::new(vec![
        TxInputNote::Unauthenticated { note: input_note.clone() }
    ])?;

    // 5. Execute the transaction locally
    let tx_args = TransactionArgs::default();

    let execution_result = match self
        .build_executor(&data_store)?
        .execute_transaction(
            network_account_id,
            block_num,
            input_notes,
            tx_args,
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return Ok(NtxDryRunResult {
                output_notes: vec![],
                unregistered_scripts: vec![],
                success: false,
                error: Some(format!("Transaction execution failed: {}", e)),
            });
        }
    };

    // 6. Extract output notes
    let output_notes: Vec<Note> = notes_from_output(execution_result.output_notes())
        .cloned()
        .collect();

    // 7. Check which scripts are unregistered
    let mut unregistered_scripts = Vec::new();
    for note in &output_notes {
        let script = note.script();
        let script_root = script.root();

        match self.rpc_api.get_note_script_by_root(script_root).await {
            Ok(_) => continue, // Already registered
            Err(RpcError::GrpcError { error_kind: GrpcError::NotFound, .. }) => {
                unregistered_scripts.push((script_root, script.clone()));
            }
            Err(_) => continue, // Ignore other errors for dry-run
        }
    }

    Ok(NtxDryRunResult {
        output_notes,
        unregistered_scripts,
        success: true,
        error: None,
    })
}
```

**Required imports** (add if not present):
```rust
use crate::rpc::domain::account::FetchedAccount;
```

---

## Files to Modify

| File | Changes | Priority |
|------|---------|----------|
| `crates/rust-client/src/transaction/request/mod.rs` | Add `NtxScriptsNotRegistered` error variant | High |
| `crates/rust-client/src/transaction/request/builder.rs` | Add `create_script_registration_note()`, `with_script_registration_notes()` | High |
| `crates/rust-client/src/transaction/mod.rs` | Add `get_unregistered_ntx_scripts()`, `NtxDryRunResult`, `dry_run_ntx()` | High |
| `crates/rust-client/src/errors.rs` | Add `CannotSimulatePrivateAccount` error (for Phase 2) | Medium |
| `crates/rust-client/src/lib.rs` | Re-export new public types | Medium |

## Existing Code to Reuse

| Component | Location | Purpose |
|-----------|----------|---------|
| `expected_future_notes` | `transaction/request/mod.rs:78-82` | Existing field for specifying future notes |
| `NoteDetails.recipient().script()` | miden-protocol | Extract script from note details |
| `get_note_script_by_root` | `rpc/mod.rs:345-349` | Check if script is registered |
| `get_account_details` | `rpc/mod.rs:179-183` | Fetch network account state |
| `TransactionExecutor` | miden-tx | Execute transactions locally |
| `NoteConsumptionChecker` | miden-tx | Check note consumability |
| `ClientDataStore` | `store/data_store.rs` | Provide data for execution |
| `notes_from_output` | `transaction/mod.rs:840-852` | Extract full notes from output |
| `create_swap_note` | miden-standards | Example of creating notes with future note expectations |

---

## Testing Plan

### Unit Tests

**File:** `crates/rust-client/src/transaction/request/mod.rs` (add to existing tests module)

```rust
#[test]
fn test_ntx_scripts_not_registered_error() {
    let scripts = vec![Word::default(), [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]];
    let error = TransactionRequestError::NtxScriptsNotRegistered { scripts: scripts.clone() };

    assert!(error.to_string().contains("not registered"));
    if let TransactionRequestError::NtxScriptsNotRegistered { scripts: err_scripts } = error {
        assert_eq!(err_scripts, scripts);
    } else {
        panic!("Wrong error variant");
    }
}
```

**File:** `crates/rust-client/src/transaction/request/builder.rs` (add tests)

```rust
#[cfg(test)]
mod registration_tests {
    use super::*;
    use miden_protocol::crypto::rand::RpoRandomCoin;

    #[test]
    fn test_create_script_registration_note() {
        let mut rng = RpoRandomCoin::new(Word::default());
        let sender_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

        // Create a simple script
        let script = NoteScript::compile("begin nop end").unwrap();

        let note = create_script_registration_note(script.clone(), sender_id, &mut rng).unwrap();

        // Verify note properties
        assert_eq!(note.metadata().note_type(), NoteType::Public);
        assert_eq!(note.metadata().sender(), sender_id);
        assert!(note.assets().is_empty());
        assert_eq!(note.script().root(), script.root());
    }

    #[test]
    fn test_with_script_registration_notes() {
        let mut rng = RpoRandomCoin::new(Word::default());
        let sender_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

        let script1 = NoteScript::compile("begin nop end").unwrap();
        let script2 = NoteScript::compile("begin push.1 drop end").unwrap();

        let request = TransactionRequestBuilder::new()
            .with_script_registration_notes(vec![script1, script2], sender_id, &mut rng)
            .unwrap()
            .build()
            .unwrap();

        // Should have created 2 output notes
        let output_notes = request.expected_output_own_notes();
        assert_eq!(output_notes.len(), 2);

        for note in output_notes {
            assert_eq!(note.metadata().note_type(), NoteType::Public);
        }
    }
}
```

### Integration Tests

**File:** `bin/integration-tests/src/tests/ntx_script_registration.rs` (new file)

```rust
//! Integration tests for NTX script registration.

use miden_client::{
    Client,
    transaction::{TransactionRequestBuilder, NtxDryRunResult},
};
use miden_protocol::note::{NoteScript, NoteType};

/// Tests that `get_unregistered_ntx_scripts` correctly identifies unregistered scripts.
#[tokio::test]
async fn test_get_unregistered_ntx_scripts() {
    // Setup: Create client and accounts
    let mut client = create_test_client().await;
    let sender_account = create_test_account(&mut client).await;

    // Create a transaction request with expected future notes containing a custom script
    let custom_script = NoteScript::compile("begin push.42 drop end").unwrap();

    // Build note details with the custom script
    let note_details = create_note_details_with_script(custom_script.clone());
    let tag = NoteTag::with_account_target(sender_account.id());

    let request = TransactionRequestBuilder::new()
        .expected_future_notes(vec![(note_details, tag)])
        .build()
        .unwrap();

    // Check for unregistered scripts
    let unregistered = client.get_unregistered_ntx_scripts(&request).await.unwrap();

    // The custom script should be unregistered
    assert_eq!(unregistered.len(), 1);
    assert_eq!(unregistered[0].0, custom_script.root());
}

/// Tests the full script registration flow.
#[tokio::test]
async fn test_script_registration_flow() {
    let mut client = create_test_client().await;
    let sender_account = create_test_account(&mut client).await;
    let mut rng = client.rng();

    // 1. Create a custom script that's not registered
    let custom_script = NoteScript::compile("begin push.42 drop end").unwrap();

    // 2. Verify it's not registered
    let result = client.rpc_api.get_note_script_by_root(custom_script.root()).await;
    assert!(matches!(result, Err(RpcError::GrpcError { error_kind: GrpcError::NotFound, .. })));

    // 3. Create and submit a registration transaction
    let registration_request = TransactionRequestBuilder::new()
        .with_script_registration_notes(vec![custom_script.clone()], sender_account.id(), &mut rng)
        .unwrap()
        .build()
        .unwrap();

    client.submit_new_transaction(sender_account.id(), registration_request).await.unwrap();

    // 4. Wait for transaction to be committed
    client.sync_state().await.unwrap();

    // 5. Verify script is now registered
    let result = client.rpc_api.get_note_script_by_root(custom_script.root()).await;
    assert!(result.is_ok());
}

/// Tests dry-run simulation for network accounts.
#[tokio::test]
async fn test_dry_run_ntx() {
    let mut client = create_test_client().await;

    // Setup: Deploy a network account that creates output notes when consuming input notes
    let network_account = deploy_test_network_account(&mut client).await;

    // Create an input note for the network account
    let input_note = create_test_note_for_account(network_account.id());

    // Run dry-run simulation
    let result = client.dry_run_ntx(network_account.id(), input_note).await.unwrap();

    // Verify simulation succeeded
    assert!(result.success, "Dry-run should succeed: {:?}", result.error);

    // Check output notes were discovered
    assert!(!result.output_notes.is_empty(), "Should have discovered output notes");

    // Log unregistered scripts if any
    for (root, _script) in &result.unregistered_scripts {
        println!("Unregistered script root: {:?}", root);
    }
}
```

**Add to test module:** `bin/integration-tests/src/tests/mod.rs`

```rust
mod ntx_script_registration;
```

---

## Open Questions

### 1. Registration Note Consumption

**Question:** Should registration notes target the sender (so they can consume them immediately) or use a special "registry" tag?

**Recommendation:** Target the sender for simplicity. The sender can optionally consume the note later to clean up, but this isn't required. The script is registered as soon as the note is created.

### 2. Auto-Registration vs Manual

**Question:** Should the client automatically add registration notes when it detects unregistered scripts, or require explicit opt-in?

**Recommendation:** Require explicit opt-in. Auto-registration could surprise users with unexpected transaction fees and output notes. Instead:
- Provide `get_unregistered_ntx_scripts()` to detect the issue
- Provide `with_script_registration_notes()` to fix it
- Let developers choose their approach

### 3. Validation in execute_transaction

**Question:** Should `execute_transaction` automatically call `get_unregistered_ntx_scripts` and return an error if scripts are unregistered?

**Recommendation:** No, keep it optional. Reasons:
- Not all transactions have `expected_future_notes`
- Validation adds RPC calls (latency)
- Some users might want to skip validation
- Better to let users call validation explicitly when needed

### 4. Dry-Run Caching

**Question:** Should we cache network account state to avoid repeated RPC calls when dry-running multiple notes for the same account?

**Recommendation:** Start without caching. Add caching later if performance becomes an issue. Network account state can change between calls, so caching could lead to incorrect results.

### 5. Storage Completeness for Dry-Run

**Question:** For network accounts with large storage maps, how should we handle incomplete storage state during simulation?

**Recommendation:** Document the limitation. The simulation uses whatever state is returned by `get_account_details`. If the account has very large storage that isn't fully returned, the simulation might fail or behave differently than the actual transaction. This is an inherent limitation of client-side simulation.

---

## Summary

This plan provides two complementary approaches for handling ntx script registration:

1. **Solution 1 (expected_future_notes validation):** Simple, developer-driven approach where scripts are specified upfront via `expected_future_notes`. The client provides methods to:
   - Check which scripts are unregistered (`get_unregistered_ntx_scripts`)
   - Create registration notes (`with_script_registration_notes`)
   - Report clear errors when scripts are missing (`NtxScriptsNotRegistered`)

2. **Solution 2 (dry-run simulation):** Dynamic approach that simulates the ntx locally via `dry_run_ntx`. Best for:
   - Debugging ntx failures
   - Discovering what scripts an ntx will use
   - Cases where the developer doesn't know (or shouldn't need to know) what the ntx produces

Both solutions share the same underlying mechanism: detecting unregistered scripts via `get_note_script_by_root` RPC and registering them by creating public notes with those scripts.

### Implementation Priority

1. **Phase 1** (High Priority): Expected future notes validation
   - Add error type
   - Add `get_unregistered_ntx_scripts`
   - Add `create_script_registration_note`
   - Add `with_script_registration_notes`

2. **Phase 2** (Medium Priority): Dry-run simulation
   - Add `NtxDryRunResult`
   - Add `dry_run_ntx`

3. **Phase 3** (Lower Priority): Integration tests and documentation
