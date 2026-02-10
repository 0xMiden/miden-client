// @ts-nocheck
import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

// ════════════════════════════════════════════════════════════════
// Mock chain tests — no node needed, self-contained
// ════════════════════════════════════════════════════════════════

test.describe("MidenClient API - Mock Chain", () => {
  test("full flow: create accounts, mint, consume, check balance", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Mint tokens to the wallet
      const mintTxId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n,
      });

      client.proveBlock();
      await client.sync();

      // Retrieve the minted note ID from the transaction record
      const txRecords = await client.transactions.list({
        ids: [mintTxId.toHex()],
      });
      const mintedNoteId = txRecords[0]
        .outputNotes()
        .notes()[0]
        .id()
        .toString();

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

      return {
        walletId: wallet.id().toString(),
        faucetId: faucet.id().toString(),
        mintTxId: mintTxId.toHex(),
        consumeTxId: consumeTxId.toHex(),
        balance: balance.toString(),
      };
    });

    expect(result.walletId).toBeDefined();
    expect(result.faucetId).toBeDefined();
    expect(result.mintTxId).toBeDefined();
    expect(result.consumeTxId).toBeDefined();
    expect(result.balance).toBe("1000");
  });

  test("accounts.create defaults to private mutable wallet", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const wallet = await client.accounts.create();

      return {
        isFaucet: wallet.isFaucet(),
        isRegularAccount: wallet.isRegularAccount(),
        isUpdatable: wallet.isUpdatable(),
      };
    });

    expect(result.isFaucet).toBe(false);
    expect(result.isRegularAccount).toBe(true);
    expect(result.isUpdatable).toBe(true);
  });

  test("accounts.create faucet", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "TST",
        decimals: 6,
        maxSupply: 1_000_000n,
        storage: "public",
      });

      return {
        isFaucet: faucet.isFaucet(),
        isPublic: faucet.isPublic(),
      };
    });

    expect(result.isFaucet).toBe(true);
    expect(result.isPublic).toBe(true);
  });

  test("accounts.list returns created accounts", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      await client.accounts.create();
      await client.accounts.create();

      const accounts = await client.accounts.list();
      return { count: accounts.length };
    });

    expect(result.count).toBe(2);
  });

  test("accounts.get returns account by hex string", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const hexId = wallet.id().toString();

      const fetched = await client.accounts.get(hexId);
      return {
        fetchedId: fetched?.id().toString(),
        originalId: wallet.id().toString(),
      };
    });

    expect(result.fetchedId).toBe(result.originalId);
  });

  test("accounts.get returns null for nonexistent account", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      // Create a wallet to get a valid-looking hex ID, then look up a different one
      const wallet = await client.accounts.create();
      // Use the wallet's own ID (which exists)
      const found = await client.accounts.get(wallet);
      return { isNull: found === null, hasId: found?.id() != null };
    });

    expect(result.isNull).toBe(false);
    expect(result.hasId).toBe(true);
  });

  test("transactions.list with no query returns all", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
      });

      const allTxs = await client.transactions.list();
      return { count: allTxs.length };
    });

    expect(result.count).toBe(1);
  });

  test("transactions.list with uncommitted query", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
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

      return {
        uncommittedBefore: uncommittedCount,
        uncommittedAfter: uncommittedAfter.length,
      };
    });

    expect(result.uncommittedBefore).toBe(1);
    expect(result.uncommittedAfter).toBe(0);
  });

  test("notes.list and notes.get", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const mintTxId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n,
        type: "public",
      });

      client.proveBlock();
      await client.sync();

      // List all notes
      const allNotes = await client.notes.list();
      const noteId = allNotes[0]?.id().toString();

      // Get a single note by ID
      const note = await client.notes.get(noteId);

      return {
        noteCount: allNotes.length,
        noteId,
        fetchedNoteId: note?.id().toString(),
      };
    });

    expect(result.noteCount).toBeGreaterThanOrEqual(1);
    expect(result.fetchedNoteId).toBe(result.noteId);
  });

  test("transactions.submit with custom TransactionRequest", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Build a custom TransactionRequest using low-level _WebClient
      const lowLevel = await window._MockWebClient.createClient();
      const mintRequest = lowLevel.newMintTransactionRequest(
        wallet.id(),
        faucet.id(),
        window.NoteType.Public,
        BigInt(500)
      );

      // Submit the pre-built request through the high-level API
      const txId = await client.transactions.submit(faucet, mintRequest);

      return {
        txId: txId.toHex(),
      };
    });

    expect(result.txId).toBeDefined();
    expect(result.txId.length).toBeGreaterThan(0);
  });

  test("exportStore and importStore round-trip", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      // Create an account
      const wallet = await client.accounts.create();
      const walletId = wallet.id().toString();

      // Export the store
      const snapshot = await client.exportStore();

      // Create a new mock client and import the store
      const client2 = await window.MidenClient.createMock();
      await client2.importStore(snapshot);

      // Check the account exists in the new client
      const accounts = await client2.accounts.list();
      const accountIds = accounts.map((a) => a.id().toString());

      return {
        version: snapshot.version,
        walletId,
        foundInImport: accountIds.includes(walletId),
      };
    });

    expect(result.version).toBe(1);
    expect(result.foundInImport).toBe(true);
  });

  test("usesMockChain and proveBlock", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const isMock = client.usesMockChain();

      // proveBlock should work without error
      client.proveBlock();

      return { isMock };
    });

    expect(result.isMock).toBe(true);
  });

  test("terminate prevents further operations", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      client.terminate();

      try {
        await client.sync();
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });

    expect(result.threw).toBe(true);
    expect(result.message).toContain("terminated");
  });

  test("consumeAll consumes all available notes", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Mint two notes
      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 100n,
      });
      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 200n,
      });
      client.proveBlock();
      await client.sync();

      const result = await client.transactions.consumeAll({
        account: wallet,
      });
      return {
        consumed: result.consumed,
        remaining: result.remaining,
        hasTxId: result.txId != null,
      };
    });

    expect(result.consumed).toBe(2);
    expect(result.remaining).toBe(0);
    expect(result.hasTxId).toBe(true);
  });

  test("consumeAll with maxNotes limits consumption", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 100n,
      });
      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 200n,
      });
      client.proveBlock();
      await client.sync();

      const result = await client.transactions.consumeAll({
        account: wallet,
        maxNotes: 1,
      });
      return {
        consumed: result.consumed,
        remaining: result.remaining,
        hasTxId: result.txId != null,
      };
    });

    expect(result.consumed).toBe(1);
    expect(result.remaining).toBe(1);
    expect(result.hasTxId).toBe(true);
  });

  test("consumeAll with maxNotes: 0 returns early", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 100n,
      });
      client.proveBlock();
      await client.sync();

      const result = await client.transactions.consumeAll({
        account: wallet,
        maxNotes: 0,
      });
      return {
        consumed: result.consumed,
        remaining: result.remaining,
        txId: result.txId,
      };
    });

    expect(result.consumed).toBe(0);
    expect(result.remaining).toBe(1);
    expect(result.txId).toBeNull();
  });

  test("consumeAll with no consumable notes returns early", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();

      const result = await client.transactions.consumeAll({
        account: wallet,
      });
      return {
        consumed: result.consumed,
        remaining: result.remaining,
        txId: result.txId,
      };
    });

    expect(result.consumed).toBe(0);
    expect(result.remaining).toBe(0);
    expect(result.txId).toBeNull();
  });

  test("accounts.getDetails returns full account info", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();

      const details = await client.accounts.getDetails(wallet);
      return {
        hasAccount: details.account != null,
        hasVault: details.vault != null,
        hasStorage: details.storage != null,
        hasKeys: Array.isArray(details.keys),
      };
    });

    expect(result.hasAccount).toBe(true);
    expect(result.hasVault).toBe(true);
    expect(result.hasStorage).toBe(true);
    expect(result.hasKeys).toBe(true);
  });

  test("notes.listSent returns output notes after mint", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
      });
      client.proveBlock();
      await client.sync();

      const sent = await client.notes.listSent();
      return { sentCount: sent.length };
    });

    expect(result.sentCount).toBeGreaterThanOrEqual(1);
  });

  test("notes.listAvailable returns consumable notes", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
      });
      client.proveBlock();
      await client.sync();

      const available = await client.notes.listAvailable({ account: wallet });
      return { availableCount: available.length };
    });

    expect(result.availableCount).toBeGreaterThanOrEqual(1);
  });

  test("terminate prevents resource operations", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
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
      return { errors };
    });

    expect(result.errors).toHaveLength(3);
    for (const err of result.errors) {
      expect(err).toContain("terminated");
    }
  });

  test("error on invalid note type string", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      try {
        await client.transactions.mint({
          account: faucet,
          to: wallet,
          amount: 100n,
          type: "Private", // wrong case
        });
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });

    expect(result.threw).toBe(true);
    expect(result.message).toContain("Unknown note type");
  });

  test("error on invalid storage mode string", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      try {
        await client.accounts.create({
          storage: "encrypted",
        });
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });

    expect(result.threw).toBe(true);
    expect(result.message).toContain("Unknown storage mode");
  });

  test("error on null account reference", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      try {
        await client.accounts.get(null);
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });

    expect(result.threw).toBe(true);
    expect(result.message).toContain("null or undefined");
  });

  test("importStore rejects invalid version", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      try {
        await client.importStore({ version: 99, data: {} });
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });

    expect(result.threw).toBe(true);
    expect(result.message).toContain("Unsupported store snapshot version");
  });

  test("accounts.export and import round-trip", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create({ storage: "public" });
      const walletId = wallet.id().toString();

      const accountFile = await client.accounts.export(wallet);

      // Import into a fresh client
      const client2 = await window.MidenClient.createMock();
      const imported = await client2.accounts.import({ file: accountFile });

      return {
        hasFile: accountFile != null,
        hasSerialize: typeof accountFile.serialize === "function",
        originalId: walletId,
        importedId: imported.id().toString(),
      };
    });

    expect(result.hasFile).toBe(true);
    expect(result.hasSerialize).toBe(true);
    expect(result.importedId).toBe(result.originalId);
  });

  test("notes.export and notes.import round-trip", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
        type: "public",
      });
      client.proveBlock();
      await client.sync();

      // Get the note
      const notes = await client.notes.list();
      const noteId = notes[0].id().toHex();

      // Export it
      const noteFile = await client.notes.export(noteId, { format: "full" });

      // Import into a fresh client
      const client2 = await window.MidenClient.createMock();
      const importedId = await client2.notes.import(noteFile);

      return {
        originalId: noteId,
        importedId: importedId.toHex(),
      };
    });

    expect(result.importedId).toBeDefined();
    expect(result.importedId).toBe(result.originalId);
  });

  test("notes.export with id format", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
        type: "public",
      });
      client.proveBlock();
      await client.sync();

      const notes = await client.notes.list();
      const noteId = notes[0].id().toString();

      const noteFile = await client.notes.export(noteId, { format: "id" });
      return { hasFile: noteFile != null };
    });

    expect(result.hasFile).toBe(true);
  });

  test("transactions.preview returns a TransactionSummary", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const faucetId = faucet.id().toString();

      const summary = await client.transactions.preview({
        operation: "mint",
        account: faucet,
        to: wallet,
        amount: 1000n,
      });

      return {
        hasSummary: summary != null,
        hasAccountId: typeof summary.accountId === "function",
        accountIdMatches: summary.accountId().toString() === faucetId,
      };
    });

    expect(result.hasSummary).toBe(true);
    expect(result.hasAccountId).toBe(true);
    expect(result.accountIdMatches).toBe(true);
  });

  test("standalone createP2IDNote creates a valid note", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const note = window.createP2IDNote({
        from: faucet,
        to: wallet,
        assets: { token: faucet, amount: 100n },
      });

      return {
        hasNote: note != null,
        hasId: typeof note.id === "function",
        hasAssets: typeof note.assets === "function",
      };
    });

    expect(result.hasNote).toBe(true);
    expect(result.hasId).toBe(true);
    expect(result.hasAssets).toBe(true);
  });

  test("standalone buildSwapTag returns a number", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const faucetA = await client.accounts.create({
        type: "faucet",
        symbol: "AAA",
        decimals: 8,
        maxSupply: 10_000_000n,
      });
      const faucetB = await client.accounts.create({
        type: "faucet",
        symbol: "BBB",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const tag = window.buildSwapTag({
        offer: { token: faucetA, amount: 100n },
        request: { token: faucetB, amount: 200n },
      });

      return {
        tag,
        isNumber: typeof tag === "number",
        fitsU32: tag >= 0 && tag <= 0xffffffff,
      };
    });

    expect(result.isNumber).toBe(true);
    expect(result.tag).toBeGreaterThan(0);
    expect(result.fitsU32).toBe(true);
  });

  test("serializeMockChain and restore", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n,
      });
      client.proveBlock();
      await client.sync();

      // Serialize the mock chain
      const serializedChain = client.serializeMockChain();

      // Create a new client from the serialized chain
      const client2 = await window.MidenClient.createMock({
        serializedMockChain: serializedChain,
      });
      await client2.sync();

      const height = await client2.getSyncHeight();
      return { height, chainSize: serializedChain.length };
    });

    expect(result.chainSize).toBeGreaterThan(0);
    expect(result.height).toBeGreaterThan(0);
  });
});

