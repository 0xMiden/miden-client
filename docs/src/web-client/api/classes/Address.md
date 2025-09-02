[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Address

# Class: Address

## Constructors

### Constructor

> **new Address**(`account_id`, `_interface`): `Address`

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### \_interface

[`AddressInterface`](../type-aliases/AddressInterface.md)

#### Returns

`Address`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### interface()

> **interface**(): [`AddressInterface`](../type-aliases/AddressInterface.md)

#### Returns

[`AddressInterface`](../type-aliases/AddressInterface.md)

***

### to\_bech32()

> **to\_bech32**(`network_id`): `string`

#### Parameters

##### network\_id

[`NetworkId`](../type-aliases/NetworkId.md)

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

> **toNoteTag**(): [`NoteTag`](NoteTag.md)

#### Returns

[`NoteTag`](NoteTag.md)

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### from\_bech32()

> `static` **from\_bech32**(`bech32`): `Address`

#### Parameters

##### bech32

`string`

#### Returns

`Address`
