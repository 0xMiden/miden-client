[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteRecipient

# Class: NoteRecipient

Target recipient information for a note, including script and serial number.

## Constructors

### Constructor

> **new NoteRecipient**(`serial_num`, `note_script`, `inputs`): `NoteRecipient`

Creates a new note recipient from serial number, script, and inputs.

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

### digest()

> **digest**(): [`Word`](Word.md)

Returns the digest identifying this recipient.

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

Returns the inputs passed to the recipient script.

#### Returns

[`NoteInputs`](NoteInputs.md)

***

### script()

> **script**(): [`NoteScript`](NoteScript.md)

Returns the script associated with this recipient.

#### Returns

[`NoteScript`](NoteScript.md)

***

### serialNum()

> **serialNum**(): [`Word`](Word.md)

Returns the recipient serial number word.

#### Returns

[`Word`](Word.md)
