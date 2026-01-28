[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / SigningInputs

# Class: SigningInputs

## Properties

### variantType

> `readonly` **variantType**: [`SigningInputsType`](../enumerations/SigningInputsType.md)

Returns which variant these signing inputs represent.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### arbitraryPayload()

> **arbitraryPayload**(): [`FeltArray`](FeltArray.md)

Returns the arbitrary payload as an array of felts.

#### Returns

[`FeltArray`](FeltArray.md)

***

### blindPayload()

> **blindPayload**(): [`Word`](Word.md)

Returns the blind payload as a word.

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

Returns the commitment to these signing inputs.

#### Returns

[`Word`](Word.md)

***

### toElements()

> **toElements**(): [`FeltArray`](FeltArray.md)

Returns the inputs as field elements.

#### Returns

[`FeltArray`](FeltArray.md)

***

### transactionSummaryPayload()

> **transactionSummaryPayload**(): [`TransactionSummary`](TransactionSummary.md)

Returns the transaction summary payload if this variant contains one.

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

Creates blind signing inputs from a single word.

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
