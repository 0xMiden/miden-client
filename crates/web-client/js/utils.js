/**
 * Shared utility functions for the MidenClient resource classes.
 * Each function accepts a `wasm` parameter (the WASM module) for constructing typed objects.
 */

/**
 * Resolves an AccountRef (string | Account | AccountId) to an AccountId.
 *
 * - Strings starting with `0x`/`0X` are parsed as hex via `AccountId.fromHex()`.
 * - Other strings are parsed as bech32 via `AccountId.fromBech32()`.
 * - Objects with an `.id()` method (Account) are resolved by calling `.id()`.
 * - Otherwise, the value is assumed to be an AccountId pass-through.
 *
 * @param {string | Account | AccountId} ref - The account reference to resolve.
 * @param {object} wasm - The WASM module.
 * @returns {AccountId} The resolved AccountId.
 */
export function resolveAccountRef(ref, wasm) {
  if (ref == null) {
    throw new Error("Account reference cannot be null or undefined");
  }
  if (typeof ref === "string") {
    if (ref.startsWith("0x") || ref.startsWith("0X")) {
      return wasm.AccountId.fromHex(ref);
    }
    return wasm.AccountId.fromBech32(ref);
  }
  if (ref && typeof ref.id === "function") {
    return ref.id();
  }
  return ref;
}

/**
 * Resolves an AccountRef to a WASM Address object.
 *
 * - Strings starting with bech32 prefixes (`m`) are parsed via `Address.fromBech32()`.
 * - Strings starting with `0x`/`0X` are parsed as hex AccountId, then wrapped in Address.
 * - Account objects are resolved via `.id()` then wrapped in Address.
 * - AccountId objects are wrapped in Address directly.
 *
 * @param {string | Account | AccountId} ref - The account reference to resolve.
 * @param {object} wasm - The WASM module.
 * @returns {Address} The resolved Address.
 */
export function resolveAddress(ref, wasm) {
  if (typeof ref === "string") {
    if (ref.startsWith("0x") || ref.startsWith("0X")) {
      const accountId = wasm.AccountId.fromHex(ref);
      return wasm.Address.fromAccountId(accountId, undefined);
    }
    return wasm.Address.fromBech32(ref);
  }
  if (ref && typeof ref.id === "function") {
    const accountId = ref.id();
    return wasm.Address.fromAccountId(accountId, undefined);
  }
  return wasm.Address.fromAccountId(ref, undefined);
}

/**
 * Resolves a NoteVisibility string to a WASM NoteType value.
 *
 * @param {string | undefined} type - "public" or "private". Defaults to "public".
 * @param {object} wasm - The WASM module.
 * @returns {number} The NoteType enum value.
 */
export function resolveNoteType(type, wasm) {
  if (type === "private") {
    return wasm.NoteType.Private;
  }
  return wasm.NoteType.Public;
}

/**
 * Resolves a storage mode string to a WASM AccountStorageMode instance.
 *
 * @param {string | undefined} mode - "private", "public", or "network". Defaults to "private".
 * @param {object} wasm - The WASM module.
 * @returns {AccountStorageMode} The storage mode instance.
 */
export function resolveStorageMode(mode, wasm) {
  switch (mode) {
    case "public":
      return wasm.AccountStorageMode.public();
    case "network":
      return wasm.AccountStorageMode.network();
    case "private":
    default:
      return wasm.AccountStorageMode.private();
  }
}

/**
 * Resolves an auth scheme string to a WASM AuthScheme enum value.
 *
 * @param {string | undefined} scheme - "falcon" or "ecdsa". Defaults to "falcon".
 * @param {object} wasm - The WASM module.
 * @returns {number} The AuthScheme enum value.
 */
export function resolveAuthScheme(scheme, wasm) {
  if (scheme === "ecdsa") {
    return wasm.AuthScheme.AuthEcdsaK256Keccak;
  }
  return wasm.AuthScheme.AuthRpoFalcon512;
}

/**
 * Hashes a seed value. Strings are hashed via SHA-256 to produce a 32-byte Uint8Array.
 * Uint8Array values are passed through unchanged.
 *
 * @param {string | Uint8Array} seed - The seed to hash.
 * @returns {Promise<Uint8Array>} The hashed seed.
 */
export async function hashSeed(seed) {
  if (seed instanceof Uint8Array) {
    return seed;
  }
  if (typeof seed === "string") {
    const encoded = new TextEncoder().encode(seed);
    const hash = await crypto.subtle.digest("SHA-256", encoded);
    return new Uint8Array(hash);
  }
  throw new TypeError(
    `Invalid seed type: expected string or Uint8Array, got ${typeof seed}`
  );
}
