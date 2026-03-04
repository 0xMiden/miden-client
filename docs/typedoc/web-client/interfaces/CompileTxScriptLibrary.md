[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompileTxScriptLibrary

# Interface: CompileTxScriptLibrary

## Properties

### component

> **component**: `AccountComponent`

AccountComponent whose procedures become available to the script.

***

### linking?

> `optional` **linking**: `"dynamic"` \| `"static"`

`"dynamic"` (default) — procedures are linked via DYNCALL at runtime.
`"static"` — procedures are inlined at compile time.
