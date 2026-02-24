// Re-export everything from the WASM module
export * from "./crates/miden_client_web";

// Import types we need for augmentation
import type {
  WebClient as WasmWebClientBase,
  SyncSummary,
  TransactionProver,
  Account,
  AccountHeader,
  AccountId,
  AccountFile,
  AccountCode,
  AccountStorage,
  AssetVault,
  Word,
  Felt,
  TransactionId,
  TransactionRequest,
  TransactionSummary,
  TransactionRecord,
  InputNoteRecord,
  OutputNoteRecord,
  ConsumableNoteRecord,
  NoteId,
  NoteFile,
  NoteTag,
  Note,
  OutputNote,
  NoteExportFormat,
} from "./crates/miden_client_web";

// Import the full namespace for the MidenArrayConstructors type
import type * as WasmExports from "./crates/miden_client_web";

// ════════════════════════════════════════════════════════════════
// Callback types for external keystore support
// ════════════════════════════════════════════════════════════════

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

// ════════════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════════════

/**
 * User-friendly auth scheme constants for MidenClient options.
 * Use `AuthScheme.Falcon` or `AuthScheme.ECDSA` instead of raw strings.
 */
export declare const AuthScheme: {
  readonly Falcon: "falcon";
  readonly ECDSA: "ecdsa";
};

/**
 * Union of all values in the AuthScheme const.
 */
export type AuthSchemeType = (typeof AuthScheme)[keyof typeof AuthScheme];

/**
 * Union of all values in the AccountType const.
 * Includes string aliases ("MutableWallet", etc.) for the simplified API
 * and numeric variants (1, 2, 3) matching the original WASM enum.
 */
export type AccountType = (typeof AccountType)[keyof typeof AccountType];

/**
 * User-friendly account type constants for the simplified API.
 * Replaces the WASM `AccountType` enum (which has internal names like
 * `RegularAccountUpdatableCode`) with readable string constants.
 */
export declare const AccountType: {
  readonly MutableWallet: "MutableWallet";
  readonly ImmutableWallet: "ImmutableWallet";
  readonly FungibleFaucet: "FungibleFaucet";
  readonly NonFungibleFaucet: 1;
  readonly RegularAccountImmutableCode: 2;
  readonly RegularAccountUpdatableCode: 3;
};

/** Union of valid AccountType string values. */
export type AccountTypeValue =
  | "MutableWallet"
  | "ImmutableWallet"
  | "FungibleFaucet";

// ════════════════════════════════════════════════════════════════
// Client options
// ════════════════════════════════════════════════════════════════

export interface ClientOptions {
  /** RPC endpoint URL. Defaults to testnet RPC. */
  rpcUrl?: string;
  /** Note transport URL (optional). */
  noteTransportUrl?: string;
  /** Auto-creates a remote prover from this URL. */
  proverUrl?: string;
  /** Hashed to 32 bytes via SHA-256. */
  seed?: string | Uint8Array;
  /** Store isolation key. */
  storeName?: string;
  /** Sync state on creation (default: false). */
  autoSync?: boolean;
  /** External keystore callbacks. */
  keystore?: {
    getKey: GetKeyCallback;
    insertKey: InsertKeyCallback;
    sign: SignCallback;
  };
}

// ════════════════════════════════════════════════════════════════
// Shared types
// ════════════════════════════════════════════════════════════════

/**
 * An account reference: hex string, bech32 string, Account, AccountHeader, or AccountId object.
 * All ID fields throughout the SDK accept any of these forms.
 */
export type AccountRef = string | Account | AccountHeader | AccountId;

/** Represents an amount of a specific token (identified by its faucet account). */
export interface Asset {
  /** Token identifier (faucet account ID). */
  token: AccountRef;
  /** Auto-converted to bigint internally. */
  amount: number | bigint;
}

export type NoteVisibility = "public" | "private";

/**
 * A note reference: hex note ID string, NoteId object, InputNoteRecord, or Note object.
 */
export type NoteInput = string | NoteId | Note | InputNoteRecord;

// ════════════════════════════════════════════════════════════════
// Account types
// ════════════════════════════════════════════════════════════════

