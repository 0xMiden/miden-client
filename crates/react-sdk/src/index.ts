// Context and Provider
export { MidenProvider, useMiden, useMidenClient } from "./context/MidenProvider";

// Query Hooks
export { useAccounts } from "./hooks/useAccounts";
export { useAccount } from "./hooks/useAccount";
export { useNotes } from "./hooks/useNotes";
export { useSyncState } from "./hooks/useSyncState";

// Mutation Hooks
export { useCreateWallet } from "./hooks/useCreateWallet";
export { useCreateFaucet } from "./hooks/useCreateFaucet";
export { useSend } from "./hooks/useSend";
export { useMint } from "./hooks/useMint";
export { useConsume } from "./hooks/useConsume";
export { useSwap } from "./hooks/useSwap";

// Types
export type {
  MidenConfig,
  MidenState,
  TransactionStage,
  QueryResult,
  MutationResult,
  SyncState,
  AccountsResult,
  AccountResult,
  AssetBalance,
  NotesFilter,
  NotesResult,
  CreateWalletOptions,
  CreateFaucetOptions,
  SendOptions,
  MintOptions,
  ConsumeOptions,
  SwapOptions,
  TransactionResult,
} from "./types";

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
} from "./types";

// Default configuration values
export { DEFAULTS } from "./types";

// Hook result types
export type { UseCreateWalletResult } from "./hooks/useCreateWallet";
export type { UseCreateFaucetResult } from "./hooks/useCreateFaucet";
export type { UseSendResult } from "./hooks/useSend";
export type { UseMintResult } from "./hooks/useMint";
export type { UseConsumeResult } from "./hooks/useConsume";
export type { UseSwapResult } from "./hooks/useSwap";
export type { UseSyncStateResult } from "./hooks/useSyncState";
