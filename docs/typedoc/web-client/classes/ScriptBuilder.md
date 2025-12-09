[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ScriptBuilder

# Class: ScriptBuilder

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### buildLibrary()

> **buildLibrary**(`library_path`, `source_code`): [`Library`](Library.md)

Given a Library Path, and a source code, turn it into a Library.
E.g. A path library can be `miden::my_contract`. When turned into a library,
this can be used from another script with an import statement, following the
previous example: `use.miden::my_contract'.

#### Parameters

##### library\_path

`string`

##### source\_code

`string`

#### Returns

[`Library`](Library.md)

***

### compileNoteScript()

> **compileNoteScript**(`program`): [`NoteScript`](NoteScript.md)

Given a Note Script's source code, compiles it with the available
modules under this builder. Returns the compiled script.

#### Parameters

##### program

`string`

#### Returns

[`NoteScript`](NoteScript.md)

***

### compileTxScript()

> **compileTxScript**(`tx_script`): [`TransactionScript`](TransactionScript.md)

Given a Transaction Script's source code, compiles it with the available
modules under this builder. Returns the compiled script.

#### Parameters

##### tx\_script

`string`

#### Returns

[`TransactionScript`](TransactionScript.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### linkDynamicLibrary()

> **linkDynamicLibrary**(`library`): `void`

This is useful to dynamically link the [Library](Library.md) of a foreign account
that is invoked using foreign procedure invocation (FPI). Its code is available
on-chain and so it does not have to be copied into the script code.

For all other use cases not involving FPI, link the library statically.
Receives as argument the library to be linked.

#### Parameters

##### library

[`Library`](Library.md)

#### Returns

`void`

***

### linkModule()

> **linkModule**(`module_path`, `module_code`): `void`

Given a module path (something like `my_lib::module`) and source code, this will
statically link it for use with scripts to be built with this builder.

#### Parameters

##### module\_path

`string`

##### module\_code

`string`

#### Returns

`void`

***

### linkStaticLibrary()

> **linkStaticLibrary**(`library`): `void`

Statically links the given library.

Static linking means the library code is copied into the script code.
Use this for most libraries that are not available on-chain.

Receives as argument the library to link.

#### Parameters

##### library

[`Library`](Library.md)

#### Returns

`void`

***

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`
