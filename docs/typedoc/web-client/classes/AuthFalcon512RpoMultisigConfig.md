[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AuthFalcon512RpoMultisigConfig

# Class: AuthFalcon512RpoMultisigConfig

Multisig auth configuration for `RpoFalcon512` signatures.

## Constructors

### Constructor

> **new AuthFalcon512RpoMultisigConfig**(`approvers`, `default_threshold`): `AuthFalcon512RpoMultisigConfig`

Build a configuration with a list of approver public key commitments and a default
threshold.

`default_threshold` must be >= 1 and <= `approvers.length`.

#### Parameters

##### approvers

[`Word`](Word.md)[]

##### default\_threshold

`number`

#### Returns

`AuthFalcon512RpoMultisigConfig`

## Properties

### approvers

> `readonly` **approvers**: [`Word`](Word.md)[]

Approver public key commitments as Words.

***

### defaultThreshold

> `readonly` **defaultThreshold**: `number`

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

### getProcThresholds()

> **getProcThresholds**(): [`ProcedureThreshold`](ProcedureThreshold.md)[]

Per-procedure thresholds.

#### Returns

[`ProcedureThreshold`](ProcedureThreshold.md)[]

***

### withProcThresholds()

> **withProcThresholds**(`proc_thresholds`): `AuthFalcon512RpoMultisigConfig`

Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.

#### Parameters

##### proc\_thresholds

[`ProcedureThreshold`](ProcedureThreshold.md)[]

#### Returns

`AuthFalcon512RpoMultisigConfig`
