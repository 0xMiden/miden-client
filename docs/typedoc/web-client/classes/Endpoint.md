[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Endpoint

# Class: Endpoint

The `Endpoint` struct represents a network endpoint, consisting of a protocol, a host, and a
port.

This struct is used to define the address of a Miden node that the client will connect to.

## Constructors

### Constructor

> **new Endpoint**(`url`): `Endpoint`

Creates an endpoint from a URL string.

#### Parameters

##### url

`string`

The URL string (e.g., <https://localhost:57291>)

#### Returns

`Endpoint`

#### Throws

throws an error if the URL is invalid

## Properties

### host

> `readonly` **host**: `string`

Returns the host of the endpoint.

***

### port

> `readonly` **port**: `number`

Returns the port of the endpoint.

***

### protocol

> `readonly` **protocol**: `string`

Returns the protocol of the endpoint.

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

### toString()

> **toString**(): `string`

Returns the string representation of the endpoint.

#### Returns

`string`

***

### devnet()

> `static` **devnet**(): `Endpoint`

Returns the endpoint for the Miden devnet.

#### Returns

`Endpoint`

***

### localhost()

> `static` **localhost**(): `Endpoint`

Returns the endpoint for a local Miden node.

Uses <http://localhost:57291>

#### Returns

`Endpoint`

***

### testnet()

> `static` **testnet**(): `Endpoint`

Returns the endpoint for the Miden testnet.

#### Returns

`Endpoint`
