[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / FetchedAccount

# Class: FetchedAccount

Account details returned by the node.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### account()

> **account**(): [`Account`](Account.md)

Returns the full account data when the account is public.

#### Returns

[`Account`](Account.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the account commitment reported by the node.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isNetwork()

> **isNetwork**(): `boolean`

Returns true when the account is a network account.

#### Returns

`boolean`

***

### isPrivate()

> **isPrivate**(): `boolean`

Returns true when the account is private.

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

Returns true when the account is public.

#### Returns

`boolean`

***

### lastBlockNum()

> **lastBlockNum**(): `number`

Returns the last block height where the account was updated.

#### Returns

`number`
