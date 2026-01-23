// Re-export everything from the WASM module
export * from "./crates/miden_client_web";

export type { WebClient as WasmWebClient } from "./crates/miden_client_web";

export {
  Account,
  AccountArray,
  AccountBuilder,
  AccountBuilderResult,
  AccountCode,
  AccountComponent,
  AccountComponentCode,
  AccountDelta,
  AccountFile,
  AccountHeader,
  AccountId,
  AccountIdArray,
  AccountInterface,
  AccountStorage,
  AccountStorageDelta,
  AccountStorageMode,
  AccountStorageRequirements,
  AccountType,
  AccountVaultDelta,
  Address,
  AdviceInputs,
  AdviceMap,
  AssetVault,
  AuthFalcon512RpoMultisigConfig,
  AuthScheme,
  AuthSecretKey,
  BasicFungibleFaucetComponent,
  BlockHeader,
  CommittedNote,
  ConsumableNoteRecord,
  Endpoint,
  ExecutedTransaction,
  Felt,
  FeltArray,
  FetchedAccount,
  FetchedNote,
  FlattenedU8Vec,
  ForeignAccount,
  ForeignAccountArray,
  FungibleAsset,
  FungibleAssetDelta,
  FungibleAssetDeltaItem,
  GetProceduresResultItem,
  InputNote,
  InputNoteRecord,
  InputNoteState,
  InputNotes,
  IntoUnderlyingByteSource,
  IntoUnderlyingSink,
  IntoUnderlyingSource,
  JsAccountUpdate,
  JsStateSyncUpdate,
  JsStorageMapEntry,
  JsStorageSlot,
  JsVaultAsset,
  Library,
  MerklePath,
  NetworkAccountTarget,
  NetworkId,
  NetworkType,
  Note,
  NoteAttachment,
  NoteAttachmentKind,
  NoteAttachmentScheme,
  NoteAndArgs,
  NoteAndArgsArray,
  NoteAssets,
  NoteConsumability,
  NoteConsumptionStatus,
  NoteDetails,
  NoteDetailsAndTag,
  NoteDetailsAndTagArray,
  NoteExecutionHint,
  NoteFile,
  NoteFilter,
  NoteFilterTypes,
  NoteHeader,
  NoteId,
  NoteIdAndArgs,
  NoteIdAndArgsArray,
  NoteInclusionProof,
  NoteInputs,
  NoteLocation,
  NoteMetadata,
  NoteRecipient,
  NoteRecipientArray,
  NoteScript,
  NoteSyncInfo,
  NoteTag,
  NoteType,
  OutputNote,
  OutputNoteArray,
  OutputNotes,
  OutputNotesArray,
  OutputNoteRecord,
  OutputNoteState,
  Package,
  PartialNote,
  ProcedureThreshold,
  Program,
  ProvenTransaction,
  PublicKey,
  RpcClient,
  Rpo256,
  CodeBuilder,
  SecretKey,
  SerializedInputNoteData,
  SerializedOutputNoteData,
  SerializedTransactionData,
  Signature,
  SigningInputs,
  SigningInputsType,
  SlotAndKeys,
  SparseMerklePath,
  StorageMap,
  StorageSlot,
  StorageSlotArray,
  SyncSummary,
  TransactionProver,
  TransactionRecord,
  TransactionRequest,
  TransactionRequestBuilder,
  TransactionResult,
  TransactionScript,
  TransactionScriptInputPair,
  TransactionScriptInputPairArray,
  TransactionStatus,
  TransactionStoreUpdate,
  TransactionSummary,
  Word,
  createAuthFalcon512RpoMultisig,
} from "./crates/miden_client_web";

// Import the full namespace for the MidenArrayConstructors type
import type * as WasmExports from "./crates/miden_client_web";

// Export the WASM WebClient type alias for users who need to reference it explicitly
export type { WebClient as WasmWebClient } from "./crates/miden_client_web";

// Callback types for external keystore support
export type GetKeyCallback = (
  pubKey: Uint8Array
) => Promise<Uint8Array | null | undefined> | Uint8Array | null | undefined;

export type InsertKeyCallback = (
  pubKey: Uint8Array,
  secretKey: Uint8Array
) => Promise<void> | void;

export type SignCallback = (
  pubKey: Uint8Array,
  signingInputs: Uint8Array
) => Promise<Uint8Array> | Uint8Array;

type MidenArrayConstructors = {
  [K in keyof typeof WasmExports as K extends `${string}Array`
    ? K
    : never]: (typeof WasmExports)[K];
};

export declare const MidenArrays: MidenArrayConstructors;

// Extend WASM WebClient but override methods that use workers
export declare class WebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped WebClient.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @param network - Optional name for the store (optional).
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    network?: string
  ): Promise<WebClient & WasmWebClient>;

  /**
   * Factory method to create and initialize a new wrapped WebClient with a remote keystore.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @param storeName - Optional name for the store (optional).
   * @param getKeyCb - Callback used to retrieve secret keys for a given public key.
   * @param insertKeyCb - Callback used to persist secret keys in the external store.
   * @param signCb - Callback used to create signatures for the provided inputs.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClientWithExternalKeystore(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string,
    getKeyCb?: GetKeyCallback,
    insertKeyCb?: InsertKeyCallback,
    signCb?: SignCallback
  ): Promise<WebClient & WasmWebClient>;

  /** Returns the default transaction prover configured on the client. */
  defaultTransactionProver(): TransactionProver;

  /**
   * Syncs the client state with the Miden node.
   *
   * This method coordinates concurrent calls using the Web Locks API:
   * - If a sync is already in progress, callers wait and receive the same result
   * - Cross-tab coordination ensures only one sync runs at a time per database
   *
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncState(): Promise<SyncSummary>;

  /**
   * Syncs the client state with the Miden node with an optional timeout.
   *
   * This method coordinates concurrent calls using the Web Locks API:
   * - If a sync is already in progress, callers wait and receive the same result
   * - Cross-tab coordination ensures only one sync runs at a time per database
   * - If a timeout is specified and exceeded, the method throws an error
   *
   * @param timeoutMs - Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncStateWithTimeout(timeoutMs?: number): Promise<SyncSummary>;

  /**
   * Terminates the underlying worker.
   */
  terminate(): void;
}

// MockWebClient class that extends the augmented WebClient
export declare class MockWebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped MockWebClient.
   *
   * @param serializedMockChain - Serialized mock chain (optional).
   * @param serializedMockNoteTransportNode - Serialized mock note transport node (optional).
   * @param seed - Seed for account initialization (optional).
   * @returns A promise that resolves to a fully initialized MockWebClient.
   */
  static createClient(
    serializedMockChain?: ArrayBuffer | Uint8Array,
    serializedMockNoteTransportNode?: ArrayBuffer | Uint8Array,
    seed?: Uint8Array
  ): Promise<MockWebClient>;

  /** Syncs the mock state and returns the resulting summary. */
  syncState(): Promise<SyncSummary>;

  /**
   * Syncs the client state with the Miden node with an optional timeout.
   *
   * @param timeoutMs - Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncStateWithTimeout(timeoutMs?: number): Promise<SyncSummary>;
}
