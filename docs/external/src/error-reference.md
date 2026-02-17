---
sidebar_position: 99
---

# Error Reference

Every error raised by the Miden client carries a **stable error code** in the format
`MIDEN-XX-NNN`. These codes are intended for programmatic matching and
troubleshooting — they never change, even if the underlying message or variant is
renamed.

## Quick-Reference Table

| Code | Variant | Module |
|------|---------|--------|
| `MIDEN-CL-001` | `ClientError::AddressAlreadyTracked` | client |
| `MIDEN-CL-002` | `ClientError::AccountAlreadyTracked` | client |
| `MIDEN-CL-003` | `ClientError::NoteTagDerivedAddressAlreadyTracked` | client |
| `MIDEN-CL-004` | `ClientError::AccountError` | client |
| `MIDEN-CL-005` | `ClientError::AccountLocked` | client |
| `MIDEN-CL-006` | `ClientError::AccountCommitmentMismatch` | client |
| `MIDEN-CL-007` | `ClientError::AccountIsPrivate` | client |
| `MIDEN-CL-008` | `ClientError::AccountNonceTooLow` | client |
| `MIDEN-CL-009` | `ClientError::AssetError` | client |
| `MIDEN-CL-010` | `ClientError::AccountDataNotFound` | client |
| `MIDEN-CL-011` | `ClientError::PartialBlockchainError` | client |
| `MIDEN-CL-012` | `ClientError::DataDeserializationError` | client |
| `MIDEN-CL-013` | `ClientError::NoteNotFoundOnChain` | client |
| `MIDEN-CL-014` | `ClientError::HexParseError` | client |
| `MIDEN-CL-015` | `ClientError::InvalidPartialMmrForest` | client |
| `MIDEN-CL-016` | `ClientError::AddNewAccountWithoutSeed` | client |
| `MIDEN-CL-017` | `ClientError::MerkleError` | client |
| `MIDEN-CL-018` | `ClientError::MissingOutputRecipients` | client |
| `MIDEN-CL-019` | `ClientError::NoteError` | client |
| `MIDEN-CL-020` | `ClientError::NoteCheckerError` | client |
| `MIDEN-CL-021` | `ClientError::NoteImportError` | client |
| `MIDEN-CL-022` | `ClientError::NoteRecordConversionError` | client |
| `MIDEN-CL-023` | `ClientError::NoteTransportError` | client |
| `MIDEN-CL-024` | `ClientError::NoConsumableNoteForAccount` | client |
| `MIDEN-CL-025` | `ClientError::RpcError` | client |
| `MIDEN-CL-026` | `ClientError::RecencyConditionError` | client |
| `MIDEN-CL-027` | `ClientError::NoteScreenerError` | client |
| `MIDEN-CL-028` | `ClientError::StoreError` | client |
| `MIDEN-CL-029` | `ClientError::TransactionExecutorError` | client |
| `MIDEN-CL-030` | `ClientError::TransactionInputError` | client |
| `MIDEN-CL-031` | `ClientError::TransactionProvingError` | client |
| `MIDEN-CL-032` | `ClientError::TransactionRequestError` | client |
| `MIDEN-CL-033` | `ClientError::AccountInterfaceError` | client |
| `MIDEN-CL-034` | `ClientError::TransactionScriptError` | client |
| `MIDEN-CL-035` | `ClientError::ClientInitializationError` | client |
| `MIDEN-CL-036` | `ClientError::NoteTagsLimitExceeded` | client |
| `MIDEN-CL-037` | `ClientError::AccountsLimitExceeded` | client |
| `MIDEN-CL-038` | `ClientError::UnsupportedAuthSchemeId` | client |
| `MIDEN-CL-039` | `ClientError::AccountRecordNotFull` | client |
| `MIDEN-CL-040` | `ClientError::AccountRecordNotPartial` | client |
| `MIDEN-IP-001` | `IdPrefixFetchError::NoMatch` | client |
| `MIDEN-IP-002` | `IdPrefixFetchError::MultipleMatches` | client |
| `MIDEN-RP-001` | `RpcError::AcceptHeaderError` | rpc |
| `MIDEN-RP-002` | `RpcError::AccountUpdateForPrivateAccountReceived` | rpc |
| `MIDEN-RP-003` | `RpcError::ConnectionError` | rpc |
| `MIDEN-RP-004` | `RpcError::DeserializationError` | rpc |
| `MIDEN-RP-005` | `RpcError::ExpectedDataMissing` | rpc |
| `MIDEN-RP-006` | `RpcError::InvalidResponse` | rpc |
| `MIDEN-RP-007` | `RpcError::GrpcError` | rpc |
| `MIDEN-RP-008` | `RpcError::NoteNotFound` | rpc |
| `MIDEN-RP-009` | `RpcError::InvalidNodeEndpoint` | rpc |
| `MIDEN-GR-001` | `GrpcError::NotFound` | rpc |
| `MIDEN-GR-002` | `GrpcError::InvalidArgument` | rpc |
| `MIDEN-GR-003` | `GrpcError::PermissionDenied` | rpc |
| `MIDEN-GR-004` | `GrpcError::AlreadyExists` | rpc |
| `MIDEN-GR-005` | `GrpcError::ResourceExhausted` | rpc |
| `MIDEN-GR-006` | `GrpcError::FailedPrecondition` | rpc |
| `MIDEN-GR-007` | `GrpcError::Cancelled` | rpc |
| `MIDEN-GR-008` | `GrpcError::DeadlineExceeded` | rpc |
| `MIDEN-GR-009` | `GrpcError::Unavailable` | rpc |
| `MIDEN-GR-010` | `GrpcError::Internal` | rpc |
| `MIDEN-GR-011` | `GrpcError::Unimplemented` | rpc |
| `MIDEN-GR-012` | `GrpcError::Unauthenticated` | rpc |
| `MIDEN-GR-013` | `GrpcError::Aborted` | rpc |
| `MIDEN-GR-014` | `GrpcError::OutOfRange` | rpc |
| `MIDEN-GR-015` | `GrpcError::DataLoss` | rpc |
| `MIDEN-GR-016` | `GrpcError::Unknown` | rpc |
| `MIDEN-AH-001` | `AcceptHeaderError::NoSupportedMediaRange` | rpc |
| `MIDEN-AH-002` | `AcceptHeaderError::ParsingError` | rpc |
| `MIDEN-RC-001` | `RpcConversionError::DeserializationError` | rpc |
| `MIDEN-RC-002` | `RpcConversionError::NotAValidFelt` | rpc |
| `MIDEN-RC-003` | `RpcConversionError::NoteTypeError` | rpc |
| `MIDEN-RC-004` | `RpcConversionError::MerkleError` | rpc |
| `MIDEN-RC-005` | `RpcConversionError::InvalidField` | rpc |
| `MIDEN-RC-006` | `RpcConversionError::InvalidInt` | rpc |
| `MIDEN-RC-007` | `RpcConversionError::MissingFieldInProtobufRepresentation` | rpc |
| `MIDEN-ST-001` | `StoreError::AssetError` | store |
| `MIDEN-ST-002` | `StoreError::AssetVaultError` | store |
| `MIDEN-ST-003` | `StoreError::AccountCodeDataNotFound` | store |
| `MIDEN-ST-004` | `StoreError::AccountDataNotFound` | store |
| `MIDEN-ST-005` | `StoreError::AccountError` | store |
| `MIDEN-ST-006` | `StoreError::AddressError` | store |
| `MIDEN-ST-007` | `StoreError::AccountIdError` | store |
| `MIDEN-ST-008` | `StoreError::AccountCommitmentMismatch` | store |
| `MIDEN-ST-009` | `StoreError::AccountKeyNotFound` | store |
| `MIDEN-ST-010` | `StoreError::AccountStorageRootNotFound` | store |
| `MIDEN-ST-011` | `StoreError::AccountStorageIndexNotFound` | store |
| `MIDEN-ST-012` | `StoreError::BlockHeaderNotFound` | store |
| `MIDEN-ST-013` | `StoreError::PartialBlockchainNodeNotFound` | store |
| `MIDEN-ST-014` | `StoreError::DataDeserializationError` | store |
| `MIDEN-ST-015` | `StoreError::DatabaseError` | store |
| `MIDEN-ST-016` | `StoreError::HexParseError` | store |
| `MIDEN-ST-017` | `StoreError::InvalidInt` | store |
| `MIDEN-ST-018` | `StoreError::NoteRecordError` | store |
| `MIDEN-ST-019` | `StoreError::MerkleStoreError` | store |
| `MIDEN-ST-020` | `StoreError::MmrError` | store |
| `MIDEN-ST-021` | `StoreError::NoteInclusionProofError` | store |
| `MIDEN-ST-022` | `StoreError::NoteTagAlreadyTracked` | store |
| `MIDEN-ST-023` | `StoreError::NoteTransportCursorNotFound` | store |
| `MIDEN-ST-024` | `StoreError::NoteScriptNotFound` | store |
| `MIDEN-ST-025` | `StoreError::ParsingError` | store |
| `MIDEN-ST-026` | `StoreError::QueryError` | store |
| `MIDEN-ST-027` | `StoreError::SmtProofError` | store |
| `MIDEN-ST-028` | `StoreError::StorageMapError` | store |
| `MIDEN-ST-029` | `StoreError::TransactionScriptError` | store |
| `MIDEN-ST-030` | `StoreError::VaultDataNotFound` | store |
| `MIDEN-ST-031` | `StoreError::WordError` | store |
| `MIDEN-NR-001` | `NoteRecordError::ConversionError` | store |
| `MIDEN-NR-002` | `NoteRecordError::NoteError` | store |
| `MIDEN-NR-003` | `NoteRecordError::NoteNotConsumable` | store |
| `MIDEN-NR-004` | `NoteRecordError::InvalidInclusionProof` | store |
| `MIDEN-NR-005` | `NoteRecordError::InvalidStateTransition` | store |
| `MIDEN-NR-006` | `NoteRecordError::StateTransitionError` | store |
| `MIDEN-TX-001` | `TransactionRequestError::AccountInterfaceError` | transaction |
| `MIDEN-TX-002` | `TransactionRequestError::AccountError` | transaction |
| `MIDEN-TX-003` | `TransactionRequestError::DuplicateInputNote` | transaction |
| `MIDEN-TX-004` | `TransactionRequestError::ForeignAccountDataMissing` | transaction |
| `MIDEN-TX-005` | `TransactionRequestError::ForeignAccountStorageSlotInvalidIndex` | transaction |
| `MIDEN-TX-006` | `TransactionRequestError::InvalidForeignAccountId` | transaction |
| `MIDEN-TX-007` | `TransactionRequestError::InputNoteNotAuthenticated` | transaction |
| `MIDEN-TX-008` | `TransactionRequestError::InputNoteAlreadyConsumed` | transaction |
| `MIDEN-TX-009` | `TransactionRequestError::InvalidNoteVariant` | transaction |
| `MIDEN-TX-010` | `TransactionRequestError::InvalidSenderAccount` | transaction |
| `MIDEN-TX-011` | `TransactionRequestError::InvalidTransactionScript` | transaction |
| `MIDEN-TX-012` | `TransactionRequestError::MerkleError` | transaction |
| `MIDEN-TX-013` | `TransactionRequestError::NoInputNotesNorAccountChange` | transaction |
| `MIDEN-TX-014` | `TransactionRequestError::NoteNotFound` | transaction |
| `MIDEN-TX-015` | `TransactionRequestError::NoteCreationError` | transaction |
| `MIDEN-TX-016` | `TransactionRequestError::P2IDNoteWithoutAsset` | transaction |
| `MIDEN-TX-017` | `TransactionRequestError::CodeBuilderError` | transaction |
| `MIDEN-TX-018` | `TransactionRequestError::ScriptTemplateError` | transaction |
| `MIDEN-TX-019` | `TransactionRequestError::StorageSlotNotFound` | transaction |
| `MIDEN-TX-020` | `TransactionRequestError::TransactionInputError` | transaction |
| `MIDEN-TX-021` | `TransactionRequestError::StorageMapError` | transaction |
| `MIDEN-TX-022` | `TransactionRequestError::AssetVaultError` | transaction |
| `MIDEN-TX-023` | `TransactionRequestError::UnsupportedAuthSchemeId` | transaction |
| `MIDEN-NT-001` | `NoteTransportError::Disabled` | note_transport |
| `MIDEN-NT-002` | `NoteTransportError::Connection` | note_transport |
| `MIDEN-NT-003` | `NoteTransportError::Deserialization` | note_transport |
| `MIDEN-NT-004` | `NoteTransportError::Network` | note_transport |
| `MIDEN-NS-001` | `NoteScreenerError::InvalidNoteInputsError` | note |
| `MIDEN-NS-002` | `NoteScreenerError::AccountDataNotFound` | note |
| `MIDEN-NS-003` | `NoteScreenerError::StoreError` | note |
| `MIDEN-NS-004` | `NoteScreenerError::NoteCheckerError` | note |
| `MIDEN-NS-005` | `NoteScreenerError::TransactionRequestError` | note |
| `MIDEN-NI-001` | `InvalidNoteInputsError::AccountError` | note |
| `MIDEN-NI-002` | `InvalidNoteInputsError::AssetError` | note |
| `MIDEN-NI-003` | `InvalidNoteInputsError::WrongNumInputs` | note |
| `MIDEN-NI-004` | `InvalidNoteInputsError::BlockNumberError` | note |
| `MIDEN-KS-001` | `KeyStoreError::StorageError` | keystore |
| `MIDEN-KS-002` | `KeyStoreError::DecodingError` | keystore |
| `MIDEN-TP-001` | `TokenParseError::MaxDecimals` | utils |
| `MIDEN-TP-002` | `TokenParseError::MultipleDecimalPoints` | utils |
| `MIDEN-TP-003` | `TokenParseError::ParseU64` | utils |
| `MIDEN-TP-004` | `TokenParseError::TooManyDecimals` | utils |
| `MIDEN-AP-001` | `AccountProofError::InconsistentAccountCommitment` | rpc |
| `MIDEN-AP-002` | `AccountProofError::InconsistentAccountId` | rpc |
| `MIDEN-AP-003` | `AccountProofError::InconsistentCodeCommitment` | rpc |
| `MIDEN-SQ-001` | `SqliteStoreError::DatabaseError` | sqlite-store |
| `MIDEN-SQ-002` | `SqliteStoreError::MigrationError` | sqlite-store |
| `MIDEN-SQ-003` | `SqliteStoreError::MissingMigrationsTable` | sqlite-store |
| `MIDEN-SQ-004` | `SqliteStoreError::MigrationHashMismatch` | sqlite-store |

