// Re-export everything from the WASM module
export * from "./crates/miden_client_web";

// Import types we need for augmentation
import type {
  WebClient as WasmWebClient,
  SyncSummary,
  TransactionProver,
} from "./crates/miden_client_web";

// Import the full namespace for the MidenArrayConstructors type
import type * as WasmExports from "./crates/miden_client_web";

// Export the WASM WebClient type alias for users who need to reference it explicitly
export type { WebClient as WasmWebClient } from "./crates/miden_client_web";

// Callback types for external keystore support
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

type MidenArrayConstructors = {
  [K in keyof typeof WasmExports as K extends `${string}Array`
    ? K
    : never]: (typeof WasmExports)[K];
};

export declare const MidenArrays: MidenArrayConstructors;

/** Payload delivered to {@link WebClient.onStateChanged} listeners. */
export interface StateChangedEvent {
  type: "stateChanged";
  /** The mutating operation that triggered the event. */
  operation?: string;
  /** The store / database name that was mutated. */
  storeName: string;
}

// WebClient wrapper class that uses a worker and forwards missing methods to WASM.
export declare class WebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped WebClient.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @param network - Optional name for the store. Setting this allows multiple clients to be used in the same browser.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    network?: string,
    logLevel?: LogLevel
  ): Promise<WebClient>;

  /**
   * Factory method to create and initialize a new wrapped WebClient with a remote keystore.
   *
   * @param rpcUrl - The RPC URL (optional).
   * @param noteTransportUrl - The note transport URL (optional).
   * @param seed - The seed for the account (optional).
   * @param storeName - Optional name for the store. Setting this allows multiple clients to be used in the same browser.
   * @param getKeyCb - Callback used to retrieve secret keys for a given public key.
   * @param insertKeyCb - Callback used to persist secret keys in the external store.
   * @param signCb - Callback used to create signatures for the provided inputs.
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClientWithExternalKeystore(
    rpcUrl?: string,
    noteTransportUrl?: string,
    seed?: Uint8Array,
    storeName?: string,
    getKeyCb?: GetKeyCallback,
    insertKeyCb?: InsertKeyCallback,
    signCb?: SignCallback,
    logLevel?: LogLevel
  ): Promise<WebClient>;

  /** Returns the default transaction prover configured on the client. */
  defaultTransactionProver(): TransactionProver;

  /**
   * Syncs the client state with the Miden node.
   *
   * This method coordinates concurrent calls using the Web Locks API:
   * - If a sync is already in progress, callers wait and receive the same result
   * - Cross-tab coordination ensures only one sync runs at a time per database
   *
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncState(): Promise<SyncSummary>;

  /**
   * Syncs the client state with the Miden node with an optional timeout.
   *
   * This method coordinates concurrent calls using the Web Locks API:
   * - If a sync is already in progress, callers wait and receive the same result
   * - Cross-tab coordination ensures only one sync runs at a time per database
   * - If a timeout is specified and exceeded, the method throws an error
   *
   * @param timeoutMs - Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncStateWithTimeout(timeoutMs?: number): Promise<SyncSummary>;

  /**
   * Register a listener that fires when another browser tab mutates the same
   * IndexedDB database (cross-tab BroadcastChannel notification, Layer 3).
   *
   * @param callback - Invoked with a {@link StateChangedEvent} payload.
   * @returns An unsubscribe function — call it to remove the listener.
   */
  onStateChanged(callback: (event: StateChangedEvent) => void): () => void;

  /**
   * Terminates the underlying worker and cleans up the BroadcastChannel.
   */
  terminate(): void;
}

// MockWebClient class that extends the WebClient wrapper
export declare class MockWebClient extends WebClient {
  /**
   * Factory method to create and initialize a new wrapped MockWebClient.
   *
   * @param serializedMockChain - Serialized mock chain (optional).
   * @param serializedMockNoteTransportNode - Serialized mock note transport node (optional).
   * @param seed - Seed for account initialization (optional).
   * @returns A promise that resolves to a fully initialized MockWebClient.
   */
  static createClient(
    serializedMockChain?: ArrayBuffer | Uint8Array,
    serializedMockNoteTransportNode?: ArrayBuffer | Uint8Array,
    seed?: Uint8Array,
    logLevel?: LogLevel
  ): Promise<MockWebClient>;

  /** Syncs the mock state and returns the resulting summary. */
  syncState(): Promise<SyncSummary>;

  /**
   * Syncs the client state with the Miden node with an optional timeout.
   *
   * @param timeoutMs - Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.
   * @returns A promise that resolves to a SyncSummary with the sync results.
   */
  syncStateWithTimeout(timeoutMs?: number): Promise<SyncSummary>;
}
