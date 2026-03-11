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

#### Returns

`Promise`\<`AccountComponent`\>

***

### txScript()

> **txScript**(`options`): `Promise`\<`TransactionScript`\>

Compile MASM source into a TransactionScript.

#### Parameters

##### options

[`CompileTxScriptOptions`](CompileTxScriptOptions.md)

#### Returns

`Promise`\<`TransactionScript`\>
