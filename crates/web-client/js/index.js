import loadWasm from "./wasm.js";
import { CallbackType, MethodName, WorkerAction } from "./constants.js";
import {
  acquireSyncLock,
  releaseSyncLock,
  releaseSyncLockWithError,
} from "./syncLock.js";
import { MidenClient } from "./client.js";
import { CompilerResource } from "./resources/compiler.js";
import {
  createP2IDNote,
  createP2IDENote,
  buildSwapTag,
  _setWasm as _setStandaloneWasm,
  _setWebClient as _setStandaloneWebClient,
} from "./standalone.js";
import { resolveAuthScheme } from "./utils.js";
export { resolveAuthScheme };
export * from "../Cargo.toml";

export const AccountType = Object.freeze({
  // WASM-compatible numeric values — usable with AccountBuilder directly
  FungibleFaucet: 0,
  NonFungibleFaucet: 1,
  RegularAccountImmutableCode: 2,
  RegularAccountUpdatableCode: 3,
  // SDK-friendly aliases (same numeric values as their WASM equivalents)
  MutableWallet: 3,
  ImmutableWallet: 2,
  ImmutableContract: 2,
  MutableContract: 3,
});

export const AuthScheme = Object.freeze({
  Falcon: "falcon",
  ECDSA: "ecdsa",
});

export const NoteVisibility = Object.freeze({
  Public: "public",
  Private: "private",
});

export const StorageMode = Object.freeze({
  Public: "public",
  Private: "private",
  Network: "network",
});

export const Linking = Object.freeze({
  Dynamic: "dynamic",
  Static: "static",
});

export { MidenClient };
export { CompilerResource };
export { createP2IDNote, createP2IDENote, buildSwapTag };

// Internal exports — used by integration tests that need direct access to the low-level WebClient proxy.
export { WebClient as WasmWebClient, MockWebClient as MockWasmWebClient };

// Method classification sets — used by scripts/check-method-classification.js to ensure
// every WASM export is explicitly categorised. Update when adding new WASM methods.
//
// Note on `SYNC_METHODS`: the classifier is "synchronous in JS" — i.e.
// `pub fn ...` in Rust, not `pub async fn ...`. Two sub-cases:
//   1. Factory methods that return a non-Promise value (`accountReader`
//      returns `AccountReader`; the transaction-request builders return
//      `TransactionRequestBuilder`; `createCodeBuilder` returns a builder).
//      Wrapping these in `_serializeWasmCall` would turn their return
//      value into `Promise<T>` and break callers that use the result
//      immediately (e.g. `const reader = client.accountReader(id);
//      await reader.nonce();`).
//   2. Sync methods that still take `&mut self` in Rust (`proveBlock`,
//      `serializeMockChain`, `setDebugMode`). Safe to opt out because JS
//      is single-threaded — the event loop cannot interleave another
//      call during their synchronous execution, so the RefCell borrow
//      is always released before any other borrow can start.
// Do NOT move a sync-in-JS method into `WRITE_METHODS` / `READ_METHODS`
// just because it takes `&mut self` or `&self`; wrapping changes its
// return shape and breaks every caller.
const SYNC_METHODS = new Set([
  "accountReader",
  "buildSwapTag",
  "createCodeBuilder",
  "lastAuthError",
  "newConsumeTransactionRequest",
  "newMintTransactionRequest",
  "newSendTransactionRequest",
  "newSwapTransactionRequest",
  "proveBlock",
  "serializeMockChain",
  "serializeMockNoteTransportNode",
  "setDebugMode",
  "storeIdentifier",
  "usesMockChain",
]);

const WRITE_METHODS = new Set([
  "addTag",
  "executeForSummary",
  "executeProgram",
  "fetchAllPrivateNotes",
  "fetchPrivateNotes",
  "forceImportStore",
  "importAccountById",
  "importAccountFile",
  "importNoteFile",
  "importPublicAccountFromSeed",
  "insertAccountAddress",
  "newAccount",
  "pruneAccountHistory",
  "removeAccountAddress",
  "removeTag",
  "removeSetting",
  "sendPrivateNote",
  "setSetting",
  "submitProvenTransaction",
]);

const READ_METHODS = new Set([
  "exportAccountFile",
  "exportNoteFile",
  "exportStore",
  "getAccount",
  "getAccountCode",
  "getAccountStorage",
  "getAccountVault",
  "getAccounts",
  "getConsumableNotes",
  "getInputNote",
  "getInputNotes",
  "getOutputNote",
  "getOutputNotes",
  "getSetting",
  "getSyncHeight",
  "getTransactions",
  "listSettingKeys",
  "listTags",
  "executeProgram",
]);

const MOCK_STORE_NAME = "mock_client_db";

// `SYNC_METHODS` is read by `createClientProxy` to decide whether a
// proxy-fallback call should stay synchronous. `WRITE_METHODS` /
// `READ_METHODS` are consulted only by the CI lint (see
// scripts/check-method-classification.js); suppress unused-variable
// warnings for those two.
void WRITE_METHODS;
void READ_METHODS;

