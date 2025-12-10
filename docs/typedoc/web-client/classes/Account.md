[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Account

# Class: Account

An account which can store assets and define rules for manipulating them.

An account consists of the following components:
- Account ID, which uniquely identifies the account and also defines basic properties of the
  account.
- Account vault, which stores assets owned by the account.
- Account storage, which is a key-value map (both keys and values are words) used to store
  arbitrary user-defined data.
- Account code, which is a set of Miden VM programs defining the public interface of the
  account.
- Account nonce, a value which is incremented whenever account state is updated.

Out of the above components account ID is always immutable (once defined it can never be
changed). Other components may be mutated throughout the lifetime of the account. However,
account state can be changed only by invoking one of account interface methods.

The recommended way to build an account is through an `AccountBuilder`, which can be
instantiated directly from a 32-byte seed.

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
