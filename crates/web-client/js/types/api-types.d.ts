// Import types needed for type references in the public API
import type {
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
  TransactionResult,
  TransactionSummary,
  TransactionRecord,
  InputNoteRecord,
  OutputNoteRecord,
  NoteId,
  NoteFile,
  NoteTag,
  Note,
  OutputNote,
  NoteExportFormat,
  StorageSlot,
  AccountComponent,
  AuthSecretKey,
  AccountStorageRequirements,
  TransactionScript,
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
 * User-friendly note visibility constants.
 * Use `NoteVisibility.Public` or `NoteVisibility.Private` instead of raw strings.
 */
export declare const NoteVisibility: {
  readonly Public: "public";
  readonly Private: "private";
};

/** Union of valid NoteVisibility string values. */
export type NoteVisibility = "public" | "private";

/**
 * User-friendly storage mode constants.
 * Use `StorageMode.Public`, `StorageMode.Private`, or `StorageMode.Network` instead of raw strings.
 */
export declare const StorageMode: {
  readonly Public: "public";
  readonly Private: "private";
  readonly Network: "network";
};

/** Union of valid StorageMode string values. */
export type StorageMode = "public" | "private" | "network";

/**
 * Union of all values in the AccountType const.
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
  readonly ImmutableContract: "ImmutableContract";
  readonly MutableContract: "MutableContract";
};

/** Union of valid AccountType string values. */
export type AccountTypeValue =
  | "MutableWallet"
  | "ImmutableWallet"
  | "FungibleFaucet"
  | "ImmutableContract"
  | "MutableContract";

// ════════════════════════════════════════════════════════════════
// Client options
// ════════════════════════════════════════════════════════════════

export interface ClientOptions {
  /**
   * RPC endpoint. Accepts shorthands or a raw URL:
   * - `"testnet"` — Miden testnet RPC (`https://rpc.testnet.miden.io`)
   * - `"devnet"` — Miden devnet RPC (`https://rpc.devnet.miden.io`)
   * - `"localhost"` / `"local"` — local node (`http://localhost:57291`)
   * - any other string — treated as a raw RPC endpoint URL
   * Defaults to the SDK testnet RPC if omitted.
   */
  rpcUrl?: "testnet" | "devnet" | "localhost" | "local" | (string & {});
  /** Note transport URL (optional). */
  noteTransportUrl?: string;
  /**
   * Prover to use for transactions. Accepts shorthands or a raw URL:
   * - `"local"` — local (in-browser) prover
   * - `"devnet"` — Miden devnet remote prover
   * - `"testnet"` — Miden testnet remote prover
   * - any other string — treated as a raw remote prover URL
   */
  proverUrl?: "local" | "devnet" | "testnet" | (string & {});
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

/**
 * A note reference: hex note ID string, NoteId object, InputNoteRecord, or Note object.
 */
export type NoteInput = string | NoteId | Note | InputNoteRecord;

// ════════════════════════════════════════════════════════════════
// Account types
// ════════════════════════════════════════════════════════════════

/** Create a wallet, faucet, or contract. Discriminated by `type` field. */
export type CreateAccountOptions =
  | WalletCreateOptions
  | FaucetCreateOptions
  | ContractCreateOptions;

export interface WalletCreateOptions {
  /** Account type. Defaults to "MutableWallet". Use AccountType enum. */
  type?: "MutableWallet" | "ImmutableWallet";
  storage?: StorageMode;
  auth?: AuthSchemeType;
  seed?: string | Uint8Array;
}

export interface FaucetCreateOptions {
  type: "FungibleFaucet";
  symbol: string;
  decimals: number;
  maxSupply: number | bigint;
  storage?: StorageMode;
  auth?: AuthSchemeType;
}

export interface ContractCreateOptions {
  type: "ImmutableContract" | "MutableContract";
  /** Raw 32-byte seed (Uint8Array). Required. */
  seed: Uint8Array;
  /** Auth secret key. Required. */
  auth: AuthSecretKey;
  /** Pre-compiled AccountComponent instances. */
  components?: AccountComponent[];
  /** Storage mode. Defaults to "public" for contracts. */
  storage?: StorageMode;
}

export interface AccountDetails {
  account: Account;
  vault: AssetVault;
  storage: AccountStorage;
  code: AccountCode | null;
  keys: Word[];
}

/**
 * Discriminated union for account import.
 *
 * - `AccountRef` (string, AccountId, Account, AccountHeader) — Import a public account by ID (fetches state from the network).
 * - `{ file: AccountFile }` — Import from a previously exported account file (works for both public and private accounts).
 * - `{ seed, type?, auth? }` — Reconstruct a **public** account from its init seed. **Does not work for private accounts** — use the account file workflow instead.
 */
export type ImportAccountInput =
  | AccountRef
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

export interface SendOptionsDefault extends TransactionOptions {
  account: AccountRef;
  to: AccountRef;
  token: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
  returnNote?: false;
  /** Block height after which the sender can reclaim the note. This is a block number, not wall-clock time. */
  reclaimAfter?: number;
  /** Block height until which the note is timelocked. This is a block number, not wall-clock time. */
  timelockUntil?: number;
}

export interface SendOptionsReturnNote extends TransactionOptions {
  account: AccountRef;
  to: AccountRef;
  token: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
  returnNote: true;
}

/** @deprecated Use SendOptionsDefault or SendOptionsReturnNote instead */
export type SendOptions = SendOptionsDefault | SendOptionsReturnNote;

export interface SendResult {
  txId: TransactionId;
  note: Note | null;
  result: TransactionResult;
}

/** Result of methods that previously returned bare TransactionId. */
export interface TransactionSubmitResult {
  txId: TransactionId;
  result: TransactionResult;
}

export interface MintOptions extends TransactionOptions {
  /** Faucet (executing account). */
  account: AccountRef;
  /** Recipient account. */
  to: AccountRef;
  /** Amount to mint. */
  amount: number | bigint;
  /** Note visibility. Defaults to "public". */
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

export interface ExecuteOptions extends TransactionOptions {
  /** Account executing the custom script. */
  account: AccountRef;
  /** Compiled TransactionScript. */
  script: TransactionScript;
  /** Foreign accounts referenced by the script. */
  foreignAccounts?: (
    | AccountRef
    | { id: AccountRef; storage?: AccountStorageRequirements }
  )[];
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
  result?: TransactionResult;
}

/**
 * Discriminated union for transaction queries.
 * Mirrors the underlying WASM TransactionFilter enum.
 */
export type TransactionQuery =
  | { status: "uncommitted" }
  | { ids: (string | TransactionId)[] }
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
  | { ids: (string | NoteId)[] };

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
  note: NoteInput;
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
  /** Create a new wallet, faucet, or contract account. Defaults to a mutable wallet if no options are provided. */
  create(options?: CreateAccountOptions): Promise<Account>;
  /**
   * Insert a pre-built account into the local store. Useful for external signer
   * integrations that construct accounts via `AccountBuilder` with custom auth commitments.
   *
   * @param account - The account to insert.
   * @param overwrite - Whether to overwrite an existing account with the same ID. Defaults to `false`.
   */
  insert(account: Account, overwrite?: boolean): Promise<void>;
  /** Retrieve an account by ID. Returns `null` if not found in the local store. */
  get(accountId: AccountRef): Promise<Account | null>;
  /** Retrieve an account locally, or import it from the network if not found. */
  getOrImport(accountId: AccountRef): Promise<Account>;
  /** List all accounts in the local store. */
  list(): Promise<AccountHeader[]>;
  /** Retrieve detailed account information including vault, storage, code, and keys. */
  getDetails(accountId: AccountRef): Promise<AccountDetails>;
  /** Get the balance of a specific token for an account. */
  getBalance(accountId: AccountRef, tokenId: AccountRef): Promise<bigint>;

  /** Import an account from the network by ID, from an exported file, or reconstruct from a seed. */
  import(input: ImportAccountInput): Promise<Account>;
  /** Export an account to an {@link AccountFile} for backup or transfer. */
  export(
    accountId: AccountRef,
    options?: ExportAccountOptions
  ): Promise<AccountFile>;

  /** Associate a Bech32 address with an account. */
  addAddress(accountId: AccountRef, address: string): Promise<void>;
  /** Remove a Bech32 address from an account. */
  removeAddress(accountId: AccountRef, address: string): Promise<void>;
}

export interface TransactionsResource {
  /** Send tokens to another account by creating a pay-to-ID note. Set `returnNote: true` to get the created note back. */
  send(
    options: SendOptionsDefault
  ): Promise<{ txId: TransactionId; note: null; result: TransactionResult }>;
  send(
    options: SendOptionsReturnNote
  ): Promise<{ txId: TransactionId; note: Note; result: TransactionResult }>;
  send(options: SendOptions): Promise<SendResult>;
  /** Mint new tokens from a faucet account. */
  mint(options: MintOptions): Promise<TransactionSubmitResult>;
  /** Consume one or more notes for an account. */
  consume(options: ConsumeOptions): Promise<TransactionSubmitResult>;
  /** Execute an atomic swap between two assets. */
  swap(options: SwapOptions): Promise<TransactionSubmitResult>;
  /** Consume all available notes for an account, up to an optional limit. Returns the count of remaining notes. */
  consumeAll(options: ConsumeAllOptions): Promise<ConsumeAllResult>;
  /** Execute a custom transaction script with optional foreign account references. */
  execute(options: ExecuteOptions): Promise<TransactionSubmitResult>;

  /** Dry-run a transaction to preview its effects without submitting it to the network. */
  preview(options: PreviewOptions): Promise<TransactionSummary>;

  /**
   * Submit a pre-built TransactionRequest.
   * Note: WASM requires accountId separately, so `account` is the first argument.
   */
  submit(
    account: AccountRef,
    request: TransactionRequest,
    options?: TransactionOptions
  ): Promise<TransactionSubmitResult>;

  /** List transactions, optionally filtered by status, IDs, or expiration. */
  list(query?: TransactionQuery): Promise<TransactionRecord[]>;

  /** Poll until a transaction is confirmed on-chain. Throws on rejection or timeout. */
  waitFor(txId: string | TransactionId, options?: WaitOptions): Promise<void>;
}

export interface NotesResource {
  /** List received (input) notes, optionally filtered by status or IDs. */
  list(query?: NoteQuery): Promise<InputNoteRecord[]>;
  /** Retrieve a note by ID. Returns `null` if not found. */
  get(noteId: NoteInput): Promise<InputNoteRecord | null>;

  /** List sent (output) notes, optionally filtered by status or IDs. */
  listSent(query?: NoteQuery): Promise<OutputNoteRecord[]>;

  /** List notes that are available for consumption by a specific account. */
  listAvailable(options: { account: AccountRef }): Promise<InputNoteRecord[]>;

  /** Import a note from a {@link NoteFile}. */
  import(noteFile: NoteFile): Promise<NoteId>;
  /** Export a note to a {@link NoteFile} for transfer or backup. */
  export(noteId: NoteInput, options?: ExportNoteOptions): Promise<NoteFile>;

  /** Fetch private notes from the note transport service. */
  fetchPrivate(options?: FetchPrivateNotesOptions): Promise<void>;
  /** Send a private note to a recipient via the note transport service. */
  sendPrivate(options: SendPrivateOptions): Promise<void>;
}

// ════════════════════════════════════════════════════════════════
// Compiler types
// ════════════════════════════════════════════════════════════════

export interface CompileComponentOptions {
  /** MASM source code for the component. */
  code: string;
  /** Initial storage slots for the component. */
  slots?: StorageSlot[];
  /**
   * When true, the component accepts all input types for Falcon-signed
   * transactions by automatically adding `exec.auth::auth_tx_rpo_falcon512`
   * to a library context. Default: true.
   *
   * **BREAKING (v0.12):** This flag was added in v0.12 and defaults to `true`.
   * Set to `false` if you compile a component that already includes its own
   * auth transaction kernel invocation or intentionally omits one.
   */
  supportAllTypes?: boolean;
}

export interface CompileTxScriptLibrary {
  /** MASM namespace for the library (e.g. "counter::module"). */
  namespace: string;
  /** MASM source code for the library. */
  code: string;
  /**
   * `"dynamic"` (default) — procedures are linked via DYNCALL at runtime.
   * `"static"` — procedures are inlined at compile time.
   */
  linking?: "dynamic" | "static";
}

export interface CompileTxScriptOptions {
  /** MASM source code for the transaction script. */
  code: string;
  /** Component libraries to link. */
  libraries?: CompileTxScriptLibrary[];
}

export interface CompilerResource {
  /** Compile MASM source into an AccountComponent. */
  component(options: CompileComponentOptions): Promise<AccountComponent>;
  /** Compile MASM source into a TransactionScript. */
  txScript(options: CompileTxScriptOptions): Promise<TransactionScript>;
}

export interface TagsResource {
  /** Add a note tag to listen for during sync. */
  add(tag: number): Promise<void>;
  /** Remove a note tag so it is no longer tracked during sync. */
  remove(tag: number): Promise<void>;
  /** List all registered note tags. */
  list(): Promise<number[]>;
}

export interface SettingsResource {
  /** Get a setting value by key. Returns `null` if not found. */
  get<T = unknown>(key: string): Promise<T | null>;
  /** Set a setting value. */
  set(key: string, value: unknown): Promise<void>;
  /** Remove a setting. */
  remove(key: string): Promise<void>;
  /** List all setting keys. */
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
  readonly compile: CompilerResource;

  /** Syncs the client state with the Miden node. */
  sync(options?: { timeout?: number }): Promise<SyncSummary>;
  /** Returns the current sync height. */
  getSyncHeight(): Promise<number>;
  /** Returns the client-level default prover. */
  readonly defaultProver: TransactionProver | null;
  /** Terminates the underlying Web Worker. After this, all method calls throw. */
  terminate(): void;

  /** Returns the identifier of the underlying store (e.g. IndexedDB database name, file path). */
  storeIdentifier(): string;

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

/** Exports the entire contents of an IndexedDB store as a JSON string. */
export declare function exportStore(storeName: string): Promise<string>;

/** Imports store contents from a JSON string, replacing all existing data. */
export declare function importStore(
  storeName: string,
  storeDump: string
): Promise<void>;

/** Returns the initialized WASM module. Throws if WASM is unavailable. */
export declare function getWasmOrThrow(): Promise<typeof WasmExports>;