## Using Error Codes

### Rust

All error types implement the `ErrorCode` trait:

```rust
use miden_client::errors::ErrorCode;

match client.sync().await {
    Ok(_) => {},
    Err(err) => {
        eprintln!("Error {}: {}", err.error_code(), err);
    },
}
```

### JavaScript (Web Client)

Every `Error` thrown by the web client includes a `code` property:

```javascript
try {
    await webClient.syncState();
} catch (err) {
    console.error(`Error ${err.code}: ${err.message}`);

    // Programmatic matching
    if (err.code === "MIDEN-RP-003") {
        // Handle connection error
    }
}
```

## Client Errors (`MIDEN-CL`)

### MIDEN-CL-001 — AddressAlreadyTracked

The client is already tracking the specified address. Attempting to add it again is a no-op conflict.

**Resolution:** Check existing tracked addresses before adding. If re-tracking is intentional, remove the address first.

### MIDEN-CL-002 — AccountAlreadyTracked

The account with the given ID is already being tracked by the client.

**Resolution:** Verify the account ID. If you need to refresh account state, use `sync` instead of re-importing.

### MIDEN-CL-003 — NoteTagDerivedAddressAlreadyTracked

The address is available but its derived note tag is already being tracked.

**Resolution:** Remove the existing note tag tracking before adding the address.

