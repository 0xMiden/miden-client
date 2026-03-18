/**
 * Platform-agnostic test helpers for mock chain tests.
 *
 * All helpers take `(client, sdk, ...)` params and use the mock chain pattern:
 * submitNewTransaction → proveBlock → syncState.
 */
// @ts-nocheck
import {
  executeAndApplyTransaction,
  waitForTransaction,
  loadNodeSdk,
  wrapNodeClient,
  createNodeIntegrationClient,
} from "./test-setup";
import { getRpcUrl, RUN_ID } from "./playwright.global.setup";
import path from "path";
import fs from "fs";
import os from "os";

let _helperCounter = 0;

function tmpDir(): string {
  const dir = path.join(
    os.tmpdir(),
    `miden-helper-${process.pid}-${++_helperCounter}`
  );
  fs.mkdirSync(path.join(dir, "keystore"), { recursive: true });
  return dir;
}

function norm(val: any): any {
  if (typeof val === "bigint") return Number(val);
  if (val instanceof BigUint64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof Uint8Array || Buffer.isBuffer(val)) return Array.from(val);
  return val;
}

// ── Mock chain transaction helpers ───────────────────────────────────

export async function setupWalletAndFaucet(
  client: any,
  sdk: any
): Promise<{
  walletId: string;
  faucetId: string;
  wallet: any;
  faucet: any;
  walletCommitment: string;
}> {
  const wallet = await client.newWallet(
    sdk.AccountStorageMode.private(),
    true,
    sdk.AuthScheme.AuthRpoFalcon512
  );
  const faucet = await client.newFaucet(
    sdk.AccountStorageMode.private(),
    false,
    "DAG",
    8,
    sdk.u64(10000000),
    sdk.AuthScheme.AuthRpoFalcon512
  );
  return {
    walletId: wallet.id().toString(),
    faucetId: faucet.id().toString(),
    walletCommitment: wallet.to_commitment().toHex(),
    wallet,
    faucet,
  };
}

/**
 * Mints tokens on the mock chain and commits the block.
 * Returns transaction ID and created note ID.
 */
export async function mockMint(
  client: any,
  sdk: any,
  targetId: any,
  faucetId: any,
  opts?: { amount?: number; publicNote?: boolean; skipSync?: boolean }
): Promise<{
  transactionId: string;
  createdNoteId: string;
  numOutputNotesCreated: number;
}> {
  const amount = opts?.amount ?? 1000;
  const noteType = opts?.publicNote
    ? sdk.NoteType.Public
    : sdk.NoteType.Private;

  const mintRequest = await client.newMintTransactionRequest(
    targetId,
    faucetId,
    noteType,
    sdk.u64(amount)
  );
  const txId = await client.submitNewTransaction(faucetId, mintRequest);

  if (!opts?.skipSync) {
    await client.proveBlock();
    await client.syncState();
  }

  const [txRecord] = await client.getTransactions(
    sdk.TransactionFilter.ids([txId])
  );
  const notes = txRecord.outputNotes().notes();
  const createdNoteId = notes[0].id().toString();

  return {
    transactionId: txId.toHex(),
    createdNoteId,
    numOutputNotesCreated: notes.length,
  };
}

/**
 * Consumes a note on the mock chain and commits the block.
 */
export async function mockConsume(
  client: any,
  sdk: any,
  accountId: any,
  noteId: string
): Promise<{
  transactionId: string;
  targetAccountBalance?: string;
  faucetId?: any;
}> {
  const inputNoteRecord = await client.getInputNote(noteId);
  if (!inputNoteRecord) throw new Error(`Note ${noteId} not found`);

  const note = inputNoteRecord.toNote();
  const consumeRequest = client.newConsumeTransactionRequest([note]);
  const txId = await client.submitNewTransaction(accountId, consumeRequest);
  await client.proveBlock();
  await client.syncState();

  return {
    transactionId: txId.toHex(),
  };
}

/**
 * Mints tokens and then consumes them, with two block advances.
 */
export async function mockMintAndConsume(
  client: any,
  sdk: any,
  accountId: any,
  faucetId: any,
  opts?: { amount?: number; publicNote?: boolean }
): Promise<{
  mintTransactionId: string;
  consumeTransactionId: string;
  createdNoteId: string;
  targetAccountBalance: string;
}> {
  const { transactionId: mintTxId, createdNoteId } = await mockMint(
    client,
    sdk,
    accountId,
    faucetId,
    opts
  );
  const { transactionId: consumeTxId } = await mockConsume(
    client,
    sdk,
    accountId,
    createdNoteId
  );

  const account = await client.getAccount(accountId);
  const balance = account.vault().getBalance(faucetId).toString();

  return {
    mintTransactionId: mintTxId,
    consumeTransactionId: consumeTxId,
    createdNoteId,
    targetAccountBalance: balance,
  };
}

