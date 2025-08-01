import { Page } from "puppeteer";
import {
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
  Library,
  Note,
  NoteAssets,
  NoteConsumability,
  NoteExecutionHint,
  NoteExecutionMode,
  NoteFilter,
  NoteFilterTypes,
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
  SecretKey,
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
  NoteAndArgs,
  NoteAndArgsArray,
} from "../dist/index";

declare global {
  interface Window {
    client: WebClient;
    remoteProverUrl: string;
    remoteProverInstance: TransactionProver;
    Account: typeof Account;
    AccountBuilder: typeof AccountBuilder;
    AccountComponent: typeof AccountComponent;
    AccountHeader: typeof AccountHeader;
    AccountId: typeof AccountId;
    AccountStorageMode: typeof AccountStorageMode;
    AccountStorageRequirements: typeof AccountStorageRequirements;
    AccountType: typeof AccountType;
    AdviceMap: typeof AdviceMap;
    Assembler: typeof Assembler;
    AssemblerUtils: typeof AssemblerUtils;
    AuthSecretKey: typeof AuthSecretKey;
    ConsumableNoteRecord: typeof ConsumableNoteRecord;
    Felt: typeof Felt;
    FeltArray: typeof FeltArray;
    ForeignAccount: typeof ForeignAccount;
    FungibleAsset: typeof FungibleAsset;
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
    TransactionResult: typeof TransactionResult;
    TransactionRequestBuilder: typeof TransactionRequestBuilder;
    TransactionScript: typeof TransactionScript;
    TransactionScriptInputPair: typeof TransactionScriptInputPair;
    TransactionScriptInputPairArray: typeof TransactionScriptInputPairArray;
    WebClient: typeof WebClient;
    Word: typeof Word;
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

declare module "./mocha.global.setup.mjs" {
  export const testingPage: Page;
}