### MIDEN-CL-004 — AccountError

A protocol-level account error occurred (e.g., invalid account data, code merge failure).

**Resolution:** Check the inner error for details. This typically indicates corrupt or incompatible account data.

### MIDEN-CL-005 — AccountLocked

The account is currently locked and cannot be modified.

**Resolution:** Wait for any in-progress transaction to complete, then retry.

### MIDEN-CL-006 — AccountCommitmentMismatch

The network account commitment does not match the imported account commitment.

**Resolution:** Sync the client to get the latest account state, then retry the import.

### MIDEN-CL-007 — AccountIsPrivate

The requested account is private and its full state is not available from the network.

**Resolution:** Private accounts must be tracked locally. Ensure you have the account data before attempting operations that require it.

### MIDEN-CL-008 — AccountNonceTooLow

The account nonce is too low to import, meaning a more recent version already exists locally.

**Resolution:** Sync the client to get the latest state, or verify you are importing the correct account version.

### MIDEN-CL-009 — AssetError

A protocol-level asset error occurred (e.g., amount exceeds maximum, invalid asset).

**Resolution:** Check the inner error. Verify asset amounts and faucet IDs.

### MIDEN-CL-010 — AccountDataNotFound

No account data was found for the specified account ID.

**Resolution:** Ensure the account has been imported or synced. Run `sync` and retry.