const buildTypedArraysExport = (exportObject) => {
  return Object.entries(exportObject).reduce(
    (exports, [exportName, _export]) => {
      if (exportName.endsWith("Array")) {
        exports[exportName] = _export;
      }
      return exports;
    },
    {}
  );
};

const deserializeError = (errorLike) => {
  if (!errorLike) {
    return new Error("Unknown error received from worker");
  }
  const { name, message, stack, cause, ...rest } = errorLike;
  const reconstructedError = new Error(message ?? "Unknown worker error");
  reconstructedError.name = name ?? reconstructedError.name;
  if (stack) {
    reconstructedError.stack = stack;
  }
  if (cause) {
    reconstructedError.cause = deserializeError(cause);
  }
  Object.entries(rest).forEach(([key, value]) => {
    if (value !== undefined) {
      reconstructedError[key] = value;
    }
  });
  return reconstructedError;
};

export const MidenArrays = {};

let wasmModule = null;
let wasmLoadPromise = null;
let webClientStaticsCopied = false;

const ensureWasm = async () => {
  if (wasmModule) {
    return wasmModule;
  }
  if (!wasmLoadPromise) {
    wasmLoadPromise = loadWasm().then((module) => {
      wasmModule = module;
      if (module) {
        Object.assign(MidenArrays, buildTypedArraysExport(module));
        if (!webClientStaticsCopied && module.WebClient) {
          copyWebClientStatics(module.WebClient);
          webClientStaticsCopied = true;
        }
        // Set WASM module for standalone utilities
        _setStandaloneWasm(module);
      }
      return module;
    });
  }
  return wasmLoadPromise;
};

export const getWasmOrThrow = async () => {
  const module = await ensureWasm();
  if (!module) {
    throw new Error(
      "Miden WASM bindings are unavailable in this environment (SSR is disabled)."
    );
  }
  return module;
};
/**
 * WebClient is a wrapper around the underlying WASM WebClient object.
 *
 * This wrapper serves several purposes:
 *
 * 1. It creates a dedicated web worker to offload computationally heavy tasks
 *    (such as creating accounts, executing transactions, submitting transactions, etc.)
 *    from the main thread, helping to prevent UI freezes in the browser.
 *
 * 2. It defines methods that mirror the API of the underlying WASM WebClient,
 *    with the intention of executing these functions via the web worker. This allows us
 *    to maintain the same API and parameters while benefiting from asynchronous, worker-based computation.
 *
 * 3. It employs a Proxy to forward any calls not designated for web worker computation
 *    directly to the underlying WASM WebClient instance.
 *
 * Additionally, the wrapper provides a static createClient function. This static method
 * instantiates the WebClient object and ensures that the necessary createClient calls are
 * performed both in the main thread and within the worker thread. This dual initialization
 * correctly passes user parameters (RPC URL and seed) to both the main-thread
 * WASM WebClient and the worker-side instance.
 *
 * Because of this implementation, the only breaking change for end users is in the way the
 * web client is instantiated. Users should now use the WebClient.createClient static call.
 */
/**
 * Create a Proxy that forwards missing properties to the underlying WASM
 * WebClient.
 *
 * Async proxy-fallback methods (every WASM method that borrows the
 * WebClient's RefCell — reads included, since `&self` and `&mut self` both
 * trip wasm-bindgen's "recursive use of an object detected" panic if
 * another borrow is live) are routed through `_serializeWasmCall` so they
 * queue on the same chain as the explicitly-wrapped methods.
 *
 * `SYNC_METHODS` opts out: they are synchronous in JS and wrapping them
 * would change their return type to `Promise<T>`, which is a breaking
 * change for consumers that use them as plain getters or builders.
 */
function createClientProxy(instance) {
  return new Proxy(instance, {
    get(target, prop, receiver) {
      if (prop in target) {
        return Reflect.get(target, prop, receiver);
      }
      if (target.wasmWebClient && prop in target.wasmWebClient) {
        const value = target.wasmWebClient[prop];
        if (typeof value === "function") {
          if (typeof prop === "string" && SYNC_METHODS.has(prop)) {
            return value.bind(target.wasmWebClient);
          }
          return (...args) =>
            target._serializeWasmCall(() =>
              value.apply(target.wasmWebClient, args)
            );
        }
        return value;
      }
      return undefined;
    },
  });
}

class WebClient {
  /**
   * Controls which worker variant is spawned when a WebClient is constructed.
   *
   * - `"auto"` (default): pick `classic` on Safari/WKWebView (where module
   *   workers have a very slow cold start), `module` everywhere else.
   * - `"module"`: always use the `.mjs` ES-module worker. Required for webpack
   *   5 / Next.js consumers so the asset tracer can see the WASM URL.
   * - `"classic"`: always use the `.js` classic-script worker. Required on
   *   Safari/WKWebView. Set this if your consumer bundler (or your host app)
   *   does not support module workers.
   *
   * Set before the first `WebClient.createClient(...)` call.
   */
  static workerMode = "auto";

