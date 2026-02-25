import loadWasm from "./wasm.js";
import { CallbackType, MethodName, WorkerAction } from "./constants.js";
import {
  acquireSyncLock,
  releaseSyncLock,
  releaseSyncLockWithError,
} from "./syncLock.js";
import { AsyncLock } from "./asyncLock.js";
import { withWriteLock } from "./webLock.js";
export * from "../Cargo.toml";

// WASM PROXY METHODS
// ================================================================================================

/**
 * Set of method names that are synchronous (non-async) on the WASM object
 * and should NOT be wrapped with the async WASM lock. Wrapping them would
 * turn their return type from T into Promise<T>, breaking callers that
 * expect a synchronous return value.
 *
 * These methods may still mutate internal client state (e.g. RNG), but they
 * complete in a single synchronous call without yielding to the event loop.
 * JavaScript's single-threaded execution guarantees they cannot interleave
 * with an in-progress async WASM call.
 *
 * When updating this set, check the Rust source for any `pub fn` (non-async)
 * method on `impl WebClient` with #[wasm_bindgen] that is NOT already handled
 * by an explicit wrapper method on the JS WebClient class.
 */
const SYNC_METHODS = new Set([
  "newMintTransactionRequest",
  "newSendTransactionRequest",
  "newConsumeTransactionRequest",
  "newSwapTransactionRequest",
  "createCodeBuilder",
  "buildSwapTag",
  "setDebugMode",
  "usesMockChain",
  "serializeMockChain",
  "serializeMockNoteTransportNode",
  "proveBlock",
]);

/**
 * Set of method names that mutate state. These are wrapped with the cross-tab
 * write lock (Layer 2) when accessed through the Proxy fallback.
 */
const WRITE_METHODS = new Set([
  "newAccount",
  "importAccountFile",
  "importAccountById",
  "importPublicAccountFromSeed",
  "importNoteFile",
  "forceImportStore",
  "addTag",
  "removeTag",
  "setSetting",
  "removeSetting",
  "insertAccountAddress",
  "removeAccountAddress",
  "sendPrivateNote",
  // fetch*PrivateNotes fetches from the note transport AND writes to IndexedDB
  "fetchPrivateNotes",
  "fetchAllPrivateNotes",
  "addAccountSecretKeyToWebStore",
  "executeForSummary",
]);

/**
 * Set of method names that are read-only (no state mutation). These are
 * wrapped with the in-process WASM lock only (Layer 1).
 *
 * This set exists purely for the CI lint check (`check-method-classification`)
 * which ensures every WASM export is explicitly classified — preventing new
 * write methods from silently defaulting to read-only.
 *
 * The Proxy behaviour is unchanged: methods not in SYNC_METHODS or
 * WRITE_METHODS already get WASM-lock-only wrapping.
 */
const READ_METHODS = new Set([
  "getAccounts",
  "getAccount",
  "getAccountAuthByPubKeyCommitment",
  "getPublicKeyCommitmentsOfAccount",
  "getInputNotes",
  "getInputNote",
  "getOutputNotes",
  "getOutputNote",
  "getConsumableNotes",
  "getTransactions",
  "getSyncHeight",
  "exportNoteFile",
  "exportStore",
  "exportAccountFile",
  "listTags",
  "getSetting",
  "listSettingKeys",
]);

/**
 * Create the Proxy that wraps a WebClient instance. The proxy:
 * - Returns properties from the wrapper (instance) first.
 * - Falls back to the underlying WASM WebClient, wrapping function calls
 *   through the in-process WASM lock (Layer 1) and, for write methods,
 *   the cross-tab write lock (Layer 2).
 */
