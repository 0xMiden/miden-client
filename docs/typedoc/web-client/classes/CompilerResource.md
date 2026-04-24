[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompilerResource

# Class: CompilerResource

## Constructors

### Constructor

> **new CompilerResource**(`inner`, `getWasm`, `client?`): `CompilerResource`

Create a standalone `CompilerResource` over a WASM `WebClient` proxy.

Normally accessed as `client.compile` on a `MidenClient`; construct
directly only when you need the compiler surface without the full
`MidenClient` wrapper (e.g. inside a framework-specific hook).

#### Parameters

##### inner

`WebClient`

The WASM `WebClient` (e.g. the `WasmWebClient` proxy).

##### getWasm

() => `Promise`\<`__module`\>

Async accessor for the WASM module, used to reach
  `AccountComponent.compile` at runtime. `getWasmOrThrow` satisfies this.

##### client?

Optional wrapper with `assertNotTerminated()`; used
  internally by `MidenClient` and may be omitted by external callers.

###### assertNotTerminated

#### Returns

`CompilerResource`

## Methods

### component()

> **component**(`options`): `Promise`\<`AccountComponent`\>

Compile MASM source into an AccountComponent.

#### Parameters

##### options

[`CompileComponentOptions`](../interfaces/CompileComponentOptions.md)

Component source code, storage slots, and auth options.

#### Returns

`Promise`\<`AccountComponent`\>

***

### noteScript()

> **noteScript**(`options`): `Promise`\<`NoteScript`\>

Compile MASM source into a NoteScript.

#### Parameters

##### options

[`CompileNoteScriptOptions`](../interfaces/CompileNoteScriptOptions.md)

Script source code and optional libraries to link.

#### Returns

`Promise`\<`NoteScript`\>

***

### txScript()

> **txScript**(`options`): `Promise`\<`TransactionScript`\>

Compile MASM source into a TransactionScript.

#### Parameters

##### options

[`CompileTxScriptOptions`](../interfaces/CompileTxScriptOptions.md)

Script source code and optional libraries to link.

#### Returns

`Promise`\<`TransactionScript`\>
