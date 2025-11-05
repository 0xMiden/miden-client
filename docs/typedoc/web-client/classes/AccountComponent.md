[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountComponent

# Class: AccountComponent

JavaScript wrapper around [`miden_client::account::component::AccountComponent`].

Represents compiled account code together with its metadata so it can be combined into new
accounts from JavaScript.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### getProcedureHash()

> **getProcedureHash**(`procedure_name`): `string`

Returns the hash of a procedure exported by this component.

#### Parameters

##### procedure\_name

`string`

Name of the exported procedure to inspect.

#### Returns

`string`

#### Throws

Throws if the procedure cannot be found.

***

### withSupportsAllTypes()

> **withSupportsAllTypes**(): `AccountComponent`

Marks the component as supporting all note types.

#### Returns

`AccountComponent`

***

### compile()

> `static` **compile**(`account_code`, `builder`, `storage_slots`): `AccountComponent`

Compiles account component source code into an executable component.

#### Parameters

##### account\_code

`string`

Account component source code in Miden assembly.

##### builder

[`ScriptBuilder`](ScriptBuilder.md)

Script builder containing the assembler state.

##### storage\_slots

[`StorageSlot`](StorageSlot.md)[]

Storage slots required by the component.

#### Returns

`AccountComponent`

#### Throws

Throws if compilation fails.

***

### createAuthComponent()

> `static` **createAuthComponent**(`secret_key`): `AccountComponent`

Builds an authentication component from a secret key.

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey.md)

#### Returns

`AccountComponent`
