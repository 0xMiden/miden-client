---
title: DAP Debugging
sidebar_position: 7
---

# DAP Debugging

The Miden client supports interactive debugging via the [Debug Adapter Protocol (DAP)](https://microsoft.github.io/debug-adapter-protocol/). You can debug both raw Miden Assembly scripts and Rust programs compiled to Miden via `midenc`. This lets you step through execution, set breakpoints, and inspect stack/memory state using any DAP-compatible client (e.g. VS Code, the `miden-debug` TUI).

## Feature Flags

Two feature flags control debugging support:

| Feature | Crate | What it enables |
|---------|-------|-----------------|
| `dap` | `miden-client`, `miden-client-cli` | Compiles in DAP support (`execute_program_with_dap`, `--start-debug-adapter` CLI flag). **Enabled by default.** |
| `testing` | `miden-client-cli` | Enables the `--offline` flag on `new-wallet`/`new-account` commands for node-less account creation. Not available in production builds. |

### Building with features

```bash
# Default build (DAP enabled)
cargo build -p miden-client-cli

# With offline mode for testing
cargo build -p miden-client-cli --features testing

# Without DAP (smaller binary)
cargo build -p miden-client-cli --no-default-features
```

## Quick Start

### 1. Create an account

With a running node:

```bash
miden-client init
miden-client new-wallet
miden-client sync
```

Or without a node (requires `testing` feature):

```bash
miden-client init
miden-client new-wallet --offline
```

### 2. Write a test script

Create a file `test_debug.masm`:

```
begin
  push.1.2
  add
  push.3
  mul
end
```

### 3. Start the DAP server

```bash
miden-client exec \
  --script-path test_debug.masm \
  --start-debug-adapter 127.0.0.1:4711
```

The client will compile the script and wait for a DAP client to connect before executing.

### 4. Connect a debugger

In a separate terminal, connect the `miden-debug` TUI:

```bash
miden-debug --dap-connect 127.0.0.1:4711
```

You can now step through execution, inspect the stack, and set breakpoints.


## How it Works

When `--start-debug-adapter` is passed:

1. The client compiles the transaction script normally.
2. Instead of using the default `FastProcessor`, it creates a `DapExecutor` (from the `miden-debug` crate) which implements the `ProgramExecutor` trait.
3. The `DapExecutor` binds a TCP listener on the specified address and waits for a DAP client connection.
4. Once connected, the DAP client controls execution (continue, step, breakpoints, inspect state).
5. If the DAP client requests a restart, the client recompiles the script from disk and re-executes — enabling an edit-and-continue workflow.
