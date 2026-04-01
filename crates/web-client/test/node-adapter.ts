/**
 * Node.js adapter for the Miden web-client napi SDK.
 *
 * Maps the Node.js SDK surface to match the browser's window.* interface,
 * so existing Playwright tests can run unchanged on both platforms.
 *
 * Key adaptations:
 * - BigInt → Number for JsU64 params (napi uses f64, browser uses BigInt)
 * - syncState() → syncStateImpl()
 * - createMockClient() with no args → createMockClient(dbPath, keystorePath, ...)
 * - Fake page.evaluate() that runs callbacks directly
 */
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";

const require = createRequire(import.meta.url);

// ── Load the napi module ──────────────────────────────────────────────

function loadSdk(): any {
  if (process.env.MIDEN_MODULE_PATH) {
    return require(process.env.MIDEN_MODULE_PATH);
  }

  const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..");
  const arch = os.arch() === "arm64" ? "aarch64" : os.arch();
  const platform =
    os.platform() === "darwin" ? "apple-darwin" : "unknown-linux-gnu";
  const target = `${arch}-${platform}`;
  const ext = os.platform() === "darwin" ? "dylib" : "so";

  const candidates = [
    path.join(
      repoRoot,
      "target",
      target,
      "release",
      `libmiden_client_web.${ext}`
    ),
    path.join(repoRoot, "target", "release", `libmiden_client_web.${ext}`),
  ];

  for (const p of candidates) {
    if (fs.existsSync(p)) {
      const nodeFile = path.join(path.dirname(p), "miden_client_web.node");
      if (
        !fs.existsSync(nodeFile) ||
        fs.statSync(p).mtimeMs > fs.statSync(nodeFile).mtimeMs
      ) {
        fs.copyFileSync(p, nodeFile);
      }
      return require(nodeFile);
    }
  }

  throw new Error(
    `napi module not found. Build with:\n` +
      `  cargo build -p miden-client-web --no-default-features --features nodejs,testing --release --target ${target}`
  );
}

// Lazy-load: only initialize when actually used (avoids crash on browser-only CI)
let _sdk: any = null;

function patchPrototype(cls: any, aliases: Record<string, string>) {
  if (!cls?.prototype) return;
  for (const [snakeCase, camelCase] of Object.entries(aliases)) {
    if (
      typeof cls.prototype[camelCase] === "function" &&
      !cls.prototype[snakeCase]
    ) {
      cls.prototype[snakeCase] = cls.prototype[camelCase];
    }
  }
}

function patchNullToUndefined(cls: any, methods: string[]) {
  if (!cls?.prototype) return;
  for (const method of methods) {
    const original = cls.prototype[method];
    if (typeof original === "function") {
      cls.prototype[method] = function (...args: any[]) {
        const result = original.apply(this, args);
        return result === null ? undefined : result;
      };
    }
  }
}

function initSdk(): any {
  const rawSdk = loadSdk();

  // Patch napi prototypes for browser compatibility
  // eslint-disable-next-line camelcase
  patchPrototype(rawSdk.Account, { to_commitment: "toCommitment" });
  // eslint-disable-next-line camelcase
  patchPrototype(rawSdk.AccountHeader, { to_commitment: "toCommitment" });

  patchNullToUndefined(rawSdk.AccountStorage, [
    "getItem",
    "getMapEntries",
    "getMapItem",
  ]);
  patchNullToUndefined(rawSdk.NoteConsumability, ["consumableAfterBlock"]);

  // Patch static methods (snake_case aliases for camelCase)
  if (rawSdk.NoteScript) {
    if (!rawSdk.NoteScript.p2id && rawSdk.NoteScript.p2Id)
      rawSdk.NoteScript.p2id = rawSdk.NoteScript.p2Id;
    if (!rawSdk.NoteScript.p2ide && rawSdk.NoteScript.p2Ide)
      rawSdk.NoteScript.p2ide = rawSdk.NoteScript.p2Ide;
  }

  return rawSdk;
}

