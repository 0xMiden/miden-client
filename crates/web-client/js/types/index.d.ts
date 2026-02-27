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