  /**
   * Decide between the module and classic worker variants based on
   * `WebClient.workerMode` and (when `auto`) the current user agent.
   * @returns {boolean} true when the classic script should be used.
   * @private
   */
  static _shouldUseClassicWorker() {
    const mode = WebClient.workerMode;
    if (mode === "module") return false;
    if (mode === "classic") return true;
    // auto: classic on Safari/WKWebView, module everywhere else.
    const ua =
      typeof navigator !== "undefined" && navigator.userAgent
        ? navigator.userAgent
        : "";
    // Chromium-based browsers (Chrome, Edge, Brave, Opera, Chromium-based
    // Android WebView) handle module workers fine.
    if (/Chrome\/|Chromium\//.test(ua)) return false;
    // Safari (desktop + iOS) and WKWebView-without-Chrome (e.g. Capacitor host)
    // both have AppleWebKit but no Chrome/Chromium in the UA. Prefer classic.
    if (/AppleWebKit/.test(ua)) return true;
    // Firefox, jsdom, node without navigator, etc. — module worker is fine.
    return false;
  }

  /**
   * Create a WebClient wrapper.
   *
   * @param {string | undefined} rpcUrl - RPC endpoint URL used by the client.
   * @param {Uint8Array | undefined} seed - Optional seed for account initialization.
   * @param {string | undefined} storeName - Optional name for the store to be used by the client.
   * @param {(pubKey: Uint8Array) => Promise<Uint8Array | null | undefined> | Uint8Array | null | undefined} [getKeyCb]
   *   - Callback to retrieve the secret key bytes for a given public key. The `pubKey`
   *   parameter is the serialized public key (from `PublicKey.serialize()`). Return the
   *   corresponding secret key as a `Uint8Array`, or `null`/`undefined` if not found. The
   *   return value may be provided synchronously or via a `Promise`.
   * @param {(pubKey: Uint8Array, AuthSecretKey: Uint8Array) => Promise<void> | void} [insertKeyCb]
   *   - Callback to persist a secret key. `pubKey` is the serialized public key, and
   *   `authSecretKey` is the serialized secret key (from `AuthSecretKey.serialize()`). May return
   *   `void` or a `Promise<void>`.
   * @param {(pubKey: Uint8Array, signingInputs: Uint8Array) => Promise<Uint8Array> | Uint8Array} [signCb]
   *   - Callback to produce serialized signature bytes for the provided inputs. `pubKey` is the
   *   serialized public key, and `signingInputs` is a `Uint8Array` produced by
   *   `SigningInputs.serialize()`. Must return a `Uint8Array` containing the serialized
   *   signature, either directly or wrapped in a `Promise`.
   * @param {string | undefined} [logLevel] - Optional log verbosity level
   *   ("error", "warn", "info", "debug", "trace", "off", or "none").
   *   When set, Rust tracing output is routed to the browser console.
   */
  constructor(
    rpcUrl,
    noteTransportUrl,
    seed,
    storeName,
    getKeyCb,
    insertKeyCb,
    signCb,
    logLevel
  ) {
    this.rpcUrl = rpcUrl;
    this.noteTransportUrl = noteTransportUrl;
    this.seed = seed;
    this.storeName = storeName;
    this.getKeyCb = getKeyCb;
    this.insertKeyCb = insertKeyCb;
    this.signCb = signCb;
    this.logLevel = logLevel;

    // Check if Web Workers are available.
    if (typeof Worker !== "undefined") {
      console.log("WebClient: Web Workers are available.");
      // Pick between the module and classic worker variants at runtime — see
      // `WebClient.workerMode` below. Both branches keep the
      // `new Worker(new URL("...", import.meta.url), ...)` form fully literal:
      // webpack 5's new-worker detector is PURELY SYNTACTIC and only triggers
      // a proper worker sub-compilation (with asset+chunk tracing into the
      // Cargo glue and the sibling WASM) when it sees that exact pattern
      // spelled inline. Hoisting either URL into a variable downgrades the
      // detection to a plain "copy file as asset" — which in turn makes the
      // worker's `await import("./Cargo-*.js")` 404 because webpack never
      // emitted a chunk for it. The bit of duplication here is load-bearing.
      //
      // - module (`.module.js` with `{ type: "module" }`): `import.meta.url`
      //   inside the Cargo glue is preserved so webpack/Vite can resolve the
      //   WASM URL statically. Preferred everywhere EXCEPT Safari/WKWebView.
      // - classic (`.js`, no options): self-contained async IIFE with
      //   `import.meta.url` rewritten to `self.location.href`; the only form
      //   Safari/WKWebView can cold-start in a reasonable time.
      if (WebClient._shouldUseClassicWorker()) {
        this.worker = new Worker(
          new URL("./workers/web-client-methods-worker.js", import.meta.url)
        );
      } else {
        this.worker = new Worker(
          new URL(
            "./workers/web-client-methods-worker.module.js",
            import.meta.url
          ),
          { type: "module" }
        );
      }

      // Map to track pending worker requests.
      this.pendingRequests = new Map();

      // Promises to track when the worker script is loaded and ready.
      this.loaded = new Promise((resolve) => {
        this.loadedResolver = resolve;
      });

      // Create a promise that resolves when the worker signals that it is fully initialized.
      this.ready = new Promise((resolve) => {
        this.readyResolver = resolve;
      });

      // Listen for messages from the worker.
      this.worker.addEventListener("message", async (event) => {
        const data = event.data;

        // Worker script loaded.
        if (data.loaded) {
          this.loadedResolver();
          return;
        }

        // Worker ready.
        if (data.ready) {
          this.readyResolver();
          return;
        }

        if (data.action === WorkerAction.EXECUTE_CALLBACK) {
          const { callbackType, args, requestId } = data;
          try {
            const callbackMapping = {
              [CallbackType.GET_KEY]: this.getKeyCb,
              [CallbackType.INSERT_KEY]: this.insertKeyCb,
              [CallbackType.SIGN]: this.signCb,
            };
            if (!callbackMapping[callbackType]) {
              throw new Error(`Callback ${callbackType} not available`);
            }
            const callbackFunction = callbackMapping[callbackType];
            let result = callbackFunction.apply(this, args);
            if (result instanceof Promise) {
              result = await result;
            }

            this.worker.postMessage({
              callbackResult: result,
              callbackRequestId: requestId,
            });
          } catch (error) {
            this.worker.postMessage({
              callbackError: error.message,
              callbackRequestId: requestId,
            });
          }
          return;
        }

        // Handle responses for method calls.
        const { requestId, error, result, methodName } = data;
        if (requestId && this.pendingRequests.has(requestId)) {
          const { resolve, reject } = this.pendingRequests.get(requestId);
          this.pendingRequests.delete(requestId);
          if (error) {
            const workerError =
              error instanceof Error ? error : deserializeError(error);
            console.error(
              `WebClient: Error from worker in ${methodName}:`,
              workerError
            );
            reject(workerError);
          } else {
            resolve(result);
          }
        }
      });

      // Once the worker script has loaded, initialize the worker.
      this.loaded.then(() => this.initializeWorker());
    } else {
      console.log("WebClient: Web Workers are not available.");
      // Worker not available; set up fallback values.
      this.worker = null;
      this.pendingRequests = null;
      this.loaded = Promise.resolve();
      this.ready = Promise.resolve();
    }

    // Lazy initialize the underlying WASM WebClient when first requested.
    this.wasmWebClient = null;
    this.wasmWebClientPromise = null;

    // Promise chain to serialize direct WASM calls that require exclusive
    // (&mut self) access. Without this, concurrent calls on the same client
    // would panic with "recursive use of an object detected" due to
    // wasm-bindgen's internal RefCell.
    this._wasmCallChain = Promise.resolve();
  }

  /**
   * Serialize a WASM call that requires exclusive (&mut self) access.
   * Concurrent calls are queued and executed one at a time.
   *
   * Wraps both the direct (in-thread) path and the worker-dispatched path.
   * On the worker path this is redundant with the worker's own message queue,
   * but harmless (the chain resolves immediately on the main thread once the
   * worker's postMessage returns). On the direct path it is load-bearing —
   * without it, concurrent main-thread callers would panic with
   * "recursive use of an object detected" (wasm-bindgen's internal RefCell).
   *
   * @param {() => Promise<any>} fn - The async function to execute.
   * @returns {Promise<any>} The result of fn.
   */
  _serializeWasmCall(fn) {
    const result = this._wasmCallChain.catch(() => {}).then(fn);
    this._wasmCallChain = result.catch(() => {});
    return result;
  }

  /**
   * Returns a promise that resolves once every serialized WASM call that
   * was already on `_wasmCallChain` when `waitForIdle()` was called has
   * settled. Use this from callers that need to perform a non-WASM-side
   * action (e.g. clear an in-memory auth key) AFTER any in-flight
   * execute / submit / sync has completed, so the WASM kernel's auth
   * callback doesn't race with the key being cleared.
   *
   * Does NOT wait for calls enqueued after `waitForIdle()` returns —
   * this is intentional, so a caller can drain and then proceed without
   * being blocked indefinitely by a concurrent workload.
   *
   * Caveat for `syncState`: `syncStateWithTimeout` awaits
   * `acquireSyncLock` (Web Locks) BEFORE wrapping its WASM call in
   * `_serializeWasmCall`, so a sync that is queued on the sync lock but
   * has not yet reached its WASM phase is not on the chain and will not
   * be awaited. Every other serialized method (`executeTransaction`,
   * `newWallet`, `submitNewTransaction`, `proveTransaction`,
   * `applyTransaction`, and the proxy-fallback reads) routes through
   * the chain synchronously on call and is always observed.
   *
   * @returns {Promise<void>}
   */
  async waitForIdle() {
    // Chain on `_wasmCallChain`; by the time this resolves, any in-flight
    // serialized call has settled. Catch so the chain state doesn't leak.
    await this._wasmCallChain.catch(() => {});
  }

  // TODO: This will soon conflict with some changes in main.
  // More context here:
  // https://github.com/0xMiden/miden-client/pull/1645?notification_referrer_id=NT_kwHOA1yg7NoAJVJlcG9zaXRvcnk7NjU5MzQzNzAyO0lzc3VlOzM3OTY4OTU1Nzk&notifications_query=is%3Aunread#discussion_r2696075480
  initializeWorker() {
    this.worker.postMessage({
      action: WorkerAction.INIT,
      args: [
        this.rpcUrl,
        this.noteTransportUrl,
        this.seed,
        this.storeName,
        !!this.getKeyCb,
        !!this.insertKeyCb,
        !!this.signCb,
        this.logLevel,
      ],
    });
  }

  async getWasmWebClient() {
    if (this.wasmWebClient) {
      return this.wasmWebClient;
    }
    if (!this.wasmWebClientPromise) {
      this.wasmWebClientPromise = (async () => {
        const wasm = await getWasmOrThrow();
        const client = new wasm.WebClient();
        this.wasmWebClient = client;
        return client;
      })();
    }
    return this.wasmWebClientPromise;
  }

  /**
   * Factory method to create and initialize a WebClient instance.
   * This method is async so you can await the asynchronous call to createClient().
   *
   * @param {string} rpcUrl - The RPC URL.
   * @param {string} noteTransportUrl - The note transport URL (optional).
   * @param {string} seed - The seed for the account.
   * @param {string | undefined} network - Optional name for the store. Setting this allows multiple clients to be used in the same browser.
   * @param {string | undefined} logLevel - Optional log verbosity level ("error", "warn", "info", "debug", "trace", "off", or "none").
   * @returns {Promise<WebClient>} The fully initialized WebClient.
   */
  static async createClient(rpcUrl, noteTransportUrl, seed, network, logLevel) {
    // Construct the instance (synchronously).
    const instance = new WebClient(
      rpcUrl,
      noteTransportUrl,
      seed,
      network,
      undefined,
      undefined,
      undefined,
      logLevel
    );

    // Set up logging on the main thread before creating the client.
    if (logLevel) {
      const wasm = await getWasmOrThrow();
      wasm.setupLogging(logLevel);
    }

    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createClient(rpcUrl, noteTransportUrl, seed, network);

    // Wait for the worker to be ready
    await instance.ready;

    return createClientProxy(instance);
  }

  /**
   * Factory method to create and initialize a WebClient instance with a remote keystore.
   * This method is async so you can await the asynchronous call to createClientWithExternalKeystore().
   *
   * @param {string} rpcUrl - The RPC URL.
   * @param {string | undefined} noteTransportUrl - The note transport URL (optional).
   * @param {string | undefined} seed - The seed for the account.
   * @param {string | undefined} storeName - Optional name for the store. Setting this allows multiple clients to be used in the same browser.
   * @param {Function | undefined} getKeyCb - The get key callback.
   * @param {Function | undefined} insertKeyCb - The insert key callback.
   * @param {Function | undefined} signCb - The sign callback.
   * @param {string | undefined} logLevel - Optional log verbosity level ("error", "warn", "info", "debug", "trace", "off", or "none").
   * @returns {Promise<WebClient>} The fully initialized WebClient.
   */
  static async createClientWithExternalKeystore(
    rpcUrl,
    noteTransportUrl,
    seed,
    storeName,
    getKeyCb,
    insertKeyCb,
    signCb,
    logLevel
  ) {
    // Construct the instance (synchronously).
    const instance = new WebClient(
      rpcUrl,
      noteTransportUrl,
      seed,
      storeName,
      getKeyCb,
      insertKeyCb,
      signCb,
      logLevel
    );

    // Set up logging on the main thread before creating the client.
    if (logLevel) {
      const wasm = await getWasmOrThrow();
      wasm.setupLogging(logLevel);
    }

    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createClientWithExternalKeystore(
      rpcUrl,
      noteTransportUrl,
      seed,
      storeName,
      getKeyCb,
      insertKeyCb,
      signCb
    );

    await instance.ready;
    return createClientProxy(instance);
  }

  /**
   * Call a method via the worker.
   * @param {string} methodName - Name of the method to call.
   * @param  {...any} args - Arguments for the method.
   * @returns {Promise<any>}
   */
  async callMethodWithWorker(methodName, ...args) {
    await this.ready;
    // Create a unique request ID.
    const requestId = `${methodName}-${Date.now()}-${Math.random()}`;
    return new Promise((resolve, reject) => {
      // Save the resolve and reject callbacks in the pendingRequests map.
      this.pendingRequests.set(requestId, { resolve, reject });
      // Send the method call request to the worker.
      this.worker.postMessage({
        action: WorkerAction.CALL_METHOD,
        methodName,
        args,
        requestId,
      });
    });
  }

  // ----- Explicitly Wrapped Methods (Worker-Forwarded) -----

  async newWallet(storageMode, mutable, authSchemeId, seed) {
    return this._serializeWasmCall(async () => {
      const wasmWebClient = await this.getWasmWebClient();
      return await wasmWebClient.newWallet(
        storageMode,
        mutable,
        authSchemeId,
        seed
      );
    });
  }

  async newFaucet(
    storageMode,
    nonFungible,
    tokenSymbol,
    decimals,
    maxSupply,
    authSchemeId
  ) {
    return this._serializeWasmCall(async () => {
      const wasmWebClient = await this.getWasmWebClient();
      return await wasmWebClient.newFaucet(
        storageMode,
        nonFungible,
        tokenSymbol,
        decimals,
        maxSupply,
        authSchemeId
      );
    });
  }

  async newAccount(account, overwrite) {
    return this._serializeWasmCall(async () => {
      const wasmWebClient = await this.getWasmWebClient();
      return await wasmWebClient.newAccount(account, overwrite);
    });
  }

  async newAccountWithSecretKey(account, secretKey) {
    return this._serializeWasmCall(async () => {
      const wasmWebClient = await this.getWasmWebClient();
      return await wasmWebClient.newAccountWithSecretKey(account, secretKey);
    });
  }

  async submitNewTransaction(accountId, transactionRequest) {
    return this._serializeWasmCall(async () => {
      try {
        if (!this.worker) {
          const wasmWebClient = await this.getWasmWebClient();
          return await wasmWebClient.submitNewTransaction(
            accountId,
            transactionRequest
          );
        }

        const wasm = await getWasmOrThrow();
        const serializedTransactionRequest = transactionRequest.serialize();
        const result = await this.callMethodWithWorker(
          MethodName.SUBMIT_NEW_TRANSACTION,
          accountId.toString(),
          serializedTransactionRequest
        );

        const transactionResult = wasm.TransactionResult.deserialize(
          new Uint8Array(result.serializedTransactionResult)
        );

        return transactionResult.id();
      } catch (error) {
        console.error("INDEX.JS: Error in submitNewTransaction:", error);
        throw error;
      }
    });
  }

  async submitNewTransactionWithProver(accountId, transactionRequest, prover) {
    return this._serializeWasmCall(async () => {
      try {
        if (!this.worker) {
          const wasmWebClient = await this.getWasmWebClient();
          return await wasmWebClient.submitNewTransactionWithProver(
            accountId,
            transactionRequest,
            prover
          );
        }

        const wasm = await getWasmOrThrow();
        const serializedTransactionRequest = transactionRequest.serialize();
        const proverPayload = prover.serialize();
        const result = await this.callMethodWithWorker(
          MethodName.SUBMIT_NEW_TRANSACTION_WITH_PROVER,
          accountId.toString(),
          serializedTransactionRequest,
          proverPayload
        );

        const transactionResult = wasm.TransactionResult.deserialize(
          new Uint8Array(result.serializedTransactionResult)
        );

        return transactionResult.id();
      } catch (error) {
        console.error(
          "INDEX.JS: Error in submitNewTransactionWithProver:",
          error
        );
        throw error;
      }
    });
  }

  async executeTransaction(accountId, transactionRequest) {
    return this._serializeWasmCall(async () => {
      try {
        if (!this.worker) {
          const wasmWebClient = await this.getWasmWebClient();
          return await wasmWebClient.executeTransaction(
            accountId,
            transactionRequest
          );
        }

        const wasm = await getWasmOrThrow();
        const serializedTransactionRequest = transactionRequest.serialize();
        const serializedResultBytes = await this.callMethodWithWorker(
          MethodName.EXECUTE_TRANSACTION,
          accountId.toString(),
          serializedTransactionRequest
        );

        return wasm.TransactionResult.deserialize(
          new Uint8Array(serializedResultBytes)
        );
      } catch (error) {
        console.error("INDEX.JS: Error in executeTransaction:", error);
        throw error;
      }
    });
  }

  async proveTransaction(transactionResult, prover) {
    return this._serializeWasmCall(async () => {
      try {
        if (!this.worker) {
          const wasmWebClient = await this.getWasmWebClient();
          return prover
            ? await wasmWebClient.proveTransactionWithProver(
                transactionResult,
                prover
              )
            : await wasmWebClient.proveTransaction(transactionResult);
        }

        const wasm = await getWasmOrThrow();
        const serializedTransactionResult = transactionResult.serialize();
        const proverPayload = prover ? prover.serialize() : null;

        const serializedProvenBytes = await this.callMethodWithWorker(
          MethodName.PROVE_TRANSACTION,
          serializedTransactionResult,
          proverPayload
        );

        return wasm.ProvenTransaction.deserialize(
          new Uint8Array(serializedProvenBytes)
        );
      } catch (error) {
        console.error("INDEX.JS: Error in proveTransaction:", error);
        throw error;
      }
    });
  }

  // Delegates to `proveTransaction`, which already routes through
  // `_serializeWasmCall` and dispatches to the WASM `proveTransactionWithProver`
  // binding when `prover` is present. Kept as a wrapper (rather than elided)
  // so the method classification lint sees an explicit match for the WASM
  // method name.
  async proveTransactionWithProver(transactionResult, prover) {
    return this.proveTransaction(transactionResult, prover);
  }

  async applyTransaction(transactionResult, submissionHeight) {
    return this._serializeWasmCall(async () => {
      try {
        if (!this.worker) {
          const wasmWebClient = await this.getWasmWebClient();
          return await wasmWebClient.applyTransaction(
            transactionResult,
            submissionHeight
          );
        }

        const wasm = await getWasmOrThrow();
        const serializedTransactionResult = transactionResult.serialize();
        const serializedUpdateBytes = await this.callMethodWithWorker(
          MethodName.APPLY_TRANSACTION,
          serializedTransactionResult,
          submissionHeight
        );

        return wasm.TransactionStoreUpdate.deserialize(
          new Uint8Array(serializedUpdateBytes)
        );
      } catch (error) {
        console.error("INDEX.JS: Error in applyTransaction:", error);
        throw error;
      }
    });
  }

  /**
   * Syncs the client state with the node.
   *
   * This method coordinates concurrent sync calls using the Web Locks API when available,
   * with an in-process mutex fallback for older browsers. If a sync is already in progress,
   * subsequent callers will wait and receive the same result (coalescing behavior).
   *
   * @returns {Promise<SyncSummary>} The sync summary
   */
  async syncState() {
    return this.syncStateWithTimeout(0);
  }

  /**
   * Syncs the client state with the node with an optional timeout.
   *
   * This method coordinates concurrent sync calls using the Web Locks API when available,
   * with an in-process mutex fallback for older browsers. If a sync is already in progress,
   * subsequent callers will wait and receive the same result (coalescing behavior).
   *
   * @param {number} timeoutMs - Timeout in milliseconds (0 = no timeout)
   * @returns {Promise<SyncSummary>} The sync summary
   */
  async syncStateWithTimeout(timeoutMs = 0) {
    // Use storeName as the database ID for lock coordination
    const dbId = this.storeName || "default";

    try {
      // Acquire the sync lock (coordinates concurrent calls)
      const lockHandle = await acquireSyncLock(dbId, timeoutMs);

      if (!lockHandle.acquired) {
        // We're coalescing - return the result from the in-progress sync
        return lockHandle.coalescedResult;
      }

      // We acquired the lock - perform the sync. Wrap the actual WASM
      // call in _serializeWasmCall so it can't race with any other
      // mutating method (executeTransaction, submitNewTransaction, etc.)
      // on the same WebClient. The outer coalescing lock stays in place
      // so concurrent syncState callers still share one in-flight sync.
      try {
        const result = await this._serializeWasmCall(async () => {
          if (!this.worker) {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.syncStateImpl();
          }
          const wasm = await getWasmOrThrow();
          const serializedSyncSummaryBytes = await this.callMethodWithWorker(
            MethodName.SYNC_STATE
          );
          return wasm.SyncSummary.deserialize(
            new Uint8Array(serializedSyncSummaryBytes)
          );
        });

        // Release the lock with the result
        releaseSyncLock(dbId, result);
        return result;
      } catch (error) {
        // Release the lock with the error
        releaseSyncLockWithError(dbId, error);
        throw error;
      }
    } catch (error) {
      console.error("INDEX.JS: Error in syncState:", error);
      throw error;
    }
  }

  /**
   * Terminates the underlying Web Worker used by this WebClient instance.
   *
   * Call this method when you're done using a WebClient to free up browser
   * resources. Each WebClient instance uses a dedicated Web Worker for
   * computationally intensive operations. Terminating releases that thread.
   *
   * After calling terminate(), the WebClient should not be used.
   */
  terminate() {
    if (this.worker) {
      this.worker.terminate();
    }
  }
}

class MockWebClient extends WebClient {
  constructor(seed, logLevel) {
    super(
      null,
      null,
      seed,
      MOCK_STORE_NAME,
      undefined,
      undefined,
      undefined,
      logLevel
    );
  }

