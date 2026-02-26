import loadWasm from "../../dist/wasm.js";
import { CallbackType, MethodName, WorkerAction } from "../constants.js";

let wasmModule = null;

const getWasmOrThrow = async () => {
  if (!wasmModule) {
    wasmModule = await loadWasm();
  }
  if (!wasmModule) {
    throw new Error(
      "Miden WASM bindings are unavailable in the worker environment."
    );
  }
  return wasmModule;
};

const serializeUnknown = (value) => {
  if (typeof value === "string") {
    return value;
  }
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
};

const serializeError = (error) => {
  if (error instanceof Error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack,
      cause: error.cause ? serializeError(error.cause) : undefined,
      code: error.code,
    };
  }

  if (typeof error === "object" && error !== null) {
    return {
      name: error.name ?? "Error",
      message: error.message ?? serializeUnknown(error),
    };
  }

  return {
    name: "Error",
    message: serializeUnknown(error),
  };
};

/**
 * Worker for executing WebClient methods in a separate thread.
 *
 * This worker offloads computationally heavy tasks from the main thread by handling
 * WebClient operations asynchronously. It imports the WASM module and instantiates a
 * WASM WebClient, then listens for messages from the main thread to perform one of two actions:
 *
 * 1. **Initialization (init):**
 *    - The worker receives an "init" message along with user parameters (RPC URL and seed).
 *    - It instantiates the WASM WebClient and calls its createClient method.
 *    - Once initialization is complete, the worker sends a `{ ready: true }` message back to signal
 *      that it is fully initialized.
 *
 * 2. **Method Invocation (callMethod):**
 *    - The worker receives a "callMethod" message with a specific method name and arguments.
 *    - It uses a mapping (defined in `methodHandlers`) to route the call to the corresponding WASM WebClient method.
 *    - Complex data is serialized before being sent and deserialized upon return.
 *    - The result (or any error) is then posted back to the main thread.
 *
 * The worker uses a message queue to process incoming messages sequentially, ensuring that only one message
 * is handled at a time.
 *
 * Additionally, the worker immediately sends a `{ loaded: true }` message upon script load. This informs the main
 * thread that the worker script is loaded and ready to receive the "init" message.
 *
 * Supported actions (defined in `WorkerAction`):
 *   - "init"       : Initialize the WASM WebClient with provided parameters.
 *   - "callMethod" : Invoke a designated method on the WASM WebClient.
 *
 * Supported method names are defined in the `MethodName` constant.
 */

// Global state variables.
let wasmWebClient = null;
let wasmSeed = null; // Seed for the WASM WebClient, if needed.
let ready = false; // Indicates if the worker is fully initialized.
let messageQueue = []; // Queue for sequential processing.
let processing = false; // Flag to ensure one message is processed at a time.

// Track pending callback requests
let pendingCallbacks = new Map();

// Timeout for pending callbacks (30 seconds)
const CALLBACK_TIMEOUT_MS = 30000;

// Define proxy functions for callbacks that communicate with main thread
const callbackProxies = {
  getKey: async (pubKey) => {
    return new Promise((resolve, reject) => {
      const requestId = `${CallbackType.GET_KEY}-${crypto.randomUUID()}`;
      const timeoutId = setTimeout(() => {
        if (pendingCallbacks.has(requestId)) {
          pendingCallbacks.delete(requestId);
          reject(new Error(`Callback ${requestId} timed out`));
        }
      }, CALLBACK_TIMEOUT_MS);
      pendingCallbacks.set(requestId, { resolve, reject, timeoutId });

      self.postMessage({
        action: WorkerAction.EXECUTE_CALLBACK,
        callbackType: CallbackType.GET_KEY,
        args: [pubKey],
        requestId,
      });
    });
  },
  insertKey: async (pubKey, secretKey) => {
    return new Promise((resolve, reject) => {
      const requestId = `${CallbackType.INSERT_KEY}-${crypto.randomUUID()}`;
      const timeoutId = setTimeout(() => {
        if (pendingCallbacks.has(requestId)) {
          pendingCallbacks.delete(requestId);
          reject(new Error(`Callback ${requestId} timed out`));
        }
      }, CALLBACK_TIMEOUT_MS);
      pendingCallbacks.set(requestId, { resolve, reject, timeoutId });

      self.postMessage({
        action: WorkerAction.EXECUTE_CALLBACK,
        callbackType: CallbackType.INSERT_KEY,
        args: [pubKey, secretKey],
        requestId,
      });
    });
  },
  sign: async (pubKey, signingInputs) => {
    return new Promise((resolve, reject) => {
      const requestId = `${CallbackType.SIGN}-${crypto.randomUUID()}`;
      const timeoutId = setTimeout(() => {
        if (pendingCallbacks.has(requestId)) {
          pendingCallbacks.delete(requestId);
          reject(new Error(`Callback ${requestId} timed out`));
        }
      }, CALLBACK_TIMEOUT_MS);
      pendingCallbacks.set(requestId, { resolve, reject, timeoutId });

      self.postMessage({
        action: WorkerAction.EXECUTE_CALLBACK,
        callbackType: CallbackType.SIGN,
        args: [pubKey, signingInputs],
        requestId,
      });
    });
  },
};

