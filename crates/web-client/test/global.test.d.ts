import { Page } from "puppeteer";
import { WebClient as WasmWebClient } from "../dist/crates/miden_client_web";
import {
  Account,
  AccountFile,
  AccountBuilder,
  AccountComponent,
  AccountDelta,
  AccountHeader,
  AccountId,
  AccountInterface,
  AccountStorageMode,
  AccountStorageRequirements,
  AccountType,
  Address,
  AddressInterface,
  AdviceMap,
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
  NetworkId,
  Note,
  NoteAssets,
  NoteConsumability,
  NoteExecutionHint,
  NoteExecutionMode,
  NoteFilter,
  NoteFile,
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
  TransactionResult,
  TransactionRequestBuilder,
  TransactionScript,
  TransactionScriptInputPair,
  TransactionScriptInputPairArray,
  TransactionSummary,
  Word,
  NoteAndArgs,
  NoteAndArgsArray,
  MidenArrays,
  ScriptBuilder,
  ScriptBuilderMode,
} from "../dist/index";
import { MockWebClient, WebClient } from "../js";

declare global {
  interface Window {
    client: WebClient & WasmWebClient;
    MockWebClient: typeof MockWebClient;
    remoteProverUrl?: string;
    remoteProverInstance: TransactionProver;
    rpcUrl?: string;
    Account: typeof Account;
    AccountFile: typeof AccountFile;
    AccountBuilder: typeof AccountBuilder;
    AccountComponent: typeof AccountComponent;
    AccountDelta: typeof AccountDelta;
    AccountStorageDelta: typeof AccountStorageDelta;
    AccountVaultDelta: typeof AccountVaultDelta;
    AccountHeader: typeof AccountHeader;
    AccountId: typeof AccountId;
    AccountInterface: typeof AccountInterface;
    AccountStorageDelta: typeof AccountStorageDelta;
    AccountStorageMode: typeof AccountStorageMode;
    AccountStorageRequirements: typeof AccountStorageRequirements;
    AccountType: typeof AccountType;
    AccountVaultDelta: typeof AccountVaultDelta;
    Address: typeof Address;
    AddressInterface: typeof AddressInterface;
    AdviceMap: typeof AdviceMap;
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
    NetworkId: typeof NetworkId;
    Note: typeof Note;
    NoteAndArgs: typeof NoteAndArgs;
    NoteAndArgsArray: typeof NoteAndArgsArray;
    NoteAssets: typeof NoteAssets;
    NoteConsumability: typeof NoteConsumability;
    NoteExecutionHint: typeof NoteExecutionHint;
    NoteExecutionMode: typeof NoteExecutionMode;
    NoteFilter: typeof NoteFilter;
    NoteFile: typeof NoteFile;
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
    TransactionResult: typeof TransactionResult;
    TransactionRequestBuilder: typeof TransactionRequestBuilder;
    TransactionScript: typeof TransactionScript;
    TransactionScriptInputPair: typeof TransactionScriptInputPair;
    TransactionScriptInputPairArray: typeof TransactionScriptInputPairArray;
    TransactionSummary: typeof TransactionSummary;
    RpcClient: typeof RpcClient;
    WebClient: typeof WebClient;
    Word: typeof Word;
    Address: typeof Address;
    MidenArrays: typeof MidenArrays;
    ScriptBuilder: typeof ScriptBuilder;
    ScriptBuilderMode: typeof ScriptBuilderMode;
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
      parseNetworkId: (networkId: string) => NetworkId;
    };
  }
}

declare module "./playwright.global.setup" {
  export const testingPage: Page;
}
