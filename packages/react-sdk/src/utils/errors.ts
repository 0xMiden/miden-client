export type MidenErrorCode =
  | "WASM_CLASS_MISMATCH"
  | "WASM_POINTER_CONSUMED"
  | "WASM_NOT_INITIALIZED"
  | "WASM_SYNC_REQUIRED"
  | "SEND_BUSY"
  | "OPERATION_BUSY"
  | "UNKNOWN";

export class MidenError extends Error {
  readonly code: MidenErrorCode;
  declare readonly cause?: unknown;

  constructor(
    message: string,
    options?: { cause?: unknown; code?: MidenErrorCode }
  ) {
    super(message);
    this.name = "MidenError";
    this.code = options?.code ?? "UNKNOWN";
    if (options?.cause !== undefined) {
      this.cause = options.cause;
    }
  }
}

interface ErrorPattern {
  test: (msg: string) => boolean;
  code: MidenErrorCode;
  message: string;
}

const ERROR_PATTERNS: ErrorPattern[] = [
  {
    test: (msg) =>
      msg.includes("_assertClass") || msg.includes("expected instance of"),
    code: "WASM_CLASS_MISMATCH",
    message:
      "WASM class identity mismatch. This usually means multiple copies of @miden-sdk/miden-sdk " +
      "are bundled. Ensure your bundler deduplicates the package. " +
      "For Vite: add resolve.dedupe and optimizeDeps.exclude for @miden-sdk/miden-sdk.",
  },
  {
    test: (msg) =>
      msg.includes("null pointer") ||
      msg.includes("already been freed") ||
      msg.includes("dereferencing a null"),
    code: "WASM_POINTER_CONSUMED",
    message:
      "WASM object was already consumed. Some WASM-bound objects can only be passed once — " +
      "if you need to reuse a value, create a fresh instance before each call.",
  },
  {
    test: (msg) =>
      msg.includes("not initialized") ||
      msg.includes("Cannot read properties of null"),
    code: "WASM_NOT_INITIALIZED",
    message:
      "Miden client is not initialized. Ensure you are inside a <MidenProvider> and the client is ready " +
      "before calling SDK methods.",
  },
  {
    test: (msg) =>
      msg.includes("state commitment mismatch") || msg.includes("stale state"),
    code: "WASM_SYNC_REQUIRED",
    message:
      "Account state is stale. Call sync() before executing transactions, or ensure no concurrent " +
      "transactions are running against the same account.",
  },
];

export function wrapWasmError(e: unknown): Error {
  if (e instanceof MidenError) return e;

  const msg = e instanceof Error ? e.message : String(e);
  for (const pattern of ERROR_PATTERNS) {
    if (pattern.test(msg)) {
      return new MidenError(pattern.message, { cause: e, code: pattern.code });
    }
  }

  if (e instanceof Error) return e;
  return new Error(msg);
}
