[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Word

# Class: Word

Represents four field elements packed together, matching the VM word type.

## Constructors

### Constructor

> **new Word**(`u64_vec`): `Word`

Creates a word from four `u64` values.

#### Parameters

##### u64\_vec

`BigUint64Array`

#### Returns

`Word`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the word into bytes.

#### Returns

`Uint8Array`

***

### toFelts()

> **toFelts**(): [`Felt`](Felt.md)[]

Returns the four field elements contained in the word.

#### Returns

[`Felt`](Felt.md)[]

***

### toHex()

> **toHex**(): `string`

Returns the hex string representation of the word.

#### Returns

`string`

***

### toU64s()

> **toU64s**(): `BigUint64Array`

Returns the four `u64` limbs contained in the word.

#### Returns

`BigUint64Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Word`

Deserializes a word from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Word`

***

### fromHex()

> `static` **fromHex**(`hex`): `Word`

Creates a Word from a hex string.
Fails if the provided string is not a valid hex representation of a Word.

#### Parameters

##### hex

`string`

#### Returns

`Word`

***

### newFromFelts()

> `static` **newFromFelts**(`felt_vec`): `Word`

Creates a word from four field elements.

#### Parameters

##### felt\_vec

[`Felt`](Felt.md)[]

#### Returns

`Word`
