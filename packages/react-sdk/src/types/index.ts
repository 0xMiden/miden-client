import type {
  WasmWebClient as WebClient,
  Account,
  AccountHeader,
  AccountId,
  AccountFile,
  InputNoteRecord,
  ConsumableNoteRecord,
  TransactionFilter,
  TransactionId,
  TransactionRecord,
  TransactionRequest,
  NoteType,
  AccountStorageMode,
} from "@miden-sdk/miden-sdk";

// Re-export SDK types for convenience
export type {
  WebClient,
  Account,
  AccountHeader,
  AccountId,
  AccountFile,
  InputNoteRecord,
  ConsumableNoteRecord,
  TransactionFilter,
  TransactionId,
  TransactionRecord,
  TransactionRequest,
  NoteType,
  AccountStorageMode,
};

// Re-export signer types for external signer providers
export type {
  SignCallback,
  SignerAccountType,
  SignerAccountConfig,
  SignerContextValue,
} from "../context/SignerContext";

export type RpcUrlConfig =
  | string
  | "devnet"
  | "testnet"
  | "localhost"
  | "local";

export type ProverConfig =
  | "local"
  | "devnet"
  | "testnet"
  | string
  | {
      url: string;
      timeoutMs?: number | bigint;
    };

export type ProverUrls = {
  devnet?: string;
  testnet?: string;
};

/** Options for passkey-based key encryption (WebAuthn PRF). */
export interface PasskeyEncryptionOptions {
  /** Existing credential ID (base64url). Omit to register a new passkey. */
  credentialId?: string;
  /** WebAuthn relying party ID. Defaults to current hostname. */
  rpId?: string;
  /** Relying party display name. Defaults to "Miden Client". */
  rpName?: string;
  /** User display name for the passkey. Defaults to "Miden Wallet User". */
  userName?: string;
}

// Provider configuration
export interface MidenConfig {
  /** RPC node URL or network name (devnet/testnet/localhost). Defaults to testnet. */
  rpcUrl?: RpcUrlConfig;
  /** Note transport URL for streaming notes. */
  noteTransportUrl?: string;
  /** Auto-sync interval in milliseconds. Set to 0 to disable. Default: 15000ms */
  autoSyncInterval?: number;
  /** Initial seed for deterministic RNG (must be 32 bytes if provided) */
  seed?: Uint8Array;
  /** Transaction prover selection (local/devnet/testnet or a remote URL). */
  prover?: ProverConfig;
  /** Optional override URLs for network provers. */
  proverUrls?: ProverUrls;
  /** Default timeout for remote prover requests in milliseconds. */
  proverTimeoutMs?: number | bigint;
  /** Store isolation key. Recommended when using passkeyEncryption (defaults to "default"). */
  storeName?: string;
  /**
   * Opt-in passkey encryption for keys at rest. Pass `true` for defaults
   * or a `PasskeyEncryptionOptions` object to reuse an existing credential.
   *
   * When `true`, checks localStorage for an existing credential and reuses it
   * if found; otherwise registers a new passkey (triggering a biometric prompt).
   *
   * Requires Chrome 116+, Safari 18+, or Edge 116+.
   * Not compatible with external signer mode — if a SignerContext is active,
   * this option is ignored.
   */
  passkeyEncryption?: boolean | PasskeyEncryptionOptions;
}

// Provider state
export interface MidenState {
  client: WebClient | null;
  isReady: boolean;
  isInitializing: boolean;
  error: Error | null;
}

// Transaction stages for mutation hooks
export type TransactionStage =
  | "idle"
  | "executing"
  | "proving"
  | "submitting"
  | "complete";

