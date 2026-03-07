[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompileTxScriptLibrary

# Interface: CompileTxScriptLibrary

## Properties

### code

> **code**: `string`

MASM source code for the library.

***

### linking?

> `optional` **linking**: `"dynamic"` \| `"static"`

`"dynamic"` (default) — procedures are linked via DYNCALL at runtime.
`"static"` — procedures are inlined at compile time.

***

### namespace

> **namespace**: `string`

MASM namespace for the library (e.g. "counter::module").
