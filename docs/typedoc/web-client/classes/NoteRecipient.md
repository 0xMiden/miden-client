[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteRecipient

# Class: NoteRecipient

Value that describes under which condition a note can be consumed.

The recipient is not an account address, instead it is a value that describes when a note can be
consumed. Because not all notes have predetermined consumer addresses, e.g. swap notes can be
consumed by anyone, the recipient is defined as the code and its storage, that when successfully
executed results in the note's consumption.

Recipient is computed as a nested hash of the serial number, the script root, and the storage
commitment, ensuring the recipient digest binds all three pieces of data together.

## Constructors

### Constructor

> **new NoteRecipient**(`serial_num`, `note_script`, `storage`): `NoteRecipient`

Creates a note recipient from its serial number, script, and storage.

#### Parameters

##### serial\_num

[`Word`](Word.md)

##### note\_script

[`NoteScript`](NoteScript.md)

##### storage

[`NoteStorage`](NoteStorage.md)

#### Returns

`NoteRecipient`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### digest()

> **digest**(): [`Word`](Word.md)

Returns the digest of the recipient data (used in the note commitment).

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### script()

> **script**(): [`NoteScript`](NoteScript.md)

Returns the script that controls consumption.

#### Returns

[`NoteScript`](NoteScript.md)

***

### serialNum()

> **serialNum**(): [`Word`](Word.md)

Returns the serial number that prevents double spends.

#### Returns

[`Word`](Word.md)

***

### storage()

> **storage**(): [`NoteStorage`](NoteStorage.md)

Returns the storage provided to the script.

#### Returns

[`NoteStorage`](NoteStorage.md)
