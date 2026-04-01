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

// Re-export all napi SDK types (equivalent to browser's `export * from "../Cargo.toml"`).
// Since we can't statically export dynamic napi bindings, we re-export common types explicitly
// and provide getNativeModule()/getWrappedSdk() for anything else.

export function getNativeModule() {
  ensureInitialized();
  return _rawSdk;
}

export function getWrappedSdk() {
  ensureInitialized();
  return _wrappedSdk;
}

// Re-export commonly used SDK types from the napi module.
// Uses a lazy getter pattern so the module is loaded on first access.
function _reexport(name) {
  return {
    get [name]() {
      ensureInitialized();
      return _wrappedSdk[name] ?? _rawSdk[name];
    },
  }[name];
}

// Account types
export const Account = /* @__PURE__ */ _reexport("Account");
export const AccountBuilder = /* @__PURE__ */ _reexport("AccountBuilder");
export const AccountComponent = /* @__PURE__ */ _reexport("AccountComponent");
export const AccountFile = /* @__PURE__ */ _reexport("AccountFile");
export const AccountHeader = /* @__PURE__ */ _reexport("AccountHeader");
export const AccountId = /* @__PURE__ */ _reexport("AccountId");
export const AccountInterface = /* @__PURE__ */ _reexport("AccountInterface");
export const AccountStorage = /* @__PURE__ */ _reexport("AccountStorage");
export const AccountStorageMode =
  /* @__PURE__ */ _reexport("AccountStorageMode");
export const AccountStorageRequirements = /* @__PURE__ */ _reexport(
  "AccountStorageRequirements"
);
export const Address = /* @__PURE__ */ _reexport("Address");

// Auth types
export const AuthSchemeNative = /* @__PURE__ */ _reexport("AuthScheme");
export const AuthSecretKey = /* @__PURE__ */ _reexport("AuthSecretKey");

// Crypto types
export const Felt = /* @__PURE__ */ _reexport("Felt");
export const Word = /* @__PURE__ */ _reexport("Word");
export const Rpo = /* @__PURE__ */ _reexport("Rpo");
export const Rpo256 = /* @__PURE__ */ _reexport("Rpo256");
export const PublicKey = /* @__PURE__ */ _reexport("PublicKey");
export const Signature = /* @__PURE__ */ _reexport("Signature");

// Asset types
export const FungibleAsset = /* @__PURE__ */ _reexport("FungibleAsset");

// Note types
export const Note = /* @__PURE__ */ _reexport("Note");
export const NoteAssets = /* @__PURE__ */ _reexport("NoteAssets");
export const NoteAttachment = /* @__PURE__ */ _reexport("NoteAttachment");
export const NoteExportFormat = /* @__PURE__ */ _reexport("NoteExportFormat");
export const NoteExecutionHint = /* @__PURE__ */ _reexport("NoteExecutionHint");
export const NoteFile = /* @__PURE__ */ _reexport("NoteFile");
export const NoteFilter = /* @__PURE__ */ _reexport("NoteFilter");
export const NoteFilterTypes = /* @__PURE__ */ _reexport("NoteFilterTypes");
export const NoteId = /* @__PURE__ */ _reexport("NoteId");
export const NoteMetadata = /* @__PURE__ */ _reexport("NoteMetadata");
export const NoteRecipient = /* @__PURE__ */ _reexport("NoteRecipient");
export const NoteScript = /* @__PURE__ */ _reexport("NoteScript");
export const NoteStorage = /* @__PURE__ */ _reexport("NoteStorage");
export const NoteTag = /* @__PURE__ */ _reexport("NoteTag");
export const NoteType = /* @__PURE__ */ _reexport("NoteType");
export const OutputNote = /* @__PURE__ */ _reexport("OutputNote");

// Transaction types
export const TransactionFilter = /* @__PURE__ */ _reexport("TransactionFilter");
export const TransactionProver = /* @__PURE__ */ _reexport("TransactionProver");
export const TransactionRequestBuilder = /* @__PURE__ */ _reexport(
  "TransactionRequestBuilder"
);

// Network types
export const NetworkId = /* @__PURE__ */ _reexport("NetworkId");
export const RpcClient = /* @__PURE__ */ _reexport("RpcClient");
export const Endpoint = /* @__PURE__ */ _reexport("Endpoint");

// Misc
export const AdviceMap = /* @__PURE__ */ _reexport("AdviceMap");
export const ForeignAccount = /* @__PURE__ */ _reexport("ForeignAccount");
export const Package = /* @__PURE__*/ _reexport("Package");
export const StorageMap = /* @__PURE__ */ _reexport("StorageMap");
export const StorageSlot = /* @__PURE__ */ _reexport("StorageSlot");
export const TokenSymbol = /* @__PURE__ */ _reexport("TokenSymbol");
