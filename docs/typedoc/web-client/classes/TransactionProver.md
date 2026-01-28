[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionProver

# Class: TransactionProver

Wrapper over local or remote transaction proving backends.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### endpoint()

> **endpoint**(): `string`

Returns the endpoint if this is a remote prover.

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

Serializes the prover configuration into a string descriptor.

Format:
- `"local"` for local prover
- `"remote|{endpoint}"` for remote prover without timeout
- `"remote|{endpoint}|{timeout_ms}"` for remote prover with timeout

Uses `|` as delimiter since it's not a valid URL character.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`payload`): `TransactionProver`

Reconstructs a prover from its serialized descriptor.

Parses the format produced by `serialize()`:
- `"local"` for local prover
- `"remote|{endpoint}"` for remote prover without timeout
- `"remote|{endpoint}|{timeout_ms}"` for remote prover with timeout

#### Parameters

##### payload

`string`

#### Returns

`TransactionProver`

***

### newLocalProver()

> `static` **newLocalProver**(): `TransactionProver`

Creates a prover that uses the local proving backend.

#### Returns

`TransactionProver`

***

### newRemoteProver()

> `static` **newRemoteProver**(`endpoint`, `timeout_ms?`): `TransactionProver`

Creates a new remote transaction prover.

Arguments:
- `endpoint`: The URL of the remote prover.
- `timeout_ms`: The timeout in milliseconds for the remote prover.

#### Parameters

##### endpoint

`string`

##### timeout\_ms?

`bigint`

#### Returns

`TransactionProver`
