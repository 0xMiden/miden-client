## Troubleshooting and transaction lifecycle

This guide helps you troubleshoot common issues and understand the end-to-end lifecycle of transactions and notes in the Miden client.

For a complete list of all error codes, see the [Error Reference](./error-reference.md).

### Actionable error hints

#### `MIDEN-CL-018` — `ClientError::MissingOutputRecipients`
- Cause: The MASM program emitted an output note whose recipient was not listed in `TransactionRequestBuilder::expected_output_recipients(...)`.
- Fix: Reconcile the MASM recipient data with the Rust note structs and update the expected recipients so that the expected recipients are part of the transaction outputs.

#### `MIDEN-TX-007` — `TransactionRequestError::InputNoteNotAuthenticated`
- Cause: A note ID included in the transaction request did not have a corresponding `InputNoteRecord` in the store, or it was not found to contain authentication data.
- Fix: Import or sync the note, so its record and inclusion proof are present before building and executing the request.

#### `MIDEN-TX-013` — `TransactionRequestError::NoInputNotesNorAccountChange`
- Cause: The transaction neither consumes input notes nor mutates tracked account state.
- Fix: Add at least one authenticated/unauthenticated input note or include an explicit account state update in the request.

#### `MIDEN-TX-019` — `TransactionRequestError::StorageSlotNotFound`
- Cause: The request referenced an account storage slot that does not exist, often because the ABI layout is incorrectly addressed (auth component is always the first component in the account component list).
- Fix: Verify the account ABI and component ordering, then adjust the slot index used in the transaction.

#### `TransactionExecutorError::ForeignAccountNotAnchoredInReference`
- Cause: The foreign account proof was generated against a different block than the request's reference block.
- Fix: Re-fetch the foreign account proof anchored at the correct reference block and retry.

#### `TransactionExecutorError::TransactionProgramExecutionFailed`
- Cause: The MASM kernel failed during execution (e.g., failed assertion or constraint violation).
- Fix: Re-run with the debug mode, capture VM diagnostics and inspect the source manager output to understand why execution failed.

#### `MIDEN-ST-008` — `ClientError::StoreError(AccountCommitmentMismatch(...))`
- Cause: The final account commitment already exists locally, usually because the transaction was applied previously.
- Fix: Sync to confirm the transaction status and avoid resubmitting it; if you need a clean slate for development, reset the store.

#### `MIDEN-CL-013` / `MIDEN-RP-008` — `NoteNotFoundOnChain` / `RpcError::NoteNotFound`
- Cause: The note has not been found on chain, or the input ID is incorrect.
- Fix: Verify the note ID, ensure it has been committed, and run sync the client before retrying.
