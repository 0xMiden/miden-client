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

Returns the hex-encoded MAST root for a procedure by name.

#### Parameters

##### procedure\_name

`string`

#### Returns

`string`

***

### getProcedures()

> **getProcedures**(): [`GetProceduresResultItem`](GetProceduresResultItem.md)[]

Returns all procedures exported by this component.

#### Returns

[`GetProceduresResultItem`](GetProceduresResultItem.md)[]

***

### withSupportsAllTypes()

> **withSupportsAllTypes**(): `AccountComponent`

Marks the component as supporting all account types.

#### Returns

`AccountComponent`

***

### compile()

> `static` **compile**(`account_code`, `builder`, `storage_slots`): `AccountComponent`

Compiles account code with the given storage slots using the provided assembler.

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

### createAuthComponentFromCommitment()

> `static` **createAuthComponentFromCommitment**(`commitment`, `auth_scheme`): `AccountComponent`

#### Parameters

##### commitment

[`Word`](Word.md)

##### auth\_scheme

[`AuthScheme`](AuthScheme.md)

#### Returns

`AccountComponent`

***

### createAuthComponentFromSecretKey()

> `static` **createAuthComponentFromSecretKey**(`secret_key`): `AccountComponent`

Builds an auth component from a secret key (`RpoFalcon512` or ECDSA k256 Keccak).

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey.md)

#### Returns

`AccountComponent`

***

### fromPackage()

> `static` **fromPackage**(`_package`, `storage_slots`): `AccountComponent`

Creates an account component from a compiled package and storage slots.

#### Parameters

##### \_package

[`Package`](Package.md)

##### storage\_slots

[`StorageSlotArray`](StorageSlotArray.md)

#### Returns

`AccountComponent`