export const sdk = new Proxy(
  {},
  {
    get(_target, prop) {
      if (!_sdk) _sdk = initSdk();
      return _sdk[prop];
    },
  }
) as any;

// ── BigInt → Number conversion ────────────────────────────────────────

/**
 * Converts BigInt values to Number. The browser SDK uses BigInt for u64 params,
 * but the Node.js napi SDK uses f64 (JavaScript number). Values up to 2^53 are
 * safe, which covers all practical token amounts.
 */
function toNum(val: any): any {
  return typeof val === "bigint" ? Number(val) : val;
}

/**
 * Normalizes arguments for napi:
 * - BigInt → Number
 * - BigUint64Array → number[]
 * - Uint8Array/Buffer → Array<number> (for Vec<u8> params)
 */
function normalizeArg(val: any): any {
  if (typeof val === "bigint") return Number(val);
  if (val instanceof BigUint64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof BigInt64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof Uint8Array || Buffer.isBuffer(val)) return Array.from(val);
  return val;
}

/**
 * Wraps a class so that constructor args and static method args are normalized.
 * Returns a Proxy that intercepts `new` and static calls.
 */
/**
 * Wraps a class so that constructor and static method args are normalized.
 * Copies all static methods/properties, wrapping functions to normalize args.
 */
function wrapClass(Cls: any): any {
  const Wrapper: any = function (...args: any[]) {
    return new Cls(...args.map(normalizeArg));
  };
  Wrapper.prototype = Cls.prototype;
  // Copy static methods with arg normalization
  for (const key of Object.getOwnPropertyNames(Cls)) {
    if (key === "prototype" || key === "length" || key === "name") continue;
    const desc = Object.getOwnPropertyDescriptor(Cls, key);
    if (desc && typeof desc.value === "function") {
      Wrapper[key] = (...args: any[]) =>
        desc.value.apply(Cls, args.map(normalizeArg));
    } else if (desc) {
      try {
        Object.defineProperty(Wrapper, key, desc);
      } catch {
        /* skip non-configurable */
      }
    }
  }
  return Wrapper;
}

// ── Client wrapper ────────────────────────────────────────────────────

/**
 * Wraps a Node.js WebClient to match the browser API surface.
 */
function wrapClient(client: any, storeName?: string): any {
  return new Proxy(client, {
    get(target, prop) {
      // syncState → syncStateImpl
      if (prop === "syncState") {
        return (...args: any[]) => target.syncStateImpl(...args);
      }
      // syncStateWithTimeout — just calls syncState (no browser lock coordination needed)
      if (prop === "syncStateWithTimeout") {
        return (_timeoutMs?: number) => target.syncStateImpl();
      }
      // storeName — used by MidenClient for lock coordination
      if (prop === "storeName") {
        return storeName || "default";
      }
      // wasmWebClient — MidenClient's proxy looks for this
      if (prop === "wasmWebClient") {
        return target;
      }
      // newWallet: convert Uint8Array/Buffer seed to plain Array for napi's Vec<u8>
      if (prop === "newWallet") {
        return (mode: any, mutable: any, authScheme: any, seed?: any) => {
          const normalizedSeed =
            seed instanceof Uint8Array || Buffer.isBuffer(seed)
              ? Array.from(seed)
              : seed;
          return target.newWallet(
            mode,
            mutable,
            authScheme,
            normalizedSeed ?? null
          );
        };
      }
      // Methods that take JsU64 (BigInt in browser, Number in Node.js)
      if (prop === "newFaucet") {
        return (
          mode: any,
          nonFungible: any,
          symbol: any,
          decimals: any,
          maxSupply: any,
          auth: any,
          seed?: any
        ) =>
          target.newFaucet(
            mode,
            nonFungible,
            symbol,
            decimals,
            toNum(maxSupply),
            auth,
            seed
          );
      }
      if (prop === "newMintTransactionRequest") {
        return (wallet: any, faucet: any, noteType: any, amount: any) =>
          target.newMintTransactionRequest(
            wallet,
            faucet,
            noteType,
            toNum(amount)
          );
      }
      if (prop === "newSendTransactionRequest") {
        return (
          sender: any,
          targetId: any,
          faucet: any,
          noteType: any,
          amount: any,
          ...rest: any[]
        ) =>
          target.newSendTransactionRequest(
            sender,
            targetId,
            faucet,
            noteType,
            toNum(amount),
            ...rest
          );
      }
      if (prop === "newSwapTransactionRequest") {
        return (
          accountId: any,
          assetAFaucet: any,
          assetAAmount: any,
          assetBFaucet: any,
          assetBAmount: any,
          ...rest: any[]
        ) =>
          target.newSwapTransactionRequest(
            accountId,
            assetAFaucet,
            toNum(assetAAmount),
            assetBFaucet,
            toNum(assetBAmount),
            ...rest
          );
      }

      const val = target[prop];
      if (typeof val === "function") {
        const bound = val.bind(target);
        return (...args: any[]) => {
          // Normalize args for napi: BigInt→number, Uint8Array→Array, etc.
          const normalizedArgs = args.map(normalizeArg);
          const result = bound(...normalizedArgs);
          // Normalize return: napi's null → undefined for Option<T>
          if (result && typeof result.then === "function") {
            return result.then((v: any) => (v === null ? undefined : v));
          }
          return result === null ? undefined : result;
        };
      }
      return val;
    },
  });
}

