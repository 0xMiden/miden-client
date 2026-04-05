[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompilerResource

# Interface: CompilerResource

## Methods

### component()

> **component**(`options`): `Promise`\<`AccountComponent`\>

Compile MASM source into an AccountComponent.

#### Parameters

##### options

[`CompileComponentOptions`](CompileComponentOptions.md)

Component source code, storage slots, and auth options.

#### Returns

`Promise`\<`AccountComponent`\>

***

### txScript()

> **txScript**(`options`): `Promise`\<`TransactionScript`\>

Compile MASM source into a TransactionScript.

#### Parameters

##### options

[`CompileTxScriptOptions`](CompileTxScriptOptions.md)

Script source code and optional libraries to link.

#### Returns

`Promise`\<`TransactionScript`\>
