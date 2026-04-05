/**
 * StorageView wraps the raw WASM AccountStorage to provide a developer-friendly
 * (and AI-agent-friendly) API.
 *
 * Key behavior: `getItem()` returns a `StorageResult` that works intuitively for
 * both Value and StorageMap slots. The result has `.toNumber()`, `.toHex()`, and
 * `.toString()` methods that do the right thing automatically. For StorageMap slots,
 * `.entries` provides access to all map entries.
 *
 * The raw WASM AccountStorage is still accessible via `.raw` for advanced use cases
 * that need the original behavior (e.g., comparing map commitment roots).
 */
export class StorageView {
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
   * Returns a StorageResult for the given slot.
   *
   * The result has convenience methods that work for both Value and StorageMap slots:
   * - `.toNumber()` — first felt as a JS number
   * - `.toBigInt()` — first felt as BigInt (full u64 precision)
   * - `.toHex()` — first felt's Word as hex string
   * - `.toString()` — renders as the number (works in JSX: {result})
   * - `.isMap` — true if this is a StorageMap slot
   * - `.entries` — all map entries (undefined for Value slots)
   * - `.word` — the underlying Word value
   *
   * For explicit key-based map reads, use `getMapItem(slotName, key)`.
   * For the raw commitment hash, use `raw.getItem(slotName)`.
   *
   * @param {string} slotName
   * @returns {StorageResult | undefined}
   */
  getItem(slotName) {
    // Check if this is a StorageMap by trying getMapEntries
    const entries = this.#storage.getMapEntries(slotName);
    if (entries !== undefined && entries !== null) {
      // StorageMap — build result from entries
      const parsedEntries = entries.map((e) => ({
        key: e.key,
        value: e.value,
        word: this.#hexToWord(e.value),
      }));
      const firstWord =
        parsedEntries.length > 0 ? parsedEntries[0].word : undefined;
      return new StorageResult(firstWord, true, parsedEntries);
    }

    // Value slot — use raw getItem
    const word = this.#storage.getItem(slotName);
    if (!word) return undefined;
    return new StorageResult(word, false, undefined);
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
   * Returns the commitment root of a storage slot as a Word.
   *
   * For Value slots, this is the stored Word itself.
   * For StorageMap slots, this is the Merkle root hash of the map — useful for:
   * - Verifying state hasn't changed between transactions
   * - Merkle inclusion proofs against the account state
   * - Comparing map state across accounts or sync cycles
   *
   * This is the raw protocol-level value. For reading stored data, use `getItem()`.
   *
   * @param {string} slotName
   * @returns {import("../Cargo.toml").Word | undefined}
   */
  getCommitment(slotName) {
    return this.#storage.getItem(slotName);
  }

  /**
   * Convenience: read the first felt of a storage slot as a JavaScript number.
   * Works for both Value and StorageMap slots.
   *
   * Note: Felts are u64-backed. Values above Number.MAX_SAFE_INTEGER (2^53 - 1)
   * will lose precision. Use `.toBigInt()` on the StorageResult for exact values.
   *
   * @param {string} slotName
   * @returns {number | undefined}
   */
  getNumber(slotName) {
    return this.getItem(slotName)?.toNumber();
  }

  /**
   * @param {string} hex
   * @returns {import("../Cargo.toml").Word | undefined}
   */
  #hexToWord(hex) {
    if (!hex || !this.#WordClass) return undefined;
    try {
      return this.#WordClass.fromHex(hex);
    } catch {
      return undefined;
    }
  }
}

/**
 * Result of reading a storage slot. Works for both Value and StorageMap slots.
 *
 * Provides a unified interface so code like `storage.getItem(name).toNumber()`
 * works regardless of the underlying slot type.
 *
 * For StorageMap slots, the convenience methods (toNumber, toHex, toBigInt)
 * operate on the first entry's value. The full map data is available via `.entries`.
 * Note: Miden storage maps are Merkle-based, so "first" is determined by key hash
 * order — deterministic for a given map state, but not meaningful as an ordering.
 */
export class StorageResult {
  #word;
  #isMap;
  #entries;

  /**
   * @param {import("../Cargo.toml").Word | undefined} word — the primary Word value
   * @param {boolean} isMap — whether this came from a StorageMap slot
   * @param {Array<{key: string, value: string, word: import("../Cargo.toml").Word | undefined}> | undefined} entries
   */
  constructor(word, isMap, entries) {
    this.#word = word;
    this.#isMap = isMap;
    this.#entries = entries;
  }

  /** True if this slot is a StorageMap. */
  get isMap() {
    return this.#isMap;
  }

  /**
   * All entries from a StorageMap slot.
   * Each entry has { key: string (hex), value: string (hex), word: Word | undefined }.
   * Returns undefined for Value slots.
   */
  get entries() {
    return this.#entries;
  }

  /**
   * The underlying Word value.
   * For Value slots: the stored Word.
   * For StorageMap slots: the first entry's value as a Word (or undefined if empty).
   */
  get word() {
    return this.#word;
  }

  /**
   * First felt as a JavaScript number.
   * WARNING: May lose precision for values > Number.MAX_SAFE_INTEGER (2^53 - 1).
   * Use .toBigInt() for exact large values.
   * @returns {number}
   */
  toNumber() {
    if (!this.#word) return 0;
    return wordToNumber(this.#word);
  }

  /**
   * First felt as a BigInt. Preserves full u64 precision.
   * @returns {bigint}
   */
  toBigInt() {
    if (!this.#word) return 0n;
    return wordToBigInt(this.#word);
  }

  /**
   * The Word's hex representation.
   * For Value slots: the stored Word hex.
   * For StorageMap slots: the first entry's value Word hex.
   * @returns {string}
   */
  toHex() {
    if (!this.#word) return "0x" + "0".repeat(64);
    return this.#word.toHex();
  }

  /**
   * Renders as the numeric value. Makes `{storageResult}` work in JSX.
   * @returns {string}
   */
  toString() {
    return String(this.toNumber());
  }

  /**
   * JSON serialization — returns the numeric value.
   */
  toJSON() {
    return this.toNumber();
  }

  /**
   * Allows `+result`, `result * 2`, etc. to work as expected.
   * @returns {number}
   */
  valueOf() {
    return this.toNumber();
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
