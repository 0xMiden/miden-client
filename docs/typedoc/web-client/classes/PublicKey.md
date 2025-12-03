[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / PublicKey

# Class: PublicKey

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

### serialize()

> **serialize**(): `Uint8Array`

Serializes the public key into bytes.

#### Returns

`Uint8Array`

***

### toCommitment()

> **toCommitment**(): [`Word`](Word.md)

Returns the commitment corresponding to this public key.

#### Returns

[`Word`](Word.md)

***

### verify()

> **verify**(`message`, `signature`): `boolean`

Verifies a blind message word against the signature.

#### Parameters

##### message

[`Word`](Word.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`boolean`

***

### verifyData()

> **verifyData**(`signing_inputs`, `signature`): `boolean`

Verifies a signature over arbitrary signing inputs.

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`boolean`

***

### deserialize()

> `static` **deserialize**(`bytes`): `PublicKey`

Deserializes a public key from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`PublicKey`

***

### recoverFrom()

> `static` **recoverFrom**(`message`, `signature`): `PublicKey`

Recovers a public key from a signature (only supported for RpoFalcon512).

#### Parameters

##### message

[`Word`](Word.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`PublicKey`
