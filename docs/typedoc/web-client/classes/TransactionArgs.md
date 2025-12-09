[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionArgs

# Class: TransactionArgs

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### adviceInputs()

> **adviceInputs**(): [`AdviceInputs`](AdviceInputs.md)

Returns advice inputs attached to the transaction.

#### Returns

[`AdviceInputs`](AdviceInputs.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getNoteArgs()

> **getNoteArgs**(`note_id`): [`Word`](Word.md)

Returns note-specific arguments for the given note ID.

#### Parameters

##### note\_id

[`NoteId`](NoteId.md)

#### Returns

[`Word`](Word.md)

***

### txScript()

> **txScript**(): [`TransactionScript`](TransactionScript.md)

Returns the transaction script if provided.

#### Returns

[`TransactionScript`](TransactionScript.md)
