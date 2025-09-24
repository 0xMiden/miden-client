[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ScriptBuilder

# Class: ScriptBuilder

## Constructors

### Constructor

> **new ScriptBuilder**(`mode`): `ScriptBuilder`

Instance a `ScriptBuilder`. Will use debug mode (or not), depending
on the mode passed when initially instanced.

#### Parameters

##### mode

[`ScriptBuilderMode`](../enumerations/ScriptBuilderMode.md)

#### Returns

`ScriptBuilder`

## Methods

### buildLibrary()

> **buildLibrary**(`library_path`, `source_code`): [`Library`](Library.md)

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

#### Parameters

##### program

`string`

#### Returns

[`NoteScript`](NoteScript.md)

***

### compileTxScript()

> **compileTxScript**(`tx_script`): [`TransactionScript`](TransactionScript.md)

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

#### Parameters

##### library

[`Library`](Library.md)

#### Returns

`void`

***

### linkModule()

> **linkModule**(`module_path`, `module_code`): `void`

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
