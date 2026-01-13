[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteMetadata

# Class: NoteMetadata

Metadata associated with a note.

Note type and tag must be internally consistent according to the following rules:

- For private and encrypted notes, the two most significant bits of the tag must be `0b11`.
- For public notes, the two most significant bits of the tag can be set to any value.

# Word layout & validity

`NoteMetadata` can be encoded into a `Word` with the following layout:

```text
1st felt: [sender_id_prefix (64 bits)]
2nd felt: [sender_id_suffix (56 bits) | note_type (2 bits) | note_execution_hint_tag (6 bits)]
3rd felt: [note_execution_hint_payload (32 bits) | note_tag (32 bits)]
4th felt: [aux (64 bits)]
```

The rationale for the above layout is to ensure the validity of each felt:
- 1st felt: Is equivalent to the prefix of the account ID so it inherits its validity.
- 2nd felt: The lower 8 bits of the account ID suffix are `0` by construction, so that they can
  be overwritten with other data. The suffix is designed such that it retains its felt validity
  even if all of its lower 8 bits are be set to `1`. This is because the most significant bit is
  always zero.
- 3rd felt: The note execution hint payload must contain at least one `0` bit in its encoding,
  so the upper 32 bits of the felt will contain at least one `0` bit making the entire felt
  valid.
- 4th felt: The `aux` value must be a felt itself.

## Constructors

### Constructor

> **new NoteMetadata**(`sender`, `note_type`, `note_tag`, `note_execution_hint`, `aux?`): `NoteMetadata`

Creates metadata for a note.

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### note\_tag

[`NoteTag`](NoteTag.md)

##### note\_execution\_hint

[`NoteExecutionHint`](NoteExecutionHint.md)

##### aux?

[`Felt`](Felt.md)

#### Returns

`NoteMetadata`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### noteType()

> **noteType**(): [`NoteType`](../enumerations/NoteType.md)

Returns whether the note is private, encrypted, or public.

#### Returns

[`NoteType`](../enumerations/NoteType.md)

***

### sender()

> **sender**(): [`AccountId`](AccountId.md)

Returns the account that created the note.

#### Returns

[`AccountId`](AccountId.md)

***

### tag()

> **tag**(): [`NoteTag`](NoteTag.md)

Returns the tag associated with the note.

#### Returns

[`NoteTag`](NoteTag.md)
