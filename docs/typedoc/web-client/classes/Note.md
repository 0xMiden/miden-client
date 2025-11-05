[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Note

# Class: Note

## Constructors

### Constructor

> **new Note**(`note_assets`, `note_metadata`, `note_recipient`): `Note`

#### Parameters

##### note\_assets

[`NoteAssets`](NoteAssets.md)

##### note\_metadata

[`NoteMetadata`](NoteMetadata.md)

##### note\_recipient

[`NoteRecipient`](NoteRecipient.md)

#### Returns

`Note`

## Methods

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

#### Returns

[`NoteAssets`](NoteAssets.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId.md)

#### Returns

[`NoteId`](NoteId.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient.md)

#### Returns

[`NoteRecipient`](NoteRecipient.md)

***

### script()

> **script**(): [`NoteScript`](NoteScript.md)

#### Returns

[`NoteScript`](NoteScript.md)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### createP2IDENote()

> `static` **createP2IDENote**(`sender`, `target`, `assets`, `reclaim_height`, `timelock_height`, `note_type`, `aux`): `Note`

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### target

[`AccountId`](AccountId.md)

##### assets

[`NoteAssets`](NoteAssets.md)

##### reclaim\_height

`number`

##### timelock\_height

`number`

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### aux

[`Felt`](Felt.md)

#### Returns

`Note`

***

### createP2IDNote()

> `static` **createP2IDNote**(`sender`, `target`, `assets`, `note_type`, `aux`): `Note`

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### target

[`AccountId`](AccountId.md)

##### assets

[`NoteAssets`](NoteAssets.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### aux

[`Felt`](Felt.md)

#### Returns

`Note`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Note`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Note`
