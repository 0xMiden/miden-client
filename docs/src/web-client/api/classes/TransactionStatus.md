---
title: TransactionStatus
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / TransactionStatus

# Class: TransactionStatus

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### getBlockNum()

> **getBlockNum**(): `number`

#### Returns

`number`

***

### getCommitTimestamp()

> **getCommitTimestamp**(): `bigint`

#### Returns

`bigint`

***

### isCommitted()

> **isCommitted**(): `boolean`

#### Returns

`boolean`

***

### isDiscarded()

> **isDiscarded**(): `boolean`

#### Returns

`boolean`

***

### isPending()

> **isPending**(): `boolean`

#### Returns

`boolean`

***

### committed()

> `static` **committed**(`block_num`, `commit_timestamp`): `TransactionStatus`

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

#### Parameters

##### cause

`string`

#### Returns

`TransactionStatus`

***

### pending()

> `static` **pending**(): `TransactionStatus`

#### Returns

`TransactionStatus`
