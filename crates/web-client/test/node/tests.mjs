/**
 * Offline Node.js tests for the Miden web-client SDK.
 *
 * Uses MockWebClient (no RPC / no network) and Node's built-in test runner.
 *
 * Usage:
 *   node --test tests.mjs
 */

import { describe, it, before, after } from "node:test";
import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync, unlinkSync } from "node:fs";
import { loadSdk } from "./load-sdk.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));

// ─── Helpers ────────────────────────────────────────────────────────────────

/** Remove a SQLite file if it exists. */
function cleanDb(path) {
  if (existsSync(path)) unlinkSync(path);
  // SQLite WAL/SHM companions
  if (existsSync(path + "-wal")) unlinkSync(path + "-wal");
  if (existsSync(path + "-shm")) unlinkSync(path + "-shm");
}

/** Set the global store path used by the Node.js SQLite backend. */
function setStorePath(name) {
  const p = resolve(__dirname, `test-${name}.sqlite`);
  cleanDb(p);
  globalThis.__MIDEN_STORE_PATH = p;
  return p;
}

let sdk;

// ─── SDK Loading ────────────────────────────────────────────────────────────

describe("SDK loading", () => {
  it("loads the WASM module and exports expected symbols", async () => {
    sdk = await loadSdk();

    const keys = Object.keys(sdk);
    assert.ok(keys.length > 0, "SDK should export at least one symbol");

    // Core classes that must be present
    const required = [
      "WebClient",
      "MockWebClient",
      "AccountStorageMode",
      "NoteType",
      "TransactionFilter",
      "NoteFilter",
      "NoteFilterTypes",
      "AuthSecretKey",
      "Word",
      "Signature",
      "PublicKey",
      "Felt",
      "SigningInputs",
    ];

    for (const name of required) {
      assert.ok(keys.includes(name), `SDK should export "${name}"`);
    }
  });
});

// ─── Mock chain – account creation ──────────────────────────────────────────

describe("Mock chain - account creation", () => {
  let client;
  let dbPath;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("account-creation");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("creates a private mutable wallet", async () => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    assert.ok(wallet.isRegularAccount(), "should be a regular account");
    assert.ok(!wallet.isFaucet(), "should not be a faucet");
    assert.ok(wallet.isUpdatable(), "should be updatable (mutable)");
    assert.ok(wallet.isPrivate(), "should be private");
    assert.ok(!wallet.isPublic(), "should not be public");
    assert.ok(wallet.isNew(), "should be new");
  });

  it("creates a public immutable wallet", async () => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.public(),
      false,
      0
    );
    assert.ok(wallet.isRegularAccount());
    assert.ok(!wallet.isFaucet());
    assert.ok(!wallet.isUpdatable(), "should be immutable");
    assert.ok(wallet.isPublic(), "should be public");
    assert.ok(!wallet.isPrivate());
  });

  it("creates a private fungible faucet", async () => {
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "PFA",
      8,
      BigInt(10000000),
      0
    );
    assert.ok(faucet.isFaucet(), "should be a faucet");
    assert.ok(!faucet.isRegularAccount());
    assert.ok(faucet.isPrivate());
    assert.ok(!faucet.isPublic());
  });

  it("creates a public fungible faucet", async () => {
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.public(),
      false,
      "PFB",
      8,
      BigInt(10000000),
      0
    );
    assert.ok(faucet.isFaucet());
    assert.ok(!faucet.isRegularAccount());
    assert.ok(faucet.isPublic());
    assert.ok(!faucet.isPrivate());
  });

  it("rejects non-fungible faucet creation", async () => {
    await assert.rejects(
      () =>
        client.newFaucet(
          sdk.AccountStorageMode.public(),
          true,
          "NFT",
          8,
          BigInt(10000000),
          0
        ),
      /Non-fungible faucets are not supported yet/
    );
  });

  it("rejects invalid token symbol", async () => {
    await assert.rejects(
      () =>
        client.newFaucet(
          sdk.AccountStorageMode.public(),
          false,
          "INVALID_TOKEN",
          8,
          BigInt(10000000),
          0
        ),
      /token symbol should have length between 1 and 6 characters/
    );
  });

  it("wallet has correct account type flags", async () => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    assert.ok(wallet.isRegularAccount());
    assert.ok(!wallet.isFaucet());
    assert.equal(wallet.nonce().toString(), "0");
  });

  it("faucet has correct account type flags", async () => {
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "FLG",
      8,
      BigInt(10000000),
      0
    );
    assert.ok(faucet.isFaucet());
    assert.ok(!faucet.isRegularAccount());
    assert.ok(!faucet.isUpdatable());
  });
});

