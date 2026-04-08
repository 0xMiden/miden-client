[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompileComponentOptions

# Interface: CompileComponentOptions

## Properties

### code

> **code**: `string`

MASM source code for the component.

***

### slots?

> `optional` **slots**: `StorageSlot`[]

Initial storage slots for the component.

***

### supportAllTypes?

> `optional` **supportAllTypes**: `boolean`

When true, the component accepts all input types for Falcon-signed
transactions by automatically adding `exec.auth::auth_tx_rpo_falcon512`
to a library context. Default: true.

**BREAKING (v0.12):** This flag was added in v0.12 and defaults to `true`.
Set to `false` if you compile a component that already includes its own
auth transaction kernel invocation or intentionally omits one.
