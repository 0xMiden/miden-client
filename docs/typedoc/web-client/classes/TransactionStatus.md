[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionStatus

# Class: TransactionStatus

Status of a transaction in the node or store.

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

### getBlockNum()

> **getBlockNum**(): `number`

Returns the block number if the transaction was committed.

#### Returns

`number`

***

### getCommitTimestamp()

> **getCommitTimestamp**(): `bigint`

Returns the commit timestamp if the transaction was committed.

#### Returns

`bigint`

***

### isCommitted()

> **isCommitted**(): `boolean`

Returns true if the transaction has been committed.

#### Returns

`boolean`

***

### isDiscarded()

> **isDiscarded**(): `boolean`

Returns true if the transaction was discarded.

#### Returns

`boolean`

***

### isPending()

> **isPending**(): `boolean`

Returns true if the transaction is still pending.

#### Returns

`boolean`

***

### committed()

> `static` **committed**(`block_num`, `commit_timestamp`): `TransactionStatus`

Creates a committed status with block number and timestamp.

#### Parameters

##### block\_num

`number`

##### commit\_timestamp

`bigint`

#### Returns

`TransactionStatus`

***

### discarded()

> `static` **discarded**(`cause`): `TransactionStatus`

Creates a discarded status from a discard cause string.

#### Parameters

##### cause

`string`

#### Returns

`TransactionStatus`

***

### pending()

> `static` **pending**(): `TransactionStatus`

Creates a pending transaction status.

#### Returns

`TransactionStatus`
