[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountComponent

# Class: AccountComponent

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### getProcedureHash()

> **getProcedureHash**(`procedure_name`): `string`

#### Parameters

##### procedure\_name

`string`

#### Returns

`string`

***

### withSupportsAllTypes()

> **withSupportsAllTypes**(): `AccountComponent`

#### Returns

`AccountComponent`

***

### compile()

> `static` **compile**(`account_code`, `builder`, `storage_slots`): `AccountComponent`

#### Parameters

##### account\_code

`string`

##### builder

[`ScriptBuilder`](ScriptBuilder.md)

##### storage\_slots

[`StorageSlot`](StorageSlot.md)[]

#### Returns

`AccountComponent`

***

### createAuthComponent()

> `static` **createAuthComponent**(`secret_key`): `AccountComponent`

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey.md)

#### Returns

`AccountComponent`

***

### fromPackage()

> `static` **fromPackage**(`_package`, `storage_slots`): `AccountComponent`

#### Parameters

##### \_package

[`Package`](Package.md)

##### storage\_slots

[`StorageSlot`](StorageSlot.md)[]

#### Returns

`AccountComponent`
