import type { WebClient as WasmWebClient } from "./crates/miden_client_web";
// Import the full namespace to derive the concrete Miden array constructor map so the
// declaration matches the actual WASM exports rather than a generic ArrayBufferView.
import type * as WasmExports from "./crates/miden_client_web";

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
  NetworkId,
  NetworkType,
  Note,
  NoteAndArgs,
  NoteAndArgsArray,
  NoteAssets,
  NoteConsumability,
  NoteConsumptionStatus,
  NoteDetails,
  NoteDetailsAndTag,
  NoteDetailsAndTagArray,
  NoteExecutionHint,
  NoteExecutionMode,
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
  TestUtils,
  TokenSymbol,
  TransactionArgs,
  TransactionFilter,
  TransactionId,
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
} from "./crates/miden_client_web";

export interface WebClientConfig {
  rpcUrl?: string;
  noteTransportUrl?: string;
  seed?: Uint8Array;
  getKeyCb?: GetKeyCallback;
  insertKeyCb?: InsertKeyCallback;
  signCb?: SignCallback;
}

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
   * @param config - The configuration object.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(
    config: WebClientConfig
  ): Promise<WebClient & WasmWebClient>;

  /**
   * Factory method to create and initialize a new wrapped WebClient with a remote keystore.
   *
   * @param config - The configuration object.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClientWithExternalKeystore(
    config: WebClientConfig
  ): Promise<WebClient & WasmWebClient>;

  /**
   * Terminates the underlying worker.
   */
  terminate(): void;
}

export declare class MockWebClient extends WebClient {
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
  ): Promise<MockWebClient & WasmWebClient>;
  /** Syncs the mock state and returns the resulting summary. */
  syncState(): Promise<SyncSummary>;
}
