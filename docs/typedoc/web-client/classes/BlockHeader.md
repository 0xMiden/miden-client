[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / BlockHeader

# Class: BlockHeader

Public header for a block, containing commitments to the chain state and the proof attesting to
the block's validity.

Key fields include the previous block commitment, block number, chain/nullifier/note roots,
transaction commitments (including the kernel), proof commitment, and a timestamp. Two derived
values are exposed:
- `sub_commitment`: sequential hash of all fields except the `note_root`.
- `commitment`: a 2-to-1 hash of the `sub_commitment` and the `note_root`.

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