function createClientProxy(instance) {
  return new Proxy(instance, {
    get(target, prop, receiver) {
      // If the property exists on the wrapper, return it.
      if (prop in target) {
        return Reflect.get(target, prop, receiver);
      }
      // Otherwise, if the wasmWebClient has it, return that.
      if (target.wasmWebClient && prop in target.wasmWebClient) {
        const value = target.wasmWebClient[prop];
        if (typeof value === "function") {
          // Synchronous methods: call directly without async wrapping.
          // These are pure-computation methods whose callers expect a
          // synchronous return value.
          if (SYNC_METHODS.has(prop)) {
            return (...args) => value.apply(target.wasmWebClient, args);
          }
          // Write methods: cross-tab lock (outer) → WASM lock (inner)
          if (WRITE_METHODS.has(prop)) {
            return (...args) =>
              target._withWrite(prop, () =>
                target._wasmLock.runExclusive(() =>
                  value.apply(target.wasmWebClient, args)
                )
              );
          }
          // Read methods: WASM lock only
          return (...args) =>
            target._wasmLock.runExclusive(() =>
              value.apply(target.wasmWebClient, args)
            );
        }
        return value;
      }
      return undefined;
    },
  });
}

// WASM MODULE LOADING
// ================================================================================================

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
      }
      return module;
    });
  }
  return wasmLoadPromise;
};

const getWasmOrThrow = async () => {
  const module = await ensureWasm();
  if (!module) {
    throw new Error(
      "Miden WASM bindings are unavailable in this environment (SSR is disabled)."
    );
  }
  return module;
};

// WEB CLIENT
// ================================================================================================

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
 * Concurrency safety is provided by three layers:
 *
 * - **Layer 1 (In-Process AsyncLock):** All main-thread WASM calls are serialized
 *   through `_wasmLock` to prevent "recursive use of an object detected" panics.
 *
 * - **Layer 2 (Cross-Tab Write Lock):** Mutating operations acquire an exclusive
 *   Web Lock (`miden-db-{storeName}`) so that writes from different tabs are
 *   serialized against the same IndexedDB database.
 *
 * - **Layer 3 (BroadcastChannel):** After every write, a notification is sent
 *   to all other tabs so they can refresh stale in-memory state.
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
export class WebClient {
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

    // Layer 1: In-process WASM lock — serializes all main-thread WASM calls.
    this._wasmLock = new AsyncLock();

    // Layer 2: Guard for the cross-tab write lock. When true, the current tab
    // already holds the Web Lock, so nested or concurrent _withWrite calls in
    // this tab skip acquiring it again (the outer call's lock already blocks
    // other tabs). The in-process _wasmLock still serializes WASM access.
    this._writeLockHeld = false;

    // Layer 3: BroadcastChannel for cross-tab state-change notifications.
    const channelName = `miden-state-${storeName || "default"}`;
    try {
      this._stateChannel =
        typeof BroadcastChannel !== "undefined"
          ? new BroadcastChannel(channelName)
          : null;
    } catch {
      this._stateChannel = null;
    }
    this._stateListeners = [];
    if (this._stateChannel) {
      this._stateChannel.onmessage = async (event) => {
        // Auto-sync: refresh in-memory Rust Client state from IndexedDB.
        // Sync coalescing (in syncLock.js) ensures concurrent syncs share the
        // same result, so rapid messages are handled without debouncing.
        try {
          await this.syncState();
        } catch {
          // Sync failure is non-fatal — the next explicit sync will retry.
        }

        // Invoke listeners AFTER syncState resolves so in-memory state
        // is guaranteed fresh when callbacks run.
        for (const listener of this._stateListeners) {
          try {
            listener(event.data);
          } catch {
            // Swallow listener errors.
          }
        }
      };
    }

    // Check if Web Workers are available.
    if (typeof Worker !== "undefined") {
      console.log("WebClient: Web Workers are available.");
      // Create the worker.
      this.worker = new Worker(
        new URL("./workers/web-client-methods-worker.js", import.meta.url),
        { type: "module" }
      );

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
  }

  // CONCURRENCY HELPERS
  // ================================================================================================

