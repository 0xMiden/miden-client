[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Account

# Class: Account

WASM wrapper around the native [`Account`], exposing its state to JavaScript.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### code()

> **code**(): [`AccountCode`](AccountCode.md)

Returns the code commitment for this account.

#### Returns

[`AccountCode`](AccountCode.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to the account header, storage, and code.

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

Returns the public keys derived from the account's authentication scheme.

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

Returns true if the account is a faucet.

#### Returns

`boolean`

***

### isNetwork()

> **isNetwork**(): `boolean`

Returns true if this is a network-owned account.

#### Returns

`boolean`

***

### isNew()

> **isNew**(): `boolean`

Returns true if the account has not yet been committed to the chain.

#### Returns

`boolean`

***

### isPrivate()

> **isPrivate**(): `boolean`

Returns true if the account storage is private.

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

Returns true if the account exposes public storage.

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

Returns true if the account is a regular account (immutable or updatable code).

#### Returns

`boolean`

***

### isUpdatable()

> **isUpdatable**(): `boolean`

Returns true if the account can update its code.

#### Returns

`boolean`

***

### nonce()

> **nonce**(): [`Felt`](Felt.md)

Returns the account nonce, which is incremented on every state update.

#### Returns

[`Felt`](Felt.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the account into bytes.

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorage`](AccountStorage.md)

Returns the account storage commitment.

#### Returns

[`AccountStorage`](AccountStorage.md)

***

### vault()

> **vault**(): [`AssetVault`](AssetVault.md)

Returns the vault commitment for this account.

#### Returns

[`AssetVault`](AssetVault.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `Account`

Restores an account from its serialized bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Account`