// ── Client factories ──────────────────────────────────────────────────

let testCounter = 0;

function tmpTestDir(): string {
  const dir = path.join(
    os.tmpdir(),
    `miden-node-test-${process.pid}-${++testCounter}`
  );
  fs.mkdirSync(path.join(dir, "keystore"), { recursive: true });
  return dir;
}

/**
 * Matches the browser's `window.MockWasmWebClient` interface.
 */
export const MockWasmWebClient = {
  createClient: async (
    seed?: any,
    serializedMockChain?: any,
    serializedMockNoteTransportNode?: any
  ) => {
    const dir = tmpTestDir();
    const client = new sdk.WebClient();
    // Convert Uint8Array/Buffer to plain Array for napi's Vec<u8>
    const norm = (v: any) =>
      v instanceof Uint8Array || Buffer.isBuffer(v) ? Array.from(v) : v;
    await client.createMockClient(
      path.join(dir, "store.db"),
      path.join(dir, "keystore"),
      norm(seed) ?? null,
      norm(serializedMockChain) ?? null,
      norm(serializedMockNoteTransportNode) ?? null
    );
    return wrapClient(client, "mock");
  },
};

/**
 * Matches the browser's `window.WasmWebClient` interface.
 */
export const WasmWebClient = {
  // Static method used by standalone.js buildSwapTag
  buildSwapTag: (...args: any[]) =>
    sdk.WebClient.buildSwapTag(...args.map(normalizeArg)),

  createClient: async (
    rpcUrl?: string,
    noteTransportUrl?: any,
    seed?: any,
    storeName?: string
  ) => {
    const dir = tmpTestDir();
    const client = new sdk.WebClient();
    const normSeed =
      seed instanceof Uint8Array || Buffer.isBuffer(seed)
        ? Array.from(seed)
        : seed;
    await client.createClient(
      rpcUrl ?? null,
      noteTransportUrl ?? null,
      normSeed ?? null,
      path.join(dir, `${storeName || "store"}.db`),
      path.join(dir, "keystore"),
      false
    );
    return wrapClient(client, storeName);
  },
};

// ── Helpers (matches window.helpers.*) ────────────────────────────────

