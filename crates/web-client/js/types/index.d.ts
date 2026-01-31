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

export interface WebClientConfig {
  rpcUrl?: string;
  noteTransportUrl?: string;
  seed?: Uint8Array;
  storeName?: string;
  getKeyCb?: GetKeyCallback;
  insertKeyCb?: InsertKeyCallback;
  signCb?: SignCallback;
}

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

type MidenArrayConstructors = {
  [K in keyof typeof WasmExports as K extends `${string}Array`
    ? K
    : never]: (typeof WasmExports)[K];
};

export declare const MidenArrays: MidenArrayConstructors;

// WebClient wrapper class that uses a worker and forwards missing methods to WASM.
export declare class WebClient extends WasmWebClient {
  /**
   * Factory method to create and initialize a new wrapped WebClient.
   *
   * @param config - Client configuration (RPC URL, optional note transport URL, optional seed, optional store name).
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClient(
    config?: WebClientConfig
  ): Promise<WebClient>;

  /**
   * Factory method to create and initialize a new wrapped WebClient with a remote keystore.
   *
   * @param config - Client configuration (RPC URL, optional note transport URL, optional seed, optional store name, and callbacks).
   * @returns A promise that resolves to a fully initialized WebClient.
   */
  static createClientWithExternalKeystore(
    config?: WebClientConfig
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
   * Terminates the underlying worker.
   */
  terminate(): void;
}

// MockWebClient class that extends the WebClient wrapper
export declare class MockWebClient extends WebClient {
  /**
   * Factory method to create and initialize a new wrapped MockWebClient.
   *
   * @param config - Client configuration (optional seed + optional serialized mock data).
   * @returns A promise that resolves to a fully initialized MockWebClient.
   */
  static createClient(
    config?: WebClientConfig
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
