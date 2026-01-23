import type {
  WebClient,
  Account,
  AccountHeader,
  AccountId,
  InputNoteRecord,
  ConsumableNoteRecord,
  TransactionId,
  NoteType,
  AccountStorageMode,
} from "@demox-labs/miden-sdk";

// Re-export SDK types for convenience
export type {
  WebClient,
  Account,
  AccountHeader,
  AccountId,
  InputNoteRecord,
  ConsumableNoteRecord,
  TransactionId,
  NoteType,
  AccountStorageMode,
};

// Provider configuration
export interface MidenConfig {
  /** RPC node URL. Defaults to testnet. */
  rpcUrl?: string;
  /** Note transport URL for streaming notes. */
  noteTransportUrl?: string;
  /** Auto-sync interval in milliseconds. Set to 0 to disable. Default: 15000ms */
  autoSyncInterval?: number;
  /** Initial seed for deterministic RNG (must be 32 bytes if provided) */
  seed?: Uint8Array;
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
  getBalance: (faucetId: string) => bigint;
}

export interface AssetBalance {
  faucetId: string;
  amount: bigint;
}

// Notes types
export interface NotesFilter {
  status?: "all" | "consumed" | "committed" | "expected" | "processing";
  accountId?: string;
}

export interface NotesResult {
  notes: InputNoteRecord[];
  consumableNotes: ConsumableNoteRecord[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
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

// Send options
export interface SendOptions {
  /** Sender account ID */
  from: string;
  /** Recipient account ID */
  to: string;
  /** Faucet ID for the asset */
  faucetId: string;
  /** Amount to send */
  amount: bigint;
  /** Note type. Default: private */
  noteType?: "private" | "public" | "encrypted";
  /** Block height after which sender can reclaim note */
  recallHeight?: number;
  /** Block height after which recipient can consume note */
  timelockHeight?: number;
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
  noteType?: "private" | "public" | "encrypted";
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
  noteType?: "private" | "public" | "encrypted";
  /** Note type for payback note. Default: private */
  paybackNoteType?: "private" | "public" | "encrypted";
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