### MIDEN-CL-011 — PartialBlockchainError

An error occurred while constructing the partial blockchain (e.g., empty block headers).

**Resolution:** This typically indicates a sync issue. Run a full sync and retry.

### MIDEN-CL-012 — DataDeserializationError

Failed to deserialize data, usually from the store or network response.

**Resolution:** This may indicate data corruption or a version mismatch. Check that client and node versions are compatible.

### MIDEN-CL-013 — NoteNotFoundOnChain

The note with the given ID was not found on chain.

**Resolution:** Verify the note ID, ensure it has been committed, and run `sync` before retrying.

### MIDEN-CL-014 — HexParseError

Failed to parse a hex-encoded string.

**Resolution:** Check the input hex string for correct length and valid characters.

### MIDEN-CL-015 — InvalidPartialMmrForest

The partial MMR forest exceeds the valid `u32` range.

**Resolution:** This indicates corrupted MMR data. Re-sync the client.

### MIDEN-CL-016 — AddNewAccountWithoutSeed

Cannot add a new account without providing a seed.

**Resolution:** Provide a seed when creating the account.

### MIDEN-CL-017 — MerkleError

An error occurred in a Merkle path operation.

**Resolution:** Check the inner error for details. This usually indicates corrupted tree data.

### MIDEN-CL-018 — MissingOutputRecipients

