[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ForeignAccount

# Class: ForeignAccount

Description of a foreign account referenced by a transaction.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### account\_id()

> **account\_id**(): [`AccountId`](AccountId.md)

Returns the ID of the foreign account.

#### Returns

[`AccountId`](AccountId.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### storage\_slot\_requirements()

> **storage\_slot\_requirements**(): [`AccountStorageRequirements`](AccountStorageRequirements.md)

Returns the required storage slots/keys for this foreign account.

#### Returns

[`AccountStorageRequirements`](AccountStorageRequirements.md)

***

### public()

> `static` **public**(`account_id`, `storage_requirements`): `ForeignAccount`

Creates a foreign account entry for a public account with given storage requirements.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### storage\_requirements

[`AccountStorageRequirements`](AccountStorageRequirements.md)

#### Returns

`ForeignAccount`