  /**
   * Execute `fn` under the cross-tab write lock and broadcast a state-change
   * notification when it completes. Safe to call re-entrantly within the same
   * tab (the inner call skips the cross-tab lock since the outer call holds it).
   *
   * @param {string} operation - Name of the operation (for the broadcast payload).
   * @param {() => Promise<T>} fn - The async work.
   * @returns {Promise<T>}
   * @template T
   */
  async _withWrite(operation, fn) {
    if (this._writeLockHeld) {
      // This tab already holds the cross-tab Web Lock — skip re-acquiring
      // it to avoid deadlock. The outer call's lock still blocks other tabs.
      return fn();
    }

    const storeName = this.storeName || "default";

    const result = await withWriteLock(storeName, async () => {
      this._writeLockHeld = true;
      try {
        return await fn();
      } finally {
        this._writeLockHeld = false;
      }
    });

    // Layer 3: notify other tabs. Skip for syncState — sync is not a
    // user-facing mutation, and broadcasting it would cause a ping-pong
    // loop (Tab A syncs → broadcasts → Tab B auto-syncs → broadcasts → …).
    if (operation !== "syncState") {
      this._broadcastStateChange(operation);
    }

    return result;
  }

  /**
   * Send a state-change notification over the BroadcastChannel (Layer 3).
   *
   * @param {string} [operation] - Human-readable name of the operation.
   */
  _broadcastStateChange(operation) {
    if (this._stateChannel) {
      try {
        this._stateChannel.postMessage({
          type: "stateChanged",
          operation,
          storeName: this.storeName || "default",
        });
      } catch {
        // BroadcastChannel may be closed — ignore.
      }
    }
  }

  /**
   * Register a listener that is called when **another tab** mutates the same
   * IndexedDB database (Layer 3). The WebClient automatically calls
   * `syncState()` before invoking listeners, so the in-memory state is
   * already refreshed when your callback runs. Use this for additional
   * work like re-fetching accounts or updating UI.
   *
   * Returns an unsubscribe function.
   *
   * @param {(event: {type: string, operation?: string, storeName: string}) => void} callback
   * @returns {() => void} Unsubscribe function.
   */
  onStateChanged(callback) {
    this._stateListeners.push(callback);
    return () => {
      this._stateListeners = this._stateListeners.filter((l) => l !== callback);
    };
  }

  // WORKER / WASM INITIALIZATION
  // ================================================================================================

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

  // FACTORY METHODS
  // ================================================================================================

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

  // EXPLICITLY WRAPPED METHODS (Worker-Forwarded + Concurrency-Safe)
  // ================================================================================================

