[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FetchedAccount

# Class: FetchedAccount

Describes the response from the `GetAccountDetails` endpoint.

The content varies based on account visibility:
- **Public accounts**: Contains the complete [`Account`] details, as these are stored on-chain
- **Private accounts**: Contains only the state commitment, since full account data is stored
  off-chain

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### account()

> **account**(): [`Account`](Account.md)

#### Returns

[`Account`](Account.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

#### Returns

[`AccountId`](AccountId.md)

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

### isPrivate()

> **isPrivate**(): `boolean`

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

#### Returns

`boolean`
