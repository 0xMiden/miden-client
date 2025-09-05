import { Page } from "puppeteer";
import { WebClient as WasmWebClient } from "../dist/crates/miden_client_web";
import {
  Account,
  AccountBuilder,
  AccountComponent,
  AccountDelta,
  AccountHeader,
  AccountId,
  AccountStorageMode,
  AccountStorageRequirements,
  AccountType,
  Address,
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
  InputNoteRecord,
  Library,
  Note,
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
  NoteTag,
  NoteType,
  OutputNote,
  OutputNotesArray,
  PublicKey,
  Rpo256,
  RpcClient,
  SecretKey,
  Signature,
  SigningInputs,
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
  NoteAndArgs,
  NoteAndArgsArray,
} from "../dist/index";
import { MockWebClient, WebClient } from "../js";

declare global {
  interface Window {
    client: WebClient & WasmWebClient;
    MockWebClient: typeof MockWebClient;
    remoteProverUrl?: string;
    remoteProverInstance: TransactionProver;
    Account: typeof Account;
    AccountBuilder: typeof AccountBuilder;
    AccountComponent: typeof AccountComponent;
    AccountDelta: typeof AccountDelta;
    AccountStorageDelta: typeof AccountStorageDelta;
    AccountVaultDelta: typeof AccountVaultDelta;
    AccountHeader: typeof AccountHeader;
    AccountId: typeof AccountId;
    AccountStorageDelta: typeof AccountStorageDelta;
    AccountStorageMode: typeof AccountStorageMode;
    AccountStorageRequirements: typeof AccountStorageRequirements;
    AccountType: typeof AccountType;
    AccountVaultDelta: typeof AccountVaultDelta;
    AdviceMap: typeof AdviceMap;
    Assembler: typeof Assembler;
    AssemblerUtils: typeof AssemblerUtils;
    AuthSecretKey: typeof AuthSecretKey;
    BasicFungibleFaucetComponent: typeof BasicFungibleFaucetComponent;
    ConsumableNoteRecord: typeof ConsumableNoteRecord;
    Endpoint: typeof Endpoint;
    Felt: typeof Felt;
    FeltArray: typeof FeltArray;
    ForeignAccount: typeof ForeignAccount;
    FungibleAsset: typeof FungibleAsset;
    FungibleAssetDelta: typeof FungibleAssetDelta;
    InputNoteRecord: typeof InputNoteRecord;
    Library: typeof Library;
    Note: typeof Note;
    NoteAndArgs: typeof NoteAndArgs;
    NoteAndArgsArray: typeof NoteAndArgsArray;
    NoteAssets: typeof NoteAssets;
    NoteConsumability: typeof NoteConsumability;
    NoteExecutionHint: typeof NoteExecutionHint;
    NoteExecutionMode: typeof NoteExecutionMode;
    NoteFilter: typeof NoteFilter;
    NoteFilterTypes: typeof NoteFilterTypes;
    NoteId: typeof NoteId;
    NoteIdAndArgs: typeof NoteIdAndArgs;
    NoteIdAndArgsArray: typeof NoteIdAndArgsArray;
    NoteInputs: typeof NoteInputs;
    NoteMetadata: typeof NoteMetadata;
    NoteRecipient: typeof NoteRecipient;
    NoteScript: typeof NoteScript;
    NoteTag: typeof NoteTag;
    NoteType: typeof NoteType;
    OutputNote: typeof OutputNote;
    OutputNotesArray: typeof OutputNotesArray;
    PublicKey: typeof PublicKey;
    Rpo256: typeof Rpo256;
    SecretKey: typeof SecretKey;
    Signature: typeof Signature;
    SigningInputs: typeof SigningInputs;
    SlotAndKeys: typeof SlotAndKeys;
    SlotAndKeysArray: typeof SlotAndKeysArray;
    StorageMap: typeof StorageMap;
    StorageSlot: typeof StorageSlot;
    TestUtils: typeof TestUtils;
    TokenSymbol: typeof TokenSymbol;
    TransactionFilter: typeof TransactionFilter;
    TransactionKernel: typeof TransactionKernel;
    TransactionProver: typeof TransactionProver;
    TransactionRequest: typeof TransactionRequest;
    TransactionStoreUpdate: typeof TransactionStoreUpdate;
    TransactionRequestBuilder: typeof TransactionRequestBuilder;
    TransactionScript: typeof TransactionScript;
    TransactionScriptInputPair: typeof TransactionScriptInputPair;
    TransactionScriptInputPairArray: typeof TransactionScriptInputPairArray;
    RpcClient: typeof RpcClient;
    WebClient: typeof WebClient;
    Word: typeof Word;
    Address: typeof Address;
    createClient: () => Promise<void>;

    // Add the helpers namespace
    helpers: {
      waitForTransaction: (
        transactionId: string,
        maxWaitTime?: number,
        delayInterval?: number
      ) => Promise<void>;
      waitForBlocks: (amountOfBlocks: number) => Promise<void>;
      refreshClient: (initSeed?: Uint8Array) => Promise<void>;
    };
  }
}

declare module "./playwright.global.setup" {
  export const testingPage: Page;
}
