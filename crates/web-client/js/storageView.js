/**
 * StorageView wraps the raw WASM AccountStorage to provide a developer-friendly
 * (and AI-agent-friendly) API.
 *
 * Key behavior: `getItem()` on a StorageMap slot returns the first entry's value
 * instead of the map commitment hash. This makes the most common usage pattern
 * work correctly without requiring knowledge of `getMapItem`.
 *
 * Note on StorageMap ordering: Miden storage maps are Merkle-based, so iteration
 * order is determined by key hashes, not insertion order. "First entry" means the
 * first entry returned by the underlying iterator — this is deterministic for a
 * given map state but not meaningful as an ordering concept.
 *
 * The raw WASM AccountStorage is still accessible via `.raw` for advanced use cases
 * that need the original behavior (e.g., comparing map commitment roots).
 */
export class StorageView {
  /** @type {import("../Cargo.toml").AccountStorage} */
  #storage;
  #WordClass;

  /**
   * @param {import("../Cargo.toml").AccountStorage} wasmStorage
   * @param {typeof import("../Cargo.toml").Word} WordClass
   */
  constructor(wasmStorage, WordClass) {
    this.#storage = wasmStorage;
    this.#WordClass = WordClass;
  }

  /**
   * The raw WASM AccountStorage, for cases where you need the original
   * primitive behavior (e.g., reading map commitment roots via raw.getItem()).
   */
  get raw() {
    return this.#storage;
  }

  /**
   * Returns the commitment to the full account storage.
   */
  commitment() {
    return this.#storage.commitment();
  }

  /**
   * Returns the names of all storage slots on this account.
   * @returns {string[]}
   */
  getSlotNames() {
    return this.#storage.getSlotNames();
  }

  /**
   * Smart read: returns the actual stored value for both Value and StorageMap slots.
   *
   * - For Value slots: returns the stored Word directly.
   * - For StorageMap slots: returns the first entry's value (NOT the commitment hash).
   *
   * To read a specific key from a StorageMap, use `getMapItem(slotName, key)`.
   * To get the raw commitment hash of a StorageMap, use `raw.getItem(slotName)`.
   *
   * @param {string} slotName
   * @returns {import("../Cargo.toml").Word | undefined}
   */
  getItem(slotName) {
    // Try to get map entries — if it returns an array, this is a StorageMap
    const entries = this.#storage.getMapEntries(slotName);
    if (entries !== undefined && entries !== null) {
      // It's a StorageMap — return the first entry's value as a Word
      if (entries.length > 0) {
        return this.#parseEntryValue(entries[0]);
      }
      return undefined; // Empty map
    }

    // Not a map — use the raw getItem for Value slots
    return this.#storage.getItem(slotName);
  }

  /**
   * Returns the value for a key in a StorageMap slot.
   * Delegates directly to the raw WASM method.
   *
   * @param {string} slotName
   * @param {import("../Cargo.toml").Word} key
   * @returns {import("../Cargo.toml").Word | undefined}
   */
  getMapItem(slotName, key) {
    return this.#storage.getMapItem(slotName, key);
  }

  /**
   * Get all key-value pairs from a StorageMap slot.
   * Returns undefined if the slot isn't a map, or an empty array if the map is empty.
   */
  getMapEntries(slotName) {
    return this.#storage.getMapEntries(slotName);
  }

  /**
   * Convenience: read the first felt of a storage slot as a JavaScript number.
   * Works for both Value and StorageMap slots.
   *
   * Note: Felts are u64-backed. Values above Number.MAX_SAFE_INTEGER (2^53 - 1)
   * will lose precision. Use `wordToBigInt(storage.getItem(slotName))` for exact
   * large values.
   *
   * @param {string} slotName
   * @returns {number | undefined}
   */
  getNumber(slotName) {
    const word = this.getItem(slotName);
    if (!word) return undefined;
    return wordToNumber(word);
  }

  /**
   * Parse a JsStorageMapEntry's value hex string into a Word.
   * @param {{ key: string, value: string }} entry
   * @returns {import("../Cargo.toml").Word | undefined}
   */
  #parseEntryValue(entry) {
    if (!entry?.value || !this.#WordClass) return undefined;
    try {
      return this.#WordClass.fromHex(entry.value);
    } catch {
      return undefined;
    }
  }
}

/**
 * Convert a Word's first felt to a BigInt.
 * Uses BigInt to preserve full u64 precision (felts are u64-backed).
 * Handles the little-endian byte order of felt serialization.
 *
 * @param {import("../Cargo.toml").Word} word
 * @returns {bigint}
 */
export function wordToBigInt(word) {
  try {
    const hex = word.toHex();
    // First felt = first 16 hex chars after "0x", little-endian byte order
    const feltHex = hex.slice(2, 18);
    const bytes = feltHex.match(/../g);
    if (!bytes) return 0n;
    return BigInt("0x" + bytes.reverse().join(""));
  } catch {
    return 0n;
  }
}

/**
 * Convert a Word's first felt to a JavaScript number.
 * WARNING: May lose precision for values > Number.MAX_SAFE_INTEGER (2^53 - 1).
 * Use wordToBigInt() when exact large values matter.
 *
 * @param {import("../Cargo.toml").Word} word
 * @returns {number}
 */
export function wordToNumber(word) {
  return Number(wordToBigInt(word));
}

/**
 * Install the StorageView wrapper on Account.prototype.storage().
 * After this, `account.storage()` returns a StorageView instead of raw AccountStorage.
 *
 * @param {object} wasmModule — the loaded WASM module containing Account, Word, etc.
 */
export function installStorageView(wasmModule) {
  const AccountProto = wasmModule.Account?.prototype;
  if (!AccountProto || !AccountProto.storage) return;

  const originalStorage = AccountProto.storage;
  const WordClass = wasmModule.Word;

  AccountProto.storage = function () {
    const raw = originalStorage.call(this);
    return new StorageView(raw, WordClass);
  };
}
