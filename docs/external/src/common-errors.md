---
title: Common Errors
---

# Troubleshooting and common errors

This guide helps you troubleshoot common issues when using the Miden Client.

## Connection and sync errors

#### `RpcError::ConnectionError`
- **Cause:** Could not reach the Miden node.
- **Fix:** Check that the node endpoint in your configuration is correct and that the node is running.

#### `RpcError::AcceptHeaderError`
- **Cause:** The node rejected the request due to a version mismatch between the client and node.
- **Fix:** Ensure your client version is compatible with the node version. Update one or the other to a compatible release.

## Account errors

#### `ClientError::AccountLocked`
- **Cause:** The account is locked because the client may be missing its latest state. This can happen when the account is shared across clients and another client executed a transaction.
- **Fix:** Run `sync` to fetch the latest state from the network.

#### `ClientError::AccountNonceTooLow`
- **Cause:** The account you are trying to import has an older nonce than the version already tracked locally.
- **Fix:** Run `sync` to ensure your local state is current, or re-export the account from a more up-to-date source.

#### `ClientError::AccountNotFoundOnChain`
- **Cause:** The account was not found on the network. It may not have been committed yet, or the ID is incorrect.
- **Fix:** Verify the account ID and ensure the account has been committed on chain. For new accounts, wait for the transaction that created it to be confirmed and run `sync`.

#### `ClientError::AccountIsPrivate`
- **Cause:** Cannot retrieve private account details from the network. Private account state is not stored on chain.
- **Fix:** Private accounts can only be accessed locally. Use the account file export/import workflow to transfer private accounts between clients.

#### `ClientError::AddNewAccountWithoutSeed`
- **Cause:** New accounts require a seed to derive their initial state.
- **Fix:** Use `Client::new_account()` (Rust) or `client.accounts.create()` (TypeScript) which generates the seed automatically. If importing manually, provide the seed.

## Note errors

#### `ClientError::NoteNotFoundOnChain` / `RpcError::NoteNotFound`
- **Cause:** The note has not been found on chain, or the note ID is incorrect.
- **Fix:** Verify the note ID, ensure it has been committed, and sync the client before retrying.

#### `ClientError::NoConsumableNoteForAccount`
- **Cause:** No notes were found that the specified account can consume.
- **Fix:** Run `sync` to fetch the latest notes from the network. Verify that notes targeting this account have been committed on chain.

#### `TransactionRequestError::InputNoteNotAuthenticated`
- **Cause:** The note needs an inclusion proof before it can be consumed as an authenticated input.
- **Fix:** Run `sync` to fetch the latest proofs from the network, then retry the transaction.

## Transaction request errors

#### `TransactionRequestError::NoInputNotesNorAccountChange`
- **Cause:** The transaction neither consumes input notes nor mutates tracked account state.
- **Fix:** Add at least one authenticated/unauthenticated input note or include an explicit account state update in the request.

#### `TransactionRequestError::InvalidSenderAccount`
- **Cause:** The sender account is not tracked by this client.
- **Fix:** Import or create the account first, then retry the transaction.

#### `TransactionRequestError::P2IDNoteWithoutAsset`
- **Cause:** A pay-to-ID (P2ID) note must transfer at least one asset to the target account.
- **Fix:** Add at least one fungible or non-fungible asset to the note.

#### `TransactionRequestError::MissingAuthenticatedInputNote`
- **Cause:** A note ID included in `authenticated_input_notes` did not have a corresponding record in the store, or it was missing authentication data.
- **Fix:** Import or sync the note so its record and inclusion proof are present before building and executing the request.

#### `TransactionRequestError::StorageSlotNotFound`
- **Cause:** The request referenced an account storage slot that does not exist. This often happens because the ABI layout is incorrectly addressed (the auth component is always the first component in the account component list).
- **Fix:** Verify the account ABI and component ordering, then adjust the slot index used in the transaction.

## Transaction execution errors

#### `ClientError::MissingOutputRecipients`
- **Cause:** The MASM program emitted an output note whose recipient was not listed in the expected output recipients.
- **Fix:** Reconcile the MASM recipient data with the note structs and update the expected recipients so they match the transaction outputs.

#### `TransactionExecutorError::ForeignAccountNotAnchoredInReference`
- **Cause:** The foreign account proof was generated against a different block than the request's reference block.
- **Fix:** Re-fetch the foreign account proof anchored at the correct reference block and retry.

#### `TransactionExecutorError::TransactionProgramExecutionFailed`
- **Cause:** The MASM kernel failed during execution (e.g., failed assertion or constraint violation).
- **Fix:** Re-run with debug mode enabled, capture VM diagnostics, and inspect the source manager output to understand why execution failed.

## Store errors

#### `ClientError::StoreError(AccountCommitmentAlreadyExists)`
- **Cause:** The final account commitment already exists locally, usually because the transaction was already applied.
- **Fix:** Sync to confirm the transaction status and avoid resubmitting it. If you need a clean slate for development, reset the store.

#### `ClientError::RecencyConditionError`
- **Cause:** The transaction failed a recency check — the reference block is too old relative to the current chain tip.
- **Fix:** Sync to get the latest block data, then rebuild and resubmit the transaction.
