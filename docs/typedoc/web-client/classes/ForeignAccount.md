[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ForeignAccount

# Class: ForeignAccount

Wrapper describing another account referenced by a transaction.

## Methods

### account\_id()

> **account\_id**(): [`AccountId`](AccountId.md)

Returns the identifier of the foreign account.

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

Returns the storage slots required for this foreign account.

#### Returns

[`AccountStorageRequirements`](AccountStorageRequirements.md)

***

### public()

> `static` **public**(`account_id`, `storage_requirements`): `ForeignAccount`

Creates a reference to a public foreign account using storage requirements.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### storage\_requirements

[`AccountStorageRequirements`](AccountStorageRequirements.md)

#### Returns

`ForeignAccount`
