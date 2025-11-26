[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionProver

# Class: TransactionProver

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

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

> `static` **deserialize**(`prover_type`, `endpoint?`, `timeout_ms?`): `TransactionProver`

#### Parameters

##### prover\_type

`string`

##### endpoint?

`string`

##### timeout\_ms?

`bigint`

#### Returns

`TransactionProver`

***

### newLocalProver()

> `static` **newLocalProver**(): `TransactionProver`

#### Returns

`TransactionProver`

***

### newRemoteProver()

> `static` **newRemoteProver**(`endpoint`, `timeout_ms?`): `TransactionProver`

#### Parameters

##### endpoint

`string`

##### timeout\_ms?

`bigint`

#### Returns

`TransactionProver`
