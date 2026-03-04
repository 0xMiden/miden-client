[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountsResource

# Interface: AccountsResource

## Methods

### addAddress()

> **addAddress**(`accountId`, `address`): `Promise`\<`void`\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

##### address

`string`

#### Returns

`Promise`\<`void`\>

***

### create()

> **create**(`options?`): `Promise`\<[`Account`](../classes/Account.md)\>

#### Parameters

##### options?

[`CreateAccountOptions`](../type-aliases/CreateAccountOptions.md)

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### export()

> **export**(`accountId`, `options?`): `Promise`\<[`AccountFile`](../classes/AccountFile.md)\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

##### options?

[`ExportAccountOptions`](ExportAccountOptions.md)

#### Returns

`Promise`\<[`AccountFile`](../classes/AccountFile.md)\>

***

### get()

> **get**(`accountId`): `Promise`\<[`Account`](../classes/Account.md)\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### getBalance()

> **getBalance**(`accountId`, `tokenId`): `Promise`\<`bigint`\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

##### tokenId

[`AccountRef`](../type-aliases/AccountRef.md)

#### Returns

`Promise`\<`bigint`\>

***

### getDetails()

> **getDetails**(`accountId`): `Promise`\<[`AccountDetails`](AccountDetails.md)\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

#### Returns

`Promise`\<[`AccountDetails`](AccountDetails.md)\>

***

### import()

> **import**(`input`): `Promise`\<[`Account`](../classes/Account.md)\>

#### Parameters

##### input

[`ImportAccountInput`](../type-aliases/ImportAccountInput.md)

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### list()

> **list**(): `Promise`\<[`AccountHeader`](../classes/AccountHeader.md)[]\>

#### Returns

`Promise`\<[`AccountHeader`](../classes/AccountHeader.md)[]\>

***

### removeAddress()

> **removeAddress**(`accountId`, `address`): `Promise`\<`void`\>

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

##### address

`string`

#### Returns

`Promise`\<`void`\>