/**
 * Sends tokens from one account to another on the mock chain.
 * Mints to sender, consumes, then sends.
 */
export async function mockSend(
  client: any,
  sdk: any,
  senderId: any,
  targetId: any,
  faucetId: any,
  opts?: { amount?: number; recallHeight?: number }
): Promise<{ sendCreatedNoteIds: string[] }> {
  // Mint to sender
  const { createdNoteId: mintedNoteId } = await mockMint(
    client,
    sdk,
    senderId,
    faucetId
  );

  // Consume the minted note
  await mockConsume(client, sdk, senderId, mintedNoteId);

  // Send
  const sendAmount = opts?.amount ?? 100;
  const sendRequest = await client.newSendTransactionRequest(
    senderId,
    targetId,
    faucetId,
    sdk.NoteType.Public,
    sdk.u64(sendAmount),
    opts?.recallHeight ?? null,
    null
  );
  const sendTxId = await client.submitNewTransaction(senderId, sendRequest);
  await client.proveBlock();
  await client.syncState();

  const [sendTxRecord] = await client.getTransactions(
    sdk.TransactionFilter.ids([sendTxId])
  );
  const sendNotes = sendTxRecord.outputNotes().notes();
  const sendCreatedNoteIds = sendNotes.map((n: any) => n.id().toString());

  return { sendCreatedNoteIds };
}

/**
 * Performs an atomic swap between two accounts on the mock chain.
 */
export async function mockSwap(
  client: any,
  sdk: any,
  accountAId: any,
  accountBId: any,
  assetAFaucetId: any,
  assetAAmount: number,
  assetBFaucetId: any,
  assetBAmount: number,
  swapNoteType: string = "private",
  paybackNoteType: string = "private"
): Promise<{
  accountAAssets: { assetId: string; amount: string }[];
  accountBAssets: { assetId: string; amount: string }[];
}> {
  const noteTypeA =
    swapNoteType === "public" ? sdk.NoteType.Public : sdk.NoteType.Private;
  const noteTypeB =
    paybackNoteType === "public" ? sdk.NoteType.Public : sdk.NoteType.Private;

  // Swap transaction
  const swapRequest = await client.newSwapTransactionRequest(
    accountAId,
    assetAFaucetId,
    sdk.u64(assetAAmount),
    assetBFaucetId,
    sdk.u64(assetBAmount),
    noteTypeA,
    noteTypeB
  );

  const expectedOutputNotes = swapRequest.expectedOutputOwnNotes();
  const expectedPaybackNoteDetails = swapRequest
    .expectedFutureNotes()
    .map((futureNote: any) => futureNote.noteDetails);

  const swapTxId = await client.submitNewTransaction(accountAId, swapRequest);
  await client.proveBlock();
  await client.syncState();

  // Consume swap note for account B
  const swapNoteId = expectedOutputNotes[0].id().toString();
  const swapNoteRecord = await client.getInputNote(swapNoteId);
  if (!swapNoteRecord) throw new Error(`Swap note ${swapNoteId} not found`);

  const swapNote = swapNoteRecord.toNote();
  const consumeRequest1 = client.newConsumeTransactionRequest([swapNote]);
  await client.submitNewTransaction(accountBId, consumeRequest1);
  await client.proveBlock();
  await client.syncState();

  // Consume payback note for account A
  const paybackNoteId = expectedPaybackNoteDetails[0].id().toString();
  const paybackNoteRecord = await client.getInputNote(paybackNoteId);
  if (!paybackNoteRecord)
    throw new Error(`Payback note ${paybackNoteId} not found`);

  const paybackNote = paybackNoteRecord.toNote();
  const consumeRequest2 = client.newConsumeTransactionRequest([paybackNote]);
  await client.submitNewTransaction(accountAId, consumeRequest2);
  await client.proveBlock();
  await client.syncState();

  // Fetch final assets
  const accountA = await client.getAccount(accountAId);
  const accountAAssets = accountA
    ?.vault()
    .fungibleAssets()
    .map((asset: any) => ({
      assetId: asset.faucetId().toString(),
      amount: asset.amount().toString(),
    }));

  const accountB = await client.getAccount(accountBId);
  const accountBAssets = accountB
    ?.vault()
    .fungibleAssets()
    .map((asset: any) => ({
      assetId: asset.faucetId().toString(),
      amount: asset.amount().toString(),
    }));

  return { accountAAssets, accountBAssets };
}

// ── Utility helpers ──────────────────────────────────────────────────

export function parseNetworkId(sdk: any, networkId: string): any {
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
      throw new Error(
        `Invalid network ID: ${networkId}. Expected one of: ${Object.keys(map).join(", ")}, or a valid custom network ID`
      );
    }
  }
  return parsed;
}

