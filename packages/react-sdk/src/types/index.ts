import { AuthScheme } from "@miden-sdk/miden-sdk";
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
  NoteId,
  AccountStorageMode,
  Note,
} from "@miden-sdk/miden-sdk";

/** Flexible account reference — hex string, bech32 string, or WASM AccountId */
export type AccountRef = string | AccountId;

// Re-export SDK types for convenience
export { AuthScheme };
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
  NoteId,
  Note,
  AccountStorageMode,
};

export type { AccountRef } from "../utils/accountParsing";

// Re-export signer types for external signer providers
export type {
  SignCallback,
  GetKeyCallback,
  InsertKeyCallback,
  SignerAccountType,
  SignerAccountConfig,
  SignerContextValue,
} from "../context/SignerContext";

export type StorageMode = "private" | "public" | "network";
export type NoteVisibility = "private" | "public";

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
  accountId?: AccountRef;
  /** Only notes from this sender (any format, normalized internally) */
  sender?: string;
  /** Exclude these note IDs */
  excludeIds?: string[];
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
  storageMode?: StorageMode;
  /** Whether code can be updated. Default: true */
  mutable?: boolean;
  /** Auth scheme. Default: AuthScheme.AuthRpoFalcon512 */
  authScheme?: AuthScheme;
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
  maxSupply: bigint | number;
  /** Storage mode. Default: private */
  storageMode?: StorageMode;
  /** Auth scheme. Default: AuthScheme.AuthRpoFalcon512 */
  authScheme?: AuthScheme;
}

// Account import options
export type ImportAccountOptions =
  | {
      type: "file";
      file: AccountFile | Uint8Array | ArrayBuffer;
    }
  | {
      type: "id";
      accountId: AccountRef;
    }
  | {
      type: "seed";
      seed: Uint8Array;
      mutable?: boolean;
      authScheme?: AuthScheme;
    };

// Send options
export interface SendOptions {
  /** Sender account ID */
  from: AccountRef;
  /** Recipient account ID */
  to: AccountRef;
  /** Asset ID to send (token id) */
  assetId: AccountRef;
  /** Amount to send (ignored when sendAll is true) */
  amount?: bigint | number;
  /** Note type. Default: private */
  noteType?: NoteVisibility;
  /** Block height after which sender can reclaim note */
  recallHeight?: number;
  /** Block height after which recipient can consume note */
  timelockHeight?: number;
  /** Arbitrary data payload attached to the note */
  attachment?: bigint[] | Uint8Array | number[];
  /** Skip auto-sync before send. Default: false */
  skipSync?: boolean;
  /** Send the full balance of this asset. When true, amount is ignored. */
  sendAll?: boolean;
  /** true = build note in JS and return the Note object (e.g. for out-of-band delivery). Default: false */
  returnNote?: boolean;
}

// Send result — txId always set; note is non-null only when returnNote is true
export interface SendResult {
  txId: string;
  note: Note | null;
}

export interface MultiSendRecipient {
  /** Recipient account ID */
  to: AccountRef;
  /** Amount to send */
  amount: bigint | number;
  /** Per-recipient note type override */
  noteType?: "private" | "public";
  /** Per-recipient attachment */
  attachment?: bigint[] | Uint8Array | number[];
}

export interface MultiSendOptions {
  /** Sender account ID */
  from: AccountRef;
  /** Asset ID to send (token id) */
  assetId: AccountRef;
  /** Recipient list */
  recipients: MultiSendRecipient[];
  /** Default note type for all recipients. Default: private */
  noteType?: NoteVisibility;
  /** Skip auto-sync before send. Default: false */
  skipSync?: boolean;
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
  accountId: AccountRef;
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
  targetAccountId: AccountRef;
  /** Faucet account to mint from */
  faucetId: AccountRef;
  /** Amount to mint */
  amount: bigint | number;
  /** Note type. Default: private */
  noteType?: NoteVisibility;
}

// Consume options
export interface ConsumeOptions {
  /** Account ID that will consume the notes */
  accountId: string;
  /** Notes to consume — accepts note IDs (hex strings), NoteId objects, InputNoteRecord, or Note objects */
  notes: (string | NoteId | InputNoteRecord | Note)[];
}

