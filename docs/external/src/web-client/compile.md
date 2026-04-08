---
title: Compile
sidebar_position: 2.5
---

# Compiling MASM with the Miden SDK

`client.compile` exposes two methods for compiling Miden Assembly (MASM) code directly from the browser without needing a direct reference to the low-level `WasmWebClient`:

| Method | Purpose |
|--------|---------|
| `compile.component({ code, slots })` | Compile MASM + storage slots into an `AccountComponent` for contract creation |
| `compile.txScript({ code, libraries? })` | Compile a transaction script, optionally linking inline libraries |

Each call creates a **fresh `CodeBuilder`**, so libraries linked in one call never leak into another.

## Compiling an Account Component

```typescript
import { MidenClient, StorageSlot } from "@miden-sdk/miden-sdk";

const contractCode = `
    use miden::protocol::active_account
    use miden::protocol::native_account
    use miden::core::word
    use miden::core::sys

    const COUNTER_SLOT = word("miden::tutorials::counter")

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
`;

try {
    const client = await MidenClient.create();

    const component = await client.compile.component({
        code: contractCode,
        slots: [StorageSlot.emptyValue("miden::tutorials::counter")]
    });

    // component is ready to pass to client.accounts.create()
    console.log("Component compiled");

    // Extract the hash of a specific procedure (needed for FPI)
    const getCountHash = component.getProcedureHash("get_count");
    console.log("get_count hash:", getCountHash);
} catch (error) {
    console.error("Compilation failed:", error.message);
}
```

## Compiling a Transaction Script

### Without libraries

```typescript
try {
    const client = await MidenClient.create();

    const script = await client.compile.txScript({
        code: `
            use external_contract::counter_contract
            begin
                call.counter_contract::increment_count
            end
        `
    });

    // script is ready to pass to client.transactions.execute()
} catch (error) {
    console.error("Script compilation failed:", error.message);
}
```

### With inline libraries

Pass an array of `{ namespace, code, linking? }` objects. The compiler builds each library and links it automatically:

```typescript
try {
    const client = await MidenClient.create();

    const script = await client.compile.txScript({
        code: `
            use external_contract::my_contract
            use miden::core::sys
            begin
                call.my_contract::do_something
                exec.sys::truncate_stack
            end
        `,
        libraries: [
            {
                namespace: "external_contract::my_contract",
                code: myContractCode,
                // linking: "dynamic" (default) | "static"
            }
        ]
    });
} catch (error) {
    console.error("Script compilation failed:", error.message);
}
```

#### `linking` modes

| Value | Behaviour | When to use |
|-------|-----------|-------------|
| `"dynamic"` (default) | Links without copying the library code into the script | FPI — the foreign contract lives on-chain; the prover fetches its code at prove time |
| `"static"` | Copies library code into the script | Off-chain libraries that must be self-contained |

## Getting a Procedure Hash (for FPI)

Foreign procedure invocation (FPI) requires knowing the **hash** of the target procedure. Compile the foreign contract's component and call `getProcedureHash()`:

```typescript
try {
    const client = await MidenClient.create();

    const counterComponent = await client.compile.component({
        code: counterContractCode,
        slots: [StorageSlot.emptyValue("miden::tutorials::counter")]
    });

    const getCountHash = counterComponent.getProcedureHash("get_count");

    // Use the hash in the transaction script
    const script = await client.compile.txScript({
        code: `
            use external_contract::count_reader_contract
            use miden::core::sys
            begin
                push.${getCountHash}
                push.${counterAccountId.suffix()}
                push.${counterAccountId.prefix()}
                call.count_reader_contract::copy_count
                exec.sys::truncate_stack
            end
        `,
        libraries: [
            { namespace: "external_contract::count_reader_contract", code: countReaderCode }
        ]
    });
} catch (error) {
    console.error("FPI script compilation failed:", error.message);
}
```

## Full Example: Compile → Create Contract → Execute Script

```typescript
import { MidenClient, AccountType, AuthSecretKey, StorageSlot } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();
    await client.sync();

    // 1. Compile the contract component
    const component = await client.compile.component({
        code: counterCode,
        slots: [StorageSlot.emptyValue("miden::tutorials::counter")]
    });

    // 2. Create the contract account
    const seed = crypto.getRandomValues(new Uint8Array(32));
    const auth = AuthSecretKey.rpoFalconWithRNG(seed);

    const contract = await client.accounts.create({
        type: AccountType.ImmutableContract,
        seed,
        auth,
        components: [component]
    });

    await client.sync();

    // 3. Compile the transaction script
    const script = await client.compile.txScript({
        code: `
            use external_contract::counter_contract
            begin
                call.counter_contract::increment_count
            end
        `,
        libraries: [
            { namespace: "external_contract::counter_contract", code: counterCode }
        ]
    });

    // 4. Execute the transaction
    const txId = await client.transactions.execute({
        account: contract.id(),
        script
    });

    console.log("Transaction ID:", txId.toHex());
} catch (error) {
    console.error("Failed:", error.message);
}
```

See [Creating transactions](./new-transactions.md) for full details on `transactions.execute()`.