  initializeWorker() {
    this.worker.postMessage({
      action: WorkerAction.INIT_MOCK,
      args: [this.seed, this.logLevel],
    });
  }

  /**
   * Factory method to create a WebClient with a mock chain for testing purposes.
   *
   * @param serializedMockChain - Serialized mock chain data (optional). Will use an empty chain if not provided.
   * @param serializedMockNoteTransportNode - Serialized mock note transport node data (optional). Will use a new instance if not provided.
   * @param seed - The seed for the account (optional).
   * @returns A promise that resolves to a MockWebClient.
   */
  static async createClient(
    serializedMockChain,
    serializedMockNoteTransportNode,
    seed,
    logLevel
  ) {
    // Construct the instance (synchronously).
    const instance = new MockWebClient(seed, logLevel);

    // Set up logging on the main thread before creating the client.
    if (logLevel) {
      const wasm = await getWasmOrThrow();
      wasm.setupLogging(logLevel);
    }

    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createMockClient(
      seed,
      serializedMockChain,
      serializedMockNoteTransportNode
    );

    // Wait for the worker to be ready
    await instance.ready;

    return createClientProxy(instance);
  }

  /**
   * Syncs the mock client state.
   *
   * This method coordinates concurrent sync calls using the Web Locks API when available,
   * with an in-process mutex fallback for older browsers. If a sync is already in progress,
   * subsequent callers will wait and receive the same result (coalescing behavior).
   *
   * @returns {Promise<SyncSummary>} The sync summary
   */
  async syncState() {
    return this.syncStateWithTimeout(0);
  }

