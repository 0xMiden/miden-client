/**
 * Platform-agnostic test setup for both browser (Playwright) and Node.js (napi).
 *
 * Provides a `run` fixture that abstracts the platform difference:
 * - Node.js: SDK operations run directly in the test process
 * - Browser: SDK operations run inside a browser page via page.evaluate()
 *
 * Tests import { test, expect } from "./test-setup" and use `run` to execute
 * SDK code that works on both platforms. Assertions stay outside `run`.
 *
 * Example:
 *   test("creates a wallet", async ({ run }) => {
 *     const result = await run(async ({ client, sdk }) => {
 *       const wallet = await client.newWallet(
 *         sdk.AccountStorageMode.private(), true, sdk.AuthScheme.AuthRpoFalcon512
 *       );
 *       return { id: wallet.id().toString() };
 *     });
 *     expect(result.id).toMatch(/^0x/);
 *   });
 */
// @ts-nocheck
import { test as base, expect, chromium, webkit } from "@playwright/test";
import type { TestInfo } from "@playwright/test";
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";
import { getRpcUrl, getProverUrl, RUN_ID } from "./playwright.global.setup";

const require = createRequire(import.meta.url);

function generateStoreName(testInfo: TestInfo): string {
  return `test_${RUN_ID}_${testInfo.testId}`;
}

// ── Node.js setup ─────────────────────────────────────────────────────

let _nodeSdk: any = null;

export function loadNodeSdk(): any {
  if (_nodeSdk) return _nodeSdk;

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
      _nodeSdk = require(nodeFile);
      return _nodeSdk;
    }
  }

  throw new Error(
    `napi module not found. Build with:\n` +
      `  cargo build -p miden-client-web --no-default-features --features nodejs,testing --release --target ${target}`
  );
}

let _nodeTestCounter = 0;

export async function createNodeMockClient(): Promise<{
  client: any;
  sdk: any;
}> {
  const rawSdk = loadNodeSdk();
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++_nodeTestCounter}`
  );
  fs.mkdirSync(path.join(tmpDir, "keystore"), { recursive: true });

  const rawClient = new rawSdk.WebClient();
  await rawClient.createMockClient(
    path.join(tmpDir, "store.db"),
    path.join(tmpDir, "keystore"),
    null,
    null,
    null
  );

  // Wrap the client to normalize napi differences
  const client = wrapNodeClient(rawClient, rawSdk);
  const sdk = createNodeSdkWrapper(rawSdk);

  return { client, sdk };
}

export async function createNodeIntegrationClient(
  rpcUrl: string,
  storeName: string
): Promise<{ client: any; sdk: any }> {
  const rawSdk = loadNodeSdk();
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++_nodeTestCounter}`
  );
  fs.mkdirSync(path.join(tmpDir, "keystore"), { recursive: true });

  const rawClient = new rawSdk.WebClient();
  await rawClient.createClient(
    rpcUrl,
    null,
    null,
    path.join(tmpDir, `${storeName}.db`),
    path.join(tmpDir, "keystore"),
    false
  );

  const client = wrapNodeClient(rawClient, rawSdk);
  const sdk = createNodeSdkWrapper(rawSdk);

  return { client, sdk };
}

/**
 * Wraps a napi WebClient to normalize differences with the browser SDK:
 * - syncState() → syncStateImpl()
 * - null → undefined for Option<T> returns
 */
