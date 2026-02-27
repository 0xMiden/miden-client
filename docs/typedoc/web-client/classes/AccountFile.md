[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountFile

# Class: AccountFile

TypeDoc entry point — curated subset of the public API.
Only types listed here (or transitively referenced) appear in generated docs.
Runtime consumers should import from index.d.ts, not this file.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### account()

> **account**(): [`Account`](Account.md)

Returns the account data.

#### Returns

[`Account`](Account.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### authSecretKeyCount()

> **authSecretKeyCount**(): `number`

Returns the number of auth secret keys included.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the `AccountFile` into a byte array

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountFile`

Deserializes a byte array into an `AccountFile`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountFile`