  async newWallet(storageMode, mutable, authSchemeId, seed) {
    return this._withWrite("newWallet", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.newWallet(
              storageMode,
              mutable,
              authSchemeId,
              seed
            );
          });
        }
        const wasm = await getWasmOrThrow();
        const serializedStorageMode = storageMode.asStr();
        const serializedAccountBytes = await this.callMethodWithWorker(
          MethodName.NEW_WALLET,
          serializedStorageMode,
          mutable,
          authSchemeId,
          seed
        );
        return wasm.Account.deserialize(new Uint8Array(serializedAccountBytes));
      } catch (error) {
        console.error("INDEX.JS: Error in newWallet:", error);
        throw error;
      }
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
    return this._withWrite("newFaucet", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
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
        const wasm = await getWasmOrThrow();
        const serializedStorageMode = storageMode.asStr();
        const serializedMaxSupply = maxSupply.toString();
        const serializedAccountBytes = await this.callMethodWithWorker(
          MethodName.NEW_FAUCET,
          serializedStorageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          serializedMaxSupply,
          authSchemeId
        );

        return wasm.Account.deserialize(new Uint8Array(serializedAccountBytes));
      } catch (error) {
        console.error("INDEX.JS: Error in newFaucet:", error);
        throw error;
      }
    });
  }

  async submitNewTransaction(accountId, transactionRequest) {
    return this._withWrite("submitNewTransaction", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.submitNewTransaction(
              accountId,
              transactionRequest
            );
          });
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
    return this._withWrite("submitNewTransactionWithProver", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.submitNewTransactionWithProver(
              accountId,
              transactionRequest,
              prover
            );
          });
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
    return this._withWrite("executeTransaction", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.executeTransaction(
              accountId,
              transactionRequest
            );
          });
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

  // proveTransaction is CPU-heavy but does NOT write to IndexedDB, so it only
  // needs the in-process WASM lock (Layer 1), not the cross-tab write lock.
  async proveTransaction(transactionResult, prover) {
    try {
      if (!this.worker) {
        return await this._wasmLock.runExclusive(async () => {
          const wasmWebClient = await this.getWasmWebClient();
          return await wasmWebClient.proveTransaction(
            transactionResult,
            prover
          );
        });
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
  }

  // submitProvenTransaction and applyTransaction sync state before the write
  // to catch cross-tab writes that occurred between execute and submit.
  // This ensures the local IndexedDB view is fresh before submission.

  async submitProvenTransaction(provenTransaction, transactionResult) {
    return this._withWrite("submitProvenTransaction", async () => {
      // Sync state to catch cross-tab writes that occurred since execute.
      // We call syncStateImpl directly (bypassing the sync lock) because we
      // already hold the write lock — calling syncState() would attempt to
      // acquire sync lock → write lock, which is the reverse of the normal
      // sync lock → write lock order and could deadlock across tabs.
      try {
        await this._wasmLock.runExclusive(async () => {
          const wasmWebClient = await this.getWasmWebClient();
          await wasmWebClient.syncStateImpl();
        });
      } catch {
        // Sync failure is non-fatal — proceed with submission.
        // The node will reject truly invalid transactions.
      }

      return await this._wasmLock.runExclusive(async () => {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.submitProvenTransaction(
          provenTransaction,
          transactionResult
        );
      });
    });
  }

  async applyTransaction(provenTransaction, transactionResult) {
    return this._withWrite("applyTransaction", async () => {
      // Sync state to catch cross-tab writes (same rationale as above).
      try {
        await this._wasmLock.runExclusive(async () => {
          const wasmWebClient = await this.getWasmWebClient();
          await wasmWebClient.syncStateImpl();
        });
      } catch {
        // Sync failure is non-fatal.
      }

      return await this._wasmLock.runExclusive(async () => {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.applyTransaction(
          provenTransaction,
          transactionResult
        );
      });
    });
  }

  // SYNC
  // ================================================================================================

  /**
   * Syncs the client state with the node.
   *
   * This method coordinates concurrent sync calls using the Web Locks API when available,
   * with an in-process mutex fallback for older browsers. If a sync is already in progress,
   * subsequent callers will wait and receive the same result (coalescing behavior).
   *
   * Sync also acquires the cross-tab write lock (Layer 2) so that it does not
   * interleave with writes from other tabs.
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
   * Lock nesting order: Sync Lock (coalescing, outer) → Write Lock → WASM Lock (inner).
   *
   * @param {number} timeoutMs - Timeout in milliseconds (0 = no timeout)
   * @returns {Promise<SyncSummary>} The sync summary
   */
  async syncStateWithTimeout(timeoutMs = 0) {
    // Use storeName as the database ID for lock coordination
    const dbId = this.storeName || "default";

    try {
      // Acquire the sync lock (coordinates concurrent calls via coalescing)
      const lockHandle = await acquireSyncLock(dbId, timeoutMs);

      if (!lockHandle.acquired) {
        // We're coalescing - return the result from the in-progress sync
        return lockHandle.coalescedResult;
      }

      // We acquired the sync lock. Now acquire the write lock so that
      // sync doesn't interleave with writes from other tabs.
      try {
        const result = await this._withWrite("syncState", async () => {
          if (!this.worker) {
            return await this._wasmLock.runExclusive(async () => {
              const wasmWebClient = await this.getWasmWebClient();
              return await wasmWebClient.syncStateImpl();
            });
          } else {
            const wasm = await getWasmOrThrow();
            const serializedSyncSummaryBytes = await this.callMethodWithWorker(
              MethodName.SYNC_STATE
            );
            return wasm.SyncSummary.deserialize(
              new Uint8Array(serializedSyncSummaryBytes)
            );
          }
        });

        // Release the sync lock with the result
        releaseSyncLock(dbId, result);
        return result;
      } catch (error) {
        // Release the sync lock with the error
        releaseSyncLockWithError(dbId, error);
        throw error;
      }
    } catch (error) {
      console.error("INDEX.JS: Error in syncState:", error);
      throw error;
    }
  }

  // LIFECYCLE
  // ================================================================================================

  terminate() {
    if (this.worker) {
      this.worker.terminate();
    }
    if (this._stateChannel) {
      try {
        this._stateChannel.close();
      } catch {
        // Already closed — ignore.
      }
      this._stateChannel = null;
    }
    this._stateListeners = [];
  }
}

// MOCK WEB CLIENT
// ================================================================================================

export class MockWebClient extends WebClient {
  constructor(seed, logLevel) {
    super(null, null, seed, "mock", undefined, undefined, undefined, logLevel);
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
        const result = await this._withWrite("syncState", async () => {
          const wasmWebClient = await this.getWasmWebClient();

          if (!this.worker) {
            return await this._wasmLock.runExclusive(() =>
              wasmWebClient.syncStateImpl()
            );
          }

          const { serializedMockChain, serializedMockNoteTransportNode } =
            await this._wasmLock.runExclusive(() => ({
              serializedMockChain: wasmWebClient.serializeMockChain().buffer,
              serializedMockNoteTransportNode:
                wasmWebClient.serializeMockNoteTransportNode().buffer,
            }));

          const wasm = await getWasmOrThrow();

          const serializedSyncSummaryBytes = await this.callMethodWithWorker(
            MethodName.SYNC_STATE_MOCK,
            serializedMockChain,
            serializedMockNoteTransportNode
          );

          return wasm.SyncSummary.deserialize(
            new Uint8Array(serializedSyncSummaryBytes)
          );
        });

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
    return this._withWrite("submitNewTransaction", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.submitNewTransaction(
              accountId,
              transactionRequest
            );
          });
        }

        const wasmWebClient = await this.getWasmWebClient();
        const wasm = await getWasmOrThrow();
        const serializedTransactionRequest = transactionRequest.serialize();
        const { serializedMockChain, serializedMockNoteTransportNode } =
          await this._wasmLock.runExclusive(() => ({
            serializedMockChain: wasmWebClient.serializeMockChain().buffer,
            serializedMockNoteTransportNode:
              wasmWebClient.serializeMockNoteTransportNode().buffer,
          }));

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
    });
  }

  async submitNewTransactionWithProver(accountId, transactionRequest, prover) {
    return this._withWrite("submitNewTransactionWithProver", async () => {
      try {
        if (!this.worker) {
          return await this._wasmLock.runExclusive(async () => {
            const wasmWebClient = await this.getWasmWebClient();
            return await wasmWebClient.submitNewTransactionWithProver(
              accountId,
              transactionRequest,
              prover
            );
          });
        }

        const wasmWebClient = await this.getWasmWebClient();
        const wasm = await getWasmOrThrow();
        const serializedTransactionRequest = transactionRequest.serialize();
        const proverPayload = prover.serialize();
        const { serializedMockChain, serializedMockNoteTransportNode } =
          await this._wasmLock.runExclusive(() => ({
            serializedMockChain: wasmWebClient.serializeMockChain().buffer,
            serializedMockNoteTransportNode:
              wasmWebClient.serializeMockNoteTransportNode().buffer,
          }));

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
    });
  }
}

// STATICS
// ================================================================================================

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