The transaction did not produce output notes with the expected recipient digests.

**Resolution:** Align `TransactionRequestBuilder::expected_output_recipients(...)` with the MASM program so the declared recipients appear in the outputs.

### MIDEN-CL-019 — NoteError

A protocol-level note error occurred.

**Resolution:** Check the inner error for details.

### MIDEN-CL-020 — NoteCheckerError

An error occurred while checking a note's consumability.

**Resolution:** Check the inner error. The note may have already been processed or may be invalid.

### MIDEN-CL-021 — NoteImportError

An error occurred while importing a note.

**Resolution:** Verify the note data format and that it has not already been imported.

### MIDEN-CL-022 — NoteRecordConversionError

Failed to convert a note record between internal representations.

**Resolution:** Check the inner `NoteRecordError` for details.

### MIDEN-CL-023 — NoteTransportError

An error occurred with the note transport layer.

**Resolution:** Check network connectivity and the note transport service URL.

### MIDEN-CL-024 — NoConsumableNoteForAccount

No consumable note was found for the specified account.

**Resolution:** Verify the account has notes available. Run `sync` to refresh.

### MIDEN-CL-025 — RpcError

An RPC communication error occurred. Check the inner `RpcError` for the specific `MIDEN-RP-*` code.

### MIDEN-CL-026 — RecencyConditionError

A recency condition was not met, typically when block references are stale.

**Resolution:** Sync the client and retry with up-to-date block references.

### MIDEN-CL-027 — NoteScreenerError

An error occurred while screening a note for relevance.

**Resolution:** Check the inner `NoteScreenerError` for the specific `MIDEN-NS-*` code.

### MIDEN-CL-028 — StoreError

A persistence layer error occurred. Check the inner `StoreError` for the specific `MIDEN-ST-*` code.

### MIDEN-CL-029 — TransactionExecutorError

The transaction executor failed. This wraps errors from the `miden-tx` crate.

**Resolution:** Check the inner error. Common causes include stale foreign account proofs and MASM execution failures. Enable debug mode for detailed diagnostics.

### MIDEN-CL-030 — TransactionInputError

Transaction inputs were invalid (e.g., duplicate input notes).

**Resolution:** Check the inner error and verify your transaction inputs.

### MIDEN-CL-031 — TransactionProvingError

The transaction prover failed to generate a proof.

**Resolution:** Check the inner error. This may indicate an issue with the proving backend.

### MIDEN-CL-032 — TransactionRequestError

The transaction request was invalid. Check the inner `TransactionRequestError` for the specific `MIDEN-TX-*` code.

### MIDEN-CL-033 — AccountInterfaceError

An error occurred with the account interface (e.g., incompatible account capabilities).

**Resolution:** Verify the account supports the operations you are attempting.

### MIDEN-CL-034 — TransactionScriptError

An error occurred with the transaction script.

**Resolution:** Check the inner error. The MASM script may contain errors.

### MIDEN-CL-035 — ClientInitializationError

The client failed to initialize.

**Resolution:** Check the error message for details. Common causes include store initialization failures and invalid configuration.

### MIDEN-CL-036 — NoteTagsLimitExceeded

The maximum number of tracked note tags has been reached.

**Resolution:** Remove unused note tags before adding new ones.

### MIDEN-CL-037 — AccountsLimitExceeded

The maximum number of tracked accounts has been reached.

**Resolution:** Remove unused accounts before adding new ones.

### MIDEN-CL-038 — UnsupportedAuthSchemeId

The authentication scheme ID is not supported.

**Resolution:** Use a supported auth scheme (Falcon512Rpo or EcdsaK256Keccak).

### MIDEN-CL-039 — AccountRecordNotFull

Expected a full account record but got a different variant.

**Resolution:** Ensure you are working with a fully synced account.

### MIDEN-CL-040 — AccountRecordNotPartial

Expected a partial account record but got a different variant.

**Resolution:** Verify the account record type matches your expectation.

## ID Prefix Fetch Errors (`MIDEN-IP`)

### MIDEN-IP-001 — NoMatch

No entities matched the provided ID prefix.

**Resolution:** Verify the prefix is correct and that the entity exists in the store.

### MIDEN-IP-002 — MultipleMatches

Multiple entities matched the ID prefix when only one was expected.

**Resolution:** Provide a longer, more specific prefix to disambiguate.

## RPC Errors (`MIDEN-RP`)

