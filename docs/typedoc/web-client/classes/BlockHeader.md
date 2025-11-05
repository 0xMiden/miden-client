[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / BlockHeader

# Class: BlockHeader

Wrapper around block header data returned by the network.

## Methods

### accountRoot()

> **accountRoot**(): [`Word`](Word.md)

Returns the account Merkle root for the block.

#### Returns

[`Word`](Word.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block number.

#### Returns

`number`

***

### chainCommitment()

> **chainCommitment**(): [`Word`](Word.md)

Returns the chain commitment accumulating historical state.

#### Returns

[`Word`](Word.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the overall commitment to the block contents.

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

Returns the note commitment root for the block.

#### Returns

[`Word`](Word.md)

***

### nullifierRoot()

> **nullifierRoot**(): [`Word`](Word.md)

Returns the nullifier set root for the block.

#### Returns

[`Word`](Word.md)

***

### prevBlockCommitment()

> **prevBlockCommitment**(): [`Word`](Word.md)

Returns the commitment of the previous block in the chain.

#### Returns

[`Word`](Word.md)

***

### proofCommitment()

> **proofCommitment**(): [`Word`](Word.md)

Returns the proof commitment attesting to block validity.

#### Returns

[`Word`](Word.md)

***

### subCommitment()

> **subCommitment**(): [`Word`](Word.md)

Returns the sub-commitment combining state roots.

#### Returns

[`Word`](Word.md)

***

### timestamp()

> **timestamp**(): `number`

Returns the timestamp assigned to the block.

#### Returns

`number`

***

### txCommitment()

> **txCommitment**(): [`Word`](Word.md)

Returns the commitment to the transactions included in the block.

#### Returns

[`Word`](Word.md)

***

### txKernelCommitment()

> **txKernelCommitment**(): [`Word`](Word.md)

Returns the commitment to transaction kernels included in the block.

#### Returns

[`Word`](Word.md)

***

### version()

> **version**(): `number`

Returns the block version number.

#### Returns

`number`