// ─── Mock chain – mint and consume ──────────────────────────────────────────

describe("Mock chain - mint and consume", () => {
  let client;
  let dbPath;
  let wallet;
  let faucet;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("mint-consume");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();

    wallet = await client.newWallet(sdk.AccountStorageMode.private(), true, 0);
    faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "MNC",
      8,
      BigInt(10000000),
      0
    );
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("mints tokens and verifies faucet output note", async () => {
    const mintRequest = client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(1000)
    );

    const mintTxId = await client.submitNewTransaction(
      faucet.id(),
      mintRequest
    );
    assert.ok(mintTxId, "submitNewTransaction should return a transaction id");

    await client.proveBlock();
    await client.syncState();

    // Verify transaction record exists
    const [mintRecord] = await client.getTransactions(
      sdk.TransactionFilter.ids([mintTxId])
    );
    assert.ok(mintRecord, "mint transaction record should exist");

    const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();
    assert.ok(mintedNoteId, "minted note should have an id");
  });

  it("consumes minted note and wallet receives balance", async () => {
    // Mint fresh tokens
    const mintRequest = client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(500)
    );

    const mintTxId = await client.submitNewTransaction(
      faucet.id(),
      mintRequest
    );
    await client.proveBlock();
    await client.syncState();

    // Get the minted note
    const [mintRecord] = await client.getTransactions(
      sdk.TransactionFilter.ids([mintTxId])
    );
    const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();

    const mintedNoteRecord = await client.getInputNote(mintedNoteId);
    assert.ok(mintedNoteRecord, "minted note should be in store");

    const mintedNote = mintedNoteRecord.toNote();

    // Consume it
    const consumeRequest = client.newConsumeTransactionRequest([mintedNote]);
    await client.submitNewTransaction(wallet.id(), consumeRequest);
    await client.proveBlock();
    await client.syncState();

    // Check balance
    const updatedWallet = await client.getAccount(wallet.id());
    const balance = updatedWallet.vault().getBalance(faucet.id());
    assert.ok(balance > BigInt(0), "wallet should have a positive balance");
  });
});

// ─── Mock chain – send transaction ──────────────────────────────────────────