### MIDEN-RP-001 — AcceptHeaderError

The server rejected the request due to accept header validation. See the inner `MIDEN-AH-*` code.

### MIDEN-RP-002 — AccountUpdateForPrivateAccountReceived

The RPC response contained an update for a private account, which should not happen.

**Resolution:** This may indicate a node misconfiguration. Report to node operators.

### MIDEN-RP-003 — ConnectionError

Failed to connect to the API server.

**Resolution:** Verify the node URL, check network connectivity, and ensure the node is running.

### MIDEN-RP-004 — DeserializationError

Failed to deserialize data from the RPC response.

**Resolution:** Ensure client and node versions are compatible.

### MIDEN-RP-005 — ExpectedDataMissing

The RPC response is missing an expected field.

**Resolution:** Ensure client and node versions are compatible. The node may be returning incomplete data.

### MIDEN-RP-006 — InvalidResponse

The RPC response is invalid or malformed.

**Resolution:** Ensure client and node versions are compatible.

### MIDEN-RP-007 — GrpcError

A gRPC-level error occurred. Check the inner `MIDEN-GR-*` code for specifics.

### MIDEN-RP-008 — NoteNotFound

The requested note was not found by the node.

**Resolution:** Verify the note ID and ensure it has been committed to the chain.

### MIDEN-RP-009 — InvalidNodeEndpoint

The node endpoint URL is invalid.

**Resolution:** Check the URL format (e.g., `https://host:port`).

## gRPC Errors (`MIDEN-GR`)

These correspond to standard gRPC status codes returned by the Miden node.

| Code | gRPC Status | Meaning |
|------|-------------|---------|
| `MIDEN-GR-001` | NOT_FOUND | The requested resource does not exist on the node |
| `MIDEN-GR-002` | INVALID_ARGUMENT | Request parameters are invalid |
| `MIDEN-GR-003` | PERMISSION_DENIED | Insufficient permissions |
| `MIDEN-GR-004` | ALREADY_EXISTS | The resource already exists |
| `MIDEN-GR-005` | RESOURCE_EXHAUSTED | Rate limited or resource quota exceeded |
| `MIDEN-GR-006` | FAILED_PRECONDITION | Precondition not met |
| `MIDEN-GR-007` | CANCELLED | The operation was cancelled |
| `MIDEN-GR-008` | DEADLINE_EXCEEDED | The operation timed out |
| `MIDEN-GR-009` | UNAVAILABLE | The node is temporarily unavailable; retry later |
| `MIDEN-GR-010` | INTERNAL | Internal server error on the node |
| `MIDEN-GR-011` | UNIMPLEMENTED | The RPC method is not implemented |
| `MIDEN-GR-012` | UNAUTHENTICATED | Request requires authentication |
| `MIDEN-GR-013` | ABORTED | The operation was aborted |
| `MIDEN-GR-014` | OUT_OF_RANGE | Value out of valid range |
| `MIDEN-GR-015` | DATA_LOSS | Unrecoverable data loss or corruption |
| `MIDEN-GR-016` | UNKNOWN | Unknown error |

## Accept Header Errors (`MIDEN-AH`)

### MIDEN-AH-001 — NoSupportedMediaRange

The server does not support the client's requested content type versions.

**Resolution:** Update your client to match the node version, or check your network settings.

### MIDEN-AH-002 — ParsingError

Failed to parse the accept header values.

**Resolution:** Check that genesis and version values are correctly formatted.

## RPC Conversion Errors (`MIDEN-RC`)

### MIDEN-RC-001 — DeserializationError

Failed to deserialize protobuf data.

### MIDEN-RC-002 — NotAValidFelt

A value received from the node is not in the valid range for a field element.

### MIDEN-RC-003 — NoteTypeError

Error converting a note type from the protobuf representation.

### MIDEN-RC-004 — MerkleError

Error in Merkle tree data received from the node.

### MIDEN-RC-005 — InvalidField

A protobuf field value could not be converted to the expected domain type.

### MIDEN-RC-006 — InvalidInt

An integer conversion failed (value out of range).

### MIDEN-RC-007 — MissingFieldInProtobufRepresentation

A required field is missing in the protobuf message.

## Store Errors (`MIDEN-ST`)

### MIDEN-ST-001 — AssetError

Asset validation failed within the store.

### MIDEN-ST-002 — AssetVaultError

Asset vault validation failed within the store.

### MIDEN-ST-003 — AccountCodeDataNotFound

Account code data with the specified root was not found in the store.

### MIDEN-ST-004 — AccountDataNotFound

Account data for the specified ID was not found in the store.

**Resolution:** Ensure the account has been imported or synced.

### MIDEN-ST-005 — AccountError

