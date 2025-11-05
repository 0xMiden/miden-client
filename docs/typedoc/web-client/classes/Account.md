[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Account

# Class: Account

JavaScript wrapper around [`miden_client::account::Account`].

Exposes read-only accessors and serialization helpers for account data inside the web client.

## Methods

### code()

> **code**(): [`AccountCode`](AccountCode.md)

Returns the executable code stored in this account.

#### Returns

[`AccountCode`](AccountCode.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the account commitment.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getPublicKeys()

> **getPublicKeys**(): [`Word`](Word.md)[]

Returns the public keys associated with this account.

#### Returns

[`Word`](Word.md)[]

***

### id()

> **id**(): [`AccountId`](AccountId.md)

Returns the account identifier.

#### Returns

[`AccountId`](AccountId.md)

***

### isFaucet()

> **isFaucet**(): `boolean`

Returns `true` if the account is a faucet account.

#### Returns

`boolean`

***

### isNew()

> **isNew**(): `boolean`

Returns `true` if the account has not been initialized yet.

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

Returns `true` if the account is public.

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

Returns `true` if the account is a regular account.

#### Returns

`boolean`

***

### isUpdatable()

> **isUpdatable**(): `boolean`

Returns `true` if the account supports updating its code.

#### Returns

`boolean`

***

### nonce()

> **nonce**(): [`Felt`](Felt.md)

Returns the account nonce as a field element.

#### Returns

[`Felt`](Felt.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes this account into raw bytes.

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorage`](AccountStorage.md)

Returns the storage associated with this account.

#### Returns

[`AccountStorage`](AccountStorage.md)

***

### vault()

> **vault**(): [`AssetVault`](AssetVault.md)

Returns the asset vault associated with this account.

#### Returns

[`AssetVault`](AssetVault.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `Account`

Deserializes an account from its byte representation.

#### Parameters

##### bytes

`Uint8Array`

Serialized account bytes.

#### Returns

`Account`

#### Throws

Throws if the bytes cannot be parsed into a valid account.