function createHelpers(getClient: () => any, setClient: (c: any) => void) {
  return {
    waitForTransaction: async (
      transactionId: string,
      maxWaitTime = 10000,
      delayInterval = 1000
    ) => {
      const client = getClient();
      let timeWaited = 0;
      while (true) {
        if (timeWaited >= maxWaitTime)
          throw new Error("Timeout waiting for transaction");
        await client.syncState();
        const uncommitted = await client.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx: any) => tx.id().toHex());
        if (!ids.includes(transactionId)) break;
        await new Promise((r) => setTimeout(r, delayInterval));
        timeWaited += delayInterval;
      }
    },

    executeAndApplyTransaction: async (
      accountId: any,
      transactionRequest: any,
      prover?: any
    ) => {
      const client = getClient();
      const result = await client.executeTransaction(
        accountId,
        transactionRequest
      );
      const proverToUse = prover ?? sdk.TransactionProver.newLocalProver();
      const proven = await client.proveTransaction(result, proverToUse);
      const submissionHeight = await client.submitProvenTransaction(
        proven,
        result
      );
      return await client.applyTransaction(result, submissionHeight);
    },

    waitForBlocks: async (amountOfBlocks: number) => {
      const client = getClient();
      let currentBlock = await client.getSyncHeight();
      let finalBlock = currentBlock + amountOfBlocks;
      while (true) {
        let syncSummary = await client.syncState();
        if (syncSummary.blockNum() >= finalBlock) return;
        await new Promise((r) => setTimeout(r, 1000));
      }
    },

    parseNetworkId: (networkId: string) => {
      const map: Record<string, any> = {
        mm: sdk.NetworkId.mainnet(),
        mtst: sdk.NetworkId.testnet(),
        mdev: sdk.NetworkId.devnet(),
      };
      let parsed = map[networkId];
      if (parsed === undefined) {
        try {
          parsed = sdk.NetworkId.custom(networkId);
        } catch {
          throw new Error(`Invalid network ID: ${networkId}`);
        }
      }
      return parsed;
    },

    refreshClient: async (initSeed?: any) => {
      // Create a new mock client (the browser version reconnects to the same IndexedDB store)
      const client = await MockWasmWebClient.createClient(
        initSeed instanceof Uint8Array ? Buffer.from(initSeed) : initSeed
      );
      setClient(client);
    },
  };
}

// ── Fake page ─────────────────────────────────────────────────────────

/**
 * Creates a fake Playwright Page object for Node.js tests.
 * page.evaluate() just runs the callback directly.
 * window.* globals are set so callbacks work unchanged.
 */
// ── Array type polyfills ──────────────────────────────────────────────
// In the browser, wasm_bindgen generates typed array wrappers (NoteAndArgsArray, etc.)
// that are constructable from JS arrays. napi doesn't export these — it accepts
// plain JS arrays directly. These polyfills wrap arrays so `new XArray([...])` works.

/**
 * Creates a polyfill class for WASM array types (NoteAndArgsArray, FeltArray, etc.).
 * napi expects plain JS arrays, so `new XArray([a, b])` just returns a plain Array
 * with extra methods that the WASM arrays have (push, get, replaceAt, length as fn).
 */
function makeArrayPolyfill(_name: string) {
  return function (items: any[]) {
    const arr = Array.isArray(items) ? [...items] : [items];
    // Add WASM-style array methods that some tests use
    (arr as any).get = (i: number) => arr[i];
    (arr as any).replaceAt = (i: number, val: any) => {
      arr[i] = val;
      return arr;
    };
    return arr;
  };
}

const arrayPolyfills: Record<string, any> = {
  AccountArray: makeArrayPolyfill("AccountArray"),
  AccountIdArray: makeArrayPolyfill("AccountIdArray"),
  ForeignAccountArray: makeArrayPolyfill("ForeignAccountArray"),
  NoteRecipientArray: makeArrayPolyfill("NoteRecipientArray"),
  OutputNoteArray: makeArrayPolyfill("OutputNoteArray"),
  StorageSlotArray: makeArrayPolyfill("StorageSlotArray"),
  TransactionScriptInputPairArray: makeArrayPolyfill(
    "TransactionScriptInputPairArray"
  ),
  FeltArray: makeArrayPolyfill("FeltArray"),
  OutputNotesArray: makeArrayPolyfill("OutputNotesArray"),
  NoteAndArgsArray: makeArrayPolyfill("NoteAndArgsArray"),
  NoteDetailsAndTagArray: makeArrayPolyfill("NoteDetailsAndTagArray"),
  NoteIdAndArgsArray: makeArrayPolyfill("NoteIdAndArgsArray"),
};