Protocol-level account error within the store.

### MIDEN-ST-006 — AddressError

Address validation error within the store.

### MIDEN-ST-007 — AccountIdError

Account ID parsing or validation error.

### MIDEN-ST-008 — AccountCommitmentMismatch

The account commitment in the store does not match the expected value.

**Resolution:** Sync the client. If the issue persists, the local store may be inconsistent.

### MIDEN-ST-009 — AccountKeyNotFound

The specified public key was not found in the store.

### MIDEN-ST-010 — AccountStorageRootNotFound

Account storage data with the specified root was not found.

### MIDEN-ST-011 — AccountStorageIndexNotFound

Account storage data at the specified index was not found.

### MIDEN-ST-012 — BlockHeaderNotFound

Block header for the specified block number was not found.

**Resolution:** Sync the client to fetch missing block headers.

### MIDEN-ST-013 — PartialBlockchainNodeNotFound

A node in the partial blockchain at the specified index was not found.

### MIDEN-ST-014 — DataDeserializationError

Failed to deserialize data from the store.

### MIDEN-ST-015 — DatabaseError

A database-level error occurred (not related to queries).

### MIDEN-ST-016 — HexParseError

Failed to parse hex data from the store.

### MIDEN-ST-017 — InvalidInt

Integer conversion failed.

### MIDEN-ST-018 — NoteRecordError

An error in a note record. See the inner `MIDEN-NR-*` code.

### MIDEN-ST-019 — MerkleStoreError

Merkle store error.

### MIDEN-ST-020 — MmrError

Error constructing or querying the MMR.

### MIDEN-ST-021 — NoteInclusionProofError

Error creating a note inclusion proof.

### MIDEN-ST-022 — NoteTagAlreadyTracked

The specified note tag is already being tracked.

**Resolution:** No action needed if this is expected. Otherwise, check for duplicate tag additions.

### MIDEN-ST-023 — NoteTransportCursorNotFound

The note transport cursor was not found in the store.

### MIDEN-ST-024 — NoteScriptNotFound

Note script with the specified root was not found.

### MIDEN-ST-025 — ParsingError

Failed to parse data retrieved from the database.

### MIDEN-ST-026 — QueryError

Failed to retrieve data from the database.

### MIDEN-ST-027 — SmtProofError

Error with a Sparse Merkle Tree proof.

### MIDEN-ST-028 — StorageMapError

Error with a storage map operation.

### MIDEN-ST-029 — TransactionScriptError

Error instantiating a transaction script from the store.

### MIDEN-ST-030 — VaultDataNotFound

Account vault data with the specified root was not found.

### MIDEN-ST-031 — WordError

Failed to parse a Word (four field elements).

## Note Record Errors (`MIDEN-NR`)

### MIDEN-NR-001 — ConversionError

Error during conversion of a note record between representations.

### MIDEN-NR-002 — NoteError

Invalid underlying note object.

### MIDEN-NR-003 — NoteNotConsumable

The note is not in a consumable state.

### MIDEN-NR-004 — InvalidInclusionProof

The note's inclusion proof is invalid.

### MIDEN-NR-005 — InvalidStateTransition

The requested state transition for the note record is not valid.

### MIDEN-NR-006 — StateTransitionError

An error occurred during a note record state transition.

## Transaction Request Errors (`MIDEN-TX`)

### MIDEN-TX-001 — AccountInterfaceError

The account interface does not support the requested operation.

### MIDEN-TX-002 — AccountError

Protocol-level account error in the transaction request.

### MIDEN-TX-003 — DuplicateInputNote

The same note ID appears more than once in the transaction inputs.

**Resolution:** Remove duplicate note IDs from the input list.

### MIDEN-TX-004 — ForeignAccountDataMissing

Foreign account data is missing from the account proof.

**Resolution:** Ensure the foreign account proof includes the required data.

### MIDEN-TX-005 — ForeignAccountStorageSlotInvalidIndex

The specified foreign account storage slot is not a map type.

### MIDEN-TX-006 — InvalidForeignAccountId

The foreign account does not have the expected storage mode.

### MIDEN-TX-007 — InputNoteNotAuthenticated

A note that should be authenticated does not contain a valid inclusion proof.

**Resolution:** Import or sync the note so its record and inclusion proof are present.

### MIDEN-TX-008 — InputNoteAlreadyConsumed

The input note has already been consumed by a previous transaction.

**Resolution:** Remove consumed notes from the transaction request.

### MIDEN-TX-009 — InvalidNoteVariant

Own notes should not be of the header variant.

### MIDEN-TX-010 — InvalidSenderAccount

The sender account ID is invalid for this transaction.

### MIDEN-TX-011 — InvalidTransactionScript