// Swap options
export interface SwapOptions {
  /** Account initiating the swap */
  accountId: AccountRef;
  /** Faucet ID of the offered asset */
  offeredFaucetId: AccountRef;
  /** Amount being offered */
  offeredAmount: bigint | number;
  /** Faucet ID of the requested asset */
  requestedFaucetId: AccountRef;
  /** Amount being requested */
  requestedAmount: bigint | number;
  /** Note type for swap note. Default: private */
  noteType?: NoteVisibility;
  /** Note type for payback note. Default: private */
  paybackNoteType?: NoteVisibility;
}

// Arbitrary transaction options
export interface ExecuteTransactionOptions {
  /** Account ID the transaction applies to */
  accountId: AccountRef;
  /** Transaction request or builder */
  request:
    | TransactionRequest
    | ((client: WebClient) => TransactionRequest | Promise<TransactionRequest>);
  /** Skip auto-sync before transaction. Default: false */
  skipSync?: boolean;
  /**
   * When set, private output notes from this transaction are delivered to the
   * given recipient after the transaction is committed. Accepts any AccountRef
   * form (hex string, bech32, AccountId, Account, AccountHeader).
   */
  privateNoteRecipient?: AccountRef;
}

// Transaction result
export interface TransactionResult {
  transactionId: string;
}

// --- useNoteStream types ---

export interface StreamedNote {
  /** Note ID (hex string) */
  id: string;
  /** Sender account ID (bech32 if available) */
  sender: string;
  /** First fungible asset amount (convenience; 0n if no fungible assets) */
  amount: bigint;
  /** All assets on the note */
  assets: NoteAsset[];
  /** The underlying InputNoteRecord for escape-hatch access */
  record: InputNoteRecord;
  /** Timestamp (ms) when this note was first observed by the SDK */
  firstSeenAt: number;
  /** Pre-decoded attachment values, or null if no attachment */
  attachment: bigint[] | null;
}

export interface UseNoteStreamOptions {
  /** Note status filter. Default: "committed" */
  status?: "all" | "consumed" | "committed" | "expected" | "processing";
  /** Only notes from this sender (any format, normalized internally) */
  sender?: string | null;
  /** Only notes first seen after this timestamp */
  since?: number;
  /** Exclude these note IDs (for cross-phase stale filtering) */
  excludeIds?: Set<string> | string[];
  /** Filter by primary asset amount */
  amountFilter?: (amount: bigint) => boolean;
}

export interface UseNoteStreamReturn {
  /** Notes matching all filter criteria */
  notes: StreamedNote[];
  /** Most recent note (convenience) */
  latest: StreamedNote | null;
  /** Mark a note as handled (excluded from future renders) */
  markHandled: (noteId: string) => void;
  /** Mark all current notes as handled */
  markAllHandled: () => void;
  /** Snapshot current state for passing to next phase */
  snapshot: () => { ids: Set<string>; timestamp: number };
  isLoading: boolean;
  error: Error | null;
}

// --- useSessionAccount types ---

export interface UseSessionAccountOptions {
  /** Callback to fund the session wallet. Receives the session wallet ID. */
  fund: (sessionAccountId: string) => Promise<void>;
  /** Asset ID of the funding token (reserved for future filtering of consumable notes) */
  assetId?: string;
  /** Wallet creation options */
  walletOptions?: {
    storageMode?: "private" | "public";
    mutable?: boolean;
    authScheme?: AuthScheme;
  };
  /** Polling interval for funding note detection (ms). Default: 3000 */
  pollIntervalMs?: number;
  /** Maximum time to wait for funding note (ms). Default: 60000 */
  maxWaitMs?: number;
  /** localStorage key prefix for persistence. Default: "miden-session" */
  storagePrefix?: string;
}

export type SessionAccountStep =
  | "idle"
  | "creating"
  | "funding"
  | "consuming"
  | "ready";

export interface UseSessionAccountReturn {
  /** Start the create->fund->consume flow */
  initialize: () => Promise<void>;
  /** Session wallet ID (bech32), or null if not yet created */
  sessionAccountId: string | null;
  /** Whether the session wallet is funded and ready */
  isReady: boolean;
  /** Current step */
  step: SessionAccountStep;
  /** Error from any step */
  error: Error | null;
  /** Clear all session data and reset */
  reset: () => void;
}

// Default values
export const DEFAULTS = {
  RPC_URL: undefined, // Will use SDK's testnet default
  AUTO_SYNC_INTERVAL: 15000,
  STORAGE_MODE: "private" as const,
  WALLET_MUTABLE: true,
  AUTH_SCHEME: AuthScheme.AuthRpoFalcon512,
  NOTE_TYPE: "private" as const,
  FAUCET_DECIMALS: 8,
} as const;
