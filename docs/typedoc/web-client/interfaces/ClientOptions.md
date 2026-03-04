[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ClientOptions

# Interface: ClientOptions

## Properties

### autoSync?

> `optional` **autoSync**: `boolean`

Sync state on creation (default: false).

***

### keystore?

> `optional` **keystore**: `object`

External keystore callbacks.

#### getKey

> **getKey**: [`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

#### insertKey

> **insertKey**: [`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

#### sign

> **sign**: [`SignCallback`](../type-aliases/SignCallback.md)

***

### noteTransportUrl?

> `optional` **noteTransportUrl**: `string`

Note transport URL (optional).

***

### proverUrl?

> `optional` **proverUrl**: `string`

Auto-creates a remote prover from this URL.

***

### rpcUrl?

> `optional` **rpcUrl**: `string`

RPC endpoint URL. Defaults to testnet RPC.

***

### seed?

> `optional` **seed**: `string` \| `Uint8Array`

Hashed to 32 bytes via SHA-256.

***

### storeName?

> `optional` **storeName**: `string`

Store isolation key.
