[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionFilter

# Class: TransactionFilter

Filter used when querying stored transactions.

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

### all()

> `static` **all**(): `TransactionFilter`

Matches all transactions.

#### Returns

`TransactionFilter`

***

### expiredBefore()

> `static` **expiredBefore**(`block_num`): `TransactionFilter`

Matches transactions that expired before the given block number.

#### Parameters

##### block\_num

`number`

#### Returns

`TransactionFilter`

***

### ids()

> `static` **ids**(`ids`): `TransactionFilter`

Matches specific transaction IDs.

#### Parameters

##### ids

[`TransactionId`](TransactionId.md)[]

#### Returns

`TransactionFilter`

***

### uncommitted()

> `static` **uncommitted**(): `TransactionFilter`

Matches transactions that are not yet committed.

#### Returns

`TransactionFilter`