// Query hook result pattern
export interface QueryResult<T> {
  data: T | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

// Mutation hook result pattern
export interface MutationResult<TData, TVariables> {
  mutate: (variables: TVariables) => Promise<TData>;
  data: TData | null;
  isLoading: boolean;
  stage: TransactionStage;
  error: Error | null;
  reset: () => void;
}

// Sync state
export interface SyncState {
  syncHeight: number;
  isSyncing: boolean;
  lastSyncTime: number | null;
  error: Error | null;
}

// Account types
export interface AccountsResult {
  accounts: AccountHeader[];
  wallets: AccountHeader[];
  faucets: AccountHeader[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export interface AccountResult {
  account: Account | null;
  assets: AssetBalance[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
  getBalance: (assetId: string) => bigint;
}

export interface AssetBalance {
  assetId: string;
  amount: bigint;
  symbol?: string;
  decimals?: number;
}

// Notes types
export interface NotesFilter {
  status?: "all" | "consumed" | "committed" | "expected" | "processing";
  accountId?: string;
}

export interface NotesResult {
  notes: InputNoteRecord[];
  consumableNotes: ConsumableNoteRecord[];
  noteSummaries: NoteSummary[];
  consumableNoteSummaries: NoteSummary[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export type TransactionStatus = "pending" | "committed" | "discarded";

export interface TransactionHistoryOptions {
  /** Single transaction ID to look up. */
  id?: string | TransactionId;
  /** List of transaction IDs to look up. */
  ids?: Array<string | TransactionId>;
  /** Custom transaction filter (overrides id/ids). */
  filter?: TransactionFilter;
  /** Refresh after provider syncs. Default: true */
  refreshOnSync?: boolean;
}

export interface TransactionHistoryResult {
  records: TransactionRecord[];
  /** Convenience record when a single ID is provided. */
  record: TransactionRecord | null;
  /** Convenience status when a single ID is provided. */
  status: TransactionStatus | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export interface AssetMetadata {
  assetId: string;
  symbol?: string;
  decimals?: number;
}

export interface NoteAsset {
  assetId: string;
  amount: bigint;
  symbol?: string;
  decimals?: number;
}

export interface NoteSummary {
  id: string;
  assets: NoteAsset[];
  sender?: string;
}

// Wallet creation options
export interface CreateWalletOptions {
  /** Storage mode. Default: private */
  storageMode?: "private" | "public" | "network";
  /** Whether code can be updated. Default: true */
  mutable?: boolean;
  /** Auth scheme: 0 = RpoFalcon512, 1 = EcdsaK256Keccak. Default: 0 */
  authScheme?: 0 | 1;
  /** Initial seed for deterministic account ID */
  initSeed?: Uint8Array;
}

// Faucet creation options
export interface CreateFaucetOptions {
  /** Token symbol (e.g., "TEST") */
  tokenSymbol: string;
  /** Number of decimals. Default: 8 */
  decimals?: number;
  /** Maximum supply */
  maxSupply: bigint;
  /** Storage mode. Default: private */
  storageMode?: "private" | "public" | "network";
  /** Auth scheme: 0 = RpoFalcon512, 1 = EcdsaK256Keccak. Default: 0 */
  authScheme?: 0 | 1;
}

// Account import options
export type ImportAccountOptions =
  | {
      type: "file";
      file: AccountFile | Uint8Array | ArrayBuffer;
    }
  | {
      type: "id";
      accountId: string | AccountId;
    }
  | {
      type: "seed";
      seed: Uint8Array;
      mutable?: boolean;
      authScheme?: 0 | 1;
    };

// Send options
export interface SendOptions {
  /** Sender account ID */
  from: string;
  /** Recipient account ID */
  to: string;
  /** Asset ID to send (token id) */
  assetId: string;
  /** Amount to send */
  amount: bigint;
  /** Note type. Default: private */
  noteType?: "private" | "public";
  /** Block height after which sender can reclaim note */
  recallHeight?: number;
  /** Block height after which recipient can consume note */
  timelockHeight?: number;
}

export interface MultiSendRecipient {
  /** Recipient account ID */
  to: string;
  /** Amount to send */
  amount: bigint;
}

export interface MultiSendOptions {
  /** Sender account ID */
  from: string;
  /** Asset ID to send (token id) */
  assetId: string;
  /** Recipient list */
  recipients: MultiSendRecipient[];
  /** Note type. Default: private */
  noteType?: "private" | "public";
}

export interface InternalTransferOptions {
  /** Sender account ID */
  from: string;
  /** Recipient account ID */
  to: string;
  /** Asset ID to send (token id) */
  assetId: string;
  /** Amount to transfer */
  amount: bigint;
  /** Note type. Default: private */
  noteType?: "private" | "public";
}

export interface InternalTransferChainOptions {
  /** Initial sender account ID */
  from: string;
  /** Ordered list of recipient account IDs */
  recipients: string[];
  /** Asset ID to send (token id) */
  assetId: string;
  /** Amount to transfer per hop */
  amount: bigint;
  /** Note type. Default: private */
  noteType?: "private" | "public";
}

export interface InternalTransferResult {
  createTransactionId: string;
  consumeTransactionId: string;
  noteId: string;
}

export interface WaitForCommitOptions {
  /** Timeout in milliseconds. Default: 10000 */
  timeoutMs?: number;
  /** Polling interval in milliseconds. Default: 1000 */
  intervalMs?: number;
}

export interface WaitForNotesOptions {
  /** Account ID to check for consumable notes */
  accountId: string;
  /** Minimum number of notes to wait for. Default: 1 */
  minCount?: number;
  /** Timeout in milliseconds. Default: 10000 */
  timeoutMs?: number;
  /** Polling interval in milliseconds. Default: 1000 */
  intervalMs?: number;
}

// Mint options
export interface MintOptions {
  /** Target account to receive minted tokens */
  targetAccountId: string;
  /** Faucet account to mint from */
  faucetId: string;
  /** Amount to mint */
  amount: bigint;
  /** Note type. Default: private */
  noteType?: "private" | "public";
}

// Consume options
export interface ConsumeOptions {
  /** Account ID that will consume the notes */
  accountId: string;
  /** List of note IDs to consume */
  noteIds: string[];
}

// Swap options
export interface SwapOptions {
  /** Account initiating the swap */
  accountId: string;
  /** Faucet ID of the offered asset */
  offeredFaucetId: string;
  /** Amount being offered */
  offeredAmount: bigint;
  /** Faucet ID of the requested asset */
  requestedFaucetId: string;
  /** Amount being requested */
  requestedAmount: bigint;
  /** Note type for swap note. Default: private */
  noteType?: "private" | "public";
  /** Note type for payback note. Default: private */
  paybackNoteType?: "private" | "public";
}

// Arbitrary transaction options
export interface ExecuteTransactionOptions {
  /** Account ID the transaction applies to */
  accountId: string | AccountId;
  /** Transaction request or builder */
  request:
    | TransactionRequest
    | ((client: WebClient) => TransactionRequest | Promise<TransactionRequest>);
}

// Transaction result
export interface TransactionResult {
  transactionId: string;
}

// Default values
export const DEFAULTS = {
  RPC_URL: undefined, // Will use SDK's testnet default
  AUTO_SYNC_INTERVAL: 15000,
  STORAGE_MODE: "private" as const,
  WALLET_MUTABLE: true,
  AUTH_SCHEME: 0 as const,
  NOTE_TYPE: "private" as const,
  FAUCET_DECIMALS: 8,
} as const;
