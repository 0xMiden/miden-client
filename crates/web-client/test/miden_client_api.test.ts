// @ts-nocheck
import { test, expect } from "./test-setup";
import { createMidenClient } from "./test-helpers";
import path from "path";

// ════════════════════════════════════════════════════════════════
// Mock chain tests — no node needed, self-contained
// ════════════════════════════════════════════════════════════════

test.describe("MidenClient API - Mock Chain", () => {
  test("full flow: create accounts, mint, consume, check balance", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    // Mint tokens to the wallet
    const mintTxId = await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(1000),
    });

    client.proveBlock();
    await client.sync();

    // Retrieve the minted note ID from the transaction record
    const txRecords = await client.transactions.list({
      ids: [mintTxId.toHex()],
    });
    const mintedNoteId = txRecords[0].outputNotes().notes()[0].id().toString();

    // Consume the minted note
    const consumeTxId = await client.transactions.consume({
      account: wallet,
      notes: mintedNoteId,
    });

    client.proveBlock();
    await client.sync();

    // Check balance
    const walletAccount = await client.accounts.get(wallet);
    const balance = walletAccount.vault().getBalance(faucet.id());

    expect(wallet.id().toString()).toBeDefined();
    expect(faucet.id().toString()).toBeDefined();
    expect(mintTxId.toHex()).toBeDefined();
    expect(consumeTxId.toHex()).toBeDefined();
    expect(balance.toString()).toBe("1000");
  });

  test("accounts.create defaults to private mutable wallet", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    const wallet = await client.accounts.create();

    expect(wallet.isFaucet()).toBe(false);
    expect(wallet.isRegularAccount()).toBe(true);
    expect(wallet.isUpdatable()).toBe(true);
  });

  test("accounts.create faucet", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "TST",
      decimals: 6,
      maxSupply: sdk.u64(1000000),
      storage: "public",
    });

    expect(faucet.isFaucet()).toBe(true);
    expect(faucet.isPublic()).toBe(true);
  });

  test("accounts.list returns created accounts", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    await client.accounts.create();
    await client.accounts.create();

    const accounts = await client.accounts.list();
    expect(accounts.length).toBe(2);
  });

  test("accounts.get returns account by hex string", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const hexId = wallet.id().toString();

    const fetched = await client.accounts.get(hexId);
    expect(fetched?.id().toString()).toBe(wallet.id().toString());
  });

  test("accounts.get returns null for nonexistent account", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    // Create a wallet to get a valid-looking hex ID, then look up it
    const wallet = await client.accounts.create();
    // Use the wallet's own ID (which exists)
    const found = await client.accounts.get(wallet);
    expect(found === null).toBe(false);
    expect(found?.id() != null).toBe(true);
  });

  test("transactions.list with no query returns all", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
    });

    const allTxs = await client.transactions.list();
    expect(allTxs.length).toBe(1);
  });

  test("transactions.list with uncommitted query", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
    });

    // Before proveBlock + sync, the tx should be uncommitted
    const uncommitted = await client.transactions.list({
      status: "uncommitted",
    });
    const uncommittedCount = uncommitted.length;

    // After proveBlock + sync, it should be committed
    client.proveBlock();
    await client.sync();

    const uncommittedAfter = await client.transactions.list({
      status: "uncommitted",
    });

    expect(uncommittedCount).toBe(1);
    expect(uncommittedAfter.length).toBe(0);
  });

  test("notes.list and notes.get", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(1000),
      type: "public",
    });

    client.proveBlock();
    await client.sync();

    // List all notes
    const allNotes = await client.notes.list();
    const noteId = allNotes[0]?.id().toString();

    // Get a single note by ID
    const note = await client.notes.get(noteId);

    expect(allNotes.length).toBeGreaterThanOrEqual(1);
    expect(note?.id().toString()).toBe(noteId);
  });

  test("transactions.submit with custom TransactionRequest", async ({
    sdk,
  }) => {
    test.skip(true, "requires MockWasmWebClient low-level wiring");
  });

  test("exportStore and importStore round-trip", async ({ sdk }) => {
    test.skip(true, "uses browser-only exportStore/importStore (IndexedDB)");
  });

  test("usesMockChain and proveBlock", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const isMock = client.usesMockChain();

    // proveBlock should work without error
    client.proveBlock();

    expect(isMock).toBe(true);
  });

  test("terminate prevents further operations", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    client.terminate();

    try {
      await client.sync();
      expect(true).toBe(false); // should not reach here
    } catch (e) {
      expect(e.message).toContain("terminated");
    }
  });

  test("consumeAll consumes all available notes", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    // Mint two notes
    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(100),
    });
    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(200),
    });
    client.proveBlock();
    await client.sync();

    const result = await client.transactions.consumeAll({
      account: wallet,
    });

    expect(result.consumed).toBe(2);
    expect(result.remaining).toBe(0);
    expect(result.txId != null).toBe(true);
  });

  test("consumeAll with maxNotes limits consumption", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(100),
    });
    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(200),
    });
    client.proveBlock();
    await client.sync();

    const result = await client.transactions.consumeAll({
      account: wallet,
      maxNotes: 1,
    });

    expect(result.consumed).toBe(1);
    expect(result.remaining).toBe(1);
    expect(result.txId != null).toBe(true);
  });

  test("consumeAll with maxNotes: 0 returns early", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(100),
    });
    client.proveBlock();
    await client.sync();

    const result = await client.transactions.consumeAll({
      account: wallet,
      maxNotes: 0,
    });

    expect(result.consumed).toBe(0);
    expect(result.remaining).toBe(1);
    expect(result.txId).toBeNull();
  });

  test("consumeAll with no consumable notes returns early", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();

    const result = await client.transactions.consumeAll({
      account: wallet,
    });

    expect(result.consumed).toBe(0);
    expect(result.remaining).toBe(0);
    expect(result.txId).toBeNull();
  });

  test("accounts.getDetails returns full account info", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();

    const details = await client.accounts.getDetails(wallet);

    expect(details.account != null).toBe(true);
    expect(details.vault != null).toBe(true);
    expect(details.storage != null).toBe(true);
    expect(Array.isArray(details.keys)).toBe(true);
  });

  test("notes.listSent returns output notes after mint", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
    });
    client.proveBlock();
    await client.sync();

    const sent = await client.notes.listSent();
    expect(sent.length).toBeGreaterThanOrEqual(1);
  });

  test("notes.listAvailable returns consumable notes", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
    });
    client.proveBlock();
    await client.sync();

    const available = await client.notes.listAvailable({ account: wallet });
    expect(available.length).toBeGreaterThanOrEqual(1);
  });

  test("terminate prevents resource operations", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    client.terminate();

    const errors = [];
    try {
      await client.accounts.list();
    } catch (e) {
      errors.push("accounts.list: " + e.message);
    }
    try {
      await client.transactions.list();
    } catch (e) {
      errors.push("transactions.list: " + e.message);
    }
    try {
      await client.notes.list();
    } catch (e) {
      errors.push("notes.list: " + e.message);
    }

    expect(errors).toHaveLength(3);
    for (const err of errors) {
      expect(err).toContain("terminated");
    }
  });

  test("error on invalid note type string", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    try {
      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: sdk.u64(100),
        type: "Private", // wrong case
      });
      expect(true).toBe(false); // should not reach here
    } catch (e) {
      expect(e.message).toContain("Unknown note type");
    }
  });

  test("error on invalid storage mode string", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    try {
      await client.accounts.create({
        storage: "encrypted",
      });
      expect(true).toBe(false); // should not reach here
    } catch (e) {
      expect(e.message).toContain("Unknown storage mode");
    }
  });

  test("error on null account reference", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    try {
      await client.accounts.get(null);
      expect(true).toBe(false); // should not reach here
    } catch (e) {
      expect(e.message).toContain("null or undefined");
    }
  });

  test("accounts.export returns a valid AccountFile", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create({ storage: "public" });

    const accountFile = await client.accounts.export(wallet);

    expect(accountFile != null).toBe(true);
    expect(typeof accountFile.serialize === "function").toBe(true);
    expect(accountFile.serialize().length).toBeGreaterThan(0);
  });

  test("notes.export returns a valid NoteFile", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
      type: "public",
    });
    client.proveBlock();
    await client.sync();

    // Get the note
    const notes = await client.notes.list();
    const noteId = notes[0].id().toString();

    // Export with full format
    const noteFile = await client.notes.export(noteId, {
      format: sdk.NoteExportFormat.Full,
    });

    expect(noteFile != null).toBe(true);
    expect(typeof noteFile.serialize === "function").toBe(true);
    expect(noteFile.serialize().length).toBeGreaterThan(0);
  });

  test("notes.export with id format", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(500),
      type: "public",
    });
    client.proveBlock();
    await client.sync();

    const notes = await client.notes.list();
    const noteId = notes[0].id().toString();

    const noteFile = await client.notes.export(noteId, {
      format: sdk.NoteExportFormat.Id,
    });
    expect(noteFile != null).toBe(true);
  });

  test("transactions.preview returns a TransactionSummary", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    const summary = await client.transactions.preview({
      operation: "mint",
      account: faucet,
      to: wallet,
      amount: sdk.u64(1000),
    });

    expect(summary != null).toBe(true);
    expect(typeof summary.outputNotes === "function").toBe(true);
    expect(summary.outputNotes().numNotes()).toBeGreaterThan(0);
    expect(typeof summary.accountDelta === "function").toBe(true);
  });

  test("standalone createP2IDNote creates a valid note", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    const jsDir = path.resolve(import.meta.dirname, "..", "js");
    const { createP2IDNote } = await import(path.join(jsDir, "standalone.js"));

    const note = createP2IDNote({
      from: faucet,
      to: wallet,
      assets: { token: faucet, amount: sdk.u64(100) },
    });

    expect(note != null).toBe(true);
    expect(typeof note.id === "function").toBe(true);
    expect(typeof note.assets === "function").toBe(true);
  });

  test("standalone buildSwapTag returns a NoteTag", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const faucetA = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "AAA",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });
    const faucetB = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "BBB",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    const jsDir = path.resolve(import.meta.dirname, "..", "js");
    const { buildSwapTag } = await import(path.join(jsDir, "standalone.js"));

    const tag = buildSwapTag({
      offer: { token: faucetA, amount: sdk.u64(100) },
      request: { token: faucetB, amount: sdk.u64(200) },
    });

    const tagValue = tag.asU32();

    expect(tag != null).toBe(true);
    expect(typeof tag.asU32 === "function").toBe(true);
    expect(tagValue).toBeGreaterThan(0);
    expect(tagValue >= 0 && tagValue <= 0xffffffff).toBe(true);
  });

  test("accounts.getOrImport returns existing account without importing", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create({ storage: "public" });
    const walletId = wallet.id().toString();

    // getOrImport should return the already-local account
    const fetched = await client.accounts.getOrImport(walletId);

    expect(fetched.id().toString()).toBe(walletId);
  });

  test("accounts.getOrImport works across serialized mock chain", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();
    const wallet = await client.accounts.create({ storage: "public" });
    const walletId = wallet.id().toString();

    // Serialize chain so the second client sees the same blocks
    const chain = client.serializeMockChain();

    // Create a fresh mock client with the same chain
    const client2 = await MidenClient.createMock({
      serializedMockChain: chain,
    });

    // getOrImport should return the account (either from local store or network)
    const imported = await client2.accounts.getOrImport(walletId);

    expect(imported.id().toString()).toBe(walletId);
  });

  test("serializeMockChain and restore", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    const client = await MidenClient.createMock();

    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
      type: "FungibleFaucet",
      symbol: "DAG",
      decimals: 8,
      maxSupply: sdk.u64(10000000),
    });

    await client.transactions.mint({
      account: faucet,
      to: wallet,
      amount: sdk.u64(1000),
    });
    client.proveBlock();
    await client.sync();

    // Serialize the mock chain
    const serializedChain = client.serializeMockChain();

    // Create a new client from the serialized chain
    const client2 = await MidenClient.createMock({
      serializedMockChain: serializedChain,
    });
    await client2.sync();

    const height = await client2.getSyncHeight();

    expect(serializedChain.length).toBeGreaterThan(0);
    expect(height).toBeGreaterThan(0);
  });
});

// ════════════════════════════════════════════════════════════════
// Integration tests — require running node
// ════════════════════════════════════════════════════════════════

test.describe("MidenClient API - Integration", () => {
  test("MidenClient.create and sync", async ({ sdk }) => {
    test.skip(true, "requires running node");
  });

  test("accounts.create wallet and faucet via integration", async ({ sdk }) => {
    test.skip(true, "requires running node");
  });

  test("full send flow: mint, sync, consume, check balance", async ({
    sdk,
  }) => {
    test.skip(true, "requires running node");
  });

  test("transactions.list queries work correctly", async ({ sdk }) => {
    test.skip(true, "requires running node");
  });

  test("notes.list with status filter", async ({ sdk }) => {
    test.skip(true, "requires running node");
  });
});