export function wrapNodeClient(rawClient: any, rawSdk: any): any {
  return new Proxy(rawClient, {
    get(target, prop) {
      if (prop === "syncState") {
        return (...args: any[]) => target.syncStateImpl(...args);
      }
      if (prop === "proveBlock") {
        return async () => {
          const guard = await target.proveBlock();
          return guard;
        };
      }
      if (prop === "newWallet") {
        return (mode: any, mutable: any, authScheme: any, seed?: any) => {
          const normSeed =
            seed instanceof Uint8Array || Buffer.isBuffer(seed)
              ? Array.from(seed)
              : seed;
          const result = target.newWallet(
            mode,
            mutable,
            authScheme,
            normSeed ?? null
          );
          if (result && typeof result.then === "function") {
            return result.then((v: any) => (v === null ? undefined : v));
          }
          return result === null ? undefined : result;
        };
      }
      const val = target[prop];
      if (typeof val === "function") {
        const bound = val.bind(target);
        return (...args: any[]) => {
          const normalizedArgs = args.map(normalizeNapiArg);
          const result = bound(...normalizedArgs);
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

/**
 * Normalizes a single argument for napi:
 * - BigInt → Number, BigUint64Array → number[], Uint8Array/Buffer → number[]
 */
function normalizeNapiArg(val: any): any {
  if (typeof val === "bigint") return Number(val);
  if (val instanceof BigUint64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof BigInt64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof Uint8Array || Buffer.isBuffer(val)) return Array.from(val);
  return val;
}

/**
 * Wraps a napi class so that constructor and static method args are normalized
 * (Uint8Array → Array, BigInt → Number, etc.).
 */
function wrapNapiClass(Cls: any): any {
  const Wrapper: any = function (...args: any[]) {
    return new Cls(...args.map(normalizeNapiArg));
  };
  Wrapper.prototype = Cls.prototype;
  for (const key of Object.getOwnPropertyNames(Cls)) {
    if (key === "prototype" || key === "length" || key === "name") continue;
    const desc = Object.getOwnPropertyDescriptor(Cls, key);
    if (desc && typeof desc.value === "function") {
      Wrapper[key] = (...args: any[]) =>
        desc.value.apply(Cls, args.map(normalizeNapiArg));
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

function patchNapiPrototypes(rawSdk: any) {
  // snake_case aliases for camelCase methods (browser uses snake_case via wasm_bindgen)
  /* eslint-disable camelcase */
  for (const [cls, aliases] of [
    [rawSdk.Account, { to_commitment: "toCommitment" }],
    [rawSdk.AccountHeader, { to_commitment: "toCommitment" }],
  ] as [any, Record<string, string>][]) {
    if (!cls?.prototype) continue;
    for (const [snake, camel] of Object.entries(aliases)) {
      if (typeof cls.prototype[camel] === "function" && !cls.prototype[snake]) {
        cls.prototype[snake] = cls.prototype[camel];
      }
    }
  }
  /* eslint-enable camelcase */

  // Patch null → undefined for Option<T> returns
  for (const [cls, methods] of [
    [rawSdk.AccountStorage, ["getItem", "getMapEntries", "getMapItem"]],
    [rawSdk.NoteConsumability, ["consumableAfterBlock"]],
  ] as [any, string[]][]) {
    if (!cls?.prototype) continue;
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

  // snake_case aliases for static methods
  if (rawSdk.NoteScript) {
    if (!rawSdk.NoteScript.p2id && rawSdk.NoteScript.p2Id)
      rawSdk.NoteScript.p2id = rawSdk.NoteScript.p2Id;
    if (!rawSdk.NoteScript.p2ide && rawSdk.NoteScript.p2Ide)
      rawSdk.NoteScript.p2ide = rawSdk.NoteScript.p2Ide;
  }
}

export function createNodeSdkWrapper(rawSdk: any): any {
  patchNapiPrototypes(rawSdk);

  return {
    ...rawSdk,
    // Wrap classes whose constructors/static methods accept Uint8Array or BigInt args
    AccountBuilder: wrapNapiClass(rawSdk.AccountBuilder),
    AccountComponent: wrapNapiClass(rawSdk.AccountComponent),
    AuthSecretKey: wrapNapiClass(rawSdk.AuthSecretKey),
    Felt: wrapNapiClass(rawSdk.Felt),
    FungibleAsset: wrapNapiClass(rawSdk.FungibleAsset),
    Word: wrapNapiClass(rawSdk.Word),
    NoteTag: wrapNapiClass(rawSdk.NoteTag),
    // u64: converts to the platform-appropriate type
    // Node.js napi uses f64 (number), browser uses BigInt
    u64: (val: number | bigint) =>
      typeof val === "bigint" ? Number(val) : val,
    // u64Array: converts an array of numbers to the platform-appropriate array type
    // Browser uses BigUint64Array, Node.js uses number[]
    u64Array: (vals: number[]) => vals,
  };
}

// ── Test helpers ──────────────────────────────────────────────────────

/**
 * Executes a transaction: execute → prove → submit → apply.
 * Works identically on both platforms.
 */
export async function executeAndApplyTransaction(
  client: any,
  sdk: any,
  accountId: any,
  transactionRequest: any,
  prover?: any
) {
  const result = await client.executeTransaction(accountId, transactionRequest);
  const proverToUse = prover ?? sdk.TransactionProver.newLocalProver();
  const proven = await client.proveTransaction(result, proverToUse);
  const submissionHeight = await client.submitProvenTransaction(proven, result);
  return await client.applyTransaction(result, submissionHeight);
}

/**
 * Waits for a transaction to be committed by polling syncState.
 */
export async function waitForTransaction(
  client: any,
  sdk: any,
  transactionId: string,
  maxWaitTime = 10000,
  delayInterval = 1000
) {
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
}

// ── Browser run() support ─────────────────────────────────────────────

/**
 * Standalone browser instance for browser-side run() calls.
 * Created lazily on first browser test, shared across all tests in the worker.
 * This avoids depending on Playwright's `page` fixture (which would trigger
 * browser launch even for nodejs tests).
 */
let _runBrowser: any = null;

async function getRunBrowser(projectName: string): Promise<any> {
  if (_runBrowser) return _runBrowser;
  const launcher = projectName === "webkit" ? webkit : chromium;
  _runBrowser = await launcher.launch();
  return _runBrowser;
}

/**
 * Sets up a browser page with the SDK, a mock client, and helpers.
 */
async function setupBrowserPage(page: any, testInfo: TestInfo) {
  const rpcUrl = getRpcUrl();
  const storeName = generateStoreName(testInfo);

  await page.goto("http://localhost:8080");

  await page.evaluate(
    async ({ rpcUrl, storeName }) => {
      // Import all SDK exports and attach to window
      const sdkExports = await import("./index.js");
      for (const [key, value] of Object.entries(sdkExports)) {
        window[key] = value;
      }
      // Restore the WASM AuthScheme enum (JS API shadows it)
      const wasm = await window.getWasmOrThrow();
      window.AuthScheme = wasm.AuthScheme;

      // Create mock client.
      // Disable the web worker so all operations run on the main thread.
      // The browser MockWebClient serializes the entire mock chain for every
      // worker-delegated operation (submitNewTransaction, syncState, etc.).
      // This serialization roundtrip has a bug that produces invalid bytes,
      // causing "failed to deserialize mock chain: unexpected EOF".
      // Without a worker, the fallback paths call the WASM client directly.
      const client = await window.MockWasmWebClient.createClient();
      if (client.worker) {
        client.worker.terminate();
        client.worker = null;
      }
      window.client = client;
      window.rpcUrl = rpcUrl;
      window.storeName = storeName;

      // ── Register helpers on window ──────────────────────────────

      window.helpers = {
        setupWalletAndFaucet: async () => {
          const c = window.client;
          const wallet = await c.newWallet(
            window.AccountStorageMode.private(),
            true,
            window.AuthScheme.AuthRpoFalcon512
          );
          const faucet = await c.newFaucet(
            window.AccountStorageMode.private(),
            false,
            "DAG",
            8,
            BigInt(10000000),
            window.AuthScheme.AuthRpoFalcon512
          );
          return {
            walletId: wallet.id().toString(),
            faucetId: faucet.id().toString(),
            walletCommitment: wallet.to_commitment().toHex(),
            wallet,
            faucet,
          };
        },

        mockMint: async (targetId, faucetId, opts) => {
          const c = window.client;
          const amount = opts?.amount ?? 1000;
          const noteType = opts?.publicNote
            ? window.NoteType.Public
            : window.NoteType.Private;
          const mintRequest = await c.newMintTransactionRequest(
            targetId,
            faucetId,
            noteType,
            BigInt(amount)
          );
          const txId = await c.submitNewTransaction(faucetId, mintRequest);
          // Save hex before TransactionFilter.ids() consumes the object
          const txIdHex = txId.toHex();
          if (!opts?.skipSync) {
            await c.proveBlock();
            await c.syncState();
          }
          const [txRecord] = await c.getTransactions(
            window.TransactionFilter.ids([txId])
          );
          const notes = txRecord.outputNotes().notes();
          return {
            transactionId: txIdHex,
            createdNoteId: notes[0].id().toString(),
            numOutputNotesCreated: notes.length,
          };
        },

        mockConsume: async (accountId, noteId) => {
          const c = window.client;
          const inputNoteRecord = await c.getInputNote(noteId);
          if (!inputNoteRecord) throw new Error(`Note ${noteId} not found`);
          const note = inputNoteRecord.toNote();
          const consumeRequest = c.newConsumeTransactionRequest([note]);
          const txId = await c.submitNewTransaction(accountId, consumeRequest);
          await c.proveBlock();
          await c.syncState();
          return { transactionId: txId.toHex() };
        },

        mockMintAndConsume: async (accountId, faucetId, opts) => {
          const h = window.helpers;
          const { transactionId: mintTxId, createdNoteId } = await h.mockMint(
            accountId,
            faucetId,
            opts
          );
          const { transactionId: consumeTxId } = await h.mockConsume(
            accountId,
            createdNoteId
          );
          const c = window.client;
          const account = await c.getAccount(accountId);
          const balance = account.vault().getBalance(faucetId).toString();
          return {
            mintTransactionId: mintTxId,
            consumeTransactionId: consumeTxId,
            createdNoteId,
            targetAccountBalance: balance,
          };
        },

        mockSend: async (senderId, targetId, faucetId, opts) => {
          const h = window.helpers;
          const c = window.client;
          const { createdNoteId: mintedNoteId } = await h.mockMint(
            senderId,
            faucetId
          );
          await h.mockConsume(senderId, mintedNoteId);
          const sendAmount = opts?.amount ?? 100;
          const sendRequest = await c.newSendTransactionRequest(
            senderId,
            targetId,
            faucetId,
            window.NoteType.Public,
            BigInt(sendAmount),
            opts?.recallHeight ?? null,
            null
          );
          const sendTxId = await c.submitNewTransaction(senderId, sendRequest);
          await c.proveBlock();
          await c.syncState();
          const [sendTxRecord] = await c.getTransactions(
            window.TransactionFilter.ids([sendTxId])
          );
          const sendNotes = sendTxRecord.outputNotes().notes();
          return {
            sendCreatedNoteIds: sendNotes.map((n) => n.id().toString()),
          };
        },

        mockSwap: async (
          accountAId,
          accountBId,
          assetAFaucetId,
          assetAAmount,
          assetBFaucetId,
          assetBAmount,
          swapNoteType,
          paybackNoteType
        ) => {
          const c = window.client;
          const noteTypeA =
            swapNoteType === "public"
              ? window.NoteType.Public
              : window.NoteType.Private;
          const noteTypeB =
            paybackNoteType === "public"
              ? window.NoteType.Public
              : window.NoteType.Private;
          const swapRequest = await c.newSwapTransactionRequest(
            accountAId,
            assetAFaucetId,
            BigInt(assetAAmount),
            assetBFaucetId,
            BigInt(assetBAmount),
            noteTypeA,
            noteTypeB
          );
          const expectedOutputNotes = swapRequest.expectedOutputOwnNotes();
          const expectedPaybackNoteDetails = swapRequest
            .expectedFutureNotes()
            .map((fn) => fn.noteDetails);
          const swapTxId = await c.submitNewTransaction(
            accountAId,
            swapRequest
          );
          await c.proveBlock();
          await c.syncState();

          // Consume swap note for account B
          const swapNoteId = expectedOutputNotes[0].id().toString();
          const swapNoteRecord = await c.getInputNote(swapNoteId);
          if (!swapNoteRecord)
            throw new Error(`Swap note ${swapNoteId} not found`);
          const swapNote = swapNoteRecord.toNote();
          const consumeReq1 = c.newConsumeTransactionRequest([swapNote]);
          await c.submitNewTransaction(accountBId, consumeReq1);
          await c.proveBlock();
          await c.syncState();

          // Consume payback note for account A
          const paybackNoteId = expectedPaybackNoteDetails[0].id().toString();
          const paybackNoteRecord = await c.getInputNote(paybackNoteId);
          if (!paybackNoteRecord)
            throw new Error(`Payback note ${paybackNoteId} not found`);
          const paybackNote = paybackNoteRecord.toNote();
          const consumeReq2 = c.newConsumeTransactionRequest([paybackNote]);
          await c.submitNewTransaction(accountAId, consumeReq2);
          await c.proveBlock();
          await c.syncState();

          // Fetch final assets
          const accountA = await c.getAccount(accountAId);
          const accountAAssets = accountA
            ?.vault()
            .fungibleAssets()
            .map((asset) => ({
              assetId: asset.faucetId().toString(),
              amount: asset.amount().toString(),
            }));
          const accountB = await c.getAccount(accountBId);
          const accountBAssets = accountB
            ?.vault()
            .fungibleAssets()
            .map((asset) => ({
              assetId: asset.faucetId().toString(),
              amount: asset.amount().toString(),
            }));
          return { accountAAssets, accountBAssets };
        },

        executeAndApplyTransaction: async (
          accountId,
          transactionRequest,
          prover
        ) => {
          const c = window.client;
          const result = await c.executeTransaction(
            accountId,
            transactionRequest
          );
          const proverToUse =
            prover ?? window.TransactionProver.newLocalProver();
          const proven = await c.proveTransaction(result, proverToUse);
          const submissionHeight = await c.submitProvenTransaction(
            proven,
            result
          );
          return await c.applyTransaction(result, submissionHeight);
        },

        waitForTransaction: async (
          transactionId,
          maxWaitTime = 10000,
          delayInterval = 1000
        ) => {
          const c = window.client;
          let timeWaited = 0;
          while (true) {
            if (timeWaited >= maxWaitTime)
              throw new Error("Timeout waiting for transaction");
            await c.syncState();
            const uncommitted = await c.getTransactions(
              window.TransactionFilter.uncommitted()
            );
            const ids = uncommitted.map((tx) => tx.id().toHex());
            if (!ids.includes(transactionId)) break;
            await new Promise((r) => setTimeout(r, delayInterval));
            timeWaited += delayInterval;
          }
        },

        parseNetworkId: (networkId) => {
          const map = {
            mm: window.NetworkId.mainnet(),
            mtst: window.NetworkId.testnet(),
            mdev: window.NetworkId.devnet(),
          };
          let parsed = map[networkId];
          if (parsed === undefined) {
            try {
              parsed = window.NetworkId.custom(networkId);
            } catch {
              throw new Error(`Invalid network ID: ${networkId}`);
            }
          }
          return parsed;
        },

        createFreshMockClient: async () => {
          const freshClient = await window.MockWasmWebClient.createClient();
          // Disable worker (same as main client) to avoid mock chain
          // serialization bug and reduce memory pressure
          if (freshClient.worker) {
            freshClient.worker.terminate();
            freshClient.worker = null;
          }
          return freshClient;
        },

        createIntegrationClient: async () => {
          try {
            const uniqueName = `int_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
            const client = await window.WasmWebClient.createClient(
              window.rpcUrl,
              undefined,
              undefined,
              uniqueName
            );
            return { client };
          } catch {
            return null;
          }
        },

        getRpcUrl: () => window.rpcUrl,
      };
    },
    { rpcUrl, storeName }
  );
}

/**
 * Creates helpers for the Node.js run() context.
 * Uses async import to avoid circular dependency with test-helpers.ts.
 */
async function createNodeRunHelpers(client: any, sdk: any): Promise<any> {
  const h = await import("./test-helpers");
  return {
    setupWalletAndFaucet: () => h.setupWalletAndFaucet(client, sdk),
    mockMint: (targetId: any, faucetId: any, opts?: any) =>
      h.mockMint(client, sdk, targetId, faucetId, opts),
    mockConsume: (accountId: any, noteId: string) =>
      h.mockConsume(client, sdk, accountId, noteId),
    mockMintAndConsume: (accountId: any, faucetId: any, opts?: any) =>
      h.mockMintAndConsume(client, sdk, accountId, faucetId, opts),
    mockSend: (senderId: any, targetId: any, faucetId: any, opts?: any) =>
      h.mockSend(client, sdk, senderId, targetId, faucetId, opts),
    mockSwap: (
      accountAId: any,
      accountBId: any,
      assetAFaucetId: any,
      assetAAmount: number,
      assetBFaucetId: any,
      assetBAmount: number,
      swapNoteType?: string,
      paybackNoteType?: string
    ) =>
      h.mockSwap(
        client,
        sdk,
        accountAId,
        accountBId,
        assetAFaucetId,
        assetAAmount,
        assetBFaucetId,
        assetBAmount,
        swapNoteType,
        paybackNoteType
      ),
    executeAndApplyTransaction: (accountId: any, req: any, prover?: any) =>
      executeAndApplyTransaction(client, sdk, accountId, req, prover),
    waitForTransaction: (txId: string, maxWait?: number, interval?: number) =>
      waitForTransaction(client, sdk, txId, maxWait, interval),
    parseNetworkId: (networkId: string) => h.parseNetworkId(sdk, networkId),
    createFreshMockClient: () => h.createFreshMockClient(sdk),
    createIntegrationClient: () => h.createIntegrationClient(),
    getRpcUrl: () => getRpcUrl(),
  };
}

// ── Fixtures ──────────────────────────────────────────────────────────

export const test = base.extend<{
  client: any;
  sdk: any;
  run: <T>(
    fn: (ctx: { client: any; sdk: any; helpers: any }) => Promise<T>
  ) => Promise<T>;
}>({
  // Keep client/sdk fixtures for backward compat with tests that don't use run
  client: async ({}, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      const { client } = await createNodeMockClient();
      await use(client);
    } else {
      // Browser: client is set up by forEachTest on window.client
      await use((globalThis as any).client);
    }
  },

  sdk: async ({}, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      const rawSdk = loadNodeSdk();
      await use(createNodeSdkWrapper(rawSdk));
    } else {
      // Browser: proxy to window.* (SDK types are set up by forEachTest)
      await use(
        new Proxy(
          {
            u64: (val: number | bigint) => BigInt(val),
            u64Array: (vals: number[]) => new BigUint64Array(vals.map(BigInt)),
          },
          {
            get(target, prop) {
              if (prop in target) return target[prop];
              return (globalThis as any)[prop];
            },
          }
        )
      );
    }
  },

  /**
   * The `run` fixture: executes SDK code on both Node.js and browser.
   *
   * - Node.js: calls the callback directly with napi client/sdk
   * - Browser: stringifies the callback, sends it to page.evaluate(),
   *   where it runs with window.client and window.* SDK types
   *
   * IMPORTANT: The callback must be self-contained — it cannot reference
   * imports or variables from the test file scope (browser serializes
   * the function body). Use only { client, sdk, helpers } from the context.
   */
  run: async ({}, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      // ── Node.js: create mock client, call callback directly ──
      const { client, sdk } = await createNodeMockClient();
      const helpers = await createNodeRunHelpers(client, sdk);

      await use(async (fn) => fn({ client, sdk, helpers }));
    } else {
      // ── Browser: create a standalone page, use page.evaluate() ──
      const browser = await getRunBrowser(testInfo.project.name);
      const context = await browser.newContext();
      const page = await context.newPage();

      await setupBrowserPage(page, testInfo);

      await use(async (fn) => {
        return page.evaluate(async (fnStr) => {
          const fn = new Function("return " + fnStr)();
          const sdk = new Proxy(
            {
              u64: (v) => BigInt(v),
              u64Array: (v) => new BigUint64Array(v.map(BigInt)),
            },
            {
              get(target, prop) {
                if (prop in target) return target[prop];
                return window[prop];
              },
            }
          );
          return fn({
            client: window.client,
            sdk,
            helpers: window.helpers,
          });
        }, fn.toString());
      });

      await page.close();
      await context.close();
    }
  },
});

export { expect };
