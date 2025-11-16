import { WebClient as WasmWebClient } from "./crates/miden_client_web";
import type {
  Account,
  AccountId,
  AccountStorageMode,
  ProvenTransaction,
  SyncSummary,
  TransactionId,
  TransactionProver,
  TransactionRequest,
  TransactionResult,
  TransactionStoreUpdate,
} from "./crates/miden_client_web";

export {
  Account,
  AccountBuilder,
  AccountComponent,
  AccountDelta,
  AccountFile,
  AccountHeader,
  AccountId,
  AccountInterface,
  AccountStorageDelta,
  AccountStorageMode,
  AccountStorageRequirements,
  AccountType,
  AccountVaultDelta,
  Address,
  AddressInterface,
  AdviceMap,
  Assembler,
  AssemblerUtils,
  AuthSecretKey,
  BasicFungibleFaucetComponent,
  ConsumableNoteRecord,
  Endpoint,
  Felt,
  FeltArray,
  ForeignAccount,
  FungibleAsset,
  FungibleAssetDelta,
  InputNoteRecord,
  InputNoteState,
  Library,
  MidenArrays,
  NetworkId,
  Note,
  NoteAndArgs,
  NoteAndArgsArray,
  NoteAssets,
  NoteConsumability,
  NoteDetails,
  NoteExecutionHint,
  NoteExecutionMode,
  NoteFile,
  NoteFilter,
  NoteFilterTypes,
  NoteId,
  NoteIdAndArgs,
  NoteIdAndArgsArray,
  NoteInputs,
  NoteMetadata,
  NoteRecipient,
  NoteScript,
  NoteTag,
  NoteType,
  OutputNote,
  OutputNotesArray,
  Package,
  PublicKey,
  Rpo256,
  RpcClient,
  SecretKey,
  TransactionId,
  TransactionResult,
  ProvenTransaction,
  SerializedAccountHeader,
  Signature,
  SigningInputs,
  SigningInputsType,
  SlotAndKeys,
  SlotAndKeysArray,
  StorageMap,
  StorageSlot,
  TestUtils,
  TokenSymbol,
  TransactionFilter,
  TransactionKernel,
  TransactionProver,
  TransactionRequest,
  TransactionStoreUpdate,
  TransactionRequestBuilder,
  TransactionScript,
  TransactionScriptInputPair,
  TransactionScriptInputPairArray,
  Word,
  SyncSummary,
} from "./crates/miden_client_web";

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
) => Promise<Array<number | string>> | Array<number | string>;

// Extend WASM WebClient but override methods that use workers
export declare class WebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped WebClient.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array
  ): Promise<WebClient & WasmWebClient>;

  /**
   * Factory method to create and initialize a new wrapped WebClient with a remote keystore.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @param getKeyCb - Callback used to retrieve secret keys for a given public key.
   * @param insertKeyCb - Callback used to persist secret keys in the external store.
   * @param signCb - Callback used to create signatures for the provided inputs.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClientWithExternalKeystore(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    getKeyCb?: GetKeyCallback,
    insertKeyCb?: InsertKeyCallback,
    signCb?: SignCallback
  ): Promise<WebClient & WasmWebClient>;

  /**
   * Creates a new wallet, optionally leveraging the worker thread for heavy computations.
   */
  newWallet(
    storageMode: AccountStorageMode,
    mutable: boolean,
    seed?: Uint8Array
  ): Promise<Account>;

  /**
   * Creates a new faucet, optionally leveraging the worker thread for heavy computations.
   */
  newFaucet(
    storageMode: AccountStorageMode,
    nonFungible: boolean,
    tokenSymbol: string,
    decimals: number,
    maxSupply: bigint
  ): Promise<Account>;

  /**
   * Executes, proves, submits a transaction, and applies the resulting update.
   */
  submitTransaction(
    transactionResult: TransactionResult,
    prover?: TransactionProver | null
  ): Promise<TransactionStoreUpdate>;

  /**
   * Executes a transaction request without submitting it to the network.
   */
  executeTransaction(
    accountId: AccountId,
    transactionRequest: TransactionRequest
  ): Promise<TransactionResult>;

  /**
   * Executes, proves, submits, and applies a new transaction request.
   */
  submitNewTransaction(
    accountId: AccountId,
    transactionRequest: TransactionRequest
  ): Promise<TransactionId>;

  /**
   * Generates a transaction proof using a worker (when available).
   */
  proveTransaction(
    transactionResult: TransactionResult,
    prover?: TransactionProver | null
  ): Promise<ProvenTransaction>;

  /** Syncs local state with the node (uses worker when possible). */
  syncState(): Promise<SyncSummary>;

  /** Returns the default transaction prover configured on the client. */
  defaultTransactionProver(): TransactionProver;

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
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array
  ): Promise<MockWebClient & WasmWebClient>;

  /** Syncs the mock state and returns the resulting summary. */
  syncState(): Promise<SyncSummary>;
}
