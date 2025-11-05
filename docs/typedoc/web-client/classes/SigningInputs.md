[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SigningInputs

# Class: SigningInputs

Wrapper for the data that gets hashed when producing a signature.

## Properties

### variantType

> `readonly` **variantType**: [`SigningInputsType`](../enumerations/SigningInputsType.md)

Returns the active signing input variant.

## Methods

### arbitraryPayload()

> **arbitraryPayload**(): [`Felt`](Felt.md)[]

Returns the arbitrary payload when the variant matches.

#### Returns

[`Felt`](Felt.md)[]

***

### blindPayload()

> **blindPayload**(): [`Word`](Word.md)

Returns the blind commitment payload when the variant matches.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the signing inputs into bytes.

#### Returns

`Uint8Array`

***

### toCommitment()

> **toCommitment**(): [`Word`](Word.md)

Returns the commitment over the signing inputs.

#### Returns

[`Word`](Word.md)

***

### toElements()

> **toElements**(): [`Felt`](Felt.md)[]

Returns the signing inputs as an array of field elements.

#### Returns

[`Felt`](Felt.md)[]

***

### transactionSummaryPayload()

> **transactionSummaryPayload**(): [`TransactionSummary`](TransactionSummary.md)

Returns the underlying transaction summary when the variant matches.

#### Returns

[`TransactionSummary`](TransactionSummary.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `SigningInputs`

Deserializes signing inputs from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`SigningInputs`

***

### newArbitrary()

> `static` **newArbitrary**(`felts`): `SigningInputs`

Creates signing inputs from arbitrary field elements.

#### Parameters

##### felts

[`Felt`](Felt.md)[]

#### Returns

`SigningInputs`

***

### newBlind()

> `static` **newBlind**(`word`): `SigningInputs`

Creates signing inputs from a single blind commitment word.

#### Parameters

##### word

[`Word`](Word.md)

#### Returns

`SigningInputs`

***

### newTransactionSummary()

> `static` **newTransactionSummary**(`summary`): `SigningInputs`

Creates signing inputs from a transaction summary.

#### Parameters

##### summary

[`TransactionSummary`](TransactionSummary.md)

#### Returns

`SigningInputs`
