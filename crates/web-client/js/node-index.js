/**
 * Node.js entry point for @miden-sdk/miden-sdk.
 *
 * Loaded automatically when Node.js resolves the package import
 * (via the "node" condition in package.json exports).
 *
 * Provides the same API as the browser entry point (index.js),
 * backed by a native napi addon with SQLite storage.
 */

import { loadNativeModule } from "./node/loader.js";
import { createSdkWrapper } from "./node/napi-compat.js";
import {
  createWasmWebClient,
  createMockWasmWebClient,
} from "./node/client-factory.js";
import { MidenClient } from "./client.js";
import {
  createP2IDNote,
  createP2IDENote,
  buildSwapTag,
  _setWasm as _setStandaloneWasm,
  _setWebClient as _setStandaloneWebClient,
} from "./standalone.js";

// ── Initialization ───────────────────────────────────────────────────

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
  _setStandaloneWasm(_wrappedSdk);
  _setStandaloneWebClient(_WasmWebClient);

  _initialized = true;
}

// Initialize on import
ensureInitialized();

// ── Enum constants (matching browser entry point) ────────────────────

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

// Internal exports (matching browser entry point)
export {
  _WasmWebClient as WasmWebClient,
  _MockWasmWebClient as MockWasmWebClient,
};

// Re-export all napi SDK types (equivalent to browser's `export * from "../Cargo.toml"`)
export function getNativeModule() {
  ensureInitialized();
  return _rawSdk;
}

export function getWrappedSdk() {
  ensureInitialized();
  return _wrappedSdk;
}