/**
 * Creates a fresh mock client (separate from the test fixture's client).
 * Useful for tests that need multiple independent clients.
 */
export async function createFreshMockClient(sdk: any): Promise<any> {
  const rawSdk = loadNodeSdk();
  const dir = tmpDir();

  const rawClient = new rawSdk.WebClient();
  await rawClient.createMockClient(
    path.join(dir, "store.db"),
    path.join(dir, "keystore"),
    null,
    null,
    null
  );

  return wrapNodeClient(rawClient, rawSdk);
}

// ── MidenClient factory ──────────────────────────────────────────────

let _midenClientClass: any = null;

function makeArrayPolyfill() {
  return function (items: any[]) {
    const arr = Array.isArray(items) ? [...items] : [items];
    (arr as any).get = (i: number) => arr[i];
    (arr as any).replaceAt = (i: number, val: any) => {
      arr[i] = val;
      return arr;
    };
    return arr;
  };
}

function wrapClass(Cls: any): any {
  const Wrapper: any = function (...args: any[]) {
    return new Cls(...args.map(norm));
  };
  Wrapper.prototype = Cls.prototype;
  for (const key of Object.getOwnPropertyNames(Cls)) {
    if (key === "prototype" || key === "length" || key === "name") continue;
    const desc = Object.getOwnPropertyDescriptor(Cls, key);
    if (desc && typeof desc.value === "function") {
      Wrapper[key] = (...args: any[]) => desc.value.apply(Cls, args.map(norm));
    } else if (desc) {
      try {
        Object.defineProperty(Wrapper, key, desc);
      } catch {
        /* skip */
      }
    }
  }
  return Wrapper;
}

/**
 * Wraps a raw napi WebClient for MidenClient compatibility.
 * Handles syncState → syncStateImpl, BigInt → Number, null → undefined.
 */
