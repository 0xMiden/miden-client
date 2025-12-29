[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AuthRpoFalcon512MultisigConfig

# Class: AuthRpoFalcon512MultisigConfig

Multisig auth configuration for `RpoFalcon512` signatures.

## Constructors

### Constructor

> **new AuthRpoFalcon512MultisigConfig**(`approvers`, `default_threshold`): `AuthRpoFalcon512MultisigConfig`

Build a configuration with a list of approver public key commitments and a default
threshold.

`default_threshold` must be >= 1 and <= `approvers.length`.

#### Parameters

##### approvers

[`Word`](Word.md)[]

##### default\_threshold

`number`

#### Returns

`AuthRpoFalcon512MultisigConfig`

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

> **withProcThresholds**(`proc_thresholds`): `AuthRpoFalcon512MultisigConfig`

Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.

#### Parameters

##### proc\_thresholds

[`ProcedureThreshold`](ProcedureThreshold.md)[]

#### Returns

`AuthRpoFalcon512MultisigConfig`
