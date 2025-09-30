## Debugging guide and transaction lifecycle (CLI client)

This guide helps you troubleshoot common issues and understand the end-to-end lifecycle of transactions and notes in the Miden CLI client.

### TL;DR checklist

> Note: This section applies to the Miden CLI client. Guidance for the Rust and Web clients may differ.

- Ensure you are running commands in the same directory that contains `miden-client.toml`.
- If you need a clean local state, delete the SQLite store file referenced by `store_filepath` (default: `store.sqlite3`). It will be recreated automatically on the next command.
- Verify your node RPC endpoint is reachable and correct in `miden-client.toml`.
- Run with debug output when troubleshooting: add `--debug` or set `MIDEN_DEBUG=true`.
- Run `miden-client sync` to refresh local state after errors involving missing data or outdated heights.

### Enable debug output

- CLI flag: `miden-client --debug <command> ...` (overrides `MIDEN_DEBUG`)
- Environment variable: `MIDEN_DEBUG=true`

When enabled, the transaction executor and script compiler emit debug logs that help diagnose MASM-level issues (you can also consult the Miden VM debugging instructions).

### Common errors and how to resolve

Below are representative errors you may encounter, their likely causes, and suggested fixes.

- RpcError.GrpcError: Unavailable/DeadlineExceeded
  - Cause: Node is down, unreachable, or behind a load balancer that blocked the request.
  - Fix: Check `rpc.endpoint` in `miden-client.toml`, verify the node is running/accessible, and retry.

- RpcError.InvalidArgument / ExpectedDataMissing / InvalidResponse
  - Cause: Malformed request parameters or unexpected server response.
  - Fix: Re-check command flags/inputs. If using partial IDs, ensure they disambiguate to a single entity. Update to the latest client if the server API has changed.

- ClientError.AccountDataNotFound(account_id)
  - Cause: The account is not known to the local store yet.
  - Fix: Create/import the account first, or run `miden-client sync` to fetch it if it exists on-chain.

- ClientError.AccountLocked(account_id)
  - Cause: Attempting to modify a locked account.
  - Fix: Unlock or use another account as appropriate.

- ClientError.StoreError(AccountCommitmentAlreadyExists(...))
  - Cause: Trying to apply a transaction whose final account commitment is already present locally.
  - Fix: Ensure you are not re-applying the same transaction. Sync and check transaction status.

- ClientError.NoteNotFoundOnChain(note_id) / RpcError.NoteNotFound(note_id)
  - Cause: The note has not been published/committed yet or the ID is incorrect.
  - Fix: Verify the note ID. If it should exist, run `miden-client sync` and retry.

- ClientError.TransactionInputError / TransactionScriptError
  - Cause: Invalid transaction inputs, script logic errors, or failing constraints.
  - Fix: Run with `--debug` to collect execution logs. Validate input notes, foreign accounts, and script assumptions.

- ClientError.TransactionProvingError
  - Cause: Local proving failed or remote prover returned an error.
  - Fix: If using remote proving, verify `remote_prover_endpoint` is reachable and add `--delegate-proving`. Check prover logs.

- Recency/Block delta errors
  - Cause: Client is too far behind the network and validation enforces a max delta.
  - Fix: Run `miden-client sync` or increase `max_block_number_delta` via `miden-client init --block-delta <N>` and re-run.

### Typical CLI outputs when debugging

```sh
# Enable debug output for a command
miden-client --debug send --sender <SENDER> --target <TARGET> --asset 100::<FAUCET_ID>

# Force non-interactive submission (e.g., CI)
miden-client send --force ...

# Refresh local state
miden-client sync
```

If you see a gRPC error, it may include a status-derived kind (e.g. `Unavailable`, `InvalidArgument`) which narrows possible causes.

### Transaction lifecycle (high-level)

```mermaid
flowchart LR
    A[Build Request] --> B[Validate Request]
    B --> C[Collect/Insert Input Notes]
    C --> D[Build Transaction Script]
    D --> E[Load Foreign Accounts]
    E --> F[Execute Transaction]
    F --> G[Validate Outputs]
    G --> H[Prove Transaction]
    H --> I[Submit to Node]
    I --> J[Persist + Track Locally]

    subgraph Tracking
      J --> K[Update Account State]
      J --> L[Update Input/Output Notes]
      J --> M[Update Tags]
    end
```

Key states the CLI surfaces:

- Transaction status: `Pending` (after execution), `Committed` (after node inclusion), `Discarded` (not included).
- Input notes: `Expected` → `Processing` → `Consumed` (after sync) or `Committed` if fetched with inclusion.

### Recovery flow

1. Re-run with `--debug` or `MIDEN_DEBUG=true` for richer logs.
2. Verify `rpc.endpoint` connectivity and timeouts.
3. Run `miden-client sync` to refresh local headers/notes.
4. If local DB is inconsistent for development purposes, delete `store.sqlite3` (or configured path) and retry.
5. Adjust `max_block_number_delta` if strict recency checks block validation.
6. If proving errors persist with a remote prover, confirm `remote_prover_endpoint` and consider running locally to isolate the issue.

### References

- CLI debug flag and environment variable are documented in `CLI` and `Config` docs.
- Common error enums originate from the client and RPC layers.