// Define a mapping from method names to handler functions.
const methodHandlers = {
  [MethodName.NEW_WALLET]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [walletStorageModeStr, mutable, authSchemeId, seed] = args;
    const walletStorageMode =
      wasm.AccountStorageMode.tryFromStr(walletStorageModeStr);
    const wallet = await wasmWebClient.newWallet(
      walletStorageMode,
      mutable,
      authSchemeId,
      seed
    );
    const serializedWallet = wallet.serialize();
    return serializedWallet.buffer;
  },
  [MethodName.NEW_FAUCET]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [
      faucetStorageModeStr,
      nonFungible,
      tokenSymbol,
      decimals,
      maxSupplyStr,
      authSchemeId,
    ] = args;
    const faucetStorageMode =
      wasm.AccountStorageMode.tryFromStr(faucetStorageModeStr);
    const maxSupply = BigInt(maxSupplyStr);
    const faucet = await wasmWebClient.newFaucet(
      faucetStorageMode,
      nonFungible,
      tokenSymbol,
      decimals,
      maxSupply,
      authSchemeId
    );
    const serializedFaucet = faucet.serialize();
    return serializedFaucet.buffer;
  },
  [MethodName.SYNC_STATE]: async () => {
    // Call the internal WASM method (sync lock is handled at the JS wrapper level)
    const syncSummary = await wasmWebClient.syncStateImpl();
    const serializedSyncSummary = syncSummary.serialize();
    return serializedSyncSummary.buffer;
  },
  [MethodName.EXECUTE_TRANSACTION]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [accountIdHex, serializedTransactionRequest] = args;
    const accountId = wasm.AccountId.fromHex(accountIdHex);
    const transactionRequestBytes = new Uint8Array(
      serializedTransactionRequest
    );
    const transactionRequest = wasm.TransactionRequest.deserialize(
      transactionRequestBytes
    );
    const result = await wasmWebClient.executeTransaction(
      accountId,
      transactionRequest
    );
    const serializedResult = result.serialize();
    return serializedResult.buffer;
  },
  [MethodName.PROVE_TRANSACTION]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [serializedTransactionResult, proverPayload] = args;
    const transactionResultBytes = new Uint8Array(serializedTransactionResult);
    const transactionResult = wasm.TransactionResult.deserialize(
      transactionResultBytes
    );

    const prover = proverPayload
      ? wasm.TransactionProver.deserialize(proverPayload)
      : undefined;

    const proven = await wasmWebClient.proveTransaction(
      transactionResult,
      prover
    );
    const serializedProven = proven.serialize();
    return serializedProven.buffer;
  },
  [MethodName.SUBMIT_NEW_TRANSACTION]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [accountIdHex, serializedTransactionRequest] = args;
    const accountId = wasm.AccountId.fromHex(accountIdHex);
    const transactionRequestBytes = new Uint8Array(
      serializedTransactionRequest
    );
    const transactionRequest = wasm.TransactionRequest.deserialize(
      transactionRequestBytes
    );

    const result = await wasmWebClient.executeTransaction(
      accountId,
      transactionRequest
    );

    const transactionId = result.id().toHex();

    const proven = await wasmWebClient.proveTransaction(result);
    const submissionHeight = await wasmWebClient.submitProvenTransaction(
      proven,
      result
    );
    const transactionUpdate = await wasmWebClient.applyTransaction(
      result,
      submissionHeight
    );

    return {
      transactionId,
      submissionHeight,
      serializedTransactionResult: result.serialize().buffer,
      serializedTransactionUpdate: transactionUpdate.serialize().buffer,
    };
  },
  [MethodName.SUBMIT_NEW_TRANSACTION_WITH_PROVER]: async (args) => {
    const wasm = await getWasmOrThrow();
    const [accountIdHex, serializedTransactionRequest, proverPayload] = args;
    const accountId = wasm.AccountId.fromHex(accountIdHex);
    const transactionRequestBytes = new Uint8Array(
      serializedTransactionRequest
    );
    const transactionRequest = wasm.TransactionRequest.deserialize(
      transactionRequestBytes
    );

    // Deserialize the prover from the serialized payload
    const prover = proverPayload
      ? wasm.TransactionProver.deserialize(proverPayload)
      : undefined;

    const result = await wasmWebClient.executeTransaction(
      accountId,
      transactionRequest
    );

    const transactionId = result.id().toHex();

    const proven = await wasmWebClient.proveTransaction(result, prover);
    const submissionHeight = await wasmWebClient.submitProvenTransaction(
      proven,
      result
    );
    const transactionUpdate = await wasmWebClient.applyTransaction(
      result,
      submissionHeight
    );

    return {
      transactionId,
      submissionHeight,
      serializedTransactionResult: result.serialize().buffer,
      serializedTransactionUpdate: transactionUpdate.serialize().buffer,
    };
  },
};