describe("Mock chain - send transaction", () => {
  let client;
  let dbPath;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("send-tx");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("sends tokens between wallets and verifies balances", async () => {
    const wallet1 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    const wallet2 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "SND",
      8,
      BigInt(10000000),
      0
    );

    // Mint tokens to wallet1
    const mintRequest = client.newMintTransactionRequest(
      wallet1.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(1000)
    );
    const mintTxId = await client.submitNewTransaction(
      faucet.id(),
      mintRequest
    );
    await client.proveBlock();
    await client.syncState();

    // Consume minted note in wallet1
    const [mintRecord] = await client.getTransactions(
      sdk.TransactionFilter.ids([mintTxId])
    );
    const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();
    const mintedNoteRecord = await client.getInputNote(mintedNoteId);
    const mintedNote = mintedNoteRecord.toNote();

    const consumeRequest = client.newConsumeTransactionRequest([mintedNote]);
    await client.submitNewTransaction(wallet1.id(), consumeRequest);
    await client.proveBlock();
    await client.syncState();

    // Verify wallet1 has 1000
    let w1Account = await client.getAccount(wallet1.id());
    assert.equal(
      w1Account.vault().getBalance(faucet.id()),
      BigInt(1000),
      "wallet1 should have 1000 tokens"
    );

    // Send 300 from wallet1 to wallet2
    const sendRequest = client.newSendTransactionRequest(
      wallet1.id(),
      wallet2.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(300),
      undefined,
      null
    );
    const sendTxId = await client.submitNewTransaction(
      wallet1.id(),
      sendRequest
    );
    await client.proveBlock();
    await client.syncState();

    // Consume sent note in wallet2
    const [sendRecord] = await client.getTransactions(
      sdk.TransactionFilter.ids([sendTxId])
    );
    const sentNoteId = sendRecord.outputNotes().notes()[0].id().toString();
    const sentNoteRecord = await client.getInputNote(sentNoteId);
    const sentNote = sentNoteRecord.toNote();

    const consumeRequest2 = client.newConsumeTransactionRequest([sentNote]);
    await client.submitNewTransaction(wallet2.id(), consumeRequest2);
    await client.proveBlock();
    await client.syncState();

    // Verify final balances
    w1Account = await client.getAccount(wallet1.id());
    const w2Account = await client.getAccount(wallet2.id());

    assert.equal(
      w1Account.vault().getBalance(faucet.id()),
      BigInt(700),
      "wallet1 should have 700 tokens after sending 300"
    );
    assert.equal(
      w2Account.vault().getBalance(faucet.id()),
      BigInt(300),
      "wallet2 should have 300 tokens"
    );
  });
});

// ─── Cryptographic primitives ───────────────────────────────────────────────

describe("Cryptographic primitives", () => {
  before(async () => {
    if (!sdk) sdk = await loadSdk();
    // Crypto tests need the WASM module loaded but no client/store
    setStorePath("crypto");
  });

  for (const [method, label] of [
    ["rpoFalconWithRNG", "Falcon512"],
    ["ecdsaWithRNG", "ECDSA"],
  ]) {
    describe(label, () => {
      it("signs and verifies a message", () => {
        const secretKey = sdk.AuthSecretKey[method]();
        const message = new sdk.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
        const signature = secretKey.sign(message);
        const isValid = secretKey.publicKey().verify(message, signature);
        assert.ok(isValid, "signature should be valid");
      });

      it("rejects wrong message", () => {
        const secretKey = sdk.AuthSecretKey[method]();
        const message = new sdk.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
        const wrongMessage = new sdk.Word(new BigUint64Array([5n, 6n, 7n, 8n]));
        const signature = secretKey.sign(message);
        const isValid = secretKey.publicKey().verify(wrongMessage, signature);
        assert.ok(!isValid, "signature should not verify wrong message");
      });

      it("rejects wrong key", () => {
        const secretKey = sdk.AuthSecretKey[method]();
        const differentKey = sdk.AuthSecretKey[method]();
        const message = new sdk.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
        const signature = secretKey.sign(message);
        const isValid = differentKey.publicKey().verify(message, signature);
        assert.ok(!isValid, "signature should not verify with different key");
      });

      it("serializes and deserializes signature", () => {
        const secretKey = sdk.AuthSecretKey[method]();
        const message = new sdk.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
        const signature = secretKey.sign(message);
        const serialized = signature.serialize();
        const deserialized = sdk.Signature.deserialize(serialized);
        const isValid = secretKey.publicKey().verify(message, deserialized);
        assert.ok(isValid, "deserialized signature should verify");
      });

      it("serializes and deserializes public key", () => {
        const secretKey = sdk.AuthSecretKey[method]();
        const publicKey = secretKey.publicKey();
        const serialized = publicKey.serialize();
        const deserialized = sdk.PublicKey.deserialize(serialized);
        const reserialized = deserialized.serialize();
        assert.deepEqual(
          Array.from(serialized),
          Array.from(reserialized),
          "public key should survive round-trip serialization"
        );
      });
    });
  }
});