export async function setupNodeGlobals(
  rpcUrl: string,
  storeName: string,
  proverUrl?: string
) {
  let currentClient: any = null;

  // All SDK types go on globalThis so "window.X" works in callbacks
  const globals: Record<string, any> = {
    // Client factories
    MockWasmWebClient,
    WasmWebClient,

    // SDK types
    Account: sdk.Account,
    AccountStorageMode: sdk.AccountStorageMode,
    AuthScheme: sdk.AuthScheme,
    NoteType: sdk.NoteType,
    TransactionFilter: sdk.TransactionFilter,
    TransactionProver: sdk.TransactionProver,
    NoteFilter: sdk.NoteFilter,
    NoteFilterTypes: sdk.NoteFilterTypes,
    AccountId: sdk.AccountId,
    // AccountType: the JS wrapper uses string-based types, not the napi enum
    AccountType: {
      MutableWallet: "MutableWallet",
      ImmutableWallet: "ImmutableWallet",
      FungibleFaucet: "FungibleFaucet",
      NonFungibleFaucet: "NonFungibleFaucet",
      ImmutableContract: "ImmutableContract",
      MutableContract: "MutableContract",
      // Also keep the napi enum values for tests that use the low-level API
      RegularAccountUpdatableCode: sdk.AccountType?.RegularAccountUpdatableCode,
      RegularAccountImmutableCode: sdk.AccountType?.RegularAccountImmutableCode,
    },
    AccountInterface: sdk.AccountInterface,
    AccountBuilder: wrapClass(sdk.AccountBuilder),
    AccountComponent: wrapClass(sdk.AccountComponent),
    AccountFile: sdk.AccountFile,
    AccountHeader: sdk.AccountHeader,
    AccountStorageRequirements: sdk.AccountStorageRequirements,
    Address: sdk.Address,
    AdviceMap: sdk.AdviceMap,
    AuthSecretKey: wrapClass(sdk.AuthSecretKey),
    AuthFalcon512RpoMultisigConfig: sdk.AuthFalcon512RpoMultisigConfig,
    createAuthFalcon512RpoMultisig: sdk.createAuthFalcon512RpoMultisig,
    BasicFungibleFaucetComponent: sdk.BasicFungibleFaucetComponent,
    Endpoint: sdk.Endpoint,
    Felt: wrapClass(sdk.Felt),
    ForeignAccount: sdk.ForeignAccount,
    FungibleAsset: wrapClass(sdk.FungibleAsset),
    MidenArrays: sdk.MidenArrays,
    NetworkId: sdk.NetworkId,
    Note: sdk.Note,
    NoteAndArgs: sdk.NoteAndArgs,
    NoteAndArgsArray: sdk.NoteAndArgsArray,
    NoteAssets: sdk.NoteAssets,
    NoteAttachment: sdk.NoteAttachment,
    NoteExportFormat: sdk.NoteExportFormat,
    NoteExecutionHint: sdk.NoteExecutionHint,
    NoteFile: sdk.NoteFile,
    NoteId: sdk.NoteId,
    NoteMetadata: sdk.NoteMetadata,
    NoteRecipient: sdk.NoteRecipient,
    NoteScript: sdk.NoteScript,
    NoteStorage: sdk.NoteStorage,
    NoteTag: wrapClass(sdk.NoteTag),
    OutputNote: sdk.OutputNote,
    Package: sdk.Package,
    ProcedureThreshold: sdk.ProcedureThreshold,
    PublicKey: sdk.PublicKey,
    Rpo: sdk.Rpo,
    Rpo256: sdk.Rpo256,
    RpcClient: sdk.RpcClient,
    Signature: sdk.Signature,
    SigningInputs: sdk.SigningInputs,
    SlotAndKeys: sdk.SlotAndKeys,
    StorageMap: sdk.StorageMap,
    StorageSlot: sdk.StorageSlot,
    TestUtils: sdk.TestUtils,
    TokenSymbol: sdk.TokenSymbol,
    TransactionRequestBuilder: sdk.TransactionRequestBuilder,
    Word: wrapClass(sdk.Word),

    // Array types (polyfills — napi accepts plain arrays, these wrap for constructor compat)
    ...arrayPolyfills,
    MidenArrays: arrayPolyfills,

    // Config
    rpcUrl,
    storeName,
    remoteProverUrl: proverUrl ?? null,
    remoteProverInstance: proverUrl
      ? sdk.TransactionProver.newRemoteProver(proverUrl, null)
      : null,

    // Helpers (set after client is created)
    helpers: {},

    // Browser-only stubs
    exportStore: async () => {
      throw new Error("exportStore is browser-only");
    },
    importStore: async () => {
      throw new Error("importStore is browser-only");
    },
    indexedDB: { databases: async () => [] },

    // getWasmOrThrow — returns the raw SDK (used by MidenClient and some tests)
    // Returns an object that includes both the sdk exports AND the array types
    getWasmOrThrow: async () => ({
      ...sdk,
      ...arrayPolyfills,
      AccountBuilder: wrapClass(sdk.AccountBuilder),
      AccountComponent: wrapClass(sdk.AccountComponent),
      AuthSecretKey: wrapClass(sdk.AuthSecretKey),
      Felt: wrapClass(sdk.Felt),
      FungibleAsset: wrapClass(sdk.FungibleAsset),
      Word: wrapClass(sdk.Word),
      NoteTag: wrapClass(sdk.NoteTag),
    }),
  };

  // Set the client getter and helpers
  const setClientFn = (client: any) => {
    currentClient = client;
    (globalThis as any).client = client;
  };
  globals.helpers = createHelpers(() => currentClient, setClientFn);

  // Put everything on globalThis.window and globalThis directly
  const w = globalThis as any;
  w.window = w;
  for (const [key, value] of Object.entries(globals)) {
    w[key] = value;
  }

  // Load JS wrapper modules (pure JS — works with napi the same way as WASM).
  try {
    const jsDir = path.resolve(import.meta.dirname, "..", "js");

    // MidenClient high-level wrapper
    const { MidenClient } = await import(path.join(jsDir, "client.js"));
    MidenClient._WasmWebClient = WasmWebClient;
    MidenClient._MockWasmWebClient = MockWasmWebClient;
    MidenClient._getWasmOrThrow = globals.getWasmOrThrow;
    w.MidenClient = MidenClient;

    // Standalone helper functions
    const standalone = await import(path.join(jsDir, "standalone.js"));
    // Wire the standalone module's internal references
    if (standalone._setWasm)
      standalone._setWasm(await globals.getWasmOrThrow());
    if (standalone._setWebClient) standalone._setWebClient(WasmWebClient);
    if (standalone.createP2IDNote) w.createP2IDNote = standalone.createP2IDNote;
    if (standalone.createP2IDENote)
      w.createP2IDENote = standalone.createP2IDENote;
    if (standalone.buildSwapTag) w.buildSwapTag = standalone.buildSwapTag;
  } catch (e) {
    // If import fails, MidenClient and standalone tests will skip
    console.warn("Failed to load JS wrapper modules:", e);
  }

  // Function to set the current client (called after createClient)
  return {
    setClient: (client: any) => {
      currentClient = client;
      w.client = client;
    },
  };
}

export function createFakePage() {
  return {
    evaluate: async (fn: Function, args?: any) => fn(args),
    goto: async () => {},
    on: (..._args: any[]) => {},
  };
}
