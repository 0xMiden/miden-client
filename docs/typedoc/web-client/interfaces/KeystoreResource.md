[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / KeystoreResource

# Interface: KeystoreResource

## Methods

### get()

> **get**(`pubKeyCommitment`): `Promise`\<`AuthSecretKey`\>

Retrieves a secret key by its public key commitment. Returns null if not found.

#### Parameters

##### pubKeyCommitment

[`Word`](../classes/Word.md)

#### Returns

`Promise`\<`AuthSecretKey`\>

***

### getAccountId()

> **getAccountId**(`pubKeyCommitment`): `Promise`\<[`AccountId`](../classes/AccountId.md)\>

Returns the account ID associated with a public key commitment, or null if not found.

#### Parameters

##### pubKeyCommitment

[`Word`](../classes/Word.md)

#### Returns

`Promise`\<[`AccountId`](../classes/AccountId.md)\>

***

### getCommitments()

> **getCommitments**(`accountId`): `Promise`\<[`Word`](../classes/Word.md)[]\>

Returns all public key commitments associated with the given account ID.

#### Parameters

##### accountId

[`AccountId`](../classes/AccountId.md)

#### Returns

`Promise`\<[`Word`](../classes/Word.md)[]\>

***

### insert()

> **insert**(`accountId`, `secretKey`): `Promise`\<`void`\>

Inserts a secret key into the keystore, associating it with the given account ID.

#### Parameters

##### accountId

[`AccountId`](../classes/AccountId.md)

##### secretKey

`AuthSecretKey`

#### Returns

`Promise`\<`void`\>

***

### remove()

> **remove**(`pubKeyCommitment`): `Promise`\<`void`\>

Removes a key from the keystore by its public key commitment.

#### Parameters

##### pubKeyCommitment

[`Word`](../classes/Word.md)

#### Returns

`Promise`\<`void`\>
