[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Address

# Class: Address

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID embedded in the address.

#### Returns

[`AccountId`](AccountId.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### interface()

> **interface**(): `"BasicWallet"`

Returns the address interface.

#### Returns

`"BasicWallet"`

***

### toBech32()

> **toBech32**(`network_id`): `string`

Encodes the address using the provided network prefix.

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

Converts the address into a note tag.

#### Returns

[`NoteTag`](NoteTag.md)

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Address`

Deserializes a byte array into an `Address`.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Address`

***

### fromAccountId()

> `static` **fromAccountId**(`account_id`, `_interface?`): `Address`

Builds an address from an account ID and optional interface.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### \_interface?

`string`

#### Returns

`Address`

***

### fromBech32()

> `static` **fromBech32**(`bech32`): `Address`

Builds an address from a bech32-encoded string.

#### Parameters

##### bech32

`string`

#### Returns

`Address`
