[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionArgs

# Class: TransactionArgs

Optional transaction arguments.

- Transaction script: a program that is executed in a transaction after all input notes scripts
  have been executed.
- Note arguments: data put onto the stack right before a note script is executed. These are
  different from note inputs, as the user executing the transaction can specify arbitrary note
  args.
- Advice inputs: Provides data needed by the runtime, like the details of public output notes.
- Account inputs: Provides account data that will be accessed in the transaction.

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