/** Create a wallet (default) or faucet. Discriminated by `type` field. */
export type CreateAccountOptions = WalletCreateOptions | FaucetCreateOptions;

export interface WalletCreateOptions {
  /** Account type. Defaults to "MutableWallet". Use AccountType enum. */
  type?: "MutableWallet" | "ImmutableWallet";
  storage?: "private" | "public";
  auth?: AuthSchemeType;
  seed?: string | Uint8Array;
}

export interface FaucetCreateOptions {
  type: "FungibleFaucet";
  symbol: string;
  decimals: number;
  maxSupply: number | bigint;
  storage?: "private" | "public";
  auth?: AuthSchemeType;
}

export interface AccountDetails {
  account: Account;
  vault: AssetVault;
  storage: AccountStorage;
  code: AccountCode | null;
  keys: Word[];
}

/** Discriminated union for account import. */
export type ImportAccountInput =
  | string
  | { file: AccountFile }
  | {
      seed: Uint8Array;
      /** Account type. Defaults to "MutableWallet". Use AccountType enum. */
      type?: "MutableWallet" | "ImmutableWallet";
      auth?: AuthSchemeType;
    };

/** Options for accounts.export(). Exists for forward-compatible extensibility. */
export interface ExportAccountOptions {}

// ════════════════════════════════════════════════════════════════
// Transaction types
// ════════════════════════════════════════════════════════════════

export interface TransactionOptions {
  waitForConfirmation?: boolean;
  /**
   * Wall-clock polling timeout in milliseconds for waitFor() (default: 60_000).
   * This is NOT a block height. For block-height-based parameters, see
   * `reclaimAfter` and `timelockUntil` on SendOptions.
   */
  timeout?: number;
  /** Override default prover. */
  prover?: TransactionProver;
}

export interface SendOptions extends TransactionOptions {
  account: AccountRef;
  to: AccountRef;
  token: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
  /** Block height after which the sender can reclaim the note. This is a block number, not wall-clock time. */
  reclaimAfter?: number;
  /** Block height until which the note is timelocked. This is a block number, not wall-clock time. */
  timelockUntil?: number;
}

export interface MintOptions extends TransactionOptions {
  /** Faucet (executing account). */
  account: AccountRef;
  to: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
}

export interface ConsumeOptions extends TransactionOptions {
  account: AccountRef;
  notes: NoteInput | NoteInput[];
}

export interface ConsumeAllOptions extends TransactionOptions {
  account: AccountRef;
  maxNotes?: number;
}

export interface SwapOptions extends TransactionOptions {
  account: AccountRef;
  offer: Asset;
  request: Asset;
  type?: NoteVisibility;
  paybackType?: NoteVisibility;
}

export interface PreviewSendOptions {
  operation: "send";
  account: AccountRef;
  to: AccountRef;
  token: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
  reclaimAfter?: number;
  timelockUntil?: number;
}

export interface PreviewMintOptions {
  operation: "mint";
  account: AccountRef;
  to: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
}

export interface PreviewConsumeOptions {
  operation: "consume";
  account: AccountRef;
  notes: NoteInput | NoteInput[];
}

export interface PreviewSwapOptions {
  operation: "swap";
  account: AccountRef;
  offer: Asset;
  request: Asset;
  type?: NoteVisibility;
  paybackType?: NoteVisibility;
}

export type PreviewOptions =
  | PreviewSendOptions
  | PreviewMintOptions
  | PreviewConsumeOptions
  | PreviewSwapOptions;

/** Status values reported during waitFor polling. */
export type WaitStatus = "pending" | "submitted" | "committed";

export interface WaitOptions {
  /** Wall-clock polling timeout in ms (default: 60_000). Set to 0 to disable timeout and poll indefinitely. */
  timeout?: number;
  /** Polling interval in ms (default: 5_000). */
  interval?: number;
  onProgress?: (status: WaitStatus) => void;
}

/** Result of consumeAll — includes count of remaining notes for pagination. */
export interface ConsumeAllResult {
  txId: TransactionId | null;
  consumed: number;
  remaining: number;
}

