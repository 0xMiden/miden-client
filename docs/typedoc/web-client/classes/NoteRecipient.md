[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteRecipient

# Class: NoteRecipient

## Constructors

### Constructor

> **new NoteRecipient**(`serial_num`, `note_script`, `inputs`): `NoteRecipient`

Creates a note recipient from its serial number, script, and inputs.

#### Parameters

##### serial\_num

[`Word`](Word.md)

##### note\_script

[`NoteScript`](NoteScript.md)

##### inputs

[`NoteInputs`](NoteInputs.md)

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

### inputs()

> **inputs**(): [`NoteInputs`](NoteInputs.md)

Returns the inputs provided to the script.

#### Returns

[`NoteInputs`](NoteInputs.md)

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
