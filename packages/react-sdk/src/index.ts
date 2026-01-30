import "./types/augmentations";
import { installAccountBech32 } from "./utils/accountBech32";

installAccountBech32();

// Context and Provider
export {
  MidenProvider,
  useMiden,
  useMidenClient,
} from "./context/MidenProvider";

// Query Hooks
export { useAccounts } from "./hooks/useAccounts";
export { useAccount } from "./hooks/useAccount";
export { useNotes } from "./hooks/useNotes";
export { useSyncState } from "./hooks/useSyncState";
export { useAssetMetadata } from "./hooks/useAssetMetadata";

// Mutation Hooks
export { useCreateWallet } from "./hooks/useCreateWallet";
export { useCreateFaucet } from "./hooks/useCreateFaucet";
export { useImportAccount } from "./hooks/useImportAccount";
export { useSend } from "./hooks/useSend";
export { useMultiSend } from "./hooks/useMultiSend";
export { useInternalTransfer } from "./hooks/useInternalTransfer";
export { useMint } from "./hooks/useMint";
export { useConsume } from "./hooks/useConsume";
export { useSwap } from "./hooks/useSwap";
export { useTransaction } from "./hooks/useTransaction";

// Types
export type {
  MidenConfig,
  RpcUrlConfig,
  ProverConfig,
  ProverUrls,
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
  AssetMetadata,
  NoteAsset,
  NoteSummary,
  CreateWalletOptions,
  CreateFaucetOptions,
  ImportAccountOptions,
  SendOptions,
  MultiSendRecipient,
  MultiSendOptions,
  InternalTransferOptions,
  InternalTransferChainOptions,
  InternalTransferResult,
  MintOptions,
  ConsumeOptions,
  SwapOptions,
  ExecuteTransactionOptions,
  TransactionResult,
} from "./types";

// Re-export SDK types for convenience
export type {
  WebClient,
  Account,
  AccountHeader,
  AccountId,
  AccountFile,
  InputNoteRecord,
  ConsumableNoteRecord,
  TransactionId,
  TransactionRequest,
  NoteType,
  AccountStorageMode,
} from "./types";

// Default configuration values
export { DEFAULTS } from "./types";

// Utilities
export { toBech32AccountId } from "./utils/accountBech32";
export { formatAssetAmount, parseAssetAmount } from "./utils/amounts";
export { getNoteSummary, formatNoteSummary } from "./utils/notes";

// Hook result types
export type { UseCreateWalletResult } from "./hooks/useCreateWallet";
export type { UseCreateFaucetResult } from "./hooks/useCreateFaucet";
export type { UseImportAccountResult } from "./hooks/useImportAccount";
export type { UseSendResult } from "./hooks/useSend";
export type { UseMultiSendResult } from "./hooks/useMultiSend";
export type { UseInternalTransferResult } from "./hooks/useInternalTransfer";
export type { UseMintResult } from "./hooks/useMint";
export type { UseConsumeResult } from "./hooks/useConsume";
export type { UseSwapResult } from "./hooks/useSwap";
export type { UseTransactionResult } from "./hooks/useTransaction";
export type { UseSyncStateResult } from "./hooks/useSyncState";
