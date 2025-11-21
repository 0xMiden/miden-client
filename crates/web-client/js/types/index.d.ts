import { WebClient as WasmWebClient } from "./crates/miden_client_web";

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
  OutputNoteArray,
  Package,
  PublicKey,
  Rpo256,
  RpcClient,
  SecretKey,
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
  TransactionId,
  TransactionKernel,
  TransactionProver,
  TransactionRequest,
  TransactionResult,
  TransactionStoreUpdate,
  TransactionRequestBuilder,
  TransactionScript,
  TransactionScriptInputPair,
  TransactionScriptInputPairArray,
  TransactionSummary,
  Word,
} from "./crates/miden_client_web";

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
    seed?: string
  ): Promise<WebClient & WasmWebClient>;

  /** Returns the default transaction prover configured on the client. */
  defaultTransactionProver(): TransactionProver;

  /**
   * Terminates the underlying worker.
   */
  terminate(): void;
}
