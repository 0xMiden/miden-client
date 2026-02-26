import { AccountsResource } from "./resources/accounts.js";
import { TransactionsResource } from "./resources/transactions.js";
import { NotesResource } from "./resources/notes.js";
import { TagsResource } from "./resources/tags.js";
import { SettingsResource } from "./resources/settings.js";
import { hashSeed } from "./utils.js";

/**
 * MidenClient wraps the existing proxy-wrapped WebClient with a resource-based API.
 *
 * Resource classes receive the proxy client and call its methods, handling all type
 * conversions (string -> AccountId, number -> BigInt, string -> enum).
 */
export class MidenClient {
  // Injected by index.js to resolve circular imports
  static _WasmWebClient = null;
  static _MockWasmWebClient = null;
  static _getWasmOrThrow = null;

  #inner;
  #getWasm;
  #terminated = false;
  #defaultProver = null;
  #isMock = false;

  constructor(inner, getWasm, defaultProver) {
    this.#inner = inner;
    this.#getWasm = getWasm;
    this.#defaultProver = defaultProver ?? null;

    this.accounts = new AccountsResource(inner, getWasm, this);
    this.transactions = new TransactionsResource(inner, getWasm, this);
    this.notes = new NotesResource(inner, getWasm, this);
    this.tags = new TagsResource(inner, getWasm, this);
    this.settings = new SettingsResource(inner, getWasm, this);
  }

  /**
   * Creates and initializes a new MidenClient.
   *
   * @param {ClientOptions} [options] - Client configuration options.
   * @returns {Promise<MidenClient>} A fully initialized client.
   */
  static async create(options) {
    const getWasm = MidenClient._getWasmOrThrow;
    const WebClientClass = MidenClient._WasmWebClient;

    if (!WebClientClass || !getWasm) {
      throw new Error(
        "MidenClient not initialized. Import from the SDK package entry point."
      );
    }

    const seed = options?.seed ? await hashSeed(options.seed) : undefined;

    // Resolve passkey encryption → keystore callbacks (before the keystore branch)
    if (options?.passkeyEncryption && options?.keystore) {
      console.warn(
        "Both passkeyEncryption and keystore provided; keystore takes precedence."
      );
    }
    if (options?.passkeyEncryption && !options?.keystore) {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const passkeyOpts =
        typeof options.passkeyEncryption === "object"
          ? options.passkeyEncryption
          : {};
      const storeName = options?.storeName || "default";
      const result = await createPasskeyKeystore(storeName, passkeyOpts);
      options = {
        ...options,
        keystore: { getKey: result.getKey, insertKey: result.insertKey },
      };
    }

    let inner;
    if (options?.keystore) {
      inner = await WebClientClass.createClientWithExternalKeystore(
        options?.rpcUrl,
        options?.noteTransportUrl,
        seed,
        options?.storeName,
        options.keystore.getKey,
        options.keystore.insertKey,
        options.keystore.sign
      );
    } else {
      inner = await WebClientClass.createClient(
        options?.rpcUrl,
        options?.noteTransportUrl,
        seed,
        options?.storeName
      );
    }

    let defaultProver = null;
    if (options?.proverUrl) {
      const wasm = await getWasm();
      defaultProver = wasm.TransactionProver.newRemoteProver(
        options.proverUrl,
        undefined
      );
    }

    const client = new MidenClient(inner, getWasm, defaultProver);

    if (options?.autoSync) {
      await client.sync();
    }

    return client;
  }

  /**
   * Creates a client preconfigured for testnet use.
   * Defaults to autoSync: true.
   *
   * @param {object} [options] - Options (autoSync can be overridden).
   * @returns {Promise<MidenClient>} A fully initialized testnet client.
   */
  static async createTestnet(options) {
    return MidenClient.create({
      ...options,
      autoSync: options?.autoSync ?? true,
    });
  }

  /**
   * Creates a mock client for testing.
   *
   * @param {MockOptions} [options] - Mock client options.
   * @returns {Promise<MidenClient>} A mock client.
   */
  static async createMock(options) {
    const getWasm = MidenClient._getWasmOrThrow;
    const MockWebClientClass = MidenClient._MockWasmWebClient;

    if (!MockWebClientClass || !getWasm) {
      throw new Error(
        "MidenClient not initialized. Import from the SDK package entry point."
      );
    }

    const seed = options?.seed ? await hashSeed(options.seed) : undefined;

    const inner = await MockWebClientClass.createClient(
      options?.serializedMockChain,
      options?.serializedNoteTransport,
      seed
    );

    const client = new MidenClient(inner, getWasm, null);
    client.#isMock = true;
    return client;
  }

  /** Returns the client-level default prover (set from ClientOptions.proverUrl). */
  get defaultProver() {
    return this.#defaultProver;
  }

  /**
   * Syncs the client state with the Miden node.
   *
   * @param {object} [opts] - Sync options.
   * @param {number} [opts.timeout] - Timeout in milliseconds (0 = no timeout).
   * @returns {Promise<SyncSummary>} The sync summary.
   */
  async sync(opts) {
    this.assertNotTerminated();
    return await this.#inner.syncStateWithTimeout(opts?.timeout ?? 0);
  }

  /**
   * Returns the current sync height.
   *
   * @returns {Promise<number>} The current sync height.
   */
  async getSyncHeight() {
    this.assertNotTerminated();
    return await this.#inner.getSyncHeight();
  }

  /**
   * Terminates the underlying Web Worker. After this, all method calls will throw.
   */
  terminate() {
    this.#terminated = true;
    this.#inner.terminate?.();
  }

  [Symbol.dispose]() {
    this.terminate();
  }

  async [Symbol.asyncDispose]() {
    this.terminate();
  }

  /**
   * Exports the client store as a versioned snapshot.
   *
   * @returns {Promise<StoreSnapshot>} The store snapshot.
   */
  async exportStore() {
    this.assertNotTerminated();
    const data = await this.#inner.exportStore();
    return { version: 1, data };
  }

  /**
   * Imports a previously exported store snapshot.
   *
   * @param {StoreSnapshot} snapshot - The store snapshot to import.
   */
  async importStore(snapshot) {
    this.assertNotTerminated();
    if (!snapshot || snapshot.version !== 1) {
      throw new Error(
        `Unsupported store snapshot version: ${snapshot?.version}. Expected version 1.`
      );
    }
    // Second arg is the store password (empty string = no encryption)
    await this.#inner.forceImportStore(snapshot.data, "");
  }

  // ── Mock-only methods ──

  /** Advances the mock chain by one block. Only available on mock clients. */
  proveBlock() {
    this.assertNotTerminated();
    this.#assertMock("proveBlock");
    return this.#inner.proveBlock();
  }

  /** Returns true if this client uses a mock chain. */
  usesMockChain() {
    return this.#isMock;
  }

  /** Serializes the mock chain state for snapshot/restore in tests. */
  serializeMockChain() {
    this.assertNotTerminated();
    this.#assertMock("serializeMockChain");
    return this.#inner.serializeMockChain();
  }

  /** Serializes the mock note transport node state. */
  serializeMockNoteTransportNode() {
    this.assertNotTerminated();
    this.#assertMock("serializeMockNoteTransportNode");
    return this.#inner.serializeMockNoteTransportNode();
  }

  // ── Internal ──

  /** @internal Throws if the client has been terminated. */
  assertNotTerminated() {
    if (this.#terminated) {
      throw new Error("Client terminated");
    }
  }

  #assertMock(method) {
    if (!this.#isMock) {
      throw new Error(`${method}() is only available on mock clients`);
    }
  }
}
