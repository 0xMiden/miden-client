---
title: Address
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / Address

# Class: Address

## Methods

### accountId()

> **accountId**(): [`AccountId`](AccountId)

#### Returns

[`AccountId`](AccountId)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### interface()

> **interface**(): [`AddressInterface`](../type-aliases/AddressInterface)

#### Returns

[`AddressInterface`](../type-aliases/AddressInterface)

***

### toBech32()

> **toBech32**(`network_id`): `string`

#### Parameters

##### network\_id

[`NetworkId`](../enumerations/NetworkId)

#### Returns

`string`

***

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toNoteTag()

> **toNoteTag**(): [`NoteTag`](NoteTag)

#### Returns

[`NoteTag`](NoteTag)

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### fromAccountId()

> `static` **fromAccountId**(`account_id`, `_interface`): `Address`

#### Parameters

##### account\_id

[`AccountId`](AccountId)

##### \_interface

[`AddressInterface`](../type-aliases/AddressInterface)

#### Returns

`Address`

***

### fromBech32()

> `static` **fromBech32**(`bech32`): `Address`

#### Parameters

##### bech32

`string`

#### Returns

`Address`
