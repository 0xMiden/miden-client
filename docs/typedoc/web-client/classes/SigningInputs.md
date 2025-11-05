[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SigningInputs

# Class: SigningInputs

## Properties

### variantType

> `readonly` **variantType**: [`SigningInputsType`](../enumerations/SigningInputsType.md)

## Methods

### arbitraryPayload()

> **arbitraryPayload**(): [`FeltArray`](FeltArray.md)

#### Returns

[`FeltArray`](FeltArray.md)

***

### blindPayload()

> **blindPayload**(): [`Word`](Word.md)

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

#### Returns

`Uint8Array`

***

### toCommitment()

> **toCommitment**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### toElements()

> **toElements**(): [`FeltArray`](FeltArray.md)

#### Returns

[`FeltArray`](FeltArray.md)

***

### transactionSummaryPayload()

> **transactionSummaryPayload**(): [`TransactionSummary`](TransactionSummary.md)

#### Returns

[`TransactionSummary`](TransactionSummary.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `SigningInputs`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`SigningInputs`

***

### newArbitrary()

> `static` **newArbitrary**(`felts`): `SigningInputs`

#### Parameters

##### felts

[`Felt`](Felt.md)[]

#### Returns

`SigningInputs`

***

### newBlind()

> `static` **newBlind**(`word`): `SigningInputs`

#### Parameters

##### word

[`Word`](Word.md)

#### Returns

`SigningInputs`

***

### newTransactionSummary()

> `static` **newTransactionSummary**(`summary`): `SigningInputs`

#### Parameters

##### summary

[`TransactionSummary`](TransactionSummary.md)

#### Returns

`SigningInputs`
