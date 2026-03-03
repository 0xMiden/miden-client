/**
 * TypeDoc entry point — curated subset of the public API.
 * Only types listed here (or transitively referenced) appear in generated docs.
 * Runtime consumers should import from index.d.ts, not this file.
 */

// Curated WASM re-exports: only types referenced in the public API
export {
  Account,
  AccountCode,
  AccountFile,
  AccountHeader,
  AccountId,
  AccountStorage,
  AssetVault,
  ConsumableNoteRecord,
  Felt,
  InputNoteRecord,
  Note,
  NoteExportFormat,
  NoteFile,
  NoteId,
  NoteTag,
  OutputNote,
  OutputNoteRecord,
  SyncSummary,
  TransactionId,
  TransactionProver,
  TransactionRecord,
  TransactionRequest,
  TransactionSummary,
  Word,
} from "./crates/miden_client_web";

// All simplified API types
export * from "./api-types";