// Add mock methods to the handler mapping.
methodHandlers[MethodName.SYNC_STATE_MOCK] = async (args) => {
  let [serializedMockChain, serializedMockNoteTransportNode] = args;
  serializedMockChain = new Uint8Array(serializedMockChain);
  serializedMockNoteTransportNode = serializedMockNoteTransportNode
    ? new Uint8Array(serializedMockNoteTransportNode)
    : null;
  await wasmWebClient.createMockClient(
    wasmSeed,
    serializedMockChain,
    serializedMockNoteTransportNode
  );

  return await methodHandlers[MethodName.SYNC_STATE]();
};

methodHandlers[MethodName.SUBMIT_NEW_TRANSACTION_MOCK] = async (args) => {
  const wasm = await getWasmOrThrow();
  let serializedMockNoteTransportNode = args.pop();
  let serializedMockChain = args.pop();
  serializedMockChain = new Uint8Array(serializedMockChain);
  serializedMockNoteTransportNode = serializedMockNoteTransportNode
    ? new Uint8Array(serializedMockNoteTransportNode)
    : null;

  wasmWebClient = new wasm.WebClient();
  await wasmWebClient.createMockClient(
    wasmSeed,
    serializedMockChain,
    serializedMockNoteTransportNode
  );

  const result = await methodHandlers[MethodName.SUBMIT_NEW_TRANSACTION](args);

  return {
    transactionId: result.transactionId,
    submissionHeight: result.submissionHeight,
    serializedTransactionResult: result.serializedTransactionResult,
    serializedTransactionUpdate: result.serializedTransactionUpdate,
    serializedMockChain: wasmWebClient.serializeMockChain().buffer,
    serializedMockNoteTransportNode:
      wasmWebClient.serializeMockNoteTransportNode().buffer,
  };
};

methodHandlers[MethodName.SUBMIT_NEW_TRANSACTION_WITH_PROVER_MOCK] = async (
  args
) => {
  const wasm = await getWasmOrThrow();
  let serializedMockNoteTransportNode = args.pop();
  let serializedMockChain = args.pop();
  serializedMockChain = new Uint8Array(serializedMockChain);
  serializedMockNoteTransportNode = serializedMockNoteTransportNode
    ? new Uint8Array(serializedMockNoteTransportNode)
    : null;

  wasmWebClient = new wasm.WebClient();
  await wasmWebClient.createMockClient(
    wasmSeed,
    serializedMockChain,
    serializedMockNoteTransportNode
  );

  const result =
    await methodHandlers[MethodName.SUBMIT_NEW_TRANSACTION_WITH_PROVER](args);

  return {
    transactionId: result.transactionId,
    submissionHeight: result.submissionHeight,
    serializedTransactionResult: result.serializedTransactionResult,
    serializedTransactionUpdate: result.serializedTransactionUpdate,
    serializedMockChain: wasmWebClient.serializeMockChain().buffer,
    serializedMockNoteTransportNode:
      wasmWebClient.serializeMockNoteTransportNode().buffer,
  };
};