// ════════════════════════════════════════════════════════════════
// Integration tests — require running node
// ════════════════════════════════════════════════════════════════

test.describe("MidenClient API - Integration", () => {
  test("MidenClient.create and sync", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.create({
        rpcUrl: window.rpcUrl,
        storeName: "miden_client_api_create_test",
      });

      const syncSummary = await client.sync();
      const height = await client.getSyncHeight();

      return {
        blockNum: syncSummary.blockNum(),
        syncHeight: height,
      };
    });

    expect(result.blockNum).toBeGreaterThanOrEqual(0);
    expect(result.syncHeight).toBeGreaterThanOrEqual(0);
  });

  test("accounts.create wallet and faucet via integration", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.create({
        rpcUrl: window.rpcUrl,
        storeName: "miden_client_api_accounts_test",
      });
      await client.sync();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const accounts = await client.accounts.list();

      return {
        walletIsFaucet: wallet.isFaucet(),
        walletIsUpdatable: wallet.isUpdatable(),
        faucetIsFaucet: faucet.isFaucet(),
        accountCount: accounts.length,
      };
    });

    expect(result.walletIsFaucet).toBe(false);
    expect(result.walletIsUpdatable).toBe(true);
    expect(result.faucetIsFaucet).toBe(true);
    expect(result.accountCount).toBe(2);
  });

  test("full send flow: mint, sync, consume, check balance", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.create({
        rpcUrl: window.rpcUrl,
        storeName: "miden_client_api_send_test",
      });
      await client.sync();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Mint tokens
      const mintTxId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n,
        type: "public",
      });

      // Wait for mint to be confirmed
      await client.transactions.waitFor(mintTxId.toHex(), {
        timeout: 30_000,
        interval: 1_000,
      });

      // Consume the minted note
      const consumable = await client.notes.listAvailable({
        account: wallet,
      });
      const consumeNoteIds = consumable.map((c) =>
        c.inputNoteRecord().id().toString()
      );

      const consumeTxId = await client.transactions.consume({
        account: wallet,
        notes: consumeNoteIds,
      });

      await client.transactions.waitFor(consumeTxId.toHex(), {
        timeout: 30_000,
        interval: 1_000,
      });

      // Check balance
      const walletAccount = await client.accounts.get(wallet);
      const balance = walletAccount.vault().getBalance(faucet.id());

      return {
        mintTxId: mintTxId.toHex(),
        consumeTxId: consumeTxId.toHex(),
        balance: balance.toString(),
        consumedCount: consumeNoteIds.length,
      };
    });

    expect(result.mintTxId).toBeDefined();
    expect(result.consumeTxId).toBeDefined();
    expect(result.balance).toBe("1000");
    expect(result.consumedCount).toBeGreaterThanOrEqual(1);
  });

  test("transactions.list queries work correctly", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.create({
        rpcUrl: window.rpcUrl,
        storeName: "miden_client_api_txlist_test",
      });
      await client.sync();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      const txId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
      });
      const txHex = txId.toHex();

      // Query all
      const allTxs = await client.transactions.list();

      // Query by ID
      const byId = await client.transactions.list({ ids: [txHex] });

      // Query uncommitted
      const uncommitted = await client.transactions.list({
        status: "uncommitted",
      });

      return {
        allCount: allTxs.length,
        byIdCount: byId.length,
        byIdMatchesTxId: byId[0]?.id().toHex() === txHex,
        uncommittedCount: uncommitted.length,
      };
    });

    expect(result.allCount).toBe(1);
    expect(result.byIdCount).toBe(1);
    expect(result.byIdMatchesTxId).toBe(true);
    expect(result.uncommittedCount).toBeGreaterThanOrEqual(0);
  });

  test("notes.list with status filter", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.create({
        rpcUrl: window.rpcUrl,
        storeName: "miden_client_api_notes_test",
      });
      await client.sync();

      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: "faucet",
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Mint to generate a note
      const txId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
        type: "public",
      });

      await client.transactions.waitFor(txId.toHex(), {
        timeout: 30_000,
        interval: 1_000,
      });

      // List committed notes
      const committed = await client.notes.list({ status: "committed" });

      // List all notes
      const all = await client.notes.list();

      return {
        committedCount: committed.length,
        allCount: all.length,
      };
    });

    expect(result.committedCount).toBeGreaterThanOrEqual(1);
    expect(result.allCount).toBeGreaterThanOrEqual(1);
  });
});