  /**
   * Syncs the mock client state with an optional timeout.
   *
   * @param {number} timeoutMs - Timeout in milliseconds (0 = no timeout)
   * @returns {Promise<SyncSummary>} The sync summary
   */
  async syncStateWithTimeout(timeoutMs = 0) {
    const dbId = this.storeName || "mock";

    try {
      const lockHandle = await acquireSyncLock(dbId, timeoutMs);

      if (!lockHandle.acquired) {
        return lockHandle.coalescedResult;
      }

      try {
        let result;
        const wasmWebClient = await this.getWasmWebClient();

        if (!this.worker) {
          result = await wasmWebClient.syncStateImpl();
        } else {
          let serializedMockChain = wasmWebClient.serializeMockChain().buffer;
          let serializedMockNoteTransportNode =
            wasmWebClient.serializeMockNoteTransportNode().buffer;

          const wasm = await getWasmOrThrow();

          const serializedSyncSummaryBytes = await this.callMethodWithWorker(
            MethodName.SYNC_STATE_MOCK,
            serializedMockChain,
            serializedMockNoteTransportNode
          );

          result = wasm.SyncSummary.deserialize(
            new Uint8Array(serializedSyncSummaryBytes)
          );
        }

        releaseSyncLock(dbId, result);
        return result;
      } catch (error) {
        releaseSyncLockWithError(dbId, error);
        throw error;
      }
    } catch (error) {
      console.error("INDEX.JS: Error in syncState:", error);
      throw error;
    }
  }

