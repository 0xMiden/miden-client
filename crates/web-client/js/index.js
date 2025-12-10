import loadWasm from "./wasm.js";
import { MethodName, WorkerAction } from "./constants.js";
export * from "../Cargo.toml";

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
export class WebClient {
  /**
   * Create a WebClient wrapper.
   *
   * @param {string | undefined} rpcUrl - RPC endpoint URL used by the client.
   * @param {Uint8Array | undefined} seed - Optional seed for account initialization.
   * @param {(pubKey: Uint8Array) => Promise<Uint8Array | null | undefined> | Uint8Array | null | undefined} [getKeyCb]
   *   - Callback to retrieve the secret key bytes for a given public key. The `pubKey`
   *   parameter is the serialized public key (from `PublicKey.serialize()`). Return the
   *   corresponding secret key as a `Uint8Array`, or `null`/`undefined` if not found. The
   *   return value may be provided synchronously or via a `Promise`.
   * @param {(pubKey: Uint8Array, secretKey: Uint8Array) => Promise<void> | void} [insertKeyCb]
   *   - Callback to persist a secret key. `pubKey` is the serialized public key, and
   *   `secretKey` is the serialized secret key (from `SecretKey.serialize()`). May return
   *   `void` or a `Promise<void>`.
   * @param {(pubKey: Uint8Array, signingInputs: Uint8Array) => Promise<Array<number | string>> | Array<number | string>} [signCb]
   *   - Callback to produce signature elements for the provided inputs. `pubKey` is the
   *   serialized public key, and `signingInputs` is a `Uint8Array` produced by
   *   `SigningInputs.serialize()`. Must return an array of numeric values (numbers or numeric
   *   strings) representing the signature elements, either directly or wrapped in a `Promise`.
   */
  constructor(rpcUrl, noteTransportUrl, seed, getKeyCb, insertKeyCb, signCb) {
    this.rpcUrl = rpcUrl;
    this.noteTransportUrl = noteTransportUrl;
    this.seed = seed;
    this.getKeyCb = getKeyCb;
    this.insertKeyCb = insertKeyCb;
    this.signCb = signCb;

    // Check if Web Workers are available.
    if (
      typeof Worker !== "undefined" &&
      !this.getKeyCb &&
      !this.insertKeyCb &&
      !this.signCb
    ) {
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
      this.worker.addEventListener("message", (event) => {
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
      this.loaded.then(() => {
        this.worker.postMessage({
          action: WorkerAction.INIT,
          args: [
            this.rpcUrl,
            this.noteTransportUrl,
            this.seed,
            this.getKeyCb,
            this.insertKeyCb,
            this.signCb,
          ],
        });
      });
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
   * @returns {Promise<WebClient>} The fully initialized WebClient.
   */
  static async createClient(rpcUrl, noteTransportUrl, seed) {
    // Construct the instance (synchronously).
    const instance = new WebClient(rpcUrl, noteTransportUrl, seed);

    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createClient(rpcUrl, noteTransportUrl, seed);

    // Wait for the worker to be ready
    await instance.ready;

    // Return a proxy that forwards missing properties to wasmWebClient.
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
            return value.bind(target.wasmWebClient);
          }
          return value;
        }
        return undefined;
      },
    });
  }

  /**
   * Factory method to create and initialize a WebClient instance with a remote keystore.
   * This method is async so you can await the asynchronous call to createClientWithExternalKeystore().
   *
   * @param {string} rpcUrl - The RPC URL.
   * @param {string | undefined} noteTransportUrl - The note transport URL (optional).
   * @param {string | undefined} seed - The seed for the account.
   * @param {Function | undefined} getKeyCb - The get key callback.
   * @param {Function | undefined} insertKeyCb - The insert key callback.
   * @param {Function | undefined} signCb - The sign callback.
   * @returns {Promise<WebClient>} The fully initialized WebClient.
   */
  static async createClientWithExternalKeystore(
    rpcUrl,
    noteTransportUrl,
    seed,
    getKeyCb,
    insertKeyCb,
    signCb
  ) {
    // Construct the instance (synchronously).
    const instance = new WebClient(
      rpcUrl,
      noteTransportUrl,
      seed,
      getKeyCb,
      insertKeyCb,
      signCb
    );
    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createClientWithExternalKeystore(
      rpcUrl,
      noteTransportUrl,
      seed,
      getKeyCb,
      insertKeyCb,
      signCb
    );

    await instance.ready;
    // Return a proxy that forwards missing properties to wasmWebClient.
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
            return value.bind(target.wasmWebClient);
          }
          return value;
        }
        return undefined;
      },
    });
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
    try {
      if (!this.worker) {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.newWallet(
          storageMode,
          mutable,
          authSchemeId,
          seed
        );
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
  }

  async newFaucet(
    storageMode,
    nonFungible,
    tokenSymbol,
    decimals,
    maxSupply,
    authSchemeId
  ) {
    try {
      if (!this.worker) {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.newFaucet(
          storageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          maxSupply,
          authSchemeId
        );
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
  }

  async submitNewTransaction(accountId, transactionRequest) {
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
  }

  async executeTransaction(accountId, transactionRequest) {
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
  }

  async proveTransaction(transactionResult, prover) {
    try {
      if (!this.worker) {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.proveTransaction(transactionResult, prover);
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

  async syncState() {
    try {
      if (!this.worker) {
        const wasmWebClient = await this.getWasmWebClient();
        return await wasmWebClient.syncState();
      }

      const wasm = await getWasmOrThrow();
      const serializedSyncSummaryBytes = await this.callMethodWithWorker(
        MethodName.SYNC_STATE
      );

      return wasm.SyncSummary.deserialize(
        new Uint8Array(serializedSyncSummaryBytes)
      );
    } catch (error) {
      console.error("INDEX.JS: Error in syncState:", error);
      throw error;
    }
  }

  terminate() {
    if (this.worker) {
      this.worker.terminate();
    }
  }
}

export class MockWebClient extends WebClient {
  constructor(seed) {
    super(null, null, seed);
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
    seed
  ) {
    // Construct the instance (synchronously).
    const instance = new MockWebClient(seed);

    // Wait for the underlying wasmWebClient to be initialized.
    const wasmWebClient = await instance.getWasmWebClient();
    await wasmWebClient.createMockClient(
      seed,
      serializedMockChain,
      serializedMockNoteTransportNode
    );

    // Wait for the worker to be ready
    await instance.ready;

    // Return a proxy that forwards missing properties to wasmWebClient.
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
            return value.bind(target.wasmWebClient);
          }
          return value;
        }
        return undefined;
      },
    });
  }

  async syncState() {
    try {
      const wasmWebClient = await this.getWasmWebClient();
      if (!this.worker) {
        return await wasmWebClient.syncState();
      }

      let serializedMockChain = wasmWebClient.serializeMockChain().buffer;
      let serializedMockNoteTransportNode =
        wasmWebClient.serializeMockNoteTransportNode().buffer;

      const wasm = await getWasmOrThrow();

      const serializedSyncSummaryBytes = await this.callMethodWithWorker(
        MethodName.SYNC_STATE_MOCK,
        serializedMockChain,
        serializedMockNoteTransportNode
      );

      return wasm.SyncSummary.deserialize(
        new Uint8Array(serializedSyncSummaryBytes)
      );
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