/**
 * Discriminated union for transaction queries.
 * Mirrors the underlying WASM TransactionFilter enum.
 */
export type TransactionQuery =
  | { status: "uncommitted" }
  | { ids: string[] }
  | { expiredBefore: number };

// ════════════════════════════════════════════════════════════════
// Note types
// ════════════════════════════════════════════════════════════════

/** Discriminated union for note queries. */
export type NoteQuery =
  | {
      status:
        | "consumed"
        | "committed"
        | "expected"
        | "processing"
        | "unverified";
    }
  | { ids: string[] };

/** Options for standalone note creation utilities. */
export interface NoteOptions {
  from: AccountRef;
  to: AccountRef;
  assets: Asset | Asset[];
  type?: NoteVisibility;
  attachment?: Felt[];
}

export interface P2IDEOptions extends NoteOptions {
  reclaimAfter?: number;
  timelockUntil?: number;
}

export interface ExportNoteOptions {
  /** Export format. Defaults to NoteExportFormat.Full. Use the NoteExportFormat enum. */
  format?: NoteExportFormat;
}

export interface FetchPrivateNotesOptions {
  mode?: "incremental" | "all";
}

export interface SendPrivateOptions {
  noteId: string;
  to: AccountRef;
}

export interface MockOptions {
  seed?: string | Uint8Array;
  serializedMockChain?: Uint8Array;
  serializedNoteTransport?: Uint8Array;
}

/** Versioned store snapshot for backup/restore. */
export interface StoreSnapshot {
  version: number;
  data: unknown;
}

// ════════════════════════════════════════════════════════════════
// Swap tag options
// ════════════════════════════════════════════════════════════════

export interface BuildSwapTagOptions {
  type?: NoteVisibility;
  offer: Asset;
  request: Asset;
}

// ════════════════════════════════════════════════════════════════
// Resource interfaces
// ════════════════════════════════════════════════════════════════

export interface AccountsResource {
  create(options?: CreateAccountOptions): Promise<Account>;
  get(accountId: AccountRef): Promise<Account | null>;
  list(): Promise<AccountHeader[]>;
  getDetails(accountId: AccountRef): Promise<AccountDetails>;
  getBalance(accountId: AccountRef, tokenId: AccountRef): Promise<bigint>;

  import(input: ImportAccountInput): Promise<Account>;
  export(
    accountId: AccountRef,
    options?: ExportAccountOptions
  ): Promise<AccountFile>;

  addAddress(accountId: AccountRef, address: string): Promise<void>;
  removeAddress(accountId: AccountRef, address: string): Promise<void>;
}

export interface TransactionsResource {
  send(options: SendOptions): Promise<TransactionId>;
  mint(options: MintOptions): Promise<TransactionId>;
  consume(options: ConsumeOptions): Promise<TransactionId>;
  swap(options: SwapOptions): Promise<TransactionId>;
  consumeAll(options: ConsumeAllOptions): Promise<ConsumeAllResult>;

  preview(options: PreviewOptions): Promise<TransactionSummary>;

  /**
   * Submit a pre-built TransactionRequest.
   * Note: WASM requires accountId separately, so `account` is the first argument.
   */
  submit(
    account: AccountRef,
    request: TransactionRequest,
    options?: TransactionOptions
  ): Promise<TransactionId>;

  list(query?: TransactionQuery): Promise<TransactionRecord[]>;

  waitFor(txId: string, options?: WaitOptions): Promise<void>;
}

export interface NotesResource {
  list(query?: NoteQuery): Promise<InputNoteRecord[]>;
  get(noteId: string): Promise<InputNoteRecord | null>;

  listSent(query?: NoteQuery): Promise<OutputNoteRecord[]>;

  listAvailable(options: {
    account: AccountRef;
  }): Promise<ConsumableNoteRecord[]>;

  import(noteFile: NoteFile): Promise<NoteId>;
  export(noteId: string, options?: ExportNoteOptions): Promise<NoteFile>;

  fetchPrivate(options?: FetchPrivateNotesOptions): Promise<void>;
  sendPrivate(options: SendPrivateOptions): Promise<void>;
}

