import { AccountsResource } from "./resources/accounts.js";
import { TransactionsResource } from "./resources/transactions.js";
import { NotesResource } from "./resources/notes.js";
import { TagsResource } from "./resources/tags.js";
import { SettingsResource } from "./resources/settings.js";
import { CompilerResource } from "./resources/compiler.js";
import { KeystoreResource } from "./resources/keystore.js";
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
    this.compile = new CompilerResource(inner, getWasm, this);
    this.keystore = new KeystoreResource(inner, this);
  }

  /**
   * Creates and initializes a new MidenClient.
   *
   * If no `rpcUrl` is provided, defaults to testnet with full configuration
   * (RPC, prover, note transport, autoSync).
   *
   * @param {ClientOptions} [options] - Client configuration options.
   * @returns {Promise<MidenClient>} A fully initialized client.
   */
  static async create(options) {
    if (!options?.rpcUrl) {
      return MidenClient.createTestnet(options);
    }

    const getWasm = MidenClient._getWasmOrThrow;
    const WebClientClass = MidenClient._WasmWebClient;

    if (!WebClientClass || !getWasm) {
      throw new Error(
        "MidenClient not initialized. Import from the SDK package entry point."
      );
    }

    const seed = options?.seed ? await hashSeed(options.seed) : undefined;

    const rpcUrl = resolveRpcUrl(options?.rpcUrl);
    const noteTransportUrl = resolveNoteTransportUrl(options?.noteTransportUrl);

    let inner;
    if (options?.keystore) {
      inner = await WebClientClass.createClientWithExternalKeystore(
        rpcUrl,
        noteTransportUrl,
        seed,
        options?.storeName,
        options.keystore.getKey,
        options.keystore.insertKey,
        options.keystore.sign
      );
    } else {
      inner = await WebClientClass.createClient(
        rpcUrl,
        noteTransportUrl,
        seed,
        options?.storeName
      );
    }

    let defaultProver = null;
    if (options?.proverUrl) {
      const wasm = await getWasm();
      defaultProver = resolveProver(options.proverUrl, wasm);
    }

    const client = new MidenClient(inner, getWasm, defaultProver);

    if (options?.autoSync) {
      await client.sync();
    }

    return client;
  }

  /**
   * Creates a client preconfigured for testnet use.
   *
   * Defaults: rpcUrl "testnet", proverUrl "testnet", noteTransportUrl "testnet", autoSync true.
   * All defaults can be overridden via options.
   *
   * @param {ClientOptions} [options] - Options to override defaults.
   * @returns {Promise<MidenClient>} A fully initialized testnet client.
   */
  static async createTestnet(options) {
    return MidenClient.create({
      rpcUrl: "testnet",
      proverUrl: "testnet",
      noteTransportUrl: "testnet",
      autoSync: true,
      ...options,
    });
  }

  /**
   * Creates a client preconfigured for devnet use.
   *
   * Defaults: rpcUrl "devnet", proverUrl "devnet", noteTransportUrl "devnet", autoSync true.
   * All defaults can be overridden via options.
   *
   * @param {ClientOptions} [options] - Options to override defaults.
   * @returns {Promise<MidenClient>} A fully initialized devnet client.
   */
  static async createDevnet(options) {
    return MidenClient.create({
      rpcUrl: "devnet",
      proverUrl: "devnet",
      noteTransportUrl: "devnet",
      autoSync: true,
      ...options,
    });
  }

  /**
   * Resolves once the WASM module is initialized and safe to use.
   *
   * Idempotent and shared across callers: the underlying loader memoizes the
   * in-flight promise, so concurrent `ready()` calls await the same
   * initialization and post-init callers resolve immediately from a cached
   * module. Safe to call from `MidenProvider`, tutorial helpers, and any
   * other consumer simultaneously.
   *
   * Useful on the `/lazy` entry (e.g. Next.js / Capacitor), where no
   * top-level await runs at import time. On the default (eager) entry this
   * is redundant — importing the module already awaits WASM — but calling it
   * is still harmless.
   *
   * @returns {Promise<void>} Resolves when WASM is initialized.
   */
  static async ready() {
    const getWasm = MidenClient._getWasmOrThrow;
    if (!getWasm) {
      throw new Error(
        "MidenClient not initialized. Import from the SDK package entry point."
      );
    }
    await getWasm();
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
   * Resolves once every WASM call queued on this client before
   * `waitForIdle()` was called (execute, submit, prove, apply, sync,
   * account creation, proxy-fallback reads) has settled. Use this from
   * callers that need to perform a non-WASM-side action — e.g. clearing
   * an in-memory auth key on wallet lock — after the kernel finishes, so
   * its auth callback doesn't race with the key being cleared.
   *
   * Does NOT wait for calls enqueued after `waitForIdle()` returns —
   * intentional, so a caller can drain and proceed without being blocked
   * indefinitely by concurrent workload.
   *
   * Caveat for `syncState`: `syncStateWithTimeout` awaits the sync lock
   * (`acquireSyncLock`, which uses Web Locks) BEFORE enqueuing its WASM
   * call, so a `syncState` that is queued on the sync lock — but has
   * not yet begun its WASM phase — is not visible to `waitForIdle` and
   * will not be awaited. Other methods (`newWallet`, `executeTransaction`,
   * etc.) route through the queue synchronously on call and are always
   * observed.
   *
   * Safe to call at any time; resolves as soon as the client is idle.
   *
   * @returns {Promise<void>}
   */
  async waitForIdle() {
    this.assertNotTerminated();
    await this.#inner.waitForIdle();
  }

  /**
   * Returns the raw JS value that the most recent sign-callback invocation
   * threw, or `null` if the last sign call succeeded (or no call has
   * happened yet).
   *
   * Useful for recovering structured metadata (e.g. a `reason: 'locked'`
   * property) that the kernel-level `auth::request` diagnostic would
   * otherwise erase. Call immediately after catching a failed
   * `transactions.submit` / `transactions.send` / `transactions.consume`.
   *
   * @returns {any} The raw thrown value, or `null`.
   */
  lastAuthError() {
    this.assertNotTerminated();
    return this.#inner.lastAuthError();
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
   * Returns the identifier of the underlying store (e.g. IndexedDB database name, file path).
   *
   * @returns {string} The store identifier.
   */
  storeIdentifier() {
    this.assertNotTerminated();
    return this.#inner.storeIdentifier();
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

const RPC_URLS = {
  testnet: "https://rpc.testnet.miden.io",
  devnet: "https://rpc.devnet.miden.io",
  localhost: "http://localhost:57291",
  local: "http://localhost:57291",
};

/**
 * Resolves an rpcUrl shorthand or raw URL into a concrete endpoint string.
 *
 * @param {string | undefined} rpcUrl - "testnet", "devnet", "localhost", "local", or a raw URL.
 * @returns {string | undefined} A fully qualified URL, or undefined to use the SDK default.
 */
function resolveRpcUrl(rpcUrl) {
  if (!rpcUrl) return undefined;
  return RPC_URLS[rpcUrl.trim().toLowerCase()] ?? rpcUrl;
}

const PROVER_URLS = {
  devnet: "https://tx-prover.devnet.miden.io",
  testnet: "https://tx-prover.testnet.miden.io",
};

const NOTE_TRANSPORT_URLS = {
  testnet: "https://transport.miden.io",
  devnet: "https://transport.devnet.miden.io",
};

/**
 * Resolves a noteTransportUrl shorthand or raw URL into a concrete endpoint string.
 *
 * @param {string | undefined} noteTransportUrl - "testnet", "devnet", or a raw URL.
 * @returns {string | undefined} A fully qualified URL, or undefined if omitted.
 */
function resolveNoteTransportUrl(noteTransportUrl) {
  if (!noteTransportUrl) return undefined;
  return (
    NOTE_TRANSPORT_URLS[noteTransportUrl.trim().toLowerCase()] ??
    noteTransportUrl
  );
}

/**
 * Resolves a proverUrl shorthand or raw URL into a TransactionProver.
 *
 * @param {string} proverUrl - "local", "devnet", "testnet", or a raw URL.
 * @param {object} wasm - Loaded WASM module.
 * @returns {object} A TransactionProver instance.
 */
function resolveProver(proverUrl, wasm) {
  const normalized = proverUrl.trim().toLowerCase();
  if (normalized === "local") {
    return wasm.TransactionProver.newLocalProver();
  }
  const remoteUrl = PROVER_URLS[normalized] ?? proverUrl;
  return wasm.TransactionProver.newRemoteProver(remoteUrl, undefined);
}
