# miden-rpc-client

Minimal Miden RPC client.

## Usage

```rust
use miden_rpc_client::MidenRpcClient;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Connect to Miden node
    let mut client = MidenRpcClient::connect("https://node.example.com").await?;

    // Get node status
    let status = client.get_status().await?;
    println!("Node version: {}", status.version);

    // Get account details
    let account_details = client.get_account_details(&account_id).await?;

    Ok(())
}
```

## Available RPC Methods

1. `get_status` - Node status information
2. `get_block_header` - Block headers with optional MMR proof
3. `submit_transaction` - Submit single proven transaction
4. `sync_state` - Full state sync (accounts, notes, nullifiers)
5. `check_nullifiers` - Nullifier proofs
6. `get_notes_by_id` - Notes matching IDs
7. `get_account_commitment` - Fetch account commitment as hex string
8. `get_account_details` - Full account details including serialized data
9. `get_account_proof` - Account state proof with storage
10. `get_block_by_number` - Raw block data
11. `submit_proven_batch` - Submit transaction batch
12. `sync_account_vault` - Account vault updates within block range
13. `sync_notes` - Note synchronization by tags
14. `sync_storage_maps` - Storage map updates within block range

For advanced usage, proto types are exported and accessible via `client_mut()`.
