---
title: TransactionProver
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / TransactionProver

# Class: TransactionProver

## Methods

### endpoint()

> **endpoint**(): `string`

#### Returns

`string`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `string`

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`prover_type`, `endpoint?`): `TransactionProver`

#### Parameters

##### prover\_type

`string`

##### endpoint?

`string`

#### Returns

`TransactionProver`

***

### newLocalProver()

> `static` **newLocalProver**(): `TransactionProver`

#### Returns

`TransactionProver`

***

### newRemoteProver()

> `static` **newRemoteProver**(`endpoint`): `TransactionProver`

#### Parameters

##### endpoint

`string`

#### Returns

`TransactionProver`
