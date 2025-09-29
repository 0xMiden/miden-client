[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountComponent

# Class: AccountComponent

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

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

> `static` **compile**(`account_code`, `assembler`, `storage_slots`): `AccountComponent`

#### Parameters

##### account\_code

`string`

##### assembler

[`Assembler`](Assembler.md)

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
