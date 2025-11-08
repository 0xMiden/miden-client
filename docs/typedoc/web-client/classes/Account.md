[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Account

# Class: Account

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### code()

> **code**(): [`AccountCode`](AccountCode.md)

#### Returns

[`AccountCode`](AccountCode.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

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

#### Returns

[`Word`](Word.md)[]

***

### id()

> **id**(): [`AccountId`](AccountId.md)

#### Returns

[`AccountId`](AccountId.md)

***

### isFaucet()

> **isFaucet**(): `boolean`

#### Returns

`boolean`

***

### isNew()

> **isNew**(): `boolean`

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

#### Returns

`boolean`

***

### isUpdatable()

> **isUpdatable**(): `boolean`

#### Returns

`boolean`

***

### nonce()

> **nonce**(): [`Felt`](Felt.md)

#### Returns

[`Felt`](Felt.md)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorage`](AccountStorage.md)

#### Returns

[`AccountStorage`](AccountStorage.md)

***

### vault()

> **vault**(): [`AssetVault`](AssetVault.md)

#### Returns

[`AssetVault`](AssetVault.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `Account`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Account`
