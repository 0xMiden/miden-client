/**
 * @miden-sdk/node - Miden Client SDK for Node.js
 *
 * Provides the same MidenClient API as the browser SDK (@miden-sdk/miden-sdk),
 * backed by a native napi addon with SQLite storage.
 *
 * Usage:
 *   import { MidenClient, AccountType } from "@miden-sdk/node";
 *   const client = await MidenClient.create({ rpcUrl: "testnet" });
 */

import { loadNativeModule } from "./loader.js";
import { createSdkWrapper } from "./napi-compat.js";
import { createWasmWebClient, createMockWasmWebClient } from "./client-factory.js";
import { MidenClient } from "./js/client.js";
import {
  createP2IDNote,
  createP2IDENote,
  buildSwapTag,
  _setWasm,
  _setWebClient,
} from "./js/standalone.js";

// ── Lazy initialization ──────────────────────────────────────────────

let _initialized = false;
let _rawSdk = null;
let _wrappedSdk = null;
let _WasmWebClient = null;
let _MockWasmWebClient = null;

function ensureInitialized() {
  if (_initialized) return;

  _rawSdk = loadNativeModule();
  _wrappedSdk = createSdkWrapper(_rawSdk);
  _WasmWebClient = createWasmWebClient(_rawSdk);
  _MockWasmWebClient = createMockWasmWebClient(_rawSdk);

  // Wire MidenClient statics
  MidenClient._WasmWebClient = _WasmWebClient;
  MidenClient._MockWasmWebClient = _MockWasmWebClient;
  MidenClient._getWasmOrThrow = async () => _wrappedSdk;

  // Wire standalone functions
  _setWasm(_wrappedSdk);
  _setWebClient(_WasmWebClient);

  _initialized = true;
}

// Initialize on import
ensureInitialized();

// ── Enum constants (matching browser SDK) ────────────────────────────

export const AccountType = Object.freeze({
  MutableWallet: "MutableWallet",
  ImmutableWallet: "ImmutableWallet",
  FungibleFaucet: "FungibleFaucet",
  ImmutableContract: "ImmutableContract",
  MutableContract: "MutableContract",
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

// ── Re-exports ───────────────────────────────────────────────────────

export { MidenClient };
export { createP2IDNote, createP2IDENote, buildSwapTag };

// Re-export the raw SDK module for advanced usage
export function getNativeModule() {
  ensureInitialized();
  return _rawSdk;
}

// Re-export the wrapped SDK (with normalized classes) for advanced usage
export function getWrappedSdk() {
  ensureInitialized();
  return _wrappedSdk;
}