function wrapClientForMidenClient(
  rawClient: any,
  rawSdk: any,
  storeName?: string
): any {
  return new Proxy(rawClient, {
    get(target, prop) {
      if (prop === "syncState")
        return (...args: any[]) => target.syncStateImpl(...args);
      if (prop === "syncStateWithTimeout") return () => target.syncStateImpl();
      if (prop === "storeName") return storeName || "mock";
      if (prop === "wasmWebClient") return target;
      if (prop === "proveBlock") return async () => target.proveBlock();
      if (prop === "newWallet") {
        return (mode: any, mutable: any, authScheme: any, seed?: any) => {
          const normSeed =
            seed instanceof Uint8Array || Buffer.isBuffer(seed)
              ? Array.from(seed)
              : seed;
          return target.newWallet(mode, mutable, authScheme, normSeed ?? null);
        };
      }
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
            norm(maxSupply),
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
            norm(amount)
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
            norm(amount),
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
            norm(assetAAmount),
            assetBFaucet,
            norm(assetBAmount),
            ...rest
          );
      }
      const val = target[prop];
      if (typeof val === "function") {
        const bound = val.bind(target);
        return (...args: any[]) => {
          const normalizedArgs = args.map(norm);
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
 * Loads, configures, and returns the MidenClient class for Node.js.
 * MidenClient.createMock() will create napi-backed mock clients.
 */
export async function createMidenClient(sdk: any): Promise<any> {
  if (_midenClientClass) return _midenClientClass;

  const rawSdk = loadNodeSdk();
  const jsDir = path.resolve(import.meta.dirname, "..", "js");

  const arrayPolyfills: Record<string, any> = {};
  for (const name of [
    "AccountArray",
    "AccountIdArray",
    "ForeignAccountArray",
    "NoteRecipientArray",
    "OutputNoteArray",
    "StorageSlotArray",
    "TransactionScriptInputPairArray",
    "FeltArray",
    "OutputNotesArray",
    "NoteAndArgsArray",
    "NoteDetailsAndTagArray",
    "NoteIdAndArgsArray",
  ]) {
    arrayPolyfills[name] = makeArrayPolyfill();
  }

  const wrappedTypes = {
    ...rawSdk,
    ...arrayPolyfills,
    AccountBuilder: wrapClass(rawSdk.AccountBuilder),
    AccountComponent: wrapClass(rawSdk.AccountComponent),
    AuthSecretKey: wrapClass(rawSdk.AuthSecretKey),
    Felt: wrapClass(rawSdk.Felt),
    FungibleAsset: wrapClass(rawSdk.FungibleAsset),
    Word: wrapClass(rawSdk.Word),
    NoteTag: wrapClass(rawSdk.NoteTag),
  };

  // MockWasmWebClient matching browser's arg order: (serializedMockChain, serializedNoteTransport, seed)
  const MockWasmWebClient = {
    createClient: async (
      serializedMockChain?: any,
      serializedNoteTransport?: any,
      seed?: any
    ) => {
      const dir = tmpDir();
      const client = new rawSdk.WebClient();
      await client.createMockClient(
        path.join(dir, "store.db"),
        path.join(dir, "keystore"),
        norm(seed) ?? null,
        norm(serializedMockChain) ?? null,
        norm(serializedNoteTransport) ?? null
      );
      return wrapClientForMidenClient(client, rawSdk, "mock");
    },
  };

  // WasmWebClient (for integration tests)
  const WasmWebClient = {
    buildSwapTag: (...args: any[]) =>
      rawSdk.WebClient.buildSwapTag(...args.map(norm)),
    createClient: async (
      rpcUrl?: string,
      noteTransportUrl?: any,
      seed?: any,
      storeName?: string
    ) => {
      const dir = tmpDir();
      const client = new rawSdk.WebClient();
      await client.createClient(
        rpcUrl ?? null,
        noteTransportUrl ?? null,
        norm(seed) ?? null,
        path.join(dir, `${storeName || "store"}.db`),
        path.join(dir, "keystore"),
        false
      );
      return wrapClientForMidenClient(client, rawSdk, storeName);
    },
  };

  const getWasmOrThrow = async () => wrappedTypes;

  const { MidenClient } = await import(path.join(jsDir, "client.js"));
  MidenClient._WasmWebClient = WasmWebClient;
  MidenClient._MockWasmWebClient = MockWasmWebClient;
  MidenClient._getWasmOrThrow = getWasmOrThrow;

  // Load standalone helpers
  try {
    const standalone = await import(path.join(jsDir, "standalone.js"));
    if (standalone._setWasm) standalone._setWasm(await getWasmOrThrow());
    if (standalone._setWebClient) standalone._setWebClient(WasmWebClient);
  } catch {
    // standalone tests will skip
  }

  _midenClientClass = MidenClient;
  return MidenClient;
}

// ── Integration test helpers ─────────────────────────────────────────

let _integrationCounter = 0;

/**
 * Creates an integration client connected to a real node.
 * Returns { client, sdk } or null if no node is reachable.
 */
export async function createIntegrationClient(): Promise<{
  client: any;
  sdk: any;
} | null> {
  const rpcUrl = getRpcUrl();
  const storeName = `integration_${RUN_ID}_${++_integrationCounter}`;
  try {
    return await createNodeIntegrationClient(rpcUrl, storeName);
  } catch {
    return null;
  }
}

/**
 * Mints tokens using integration flow (executeAndApplyTransaction + waitForTransaction).
 * Requires a running node.
 */
export async function integrationMint(
  client: any,
  sdk: any,
  targetId: any,
  faucetId: any,
  opts?: { amount?: number; publicNote?: boolean; sync?: boolean }
): Promise<{
  transactionId: string;
  createdNoteId: string;
  numOutputNotesCreated: number;
}> {
  const amount = opts?.amount ?? 1000;
  const noteType = opts?.publicNote
    ? sdk.NoteType.Public
    : sdk.NoteType.Private;
  const shouldSync = opts?.sync !== false;

  await client.syncState();

  const mintRequest = await client.newMintTransactionRequest(
    targetId,
    faucetId,
    noteType,
    sdk.u64(amount)
  );
  const result = await executeAndApplyTransaction(
    client,
    sdk,
    faucetId,
    mintRequest
  );

  const transactionId = result.executedTransaction().id().toHex();
  const createdNoteId = result.createdNotes().notes()[0].id().toString();
  const numOutputNotesCreated = result.createdNotes().numNotes();

  if (shouldSync) {
    await waitForTransaction(client, sdk, transactionId);
  }

  return { transactionId, createdNoteId, numOutputNotesCreated };
}

/**
 * Consumes a note using integration flow.
 */
export async function integrationConsume(
  client: any,
  sdk: any,
  accountId: any,
  faucetId: any,
  noteId: string
): Promise<{ transactionId: string; targetAccountBalance: string }> {
  await client.syncState();

  const inputNoteRecord = await client.getInputNote(noteId);
  if (!inputNoteRecord) throw new Error(`Note ${noteId} not found`);

  const note = inputNoteRecord.toNote();
  const consumeRequest = client.newConsumeTransactionRequest([note]);
  const result = await executeAndApplyTransaction(
    client,
    sdk,
    accountId,
    consumeRequest
  );

  const transactionId = result.executedTransaction().id().toHex();
  await waitForTransaction(client, sdk, transactionId);

  const account = await client.getAccount(accountId);
  const balance = account.vault().getBalance(faucetId).toString();

  return { transactionId, targetAccountBalance: balance };
}
