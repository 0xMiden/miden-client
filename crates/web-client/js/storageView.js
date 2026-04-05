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
/** @param {string} hex @param {typeof Word} WordClass @returns {Word | undefined} */
function hexToWord(hex, WordClass) {
  if (!hex || !WordClass) return undefined;
  try {
    return WordClass.fromHex(hex);
  } catch {
    return undefined;
  }
}

export class StorageView {
  #storage;
  #WordClass;

  /**
   * @param {AccountStorage} wasmStorage
   * @param {typeof Word} WordClass
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
    // Type detection + value retrieval in one pass.
    // We call getMapEntries to detect maps, but defer parsing the entries
    // until .entries is actually accessed (lazy). Only the first entry's
    // Word is parsed eagerly for the convenience methods (toNumber, etc.).
    const rawEntries = this.#storage.getMapEntries(slotName);
    if (rawEntries !== undefined && rawEntries !== null) {
      // StorageMap — parse only the first entry eagerly
      const firstWord =
        rawEntries.length > 0
          ? hexToWord(rawEntries[0].value, this.#WordClass)
          : undefined;
      return new StorageResult(firstWord, true, rawEntries, this.#WordClass);
    }

    // Value slot — use raw getItem
    const word = this.#storage.getItem(slotName);
    if (!word) return undefined;
    return new StorageResult(word, false, undefined, this.#WordClass);
  }

  /**
   * Returns the value for a key in a StorageMap slot.
   * Delegates directly to the raw WASM method.
   *
   * @param {string} slotName
   * @param {Word} key
   * @returns {Word | undefined}
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
   * @returns {Word | undefined}
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
  #rawEntries; // Raw JsStorageMapEntry[] from WASM — parsed lazily
  #parsedEntries; // Parsed entries with Word objects — created on first .entries access
  #WordClass;

  /**
   * @param {Word | undefined} word — the primary Word value (first entry for maps)
   * @param {boolean} isMap — whether this came from a StorageMap slot
   * @param {Array | undefined} rawEntries — raw WASM entries (parsed lazily on .entries access)
   * @param {typeof Word} WordClass — Word constructor for hex parsing
   */
  constructor(word, isMap, rawEntries, WordClass) {
    this.#word = word;
    this.#isMap = isMap;
    this.#rawEntries = rawEntries;
    this.#WordClass = WordClass;
  }

  /** True if this slot is a StorageMap. */
  get isMap() {
    return this.#isMap;
  }

  /**
   * All entries from a StorageMap slot (lazily parsed on first access).
   * Each entry has { key: string (hex), value: string (hex), word: Word | undefined }.
   * Returns undefined for Value slots.
   */
  get entries() {
    if (!this.#isMap) return undefined;
    if (this.#parsedEntries) return this.#parsedEntries;
    if (!this.#rawEntries) return [];

    // Parse entries lazily — only when the user actually accesses .entries
    this.#parsedEntries = this.#rawEntries.map((e) => ({
      key: e.key,
      value: e.value,
      word: hexToWord(e.value, this.#WordClass),
    }));
    this.#rawEntries = undefined; // Free raw entries
    return this.#parsedEntries;
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
   * Returns all four Felts of the stored Word as an array.
   * Pass-through to Word.toFelts() — ensures code that expects a Word-like
   * object (e.g., `result.toFelts()[0].asInt()`) works on StorageResult.
   * @returns {Felt[]}
   */
  toFelts() {
    if (!this.#word) return [];
    return this.#word.toFelts();
  }

  /**
   * The first Felt of the stored Word.
   * Returns the WASM Felt object — use .asInt() to get its BigInt value.
   * @returns {Felt | undefined}
   */
  felt() {
    if (!this.#word) return undefined;
    const felts = this.#word.toFelts();
    return felts?.[0];
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
   * JSON serialization — returns the value as a string to avoid
   * precision loss for large u64 felt values.
   */
  toJSON() {
    return this.toBigInt().toString();
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
 * @param {Word} word
 * @returns {bigint}
 */
export function wordToBigInt(word) {
  try {
    const hex = word.toHex();
    // Word.toHex() returns "0x" + 64 hex chars (4 felts × 16 hex chars each).
    // Each felt is serialized as 8 little-endian bytes, so we take the first 16
    // hex chars (first felt) and reverse the byte pairs to get the integer value.
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
 * @param {Word} word
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
