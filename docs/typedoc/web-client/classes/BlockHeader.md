[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / BlockHeader

# Class: BlockHeader

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountRoot()

> **accountRoot**(): [`Word`](Word.md)

Returns the account root commitment.

#### Returns

[`Word`](Word.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block height.

#### Returns

`number`

***

### chainCommitment()

> **chainCommitment**(): [`Word`](Word.md)

Returns the chain commitment.

#### Returns

[`Word`](Word.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to the block contents.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### noteRoot()

> **noteRoot**(): [`Word`](Word.md)

Returns the note commitment root.

#### Returns

[`Word`](Word.md)

***

### nullifierRoot()

> **nullifierRoot**(): [`Word`](Word.md)

Returns the nullifier root commitment.

#### Returns

[`Word`](Word.md)

***

### prevBlockCommitment()

> **prevBlockCommitment**(): [`Word`](Word.md)

Returns the commitment of the previous block.

#### Returns

[`Word`](Word.md)

***

### proofCommitment()

> **proofCommitment**(): [`Word`](Word.md)

Returns the proof commitment.

#### Returns

[`Word`](Word.md)

***

### subCommitment()

> **subCommitment**(): [`Word`](Word.md)

Returns the commitment to block metadata.

#### Returns

[`Word`](Word.md)

***

### timestamp()

> **timestamp**(): `number`

Returns the block timestamp.

#### Returns

`number`

***

### txCommitment()

> **txCommitment**(): [`Word`](Word.md)

Returns the transaction commitment.

#### Returns

[`Word`](Word.md)

***

### txKernelCommitment()

> **txKernelCommitment**(): [`Word`](Word.md)

Returns the transaction kernel commitment.

#### Returns

[`Word`](Word.md)

***

### version()

> **version**(): `number`

Returns the header version.

#### Returns

`number`
