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

### createAuthComponentFromCommitment()

> `static` **createAuthComponentFromCommitment**(`commitment`, `auth_scheme`): `AccountComponent`

#### Parameters

##### commitment

[`Word`](Word.md)

##### auth\_scheme

[`AuthScheme`](../enumerations/AuthScheme.md)

#### Returns

`AccountComponent`

***

### createAuthComponentFromSecretKey()

> `static` **createAuthComponentFromSecretKey**(`secret_key`): `AccountComponent`

Builds an auth component from a secret key, inferring the auth scheme from the key type.

#### Parameters

##### secret\_key

[`AuthSecretKey`](AuthSecretKey.md)

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
