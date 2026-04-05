// Re-export everything from the WASM module
export * from "./crates/miden_client_web";

// Re-export all simplified API types
export * from "./api-types";

// Import types needed for the @internal class declarations below
import type {
  WebClient as WasmWebClientBase,
  SyncSummary,
} from "./crates/miden_client_web";
import type {
  GetKeyCallback,
  InsertKeyCallback,
  SignCallback,
} from "./api-types";

export type LogLevel =
  | "error"
  | "warn"
  | "info"
  | "debug"
  | "trace"
  | "off"
  | "none";

/**
 * Initializes the tracing subscriber that routes Rust log output to the
 * browser console. Call once per thread (main thread / Web Worker).
 * Subsequent calls on the same thread are harmless no-ops.
 *
 * @param logLevel - The maximum log level to display.
 */
export declare function setupLogging(logLevel: LogLevel): void;

// ════════════════════════════════════════════════════════════════
// StorageView — wraps WASM AccountStorage with smart getItem()
// ════════════════════════════════════════════════════════════════

import type { AccountStorage, Word, Felt } from "./crates/miden_client_web";

/**
 * Result of reading a storage slot via `StorageView.getItem()`.
 * Works for both Value and StorageMap slots.
 */
export declare class StorageResult {
  /** True if this slot is a StorageMap. */
  get isMap(): boolean;

  /**
   * All entries from a StorageMap slot.
   * Each entry has `key` (hex), `value` (hex), and `word` (parsed Word or undefined).
   * Returns undefined for Value slots.
   */
  get entries():
    | Array<{ key: string; value: string; word: Word | undefined }>
    | undefined;

  /** The underlying Word value. */
  get word(): Word | undefined;

  /** The first Felt of the stored Word. */
  felt(): Felt | undefined;

  /**
   * First felt as a JavaScript number.
   * WARNING: May lose precision for values > Number.MAX_SAFE_INTEGER.
   */
  toNumber(): number;

  /** First felt as a BigInt. Preserves full u64 precision. */
  toBigInt(): bigint;

  /** The Word's hex representation. */
  toHex(): string;

  /** Renders as the numeric value. Makes `{result}` work in JSX. */
  toString(): string;

  /** Returns the numeric value. */
  toJSON(): number;

  /** Allows arithmetic: `+result`, `result * 2`. */
  valueOf(): number;
}

/**
 * Wraps WASM AccountStorage with a developer-friendly API.
 *
 * `getItem()` returns a `StorageResult` that works intuitively for both
 * Value and StorageMap slots. The raw WASM AccountStorage is accessible
 * via `.raw`.
 *
 * Installed on `Account.prototype.storage()` at WASM load time.
 */
export declare class StorageView {
  /** The raw WASM AccountStorage. */
  get raw(): AccountStorage;

  /** Returns the commitment to the full account storage. */
  commitment(): Word;

  /** Returns the names of all storage slots. */
  getSlotNames(): string[];

  /**
   * Smart read: returns a `StorageResult` for the given slot.
   * For Value slots: wraps the stored Word.
   * For StorageMap slots: wraps the first entry's value, with all entries in `.entries`.
   */
  getItem(slotName: string): StorageResult | undefined;

  /** Returns the value for a key in a StorageMap slot. */
  getMapItem(slotName: string, key: Word): Word | undefined;

  /** Get all key-value pairs from a StorageMap slot. */
  getMapEntries(
    slotName: string
  ): Array<{ key: string; value: string }> | undefined;

  /**
   * Returns the commitment root of a storage slot.
   * For Value slots: the stored Word. For StorageMap slots: the Merkle root hash.
   * Useful for proofs, state comparison, and syncing.
   */
  getCommitment(slotName: string): Word | undefined;

  /**
   * Convenience: first felt as a JavaScript number.
   * Works for both slot types.
   */
  getNumber(slotName: string): number | undefined;
}

/** Convert a Word's first felt to a BigInt (full u64 precision). */
export declare function wordToBigInt(word: Word): bigint;

/** Convert a Word's first felt to a number (may lose precision > 2^53). */
export declare function wordToNumber(word: Word): number;

// ════════════════════════════════════════════════════════════════
// Internal exports (not public API — for tests and advanced usage)
// ════════════════════════════════════════════════════════════════

/** @internal Low-level WebClient wrapper. Use MidenClient instead. */
export declare class WasmWebClient extends WasmWebClientBase {
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string,
    logLevel?: LogLevel
  ): Promise<WasmWebClient>;

  static createClientWithExternalKeystore(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string,
    getKeyCb?: GetKeyCallback,
    insertKeyCb?: InsertKeyCallback,
    signCb?: SignCallback,
    logLevel?: LogLevel
  ): Promise<WasmWebClient>;

  syncState(): Promise<SyncSummary>;
  syncStateWithTimeout(timeoutMs: number): Promise<SyncSummary>;
  setSignCb(signCb: SignCallback | null | undefined): void;
  onStateChanged(callback: (event: any) => void): (() => void) | undefined;
  terminate(): void;
}

/** @internal Low-level MockWebClient wrapper. Use MidenClient.createMock() instead. */
export declare class MockWasmWebClient extends WasmWebClient {
  static createClient(
    serializedMockChain?: Uint8Array,
    serializedMockNoteTransportNode?: Uint8Array,
    seed?: Uint8Array,
    logLevel?: LogLevel
  ): Promise<MockWasmWebClient>;

  proveBlock(): void;
  serializeMockChain(): Uint8Array;
  serializeMockNoteTransportNode(): Uint8Array;
}