/**
 * Process a single message event.
 */
async function processMessage(event) {
  const { action, args, methodName, requestId } = event.data;
  try {
    if (action === WorkerAction.INIT) {
      const [
        rpcUrl,
        noteTransportUrl,
        seed,
        storeName,
        hasGetKeyCb,
        hasInsertKeyCb,
        hasSignCb,
        logLevel,
      ] = args;
      const wasm = await getWasmOrThrow();

      if (logLevel) {
        wasm.setupLogging(logLevel);
      }

      wasmWebClient = new wasm.WebClient();

      // Check if any callbacks are provided
      const useExternalKeystore = hasGetKeyCb || hasInsertKeyCb || hasSignCb;

      if (useExternalKeystore) {
        // Use callback proxies that communicate with the main thread
        await wasmWebClient.createClientWithExternalKeystore(
          rpcUrl,
          noteTransportUrl,
          seed,
          storeName,
          hasGetKeyCb ? callbackProxies.getKey : undefined,
          hasInsertKeyCb ? callbackProxies.insertKey : undefined,
          hasSignCb ? callbackProxies.sign : undefined
        );
      } else {
        await wasmWebClient.createClient(
          rpcUrl,
          noteTransportUrl,
          seed,
          storeName
        );
      }

      wasmSeed = seed;
      ready = true;
      self.postMessage({ ready: true });
      return;
    } else if (action === WorkerAction.INIT_MOCK) {
      const [seed, logLevel] = args;
      const wasm = await getWasmOrThrow();

      if (logLevel) {
        wasm.setupLogging(logLevel);
      }

      wasmWebClient = new wasm.WebClient();
      await wasmWebClient.createMockClient(seed);

      wasmSeed = seed;
      ready = true;
      self.postMessage({ ready: true });
      return;
    } else if (action === WorkerAction.CALL_METHOD) {
      if (!ready) {
        throw new Error("Worker is not ready. Please initialize first.");
      }
      if (!wasmWebClient) {
        throw new Error("WebClient not initialized in worker.");
      }
      // Look up the handler from the mapping.
      const handler = methodHandlers[methodName];
      if (!handler) {
        throw new Error(`Unsupported method: ${methodName}`);
      }
      const result = await handler(args);
      self.postMessage({ requestId, result, methodName });
      return;
    } else {
      throw new Error(`Unsupported action: ${action}`);
    }
  } catch (error) {
    const serializedError = serializeError(error);
    console.error(
      "WORKER: Error occurred - %s",
      serializedError.message,
      error
    );
    self.postMessage({ requestId, error: serializedError, methodName });
  }
}

/**
 * Process messages one at a time from the messageQueue.
 */
async function processQueue() {
  if (processing || messageQueue.length === 0) return;
  processing = true;
  const event = messageQueue.shift();
  try {
    await processMessage(event);
  } finally {
    processing = false;
    processQueue(); // Process next message in queue.
  }
}

// Enqueue incoming messages and process them sequentially.
self.onmessage = (event) => {
  if (
    event.data.callbackRequestId &&
    pendingCallbacks.has(event.data.callbackRequestId)
  ) {
    const { callbackRequestId, callbackResult, callbackError } = event.data;
    const { resolve, reject, timeoutId } = pendingCallbacks.get(callbackRequestId);
    clearTimeout(timeoutId);
    pendingCallbacks.delete(callbackRequestId);
    if (!callbackError) {
      resolve(callbackResult);
    } else {
      reject(new Error(callbackError));
    }
    return;
  }
  messageQueue.push(event);
  processQueue();
};

// Immediately signal that the worker script has loaded.
// This tells the main thread that the file is fully loaded before sending the "init" message.
self.postMessage({ loaded: true });
