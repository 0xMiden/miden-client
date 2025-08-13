[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SigningInputs

# Class: SigningInputs

## Properties

### variantType

> `readonly` **variantType**: `string`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### sign()

> **sign**(`secret_key`): [`Signature`](Signature.md)

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey.md)

#### Returns

[`Signature`](Signature.md)

***

### toCommitment()

> **toCommitment**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### toElements()

> **toElements**(): [`Felt`](Felt.md)[]

#### Returns

[`Felt`](Felt.md)[]

***

### verify()

> **verify**(`public_key`, `signature`): `boolean`

#### Parameters

##### public\_key

[`PublicKey`](PublicKey.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`boolean`

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