  async submitNewTransaction(accountId, transactionRequest) {
    try {
      if (!this.worker) {
        return await super.submitNewTransaction(accountId, transactionRequest);
      }

      const wasmWebClient = await this.getWasmWebClient();
      const wasm = await getWasmOrThrow();
      const serializedTransactionRequest = transactionRequest.serialize();
      const serializedMockChain = wasmWebClient.serializeMockChain().buffer;
      const serializedMockNoteTransportNode =
        wasmWebClient.serializeMockNoteTransportNode().buffer;

      const result = await this.callMethodWithWorker(
        MethodName.SUBMIT_NEW_TRANSACTION_MOCK,
        accountId.toString(),
        serializedTransactionRequest,
        serializedMockChain,
        serializedMockNoteTransportNode
      );

      const newMockChain = new Uint8Array(result.serializedMockChain);
      const newMockNoteTransportNode = result.serializedMockNoteTransportNode
        ? new Uint8Array(result.serializedMockNoteTransportNode)
        : undefined;

      const transactionResult = wasm.TransactionResult.deserialize(
        new Uint8Array(result.serializedTransactionResult)
      );

      if (!(this instanceof MockWebClient)) {
        return transactionResult.id();
      }

      this.wasmWebClient = new wasm.WebClient();
      this.wasmWebClientPromise = Promise.resolve(this.wasmWebClient);
      await this.wasmWebClient.createMockClient(
        this.seed,
        newMockChain,
        newMockNoteTransportNode
      );

      return transactionResult.id();
    } catch (error) {
      console.error("INDEX.JS: Error in submitNewTransaction:", error);
      throw error;
    }
  }