export interface TagsResource {
  add(tag: number): Promise<void>;
  remove(tag: number): Promise<void>;
  list(): Promise<number[]>;
}

export interface SettingsResource {
  get<T = unknown>(key: string): Promise<T | null>;
  set(key: string, value: unknown): Promise<void>;
  remove(key: string): Promise<void>;
  listKeys(): Promise<string[]>;
}

// ════════════════════════════════════════════════════════════════
// MidenClient
// ════════════════════════════════════════════════════════════════

export declare class MidenClient {
  /** Creates and initializes a new MidenClient. */
  static create(options?: ClientOptions): Promise<MidenClient>;
  /** Creates a client preconfigured for testnet use. Defaults to autoSync: true. */
  static createTestnet(options?: ClientOptions): Promise<MidenClient>;
  /** Creates a mock client for testing. */
  static createMock(options?: MockOptions): Promise<MidenClient>;

  readonly accounts: AccountsResource;
  readonly transactions: TransactionsResource;
  readonly notes: NotesResource;
  readonly tags: TagsResource;
  readonly settings: SettingsResource;

  /** Syncs the client state with the Miden node. */
  sync(options?: { timeout?: number }): Promise<SyncSummary>;
  /** Returns the current sync height. */
  getSyncHeight(): Promise<number>;
  /** Returns the client-level default prover. */
  readonly defaultProver: TransactionProver | null;
  /** Terminates the underlying Web Worker. After this, all method calls throw. */
  terminate(): void;

  /** Exports the client store as a versioned snapshot. */
  exportStore(): Promise<StoreSnapshot>;
  /** Imports a previously exported store snapshot. */
  importStore(snapshot: StoreSnapshot): Promise<void>;

  /** Advances the mock chain by one block. Only available on mock clients. */
  proveBlock(): void;
  /** Returns true if this client uses a mock chain. */
  usesMockChain(): boolean;
  /** Serializes the mock chain state for snapshot/restore in tests. */
  serializeMockChain(): Uint8Array;
  /** Serializes the mock note transport node state. */
  serializeMockNoteTransportNode(): Uint8Array;

  [Symbol.dispose](): void;
  [Symbol.asyncDispose](): Promise<void>;
}

// ════════════════════════════════════════════════════════════════
// Standalone utilities (tree-shakeable)
// ════════════════════════════════════════════════════════════════

/** Creates a P2ID (Pay-to-ID) note. */
export declare function createP2IDNote(options: NoteOptions): OutputNote;

/** Creates a P2IDE (Pay-to-ID with Expiration) note. */
export declare function createP2IDENote(options: P2IDEOptions): OutputNote;

/** Builds a swap tag for note matching. Returns a NoteTag (use `.asU32()` for the numeric value). */
export declare function buildSwapTag(options: BuildSwapTagOptions): NoteTag;

/** Returns the initialized WASM module. Throws if WASM is unavailable. */
export declare function getWasmOrThrow(): Promise<typeof WasmExports>;

// ════════════════════════════════════════════════════════════════
// Internal exports (not public API — for tests and advanced usage)
// ════════════════════════════════════════════════════════════════

/** @internal Low-level WebClient wrapper. Use MidenClient instead. */
export declare class WasmWebClient extends WasmWebClientBase {
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string
  ): Promise<WasmWebClient>;

  static createClientWithExternalKeystore(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string,
    getKeyCb?: GetKeyCallback,
    insertKeyCb?: InsertKeyCallback,
    signCb?: SignCallback
  ): Promise<WasmWebClient>;

  syncState(): Promise<SyncSummary>;
  syncStateWithTimeout(timeoutMs: number): Promise<SyncSummary>;
  terminate(): void;
}

/** @internal Low-level MockWebClient wrapper. Use MidenClient.createMock() instead. */
export declare class MockWasmWebClient extends WasmWebClient {
  static createClient(
    serializedMockChain?: Uint8Array,
    serializedMockNoteTransportNode?: Uint8Array,
    seed?: Uint8Array
  ): Promise<MockWasmWebClient>;

  proveBlock(): void;
  serializeMockChain(): Uint8Array;
  serializeMockNoteTransportNode(): Uint8Array;
}
