/**
 * Node.js-specific client factories that match the browser SDK's interface.
 *
 * The browser SDK creates clients with IndexedDB store names.
 * Node.js uses SQLite file paths and filesystem keystores.
 * These factories bridge the difference so MidenClient.create() works on both.
 */
import path from "path";
import fs from "fs";
import os from "os";
import { wrapClient, normalizeArg } from "./napi-compat.js";

let _counter = 0;

function createTempDir(label) {
  const dir = path.join(
    os.tmpdir(),
    `miden-${label}-${process.pid}-${Date.now()}-${++_counter}`
  );
  fs.mkdirSync(path.join(dir, "keystore"), { recursive: true });
  return dir;
}

function normBytes(val) {
  if (val instanceof Uint8Array || Buffer.isBuffer(val)) return Array.from(val);
  return val;
}

/**
 * Creates the WasmWebClient factory for Node.js.
 *
 * Matches the browser interface:
 *   WasmWebClient.createClient(rpcUrl, noteTransportUrl, seed, storeName, debugMode)
 *   WasmWebClient.createClientWithExternalKeystore(rpcUrl, noteTransportUrl, seed, storeName, getKey, insertKey, sign, debugMode)
 *   WasmWebClient.buildSwapTag(...)
 *
 * @param {object} rawSdk - The raw napi SDK module.
 * @param {object} [options]
 * @param {string} [options.dataDir] - Base directory for SQLite stores. Defaults to os.tmpdir().
 */
export function createWasmWebClient(rawSdk, options) {
  return {
    buildSwapTag: (...args) =>
      rawSdk.WebClient.buildSwapTag(...args.map(normalizeArg)),

    createClient: async (
      rpcUrl,
      noteTransportUrl,
      seed,
      storeName,
      debugMode
    ) => {
      const dir = options?.dataDir
        ? path.join(options.dataDir, storeName || "default")
        : createTempDir(storeName || "client");

      if (options?.dataDir) {
        fs.mkdirSync(path.join(dir, "keystore"), { recursive: true });
      }

      const client = new rawSdk.WebClient();
      await client.createClient(
        rpcUrl ?? null,
        noteTransportUrl ?? null,
        normBytes(seed) ?? null,
        path.join(dir, `${storeName || "store"}.db`),
        path.join(dir, "keystore"),
        debugMode ?? false
      );
      return wrapClient(client, storeName);
    },

    createClientWithExternalKeystore: async (
      rpcUrl,
      noteTransportUrl,
      seed,
      storeName,
      _getKey,
      _insertKey,
      _sign,
      debugMode
    ) => {
      // Node.js uses filesystem keystore -- external keystore callbacks are not supported.
      // Fall back to regular client creation.
      const dir = options?.dataDir
        ? path.join(options.dataDir, storeName || "default")
        : createTempDir(storeName || "client");

      if (options?.dataDir) {
        fs.mkdirSync(path.join(dir, "keystore"), { recursive: true });
      }

      const client = new rawSdk.WebClient();
      await client.createClient(
        rpcUrl ?? null,
        noteTransportUrl ?? null,
        normBytes(seed) ?? null,
        path.join(dir, `${storeName || "store"}.db`),
        path.join(dir, "keystore"),
        debugMode ?? false
      );
      return wrapClient(client, storeName);
    },
  };
}

/**
 * Creates the MockWasmWebClient factory for Node.js.
 *
 * Matches the browser interface:
 *   MockWasmWebClient.createClient(serializedMockChain, serializedNoteTransport, seed)
 *
 * @param {object} rawSdk - The raw napi SDK module.
 */
export function createMockWasmWebClient(rawSdk) {
  return {
    createClient: async (
      serializedMockChain,
      serializedNoteTransport,
      seed
    ) => {
      const dir = createTempDir("mock");
      const client = new rawSdk.WebClient();
      await client.createMockClient(
        path.join(dir, "store.db"),
        path.join(dir, "keystore"),
        normBytes(seed) ?? null,
        normBytes(serializedMockChain) ?? null,
        normBytes(serializedNoteTransport) ?? null
      );
      return wrapClient(client, "mock");
    },
  };
}