  async submitNewTransactionWithProver(accountId, transactionRequest, prover) {
    try {
      if (!this.worker) {
        return await super.submitNewTransactionWithProver(
          accountId,
          transactionRequest,
          prover
        );
      }

      const wasmWebClient = await this.getWasmWebClient();
      const wasm = await getWasmOrThrow();
      const serializedTransactionRequest = transactionRequest.serialize();
      const proverPayload = prover.serialize();
      const serializedMockChain = wasmWebClient.serializeMockChain().buffer;
      const serializedMockNoteTransportNode =
        wasmWebClient.serializeMockNoteTransportNode().buffer;

      const result = await this.callMethodWithWorker(
        MethodName.SUBMIT_NEW_TRANSACTION_WITH_PROVER_MOCK,
        accountId.toString(),
        serializedTransactionRequest,
        proverPayload,
        serializedMockChain,
        serializedMockNoteTransportNode
      );

      const newMockChain = new Uint8Array(result.serializedMockChain);
      const newMockNoteTransportNode = result.serializedMockNoteTransportNode
        ? new Uint8Array(result.serializedMockNoteTransportNode)
        : undefined;

      const transactionResult = wasm.TransactionResult.deserialize(
        new Uint8Array(result.serializedTransactionResult)
      );

      if (!(this instanceof MockWebClient)) {
        return transactionResult.id();
      }

      this.wasmWebClient = new wasm.WebClient();
      this.wasmWebClientPromise = Promise.resolve(this.wasmWebClient);
      await this.wasmWebClient.createMockClient(
        this.seed,
        newMockChain,
        newMockNoteTransportNode
      );

      return transactionResult.id();
    } catch (error) {
      console.error(
        "INDEX.JS: Error in submitNewTransactionWithProver:",
        error
      );
      throw error;
    }
  }
}

function copyWebClientStatics(WasmWebClient) {
  if (!WasmWebClient) {
    return;
  }
  Object.getOwnPropertyNames(WasmWebClient).forEach((prop) => {
    if (
      typeof WasmWebClient[prop] === "function" &&
      prop !== "constructor" &&
      prop !== "prototype"
    ) {
      WebClient[prop] = WasmWebClient[prop];
    }
  });
}

// Wire MidenClient dependencies (resolves circular import)
MidenClient._WasmWebClient = WebClient;
MidenClient._MockWasmWebClient = MockWebClient;
MidenClient._getWasmOrThrow = getWasmOrThrow;
_setStandaloneWebClient(WebClient);
