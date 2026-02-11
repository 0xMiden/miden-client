# Changelog

## 0.14.0 (TBD)

### Enhancements

* Updated the `GrpcClient` to fetch the RPC limits from the node ([#1724](https://github.com/0xMiden/miden-client/pull/1724)) ([#1737](https://github.com/0xMiden/miden-client/pull/1737)).
* Added typed error parsing for node RPC endpoints, enabling programmatic error handling instead of string parsing ([#1734](https://github.com/0xMiden/miden-client/pull/1734)).
* Added `--rpc-status` flag to `miden-client info` command to display RPC node status information including node version, genesis commitment, store status, and block producer status; also added `get_status_unversioned` to `NodeRpcClient` trait ([#1742](https://github.com/0xMiden/miden-client/pull/1742)).
* Prevent a potential unwrap panic in `insert_storage_map_nodes_for_map` ([#1750](https://github.com/0xMiden/miden-client/pull/1750)).

### Changes

* [BREAKING] Added a `AccountReader` accessible through `Client::account_reader` to read account data without needing to load the whole `Account` ([#1713](https://github.com/0xMiden/miden-client/pull/1713), [#1716](https://github.com/0xMiden/miden-client/pull/1716)).
* Added implementation for the `get_public_key` method on the `FilesystemKeystore` and `WebKeystore` ([#1731](https://github.com/0xMiden/miden-client/pull/1731)).

## 0.13.1 (TBD)

* Added the `@miden-sdk/react` hooks library (see [its own changelog](packages/react-sdk/CHANGELOG.md)) ([#1711](https://github.com/0xMiden/miden-client/pull/1711)).

## 0.13.0 (2026-01-28)

* [BREAKING] Removed `getRpoFalcon512PublicKeyAsWord` and `getEcdsaK256KeccakPublicKeyAsWord` in `AuthSecretKey`
* Improved auth scheme handling across the Rust and web clients (typed `build_wallet_id`, unified transaction tests, new shared `getPublicKeyAsWord` binding, and refreshed typedoc output) ([#1556](https://github.com/0xMiden/miden-client/pull/1556)).
* [BREAKING] Typed the `auth_scheme` plumbing across the Rust WebClient ID-building helpers and aligned the WebClient bindings with the native enum to avoid passing raw identifiers ([#1546](https://github.com/0xMiden/miden-client/pull/1546)).
* [BREAKING] WebClient `AccountComponent.createAuthComponentFromCommitment` now takes `AuthScheme` (enum) instead of a numeric scheme id. The old `AccountComponent.createAuthComponent` method was removed; use `createAuthComponentFromSecretKey` instead ([#1578](https://github.com/0xMiden/miden-client/issues/1578)).
* Changed `blockNum` type from `string` to `number` in WebClient transaction interfaces for better type safety and consistency ([#1528](https://github.com/0xMiden/miden-client/pull/1528)).
* Consolidated `FetchedNote` fields into `NoteHeader` ([#1536](https://github.com/0xMiden/miden-client/pull/1536)).
* Tied the web client's IndexedDB schema to the running package version, automatically recreating or wiping stale stores and applying the same guard to `forceImportStore` ([#1576](https://github.com/0xMiden/miden-client/pull/1576)).
* Added the `--remote-prover-timeout` configuration to the CLI ([#1551](https://github.com/0xMiden/miden-client/pull/1551)).
* Surface WASM worker errors to the JS wrapper with their original stacks for clearer diagnostics ([#1565](https://github.com/0xMiden/miden-client/issues/1565)).
* Added doc_cfg as top level cfg_attr to turn on feature annotations in docs.rs and added make targets to serve the docs ([#1543](https://github.com/0xMiden/miden-client/pull/1543)).
* Updated `DataStore` implementation to prevent retrieving whole `vault` and `storage` ([#1419](https://github.com/0xMiden/miden-client/pull/1419))
* Added RPC limit handling for `sync_nullifiers` endpoint ([#1590](https://github.com/0xMiden/miden-client/pull/1590)).
* Added pagination handling for `sync_storage_maps` and `sync_account_vault` RPC endpoints.
* Added a convenience function `fromBech32` to turn a bech32 string into an AccountId ([#1607](https://github.com/0xMiden/miden-client/pull/1607)).
* [BREAKING] Refactored the fields in retrieved notes in the WebClient: now the inclusion proof has been factored out and is always accessible ([#1606](https://github.com/0xMiden/miden-client/pull/1606)).
* [BREAKING] Renamed `NodeRpcClient::get_account_proofs` to `NodeRpcClient::get_account_proof` & added `account_state` parameter (block at which we want to retrieve the proof) ([#1616](https://github.com/0xMiden/miden-client/pull/1616)).
* [BREAKING] Refactored `NetworkId` to allow custom networks ([#1612](https://github.com/0xMiden/miden-client/pull/1612)).
* [BREAKING] Removed `toBech32Custom` and implemented custom id conversion for wasm derived class `NetworkId` ([#1612](https://github.com/0xMiden/miden-client/pull/1612)).
* [BREAKING] Remove `SecretKey` model and consolidated functionality into `AuthSecretKey` ([#1592](https://github.com/0xMiden/miden-client/issues/1380))
* Incremented the limits for various RPC calls to accommodate larger data sets ([#1621](https://github.com/0xMiden/miden-client/pull/1621)).
* [BREAKING] Introduced named storage slots, changed `FilesystemKeystore` to not be generic over RNG ([#1626](https://github.com/0xMiden/miden-client/pull/1626)).
* Added `submit_new_transaction_with_prover` to the Rust client and `submitNewTransactionWithProver` to the WebClient([#1622](https://github.com/0xMiden/miden-client/pull/1622)).
* Fixed MMR reconstruction code and fixed how block authentication paths are adjusted ([#1633](https://github.com/0xMiden/miden-client/pull/1633)).
* Added WebClient bindings and RPC helpers for additional account, note, and validation workflows ([#1638](https://github.com/0xMiden/miden-client/pull/1638)).
* [BREAKING] Modified JS binding for `AccountComponent::compile` which now takes an `AccountComponentCode` built with the newly added binding `CodeBuilder::compile_account_component_code` ([#1627](https://github.com/0xMiden/miden-client/pull/1627)).
* Expanded the `GrpcClient` API with methods to fetch account proofs and rebuild the slots for an account ([#1591](https://github.com/0xMiden/miden-client/pull/1591)).
* [BREAKING] `WebClient.addAccountSecretKeyToWebStore` now takes an additional parameter: an account ID. This will link the ID with the secret key in the WebStore. Added `WebClient.getPublicKeyCommitmentsOfAccount` method that will return a list of related public key commitments for the given account ID ([#1608](https://github.com/0xMiden/miden-client/pull/1608)).
* [BREAKING] Added naming to `IndexedDB` store to allow multiple WebClient instances to run in the same browser; `WebClient.createClient` now takes an optional DB name (otherwise defaults to name based on the endpoint/network) ([#1645](https://github.com/0xMiden/miden-client/pull/1645)).
* [BREAKING] Simplified the `NoteScreener` API, removing `NoteRelevance` in favor of `NoteConsumptionStatus`; exposed JS bindings for consumption check results ([#1630](https://github.com/0xMiden/miden-client/pull/1630)).
* [BREAKING] Replaced `TransactionRequestBuilder::unauthenticated_input_notes` & `TransactionRequestBuilder::authenticated_input_notes` for `TransactionRequestBuilder::input_notes`, now the user passes a list of notes which the `Client` itself determines the authentication status of ([#1624](https://github.com/0xMiden/miden-client/issues/1624)).
* Updated `SqliteStore`: replaced `MerkleStore` with `SmtForest` and introduced `AccountSmtForest`; simplified queries ([#1526](https://github.com/0xMiden/miden-client/pull/1526), [#1663](https://github.com/0xMiden/miden-client/pull/1663)).
* Added filter to store query to improve how the MMR is built ([#1681](https://github.com/0xMiden/miden-client/pull/1681)).
* [BREAKING] Required the client RNG to be `Send + Sync` (via the `ClientFeltRng` marker and `ClientRngBox` alias) so `Client` can be `Send + Sync` ([#1677](https://github.com/0xMiden/miden-client/issues/1677)).
* Fixed a race condition in `pruneIrrelevantBlocks` that could delete the current block header when multiple tabs share IndexedDB, causing sync to panic ([#1650](https://github.com/0xMiden/miden-client/pull/1650)).
* Fixed a race condition where concurrent sync operations could cause sync height to go backwards, leading to block header deletion and subsequent panics ([#1650](https://github.com/0xMiden/miden-client/pull/1650)).
* Changed `get_current_partial_mmr` to return a `StoreError::BlockHeaderNotFound` error instead of panicking when the block header is missing ([#1650](https://github.com/0xMiden/miden-client/pull/1650)).
* Added `CliClient` wrapper and `CliConfig::from_system()` to allow creating a CLI-configured client programmatically ([#1642](https://github.com/0xMiden/miden-client/pull/1642)).
* [BREAKING] Updated `BlockNumber` IndexedDB type: changed from `string` to `number` ([#1684](https://github.com/0xMiden/miden-client/pull/1684)).
* [BREAKING] Upgraded to protocol 0.13: exposed and aligned note-related structs to WebClient; `NoteTag` and `NoteAttachment` APIs updated renamed `NoteTag.fromAccountId` to `withAccountTarget`, added `withCustomAccountTarget`; added `NoteAttachmentScheme` wrapper and content accessors (`asWord`, `asArray`) to `NoteAttachment`; removed `NoteExecutionMode` ([#1685](https://github.com/0xMiden/miden-client/pull/1685)).
* Added sync lock to coordinate concurrent `syncState()` calls in the WebClient using the Web Locks API, with coalescing behavior where concurrent callers share results from an in-progress sync ([#1690](https://github.com/0xMiden/miden-client/pull/1690)).
* [BREAKING] Removed the `payback_note_type` field from the swap command ([#1700](https://github.com/0xMiden/miden-client/pull/1700)).

## 0.12.6 (2026-01-08)

* Enabled Workers with `createClientWithExternalKeystore` via callbacks ([#1569](https://github.com/0xMiden/miden-client/pull/1569)).
* Added `executeForSummary` method to WebClient that executes a transaction and returns a `TransactionSummary`, handling both authorized and unauthorized transactions ([#1620](https://github.com/0xMiden/miden-client/pull/1620)).
* Added WebClient bindings for the RPO Falcon512 multisig auth component ([#1620](https://github.com/0xMiden/miden-client/pull/1620)).
* Added seed to `AccountStatus::Locked` variant in `AccountRecord` to track private accounts that are locked due to a mismatch in the account commitment ([#1665](https://github.com/0xMiden/miden-client/pull/1665)).

## 0.12.5 (2025-12-01)

* Removed the top-level await from the web-client JS entry point by lazily loading the WASM module, allowing `@miden-sdk/miden-sdk` to be imported normally (including in Next.js SSR builds), and updated the worker bootstrap to match.
* Changed the default note transport endpoint from `localhost` to `https://transport.miden.io` ([#1574](https://github.com/0xMiden/miden-client/pull/1574)).
* Fixed a bug where insertions in the `Addresses` table in the IndexedDB Store resulted in the `id` and `address` fields being inverted with each other ([#1532](https://github.com/0xMiden/miden-client/pull/1532)).
* Changed the note script pre-loading step to include all expected scripts based on specified recipients ([#1539](https://github.com/0xMiden/miden-client/pull/1539)).
* Added methods to `Package` exposing inner `Program`/`Library`. Also implemented `fromPackage` methods for `NoteScript` & `TransactionScript` ([#1550](https://github.com/0xMiden/miden-client/pull/1550)).
* Added RPC limit handling for `check_nullifiers` and `get_notes_by_id` ([#1558](https://github.com/0xMiden/miden-client/pull/1558)).
* Fixed account rollback bug by not loading already discarded transaction on sync state ([#1567](https://github.com/0xMiden/miden-client/pull/1567)).
* Added `--version` flag to client CLI ([#1586](https://github.com/0xMiden/miden-client/pull/1586)).
* Refactored note fetching from the transport layer, calling now `import_note()` on retrieved notes ([#1579](https://github.com/0xMiden/miden-client/pull/1579)).

## Miden Client CLI - 0.12.4 (2025-11-17)

* Fixed CLI install process to statically include account component package files ([#1530](https://github.com/0xMiden/miden-client/pull/1530)).

## 0.12.3 (2025-11-16)

* Added `recoverFrom()` function to WASM `PublicKey` and added back `TransactionSummary` back to `index.d.ts` ([#1513](https://github.com/0xMiden/miden-client/pull/1513)).
* Added `hasProcedure` to `AccountCode` and `getProcedures` to `AccountComponent` in the WebClient ([#1517](https://github.com/0xMiden/miden-client/pull/1517)).
* Retrieve inclusion proofs for fetched notes from the Note Transport layer ([#1495](https://github.com/0xMiden/miden-client/pull/1495)).
* Added ECDSA auth component to the rust-client & web-client ([#1527](https://github.com/0xMiden/miden-client/pull/1527))

## 0.12.2 (2025-11-12)

* Added `prover()` setter to `ClientBuilder` to allow configuring custom transaction provers ([#1499](https://github.com/0xMiden/miden-client/pull/1499)).
* Added `AccountStorageMode` getters for `Account` and `AccountId`. [(#1509)](https://github.com/0xMiden/miden-client/pull/1509).
* Allowed `new-account` command to create accounts with non-Falcon auth components ([#1443](https://github.com/0xMiden/miden-client/pull/1443)).
* Added new `.miden` directory for configuration files at the client CLI ([#1464](https://github.com/0xMiden/miden-client/pull/1464)).
* Added bindings for the new ECDSA auth scheme [(#1478)](https://github.com/0xMiden/miden-client/pull/1478).
* Exposed all auth packages from `miden-base`: `no-auth`, `multisig-auth`, and `acl-auth` components are now available in the CLI under `packages/auth/` subdirectory ([#1132](https://github.com/0xMiden/miden-client/issues/1132)).

## 0.12.0 (2025-11-10)

### Features

* Added support for getting specific vault and storage elements from `Store` along with their proofs ([#1164](https://github.com/0xMiden/miden-client/pull/1164)).
* Implemented functions for lazy loading on webstore [(#1184)](https://github.com/0xMiden/miden-client/pull/1184).
* Separated `migrations` and `settings` tables [(#1287)](https://github.com/0xMiden/miden-client/pull/1287).
* Added single default address on account creation ([#1308](https://github.com/0xMiden/miden-client/pull/1308)).
* Added a `GetNoteScriptByRoot` call to the `RpcClient` ([#1311](https://github.com/0xMiden/miden-client/pull/1311)).
* Implemented account lazy loading with more granular account data getters ([#1321](https://github.com/0xMiden/miden-client/pull/1321)).
* Added `NoAuth` component to the web client ([#1330](https://github.com/0xMiden/miden-client/pull/1330)).
* Implemented shared source manager for better error reporting ([#1275](https://github.com/0xMiden/miden-client/pull/1275)).
* Added `getMapEntries` method to `AccountStorage` in web client for iterating storage map entries ([#1323](https://github.com/0xMiden/miden-client/pull/1323)).
* Added `Address` addition and removal for accounts ([#1367](https://github.com/0xMiden/miden-client/pull/1367)).
* Refactored code into their own files and added `ProvenTransaction` and `TransactionStoreUpdate` bindings for the WebClient ([#1408](https://github.com/0xMiden/miden-client/pull/1408)).
* Added `NoteFile` type, used for exporting and importing `Notes`([#1378](https://github.com/0xMiden/miden-client/pull/1383)).
* Build `IndexedDB` code from a `build.rs` instead of pushing artifacts to the repo ([#1409](https://github.com/0xMiden/miden-client/pull/1409)).
* Implemented missing RPC endpoints: `/SyncStorageMaps`, `/SyncAccountVault` & `/SyncTransactions` ([#1362](https://github.com/0xMiden/miden-client/pull/1362)).
* Updated `submit_proven_transaction()` to include `TransactionInputs` for validator ([#1421](https://github.com/0xMiden/miden-client/pull/1421)).
* [BREAKING] Replaced `AccountComponentTemplates` for `Packages` for account creation ([#1313](https://github.com/0xMiden/miden-client/pull/1313)).
* Added support for silently initializing the client CLI ([#1424](https://github.com/0xMiden/miden-client/pull/1424)).
* Started allowing for note ID prefixes in CLI `notes --send` ([#1433](https://github.com/0xMiden/miden-client/pull/1433)).
* Refactored note scripts to be pre-loaded into the store instead of providing them through advice inputs ([#1426](https://github.com/0xMiden/miden-client/pull/1426)).
* [BREAKING] Refactored client transaction APIs and the new `TransactionResult` type ([#1407](https://github.com/0xMiden/miden-client/pull/1407)).
* Introduce an account and note tag limit to be tracked by the client. ([#1476](https://github.com/0xMiden/miden-client/pull/1476)).
* Added ability to create `AccountComponent` from a `Package` and `StorageSlot` array in the Web Client ([#1469](https://github.com/0xMiden/miden-client/pull/1469)).
* Added new global default .miden directory in HOME path at the client CLI ([#1465](https://github.com/0xMiden/miden-client/pull/1465))

### Changes

* [BREAKING] Incremented MSRV to 1.90.
* Added typed arrays for each public web-client model/struct ([#1292](https://github.com/0xMiden/miden-client/pull/1292))
* [BREAKING] Unified chain tip and block number types to use `BlockNumber` instead of `u32` ([#1415](https://github.com/0xMiden/miden-client/pull/1415)).
* Modified the RPC client to avoid reconnection when setting commitment header ([#1166](https://github.com/0xMiden/miden-client/pull/1166)).
* [BREAKING] Moved `SqliteStore` and `WebStore` into their own separate crates ([#1253](https://github.com/0xMiden/miden-client/pull/1253)).
* [BREAKING] Added `block_to` parameter to `NodeRpcClient::sync_nullifiers` for better pagination control ([#1309](https://github.com/0xMiden/miden-client/pull/1309)).
* [BREAKING] Removed `web-tonic` feature ([#1268](https://github.com/0xMiden/miden-client/pull/1268)).
* [BREAKING] Updated Web Client account store functions from insert to upsert ([#1274](https://github.com/0xMiden/miden-client/pull/1274)).
* [BREAKING] Added connectivity to the Transport Layer, adding a new `Client` field and `Store` methods ([#1296](https://github.com/0xMiden/miden-client/pull/1296)).
* Removed `miden-lib` and `miden-objects` dependencies from web client & cli ([#1333](https://github.com/0xMiden/miden-client/pull/1333)).
* Add more context to errors when deserializing objects ([#1336](https://github.com/0xMiden/miden-client/pull/1336))
* [BREAKING] Renamed `TonicRpcClient` to `GrpcClient` and `tonic_rpc_client()` method to `grpc_client()` ([#1360](https://github.com/0xMiden/miden-client/pull/1360)).
* [BREAKING] Removed WebClient's `compileNoteScript` method and both `TransactionScript` and `NoteScript` compile methods; the new `ScriptBuilder` should be used instead ([#1331](https://github.com/0xMiden/miden-client/pull/1274)).
* [BREAKING] Implemented `AccountFile` in the WebClient ([#1258](https://github.com/0xMiden/miden-client/pull/1258)).
* [BREAKING] Added remote key storage and signature requesting to the `WebKeyStore` ([#1371](https://github.com/0xMiden/miden-client/pull/1371)).
* Added `sqlite_store` under `ClientBuilderSqliteExt` method to the `ClientBuilder` ([#1416](https://github.com/0xMiden/miden-client/pull/1416)).
* [BREAKING] Updated the Web Client to integrate Note Transport ([#1374](https://github.com/0xMiden/miden-client/pull/1374)).
* [BREAKING] Refactored transaction APIs to support more granular updates in the transaction lifecycle ([#1407](https://github.com/0xMiden/miden-client/pull/1407)).
* Updated Dexie indexes and SQL schema; fixed sync-related transaction state bug ([#1452](https://github.com/0xMiden/miden-client/pull/1452)).
* Started syncing output note nullifiers by default, to track when they are consumed ([#1452](https://github.com/0xMiden/miden-client/pull/1452)).
* Expanded some `ClientError` variants to contain explanations and hints about the errors ([#1462](https://github.com/0xMiden/miden-client/pull/1462)).
* [BREAKING] Removed debug mode from the client, migrated to VM 0.20 ([#1629](https://github.com/0xMiden/miden-client/pull/1629)).

## 0.11.11 (2025-10-16)

* Added missing details to `SigningInputs` object to fetch underlying data type ([#1389](https://github.com/0xMiden/miden-client/pull/1389)).

## 0.11.10 (2025-10-15)

* Optimized sync-related lookups and RPC requests ([#1387](https://github.com/0xMiden/miden-client/pull/1387)).

## 0.11.9 (2025-10-08)

* Fixed a bug where StateSync failed when called multiple times while using Safari ([#1377](https://github.com/0xMiden/miden-client/pull/1377)).
* Implemented new note compatibility checker [(#1376)](https://github.com/0xMiden/miden-client/pull/1376).
* Added indexes to improve sync process performance [(#1363)](https://github.com/0xMiden/miden-client/pull/1363).

## 0.11.8 (2025-09-29)

* Added `serialize` and `deserialize` methods for `NoteScript` [(#1117)](https://github.com/0xMiden/miden-client/pull/1117).

## 0.11.7 (2025-09-26)

* Fixed an issue where `AccountId` was being left as null-pointer ([#1340](https://github.com/0xMiden/miden-client/pull/1340)).

## 0.11.6 (2025-09-18)

* Added a way to retrieve a secret key in the client given a pub key ([#1293](https://github.com/0xMiden/miden-client/pull/1293)).
* Reexported all authentication components from `miden-lib` ([#1297](https://github.com/0xMiden/miden-client/pull/1297)).
* Added `Signature` to the list of exported types in `index.d.ts`([#1303](https://github.com/0xMiden/miden-client/pull/1303)).
* Patched `miden-base` dependencies to 0.11.4 ([#1314](https://github.com/0xMiden/miden-client/pull/1314)).

## 0.11.4 (2025-09-11)

* Added a mutable getter for `TransactionRequest`'s advice map ([#1254](https://github.com/0xMiden/miden-client/pull/1254)).
* Added a way to retrieve map items in web client ([#1282](https://github.com/0xMiden/miden-client/pull/1282)).
* Defined `AccountInterface.Unspecified` in web client ([#1286](https://github.com/0xMiden/miden-client/pull/1286)).
* Removed `AccountId.fromBech32` ([#1288](https://github.com/0xMiden/miden-client/pull/1288)).

## 0.11.3 (2025-09-08)

* Refreshed dependencies ([#1269](https://github.com/0xMiden/miden-client/pull/1269)).

## 0.11.2 (2025-09-02)

* Added WASM bindings for the `Address` type from the miden_objects crate ([#1244](https://github.com/0xMiden/miden-client/pull/1244)).
* Updated index.d.ts file to reflect recent address changes + updates to `NetworkId` enum ([#1249](https://github.com/0xMiden/miden-client/pull/1249))

## 0.11.1 (2025-08-31)

### Fixes

* Added JS files generated from TypeScript ([#1218](https://github.com/0xMiden/miden-client/pull/1218)).
* Changed method for automatically picking up tests for integraion tests binary ([#1219](https://github.com/0xMiden/miden-client/pull/1219)).

## 0.11.0 (2025-08-30)

### Features

* Added ability to convert `Word` to `U64` array and `Felt` array in Web Client ([#1041](https://github.com/0xMiden/miden-client/pull/1041)).
* [BREAKING] Added genesis commitment header to `TonicRpcClient` requests ([#1045](https://github.com/0xMiden/miden-client/pull/1045)).
* Added `TokenSymbol` type to Web Client ([#1046](https://github.com/0xMiden/miden-client/pull/1046)).
* Implemented missing endpoints for the `MockRpcApi` ([#1074](https://github.com/0xMiden/miden-client/pull/1074)).
* Added bindings for retrieving storage `AccountDelta` in the web client ([#1098](https://github.com/0xMiden/miden-client/pull/1098)).
* Added `TransactionSummary`, `AccountDelta`, and `BasicFungibleFaucet` types to Web Client ([#1115](https://github.com/0xMiden/miden-client/pull/1115)).
* Added authentication arguments support to `TransactionRequest` ([#1121](https://github.com/0xMiden/miden-client/pull/1121)).
* Added `multicall` support for the CLI ([#1141](https://github.com/0xMiden/miden-client/pull/1141)).
* Added `SigningInputs` to Web Client ([#1160](https://github.com/0xMiden/miden-client/pull/1160)).
* Added an `RpcClient` to the Web Client, with a `getNotesById` call ([#1191](https://github.com/0xMiden/miden-client/pull/1191)).

### Changes

* [BREAKING] Incremented MSRV to 1.88.
* Introduced enums instead of booleans for public APIs ([#1042](https://github.com/0xMiden/miden-client/pull/1042)).
* [BREAKING] Updated `toBech32` AccountID method: it now expects a parameter to specify the NetworkID ([#1043](https://github.com/0xMiden/miden-client/pull/1043)).
* [BREAKING] Updated `applyStateSync` to receive a single object and then write the changes in a single transaction ([#1050](https://github.com/0xMiden/miden-client/pull/1050)).
* [BREAKING] Refactored `OnNoteReceived` callback to return enum with update action ([#1051](https://github.com/0xMiden/miden-client/pull/1051)).
* [BREAKING] Made authenticator optional for `ClientBuilder` and `Client::new`. The authenticator parameter is now optional, allowing clients to be created without authentication capabilities ([#1056](https://github.com/0xMiden/miden-client/pull/1056)).
* [BREAKING] `insertAccountRecord` changed the order of some parameters [(#1068)](https://github.com/0xMiden/miden-client/pull/1068).
* The rust-client has now a simple TypeScript setup for its JS code [(#1068)](https://github.com/0xMiden/miden-client/pull/1068).
* Added the `miden-client-integration-tests` binary for running integration tests against a remote node ([#1075](https://github.com/0xMiden/miden-client/pull/1075)).
* [BREAKING] Changed `OnNoteReceived` from closure to trait object ([#1080](https://github.com/0xMiden/miden-client/pull/1080)).
* `NoteScript` now has a `toString` method that prints its own MAST source [(#1082)](https://github.com/0xMiden/miden-client/pull/1082).
* Added support for `MockRpcApi` to web client ([#1096](https://github.com/0xMiden/miden-client/pull/1096)).
* [BREAKING] Implemented asynchronous execution hosts and removed web key store workarounds [(#1104)](https://github.com/0xMiden/miden-client/pull/1104).
* Exposed signatures and serialization for public keys and secret keys [(#1107)](https://github.com/0xMiden/miden-client/pull/1107).
* Added a `exportAccount` method in Web Client ([#1111](https://github.com/0xMiden/miden-client/pull/1111)).
* Exposed additional `TransactionFilter` filters in Web Client ([#1114](https://github.com/0xMiden/miden-client/pull/1114)).
* Refactored internal structure of account vault and storage Sqlite tables ([#1128](https://github.com/0xMiden/miden-client/pull/1128)).
* Added a `NoteScript` getter for the Web Client `Note` model ([#1135](https://github.com/0xMiden/miden-client/pull/1135/)).
* Account related records are now directly stored as Uint8Arrays instead of using Blobs, this fixes a bug with Webkit-based browsers [(#1137)](https://github.com/0xMiden/miden-client/pull/1137).
* [BREAKING] Fixed `createP2IDNote` and `createP2IDENote` convenience functions in the Web Client ([#1142](https://github.com/0xMiden/miden-client/pull/1142)).
* Store changes after transaction execution no longer require fetching the whole account state ([#1147](https://github.com/0xMiden/miden-client/pull/1147)).
* [BREAKING] Use typescript for web_store files: transactions.js & sync.js; add some utils to avoid error-related boilerplate [(#1151)](https://github.com/0xMiden/miden-client/pull/1151). Breaking change: `upsertTransactionRecord` has changed the order of its parameters.
* [BREAKING] Renamed `export/importNote` to `export/importNoteFile`, expose serialization functions for `Note` in Web Client ([#1159](https://github.com/0xMiden/miden-client/pull/1159)).
* Reexported utils to parse token amounts as base units ([#1161](https://github.com/0xMiden/miden-client/pull/1161)).
* Every JS file under `rust-client's` `web store` is now using Typescript ([#1171](https://github.com/0xMiden/miden-client/pull/1171)).
* [BREAKING] The WASM import has been changed into an async function to avoid issues with top-level awaits and some vite projects. ([#1172])(<https://github.com/0xMiden/miden-client/pull/1172>).
* Tracked creation and committed timestamps for `TransactionRecord` ([#1173](https://github.com/0xMiden/miden-client/pull/1173)).
* [BREAKING] Removed `AccountId` to bech32 conversions and the `get_account_state_delta` RPC endpoint  ([#1177](https://github.com/0xMiden/miden-client/pull/1177)).
* [BREAKING] Changed `exportNoteFile` to fail fast on invalid export type ([#1198](https://github.com/0xMiden/miden-client/pull/1198)).
* [BREAKING] Refactored RPC errors ([#1202](https://github.com/0xMiden/miden-client/pull/1202)).
* Accounts are now retrieved partially when reading transaction inputs ([#1438](https://github.com/0xMiden/miden-client/pull/1438)).

## 0.10.2 (2025-08-04)

### Fixes

* Added `AuthScheme::NoAuth` support to `Client` (#1123).

## 0.10.1 (2025-07-26)

* Avoid passing unneeded nodes to `PartialMmr::from_parts` (#1081).

## 0.10.0 (2025-07-12)

### Features

* Added support for FPI in Web Client (#958).
* Exposed `bech32` account IDs in Web Client (#978).
* Added transaction script argument support to `TransactionRequest` (#1017).
* [BREAKING] Added support for timelock P2IDE notes (#1020).

### Changes

* Replaced deprecated #[clap(...)] with #[command(...)] and #[arg(...)] (#897).
* [BREAKING] Renamed `miden-cli` crate to `miden-client-cli`, and the `miden` executable to `miden-client` (#960).
* [BREAKING] Merged `concurrent` feature with `std` (#974).
* [BREAKING] Changed `TransactionRequest` to use expected output recipients instead of output notes (#976).
* [BREAKING] Removed `TransactionExecutor` from `Client` and `NoteScreener` (#998).
* Enforced input note order in `TransactionRequest` (#1001).
* Added check for duplicate input notes in `TransactionRequest` (#1001).
* [BREAKING] Renamed P2IDR to P2IDE (#1016).
* [BREAKING] Removed `with_` prefix from builder functions (#1018).
* Added a way to instantiate a `ScriptBuilder` from `Client` (#1022).
* [BREAKING] Removed `relevant_notes` from `TransactionResult` (#1030).
* Changed sync to store notes regardless of consumption checks if it matched a tracked tag (#1031).

### Fixes

* Fixed Intermittent Block Header Error During Sync in Web Client (#997).
* Fixed Swap Transaction Request in Web Client (#1002)

## v0.9.4 (2025-07-02)

* Support Operations From Counter Contract FPI Example in Web Client (#958).

## v0.9.3 (2025-06-28)

* Fixed a bug where some partial MMR nodes were missing and causing problems with note consumption (#995).

## 0.9.2 (2025-06-11)

* Refresh dependencies (#972).

### Features

* Added necessary methods to support network transactions in the Web Client (#955).

### Changes

* Fixed wasm-opt options to improve performance of generated wasm (#961).

### Fixes

* Fixed bug where network accounts were not being updated correctly in the client (#955).

## 0.9.0 (2025-05-30)

### Features

* Added support for `bech32` account IDs in the CLI (#840).
* Added support for MASM account component libraries in Web Client (#900).
* Added support for RPC client/server version matching through HTTP ACCEPT header (#912).
* Added a way to ignore invalid input notes when consuming them in a transaction (#898).
* Added `NoteUpdate` type to the note update tracker to distinguish between different types of updates (#821).
* Updated `TonicRpcClient` and `Store` traits to be subtraits of `Send` and `Sync` (#926).
* Updated `TonicRpcClient` and `Store` trait functions to return futures which are `Send` (#926).

### Changes

* Updated Web Client README and Documentation (#808).
* [BREAKING] Removed `script_roots` mod in favor of `WellKnownNote` (#834).
* Made non-default options lowercase when prompting for transaction confirmation (#843)
* [BREAKING] Updated keystore to accept arbitrarily large public keys (#833).
* Added Examples to Mdbook for Web Client (#850).
* Added account code to `miden account --show` command (#835).
* Changed exec's input file format to TOML instead of JSON (#870).
* [BREAKING] Client's methods renamed after `PartialMmr` change to `PartialBlockchain` (#894).
* [BREAKING] Made the maximum number of blocks the client can be behind the network customizable (#895).
* Improved Web Client Publishing Flow on Next Branch (#906).
* [BREAKING] Refactored `TransactionRequestBuilder` preset builders (#901).
* Improved the consumability check of the `NoteScreener` (#898).
* Exposed new test utilities in the `testing` feature (#882).
* [BREAKING] Added `tx_graceful_blocks` to `Client` constructor and refactored `TransactionRecord` (#848).
* [BREAKING] Updated the client so that only relevant block headers are stored (#828).
* [BREAKING] Added `DiscardCause` for transactions (#853).
* Chained pending transactions get discarded when one of the transactions in the chain is discarded (#889).
* [BREAKING] Renamed `NetworkNote` and `AccountDetails` to `FetchedNote` and `FetchedAccount` respectively (#931).
* Fixed wasm-opt options to improve performance of generated wasm. wasm-opt settings were broken before.

## 0.8.2 (TBD)

* Converted Web Client `NoteType` class to `enum` (#831)
* Exported `import_account_by_id` function to Web Client (#858)
* Fixed duplicate key bug in `import_account` (#899)

## 0.8.1 (2025-03-28)

### Features

* Added wallet generation from seed & import from seed on web SDK (#710).
* [BREAKING] Generalized `miden new-account` CLI command (#728).
* Added support to import public accounts to `Client` (#733).
* Added import/export for web client db (#740).
* Added `ClientBuilder` for client initialization (#741).
* [BREAKING] Merged `TonicRpcClient` with `WebTonicRpcClient` and added missing endpoints (#744).
* Added support for script execution in the `Client` and CLI (#777).
* Added note code to `miden notes --show` command (#790).
* Added Delegated Proving Support to All Transaction Types in Web Client (#792).

### Changes

* Added check for empty pay to ID notes (#714).
* [BREAKING] Refactored authentication out of the `Client` and added new separate authenticators (#718).
* Added `ClientBuilder` for client initialization (#741).
* [BREAKING] Removed `KeyStore` trait and added ability to provide signatures to `FilesystemKeyStore` and `WebKeyStore` (#744).
* Moved error handling to the `TransactionRequestBuilder::build()` (#750).
* Re-exported `RemoteTransactionProver` in `rust-client` (#752).
* [BREAKING] Added starting block number parameter to `CheckNullifiersByPrefix` and removed nullifiers from `SyncState` (#758).
* Added recency validations for the client (#776).
* [BREAKING] Updated client to Rust 2024 edition (#778).
* [BREAKING] Removed the `TransactionScriptBuilder` and associated errors from the `rust-client` (#781).
* [BREAKING] Renamed "hash" with "commitment" for block headers, note scripts and accounts (#788, #789).
* [BREAKING] Removed `Rng` generic from `Client` and added support for different keystores and RNGs in `ClientBuilder`  (#782).
* Web client: Exposed `assets` iterator for `AssetVault` (#783)
* Updated protobuf bindings generation to use `miden-node-proto-build` crate (#807).

### Fixes

* [BREAKING] Changed Snake Case Variables to Camel Case in JS/TS Files (#767).
* Fixed Web Keystore (#779).
* Fixed case where the `CheckNullifiersByPrefix` response contained nullifiers after the client's sync height (#784).

## 0.7.2 (2025-03-05) -  `miden-client-web` and `miden-client` crates

### Changes

* [BREAKING] Added initial Web Workers implementation to web client (#720, #743).
* Web client: Exposed `InputNotes` iterator and `assets` getter (#757).
* Web client: Exported `TransactionResult` in typings (#768).
* Implemented serialization and deserialization for `SyncSummary` (#725).

### Fixes

* Web client: Fixed submit transaction; Typescript types now match underlying Client call (#760).

## 0.7.0 (2025-01-28)

### Features

* [BREAKING] Implemented support for overwriting of accounts when importing (#612).
* [BREAKING] Added `AccountRecord` with information about the account's status (#600).
* [BREAKING] Added `TransactionRequestBuilder` for building `TransactionRequest` (#605).
* Added caching for foreign account code (#597).
* Added support for unauthenticated notes consumption in the CLI (#609).
* [BREAKING] Added foreign procedure invocation support for private accounts (#619).
* [BREAKING] Added support for specifying map storage slots for FPI (#645)
* Limited the number of decimals that an asset can have (#666).
* [BREAKING] Removed the `testing` feature from the CLI (#670).
* Added per transaction prover support to the web client (#674).
* [BREAKING] Added `BlockNumber` structure (#677).
* Created functions for creating standard notes and note scripts easily on the web client (#686).
* [BREAKING] Renamed plural modules to singular (#687).
* [BREAKING] Made `idxdb` only usable on WASM targets (#685).
* Added fixed seed option for web client generation (#688).
* [BREAKING] Updated `init` command in the CLI to receive a `--network` flag (#690).
* Improved CLI error messages (#682).
* [BREAKING] Renamed APIs for retrieving account information to use the `try_get_*` naming convention, and added/improved module documentation (#683).
* Enabled TLS on tonic client (#697).
* Added account creation from component templates (#680).
* Added serialization for `TransactionResult` (#704).

### Fixes

* Print MASM debug logs when executing transactions (#661).
* Web Store Minor Logging and Error Handling Improvements (#656).
* Web Store InsertChainMmrNodes Duplicate Ids Causes Error (#627).
* Fixed client bugs where some note metadata was not being updated (#625).
* Added Sync Loop to Integration Tests for Small Speedup (#590).
* Added Serial Num Parameter to Note Recipient Constructor in the Web Client (#671).

### Changes

* [BREAKING] Refactored the sync process to use a new `SyncState` component (#650).
* [BREAKING] Return `None` instead of `Err` when an entity is not found (#632).
* Add support for notes without assets in transaction requests (#654).
* Refactored RPC functions and structs to improve code quality (#616).
* [BREAKING] Added support for new two `Felt` account ID (#639).
* [BREAKING] Removed unnecessary methods from `Client` (#631).
* [BREAKING] Use `thiserror` 2.0 to derive errors (#623).
* [BREAKING] Moved structs from `miden-client::rpc` to `miden-client::rpc::domain::*` and changed prost-generated code location (#608, #610, #615).
* Refactored `Client::import_note` to return an error when the note is already being processed (#602).
* [BREAKING] Added per transaction prover support to the client (#599).
* [BREAKING] Removed unused dependencies (#584).

## 0.6.0 (2024-11-08)

### Features

* Added FPI (Foreign Procedure Invocation) support for `TransactionRequest` (#560).
* [BREAKING] Added transaction prover component to `Client` (#550).
* Added WASM consumable notes API + improved note models (#561).
* Added remote prover support to the web client with CI tests (#562).
* Added delegated proving for web client + improved note models (#566).
* Enabled setting expiration delta for `TransactionRequest` (#553).
* Implemented `GetAccountProof` endpoint (#556).
* [BREAKING] Added support for committed and discarded transactions (#531).
* [BREAKING] Added note tags for future notes in `TransactionRequest` (#538).
* Added support for multiple input note inserts at once (#538).
* Added support for custom transactions in web client (#519).
* Added support for remote proving in the CLI (#552).
* Added Transaction Integration Tests for Web Client (#569).
* Added WASM Input note tests + updated input note models (#554)
* Added Account Integration Tests for Web Client (#532).

### Fixes

* Fixed WASM + added additional WASM models (#548).
* [BREAKING] Added IDs to `SyncSummary` fields (#513).
* Added better error handling for WASM sync state (#558).
* Fixed Broken WASM (#519).
* [BREAKING] Refactored Client struct to use trait objects for inner struct fields (#539).
* Fixed panic on export command without type (#537).

### Changes

* Moved note update logic outside of the `Store` (#559).
* [BREAKING] Refactored the `Store` structure and interface for input notes (#520).
* [BREAKING] Replaced `maybe_await` from `Client` and `Store` with `async`, removed `async` feature (#565, #570).
* [BREAKING] Refactored `OutputNoteRecord` to use states and transitions for updates (#551).
* Rebuilt WASM with latest dependencies (#575).
* [BREAKING] Removed serde's de/serialization from `NoteRecordDetails` and `NoteStatus` (#514).
* Added new variants for the `NoteFilter` struct (#538).
* [BREAKING] Re-exported `TransactionRequest` from submodule, renamed `AccountDetails::Offchain` to `AccountDetails::Private`, renamed `NoteDetails::OffChain` to `NoteDetails::Private` (#508).
* Expose full SyncSummary from WASM (#555).
* [BREAKING] Changed `PaymentTransactionData` and `TransactionRequest` to allow for multiple assets per note (#525).
* Added dedicated separate table for tracked tags (#535).
* [BREAKING] Renamed `off-chain` and `on-chain` to `private` and `public` respectively for the account storage modes (#516).

## v0.5.0 (2024-08-27)

### Features

* Added support for decimal values in the CLI (#454).
* Added serialization for `TransactionRequest` (#471).
* Added support for importing committed notes from older blocks than current (#472).
* Added support for account export in the CLI (#479).
* Added the Web Client Crate (#437)
* Added testing suite for the Web Client Crate (#498)
* Fixed typing for the Web Client Crate (#521)
* [BREAKING] Refactored `TransactionRequest` to represent a generalized transaction (#438).

### Enhancements

* Added conversions for `NoteRecordDetails` (#392).
* Ignored stale updates received during sync process (#412).
* Changed `TransactionRequest` to use `AdviceInputs` instead of `AdviceMap` (#436).
* Tracked token symbols with config file (#441).
* Added validations in transaction requests (#447).
* [BREAKING] Track expected block height for notes (#448).
* Added validation for consumed notes when importing (#449).
* [BREAKING] Removed `TransactionTemplate` and `account_id` from `TransactionRequest` (#478).

### Changes

* Refactor `TransactionRequest` constructor (#434).
* [BREAKING] Refactored `Client` to merge submit_transaction and prove_transaction (#445).
* Change schema and code to to reflect changes to `NoteOrigin` (#463).
* [BREAKING] Updated Rust Client to use the new version of `miden-base` (#492).

### Fixes

* Fixed flaky integration tests (#410).
* Fixed `get_consumable_notes` to consider block header information for consumability (#432).

## v0.4.1 (2024-07-08) - `miden-client` crete only

* Fixed the build script to avoid updating generated files in docs.rs environment (#433).

## v0.4.0 (2024-07-05)

### Features

* [BREAKING] Separated `prove_transaction` from `submit_transaction` in `Client`. (#339)
* Note importing in client now uses the `NoteFile` type (#375).
* Added `wasm` and `async` feature to make the code compatible with WASM-32 target (#378).
* Added WebStore to the miden-client to support WASM-compatible store mechanisms (#401).
* Added WebTonicClient to the miden-client to support WASM-compatible RPC calls (#409).
* [BREAKING] Added unauthenticated notes to `TransactionRequest` and necessary changes to consume unauthenticated notes with the client (#417).
* Added advice map to `TransactionRequest` and updated integration test with example using the advice map to provide more than a single `Word` as `NoteArgs` for a note (#422).
* Made the client `no_std` compatible (#428).

### Enhancements

* Fixed the error message when trying to consume a pending note (now it shows that the transaction is not yet ready to be consumed).
* Added created and consumed note info when printing the transaction summary on the CLI. (#348).
* [BREAKING] Updated CLI commands so assets are now passed as `<AMOUNT>::<FAUCET_ACCOUNT_ID>` (#349).
* Changed `consume-notes` to pick up the default account ID if none is provided, and to consume all notes that are consumable by the ID if no notes are provided to the list. (#350).
* Added integration tests using the CLI (#353).
* Simplified and separated the `notes --list` table (#356).
* Fixed bug when exporting a note into a file (#368).
* Added a new check on account creation / import on the CLI to set the account as the default one if none is set (#372).
* Changed `cargo-make` usage for `make` and `Makefile.toml` for a regular `Makefile` (#359).
* [BREAKING] Library API reorganization (#367).
* New note status added to reflect more possible states (#355).
* Renamed "pending" notes to "expected" notes (#373).
* Implemented retrieval of executed transaction info (id, commit height, account_id) from sync state RPC endpoint (#387).
* Added build script to import Miden node protobuf files to generate types for `tonic_client` and removed `miden-node-proto` dependency (#395).
* [BREAKING] Split cli and client into workspace (#407).
* Moved CLI tests to the `miden-cli` crate (#413).
* Restructured the client crate module organization (#417).

## v0.3.1 (2024-05-22)

* No changes; re-publishing to crates.io to re-build documentation on docs.rs.

## v0.3.0 (2024-05-17)

* Added swap transactions and example flows on integration tests.
* Flatten the CLI subcommand tree.
* Added a mechanism to retrieve MMR data whenever a note created on a past block is imported.
* Changed the way notes are added to the database based on `ExecutedTransaction`.
* Added more feedback information to commands `info`, `notes list`, `notes show`, `account new`, `notes import`, `tx new` and `sync`.
* Add `consumer_account_id` to `InputNoteRecord` with an implementation for sqlite store.
* Renamed the CLI `input-notes` command to `notes`. Now we only export notes that were created on this client as the result of a transaction.
* Added validation using the `NoteScreener` to see if a block has relevant notes.
* Added flags to `init` command for non-interactive environments
* Added an option to verify note existence in the chain before importing.
* Add new store note filter to fetch multiple notes by their id in a single query.
* [BREAKING] `Client::new()` now does not need a `data_store_store` parameter, and `SqliteStore`'s implements interior mutability.
* [BREAKING] The store's `get_input_note` was replaced by `get_input_notes` and a `NoteFilter::Unique` was added.
* Refactored `get_account` to create the account from a single query.
* Added support for using an account as the default for the CLI
* Replace instead of ignore note scripts with when inserting input/output notes with a previously-existing note script root to support adding debug statements.
* Added RPC timeout configuration field
* Add off-chain account support for the tonic client method `get_account_update`.
* Refactored `get_account` to create the account from a single query.
* Admit partial account IDs for the commands that need them.
* Added nextest to be used as test runner.
* Added config file to run integration tests against a remote node.
* Added `CONTRIBUTING.MD` file.
* Renamed `format` command from `Makefile.toml` to `check-format` and added a new `format` command that applies the formatting.
* Added methods to get output notes from client.
* Added a `input-notes list-consumable` command to the CLI.

## 0.2.1 (2024-04-24)

* Added ability to start the client in debug mode (#283).

## 0.2.0 (2024-04-14)

* Added an `init` command to the CLI.
* Added support for on-chain accounts.
* Added support for public notes.
* Added `NoteScreener` struct capable of detecting notes consumable by a client (via heuristics), for storing only relevant notes.
* Added `TransactionRequest` for defining transactions with arbitrary scripts, inputs and outputs and changed the client API to use this definition.
* Added `ClientRng` trait for randomness component within `Client`.
* Refactored integration tests to be run as regular rust tests.
* Normalized note script fields for input note and output note tables in SQLite implementation.
* Added support for P2IDR (pay-to-id with recall) transactions on both the CLI and the lib.
* Removed the `mock-data` command from the CLI.

## 0.1.0 (2024-03-15)

* Initial release.