// ─── Account queries ────────────────────────────────────────────────────────

describe("Account queries", () => {
  let client;
  let dbPath;
  let walletId;
  let faucetId;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("account-queries");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();

    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    walletId = wallet.id();

    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "AQR",
      8,
      BigInt(10000000),
      0
    );
    faucetId = faucet.id();
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("getAccounts returns created accounts", async () => {
    const accounts = await client.getAccounts();
    assert.ok(accounts.length >= 2, "should have at least 2 accounts");

    const ids = accounts.map((a) => a.id().toString());
    assert.ok(ids.includes(walletId.toString()), "should include the wallet");
    assert.ok(ids.includes(faucetId.toString()), "should include the faucet");
  });

  it("getAccount returns full account state", async () => {
    const account = await client.getAccount(walletId);
    assert.ok(account, "account should exist");
    assert.equal(
      account.id().toString(),
      walletId.toString(),
      "account id should match"
    );
    assert.ok(account.vault(), "account should have a vault");
    assert.ok(account.storage(), "account should have storage");
    assert.ok(account.code(), "account should have code");
  });
});

// ─── Transaction queries ────────────────────────────────────────────────────

describe("Transaction queries", () => {
  let client;
  let dbPath;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("tx-queries");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("getTransactions returns submitted transactions", async () => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      0
    );
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "TXQ",
      8,
      BigInt(10000000),
      0
    );

    // Submit a mint transaction
    const mintRequest = client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(100)
    );
    const mintTxId = await client.submitNewTransaction(
      faucet.id(),
      mintRequest
    );

    // Before proveBlock, transaction should be uncommitted
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );
    const uncommittedIds = uncommitted.map((tx) => tx.id().toHex());
    assert.ok(
      uncommittedIds.includes(mintTxId.toHex()),
      "mint tx should be uncommitted before proveBlock"
    );

    // After proveBlock + sync, all should appear in getTransactions(all())
    await client.proveBlock();
    await client.syncState();

    const all = await client.getTransactions(sdk.TransactionFilter.all());
    assert.ok(all.length >= 1, "should have at least 1 transaction");

    const allIds = all.map((tx) => tx.id().toHex());
    assert.ok(
      allIds.includes(mintTxId.toHex()),
      "mint tx should appear in all transactions"
    );

    // After proving, uncommitted should be empty
    const uncommittedAfter = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );
    assert.equal(
      uncommittedAfter.length,
      0,
      "no uncommitted transactions after proveBlock"
    );
  });
});

// ─── Note queries ───────────────────────────────────────────────────────────

describe("Note queries", () => {
  let client;
  let dbPath;
  let wallet;
  let faucet;

  before(async () => {
    if (!sdk) sdk = await loadSdk();
    dbPath = setStorePath("note-queries");
    client = await sdk.MockWebClient.createClient();
    await client.syncState();

    wallet = await client.newWallet(sdk.AccountStorageMode.private(), true, 0);
    faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "NTQ",
      8,
      BigInt(10000000),
      0
    );

    // Mint to produce notes
    const mintRequest = client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Public,
      BigInt(1000)
    );
    await client.submitNewTransaction(faucet.id(), mintRequest);
    await client.proveBlock();
    await client.syncState();
  });

  after(() => {
    if (client) client.terminate();
    if (dbPath) cleanDb(dbPath);
  });

  it("getInputNotes returns expected notes after mint", async () => {
    const filter = new sdk.NoteFilter(sdk.NoteFilterTypes.All);
    const notes = await client.getInputNotes(filter);
    assert.ok(notes.length >= 1, "should have at least 1 input note");
  });

  it("getOutputNotes returns expected notes after mint", async () => {
    const filter = new sdk.NoteFilter(sdk.NoteFilterTypes.All);
    const notes = await client.getOutputNotes(filter);
    assert.ok(notes.length >= 1, "should have at least 1 output note");
  });
});