The transaction script is invalid.

### MIDEN-TX-012 — MerkleError

Merkle error in the transaction request.

### MIDEN-TX-013 — NoInputNotesNorAccountChange

The transaction has no input notes and no account state change.

**Resolution:** Add at least one input note or include an explicit account state update.

### MIDEN-TX-014 — NoteNotFound

A referenced note was not found.

### MIDEN-TX-015 — NoteCreationError

Error creating an output note.

### MIDEN-TX-016 — P2IDNoteWithoutAsset

A pay-to-ID note must contain at least one asset.

**Resolution:** Add at least one asset to the P2ID note.

### MIDEN-TX-017 — CodeBuilderError

Error building the transaction script code.

### MIDEN-TX-018 — ScriptTemplateError

Error with the transaction script template.

### MIDEN-TX-019 — StorageSlotNotFound

The storage slot was not found in the specified account.

**Resolution:** Verify the account ABI and component ordering. The auth component is always the first slot.

### MIDEN-TX-020 — TransactionInputError

Error building the transaction input notes.

### MIDEN-TX-021 — StorageMapError

Error with a storage map operation in the transaction.

### MIDEN-TX-022 — AssetVaultError

Asset vault error in the transaction.

### MIDEN-TX-023 — UnsupportedAuthSchemeId

The authentication scheme ID is not supported.

## Note Transport Errors (`MIDEN-NT`)

### MIDEN-NT-001 — Disabled

Note transport is not enabled.

**Resolution:** Enable note transport by providing a transport URL when creating the client.

### MIDEN-NT-002 — Connection

Failed to connect to the note transport service.

**Resolution:** Check the transport URL and network connectivity.

### MIDEN-NT-003 — Deserialization

Failed to deserialize note transport data.

### MIDEN-NT-004 — Network

A network error occurred in the note transport layer.

## Note Screener Errors (`MIDEN-NS`)

### MIDEN-NS-001 — InvalidNoteInputsError

Note inputs are invalid. See the inner `MIDEN-NI-*` code.

### MIDEN-NS-002 — AccountDataNotFound

Account data needed for note screening was not found.

### MIDEN-NS-003 — StoreError

Store error during note screening. See the inner `MIDEN-ST-*` code.

### MIDEN-NS-004 — NoteCheckerError

Error from the note consumption checker.

### MIDEN-NS-005 — TransactionRequestError

Error building a transaction request during note screening.

## Invalid Note Inputs Errors (`MIDEN-NI`)

### MIDEN-NI-001 — AccountError

Account error found in a note's inputs.

### MIDEN-NI-002 — AssetError

Asset error found in a note's inputs.

### MIDEN-NI-003 — WrongNumInputs

The note has an unexpected number of inputs.

### MIDEN-NI-004 — BlockNumberError

A note input representing a block number has an invalid value.

## Keystore Errors (`MIDEN-KS`)

### MIDEN-KS-001 — StorageError

Failed to read or write key data to the keystore backend.

### MIDEN-KS-002 — DecodingError

Failed to decode key data.

## Token Parse Errors (`MIDEN-TP`)

### MIDEN-TP-001 — MaxDecimals

The number of decimals exceeds the maximum allowed.

### MIDEN-TP-002 — MultipleDecimalPoints

The token amount string contains more than one decimal point.

### MIDEN-TP-003 — ParseU64

Failed to parse the token amount as a `u64`.

### MIDEN-TP-004 — TooManyDecimals

The token amount has more decimal places than allowed.

## Account Proof Errors (`MIDEN-AP`)

### MIDEN-AP-001 — InconsistentAccountCommitment

The account commitment in the proof does not match the account header's commitment.

### MIDEN-AP-002 — InconsistentAccountId

The account ID in the proof does not match the account header's ID.

### MIDEN-AP-003 — InconsistentCodeCommitment

The code commitment in the proof does not match the account header's code commitment.

## SQLite Store Errors (`MIDEN-SQ`)

### MIDEN-SQ-001 — DatabaseError

A SQLite database error occurred.

**Resolution:** Check file permissions, disk space, and that the database file is not corrupted.

### MIDEN-SQ-002 — MigrationError

A database migration failed.

**Resolution:** Check the migration logs. The database may need manual intervention or a fresh start.

### MIDEN-SQ-003 — MissingMigrationsTable

The migrations table is missing from the database.

**Resolution:** The database may be uninitialized or corrupted. Re-initialize or restore from backup.

### MIDEN-SQ-004 — MigrationHashMismatch

Migration hashes do not match, indicating the database was migrated with a different set of migrations.

**Resolution:** This usually means the database was created by a different version of the client. Use the correct client version or re-initialize the database.
