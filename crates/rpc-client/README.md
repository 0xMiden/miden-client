# miden-rpc-client

Minimal Miden RPC client.

## Available RPC Methods

1. `Status` - Node status information
2. `CheckNullifiers` - Nullifier proofs
3. `CheckNullifiersByPrefix` - Nullifiers matching prefixes
4. `GetAccountDetails` - Account state by ID
5. `GetAccountProofs` - Account state proofs with storage
6. `GetBlockByNumber` - Raw block data
7. `GetBlockHeaderByNumber` - Block headers with optional MMR proof
8. `GetNotesById` - Notes matching IDs
9. `SubmitProvenTransaction` - Submit single transaction
10. `SubmitProvenBatch` - Submit transaction batch
11. `SyncAccountVault` - Account vault updates
12. `SyncNotes` - Note synchronization
13. `SyncState` - Full state sync
14. `SyncStorageMaps` - Storage map updates

For advanced usage, proto types are exported and accessible via `client_mut()`.
