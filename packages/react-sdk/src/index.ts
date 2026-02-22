import "./types/augmentations";
import { installAccountBech32 } from "./utils/accountBech32";

installAccountBech32();

// Context and Provider
export {
  MidenProvider,
  useMiden,
  useMidenClient,
} from "./context/MidenProvider";

// Signer Context (for external signer providers)
export { SignerContext, useSigner } from "./context/SignerContext";

// Query Hooks
export { useAccounts } from "./hooks/useAccounts";
export { useAccount } from "./hooks/useAccount";
export { useNotes } from "./hooks/useNotes";
export { useNoteStream } from "./hooks/useNoteStream";
export { useTransactionHistory } from "./hooks/useTransactionHistory";
export { useSyncState } from "./hooks/useSyncState";
export { useAssetMetadata } from "./hooks/useAssetMetadata";

// Mutation Hooks
export { useCreateWallet } from "./hooks/useCreateWallet";
export { useCreateFaucet } from "./hooks/useCreateFaucet";
export { useImportAccount } from "./hooks/useImportAccount";
export { useSend } from "./hooks/useSend";
export { useMultiSend } from "./hooks/useMultiSend";
export { useInternalTransfer } from "./hooks/useInternalTransfer";
export { useWaitForCommit } from "./hooks/useWaitForCommit";
export { useWaitForNotes } from "./hooks/useWaitForNotes";
export { useMint } from "./hooks/useMint";
export { useConsume } from "./hooks/useConsume";
export { useSwap } from "./hooks/useSwap";
export { useTransaction } from "./hooks/useTransaction";
export { useSessionAccount } from "./hooks/useSessionAccount";

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
  TransactionHistoryOptions,
  TransactionHistoryResult,
  TransactionStatus,
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
  WaitForCommitOptions,
  WaitForNotesOptions,
  MintOptions,
  ConsumeOptions,
  SwapOptions,
  ExecuteTransactionOptions,
  TransactionResult,
  // Note stream types
  StreamedNote,
  UseNoteStreamOptions,
  UseNoteStreamReturn,
  // Session account types
  UseSessionAccountOptions,
  UseSessionAccountReturn,
  SessionAccountStep,
  // Signer types (for external signer providers)
  SignCallback,
  SignerAccountType,
  SignerAccountConfig,
  SignerContextValue,
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
  TransactionFilter,
  TransactionRecord,
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
export { normalizeAccountId, accountIdsEqual } from "./utils/accountId";
export {
  readNoteAttachment,
  createNoteAttachment,
} from "./utils/noteAttachment";
export type { NoteAttachmentData } from "./utils/noteAttachment";
export { bytesToBigInt, bigIntToBytes, concatBytes } from "./utils/bytes";
export { MidenError, wrapWasmError } from "./utils/errors";
export type { MidenErrorCode } from "./utils/errors";
export { waitForWalletDetection } from "./utils/walletDetection";
export type { WalletAdapterLike } from "./utils/walletDetection";
export {
  migrateStorage,
  clearMidenStorage,
  createMidenStorage,
} from "./utils/storage";
export type { MigrateStorageOptions } from "./utils/storage";

// Hook result types
export type { UseCreateWalletResult } from "./hooks/useCreateWallet";
export type { UseCreateFaucetResult } from "./hooks/useCreateFaucet";
export type { UseImportAccountResult } from "./hooks/useImportAccount";
export type { UseSendResult } from "./hooks/useSend";
export type { UseMultiSendResult } from "./hooks/useMultiSend";
export type { UseInternalTransferResult } from "./hooks/useInternalTransfer";
export type { UseWaitForCommitResult } from "./hooks/useWaitForCommit";
export type { UseWaitForNotesResult } from "./hooks/useWaitForNotes";
export type { UseMintResult } from "./hooks/useMint";
export type { UseConsumeResult } from "./hooks/useConsume";
export type { UseSwapResult } from "./hooks/useSwap";
export type { UseTransactionResult } from "./hooks/useTransaction";
export type { UseSyncStateResult } from "./hooks/useSyncState";
export type { UseTransactionHistoryResult } from "./hooks/useTransactionHistory";
