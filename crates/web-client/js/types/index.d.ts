import { WebClient as WasmWebClient } from "./crates/miden_client_web";

export {
  Account,
  AccountBuilder,
  AccountComponent,
  AccountHeader,
  AccountId,
  AccountStorageMode,
  AccountStorageRequirements,
  AccountType,
  AdviceMap,
  Assembler,
  AssemblerUtils,
  AuthSecretKey,
  ConsumableNoteRecord,
  Felt,
  FeltArray,
  ForeignAccount,
  FungibleAsset,
  InputNoteState,
  Library,
  NewSwapTransactionResult,
  Note,
  NoteAndArgs,
  NoteAndArgsArray,
  NoteAssets,
  NoteConsumability,
  NoteExecutionHint,
  NoteExecutionMode,
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
  PublicKey,
  Rpo256,
  SecretKey,
  SerializedAccountHeader,
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
  TransactionResult,
  TransactionRequestBuilder,
  TransactionScript,
  TransactionScriptInputPair,
  TransactionScriptInputPairArray,
  Word,
  WebClient,
} from "./crates/miden_client_web";

// Extend WASM WebClient but override methods that use workers
export declare class WebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped WebClient.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param seed - The seed for the account (optional).
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(rpcUrl?: string, seed?: string): Promise<WebClient>;

  /**
   * Terminates the underlying worker.
   */
  terminate(): void;
}
