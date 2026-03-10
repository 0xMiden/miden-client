---
title: Compile
sidebar_position: 3
---

# Compiling MASM

The Miden Client provides a `CodeBuilder` for compiling Miden Assembly (MASM) code into executable components, transaction scripts, and note scripts.

## Get a CodeBuilder

```rust
let code_builder = client.code_builder();
```

Each call returns a fresh `CodeBuilder`, so libraries linked in one instance never leak into another.

## Compile an account component

Compile MASM source code into an `AccountComponentCode` for use in account creation:

```rust
let component_code = client.code_builder()
    .compile_account_component_code(
        "
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::sys

        const COUNTER_SLOT = word(\"miden::tutorials::counter\")

        pub proc get_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            exec.sys::truncate_stack
        end

        pub proc increment_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            add.1
            push.COUNTER_SLOT[0..2] exec.native_account::set_item
            exec.sys::truncate_stack
        end
        "
    )?;
```

The resulting `AccountComponentCode` can be passed to `AccountBuilder` when creating custom contract accounts.

## Compile a transaction script

### Without libraries

```rust
let tx_script = client.code_builder()
    .compile_tx_script(
        "
        use external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end
        "
    )?;
```

### With linked libraries

Link external libraries before compiling. Libraries can be linked dynamically (for on-chain contracts via FPI) or statically (for self-contained scripts):

```rust
let mut cb = client.code_builder();

// Build a library from source
let library = cb.build_library(
    "external_contract::counter_contract",
    counter_contract_code,
)?;

// Link dynamically (default for FPI — code fetched at prove time)
cb.link_dynamic_library(&library)?;

// Or link statically (code copied into the script)
// cb.link_static_library(&library)?;

let tx_script = cb.compile_tx_script(
    "
    use external_contract::counter_contract
    begin
        call.counter_contract::increment_count
    end
    "
)?;
```

#### Linking modes

| Mode | Method | When to use |
|------|--------|-------------|
| Dynamic | `link_dynamic_library()` | FPI — the foreign contract lives on-chain; the prover fetches its code at prove time |
| Static | `link_static_library()` | Off-chain libraries that must be self-contained |

You can also link raw module source code directly:

```rust
let mut cb = client.code_builder();
cb.link_module("external_contract::my_module", module_source_code)?;
let tx_script = cb.compile_tx_script("...")?;
```

## Compile a note script

```rust
let note_script = client.code_builder()
    .compile_note_script(
        "
        use.miden::note
        begin
            exec.note::get_inputs
            # ... process inputs
        end
        "
    )?;
```

## Full example: compile, create, execute

```rust
use miden_client::builder::ClientBuilder;
use miden_client::account::AccountBuilder;

// 1. Compile the contract component
let component_code = client.code_builder()
    .compile_account_component_code(counter_code)?;

// 2. Create the contract account
let (account, seed) = AccountBuilder::new(client.rng())
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Public)
    .with_component(component_code, AccountComponentMetadata::new(
        InitStorageData::default(),
        AccountComponentMetadataVersion::latest(),
    )?)
    .build()?;

client.add_account(&account, false).await?;

// 3. Compile the transaction script
let mut cb = client.code_builder();
let library = cb.build_library(
    "external_contract::counter_contract",
    counter_code,
)?;
cb.link_dynamic_library(&library)?;

let tx_script = cb.compile_tx_script(
    "
    use external_contract::counter_contract
    begin
        call.counter_contract::increment_count
    end
    "
)?;

// 4. Execute the transaction
let tx_request = TransactionRequestBuilder::new()
    .with_custom_script(tx_script)?
    .build()?;

let tx_result = client.new_transaction(account.id(), tx_request).await?;
client.submit_transaction(tx_result).await?;
```

See [Creating transactions](./new-transactions.md) for more on `TransactionRequestBuilder`.
