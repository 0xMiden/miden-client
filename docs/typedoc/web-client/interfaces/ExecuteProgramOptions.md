[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ExecuteProgramOptions

# Interface: ExecuteProgramOptions

## Properties

### account

> **account**: [`AccountRef`](../type-aliases/AccountRef.md)

Account to execute the program against.

***

### adviceInputs?

> `optional` **adviceInputs**: `AdviceInputs`

Advice inputs for the execution. Defaults to empty.

***

### foreignAccounts?

> `optional` **foreignAccounts**: ([`AccountRef`](../type-aliases/AccountRef.md) \| \{ `id`: [`AccountRef`](../type-aliases/AccountRef.md); `storage?`: `AccountStorageRequirements`; \})[]

Foreign accounts referenced by the script.

***

### script

> **script**: `TransactionScript`

Compiled TransactionScript to execute.
