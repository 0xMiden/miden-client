import loadWasm from "../../dist/wasm.js";
const wasm = await loadWasm();
import { MethodName, WorkerAction } from "../constants.js";

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

// Define a mapping from method names to handler functions.
const methodHandlers = {
  [MethodName.NEW_WALLET]: async (args) => {
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
    const syncSummary = await wasmWebClient.syncState();
    const serializedSyncSummary = syncSummary.serialize();
    return serializedSyncSummary.buffer;
  },
  [MethodName.EXECUTE_TRANSACTION]: async (args) => {
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
    const [serializedTransactionResult, proverPayload] = args;
    const transactionResultBytes = new Uint8Array(serializedTransactionResult);
    const transactionResult = wasm.TransactionResult.deserialize(
      transactionResultBytes
    );

    let prover;
    if (proverPayload) {
      if (proverPayload === "local") {
        prover = wasm.TransactionProver.newLocalProver();
      } else if (proverPayload.startsWith("remote:")) {
        const endpoint = proverPayload.slice("remote:".length);
        if (!endpoint) {
          throw new Error("Remote prover requires an endpoint");
        }
        prover = wasm.TransactionProver.newRemoteProver(endpoint);
      } else {
        throw new Error("Invalid prover tag received in worker");
      }
    }

    const proven = await wasmWebClient.proveTransaction(
      transactionResult,
      prover
    );
    const serializedProven = proven.serialize();
    return serializedProven.buffer;
  },
  [MethodName.SUBMIT_NEW_TRANSACTION]: async (args) => {
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

/**
 * Process a single message event.
 */
async function processMessage(event) {
  const { action, args, methodName, requestId } = event.data;
  try {
    if (action === WorkerAction.INIT) {
      const [rpcUrl, noteTransportUrl, seed] = args;
      // Initialize the WASM WebClient.
      wasmWebClient = new wasm.WebClient();
      await wasmWebClient.createClient(rpcUrl, noteTransportUrl, seed);

      wasmSeed = seed;
      ready = true;
      // Signal that the worker is fully initialized.
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
      self.postMessage({ requestId, result });
      return;
    } else {
      throw new Error(`Unsupported action: ${action}`);
    }
  } catch (error) {
    console.error(`WORKER: Error occurred - ${error}`);
    self.postMessage({ requestId, error: error });
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
  messageQueue.push(event);
  processQueue();
};

// Immediately signal that the worker script has loaded.
// This tells the main thread that the file is fully loaded before sending the "init" message.
self.postMessage({ loaded: true });
