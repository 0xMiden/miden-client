[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Address

# Class: Address

Wrapper around account addresses that can be exposed to JavaScript.

## Methods

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the underlying account identifier, if this is an account address.

#### Returns

[`AccountId`](AccountId.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### interface()

> **interface**(): [`AddressInterface`](../type-aliases/AddressInterface.md)

Returns the interface associated with this address.

#### Returns

[`AddressInterface`](../type-aliases/AddressInterface.md)

***

### toBech32()

> **toBech32**(`network_id`): `string`

Encodes the address into the network-specific bech32 representation.

#### Parameters

##### network\_id

[`NetworkId`](../enumerations/NetworkId.md)

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

Converts the address into a note tag for note filters.

#### Returns

[`NoteTag`](NoteTag.md)

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### fromAccountId()

> `static` **fromAccountId**(`account_id`, `_interface`): `Address`

Builds an address from an account identifier and interface name.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### \_interface

`string`

#### Returns

`Address`

***

### fromBech32()

> `static` **fromBech32**(`bech32`): `Address`

Parses an address from its bech32 representation.

#### Parameters

##### bech32

`string`

#### Returns

`Address`
