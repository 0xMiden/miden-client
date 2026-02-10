import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("Store Isolation Tests", () => {
  test("default store name follows MidenClientDB_{network_id} pattern", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        undefined
      );
      await client.syncState();

      const databases = await window.indexedDB.databases();
      const dbNames = databases.map((db) => db.name);

      return {
        dbNames,
      };
    });

    const hasDefaultPattern = result.dbNames.some(
      (name) => name && name.startsWith("MidenClientDB_")
    );
    expect(hasDefaultPattern).toBe(true);
  });

  test("creates separate stores with isolated accounts", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client1 = window.client;

      await client1.newWallet(window.AccountStorageMode.private(), true, 0);

      const client2 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "IsolatedStore1"
      );
      await client2.syncState();

      const databases = await window.indexedDB.databases();
      const dbNames = databases.map((db) => db.name);

      const accounts1 = await client1.getAccounts();
      const accounts2 = await client2.getAccounts();

      return {
        accounts1Len: accounts1.length,
        accounts2Len: accounts2.length,
        dbNames,
        storeName: window.storeName,
      };
    });

    expect(result.dbNames).toContain(result.storeName);
    expect(result.dbNames).toContain("IsolatedStore1");

    expect(result.accounts1Len).toBe(1);
    expect(result.accounts2Len).toBe(0);
  });

  test("reconnecting to same store preserves data", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client1 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "PersistentStore"
      );
      await client1.syncState();

      const wallet = await client1.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const walletId = wallet.id().toString();

      const client1b = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "PersistentStore"
      );

      const accounts = await client1b.getAccounts();
      const accountIds = accounts.map((a: any) => a.id().toString());

      return {
        originalWalletId: walletId,
        reconnectedAccountIds: accountIds,
        accountCount: accounts.length,
      };
    });

    expect(result.accountCount).toBe(1);
    expect(result.reconnectedAccountIds).toContain(result.originalWalletId);
  });

  test("custom store name creates isolated database", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const customStoreName = "MyCustomStore_v1";
      const client = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        customStoreName
      );
      await client.syncState();

      await client.newWallet(window.AccountStorageMode.private(), true, 0);
      await client.newWallet(window.AccountStorageMode.private(), true, 0);

      const databases = await window.indexedDB.databases();
      const dbNames = databases.map((db) => db.name);

      const accounts = await client.getAccounts();

      return {
        dbNames,
        expectedDbName: customStoreName,
        accountCount: accounts.length,
      };
    });

    expect(result.dbNames).toContain(result.expectedDbName);
    expect(result.accountCount).toBe(2);
  });

  test("concurrent writes to different stores don't interfere", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client1 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "ConcurrentStore1"
      );
      const client2 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "ConcurrentStore2"
      );

      await Promise.all([client1.syncState(), client2.syncState()]);

      await Promise.all([
        client1.newWallet(window.AccountStorageMode.private(), true, 0),
        client1.newWallet(window.AccountStorageMode.private(), true, 0),
        client2.newWallet(window.AccountStorageMode.private(), true, 0),
      ]);

      const [accounts1, accounts2] = await Promise.all([
        client1.getAccounts(),
        client2.getAccounts(),
      ]);

      return {
        accounts1Len: accounts1.length,
        accounts2Len: accounts2.length,
      };
    });

    expect(result.accounts1Len).toBe(2);
    expect(result.accounts2Len).toBe(1);
  });

  test("multiple accounts per store remain isolated", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client1 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "MultiAccount1"
      );
      const client2 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "MultiAccount2"
      );

      await Promise.all([client1.syncState(), client2.syncState()]);

      const wallet1a = await client1.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const wallet1b = await client1.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const wallet1c = await client1.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const wallet2a = await client2.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const accounts1 = await client1.getAccounts();
      const accounts2 = await client2.getAccounts();

      const ids1 = accounts1.map((a: any) => a.id().toString());
      const ids2 = accounts2.map((a: any) => a.id().toString());

      return {
        accounts1Len: accounts1.length,
        accounts2Len: accounts2.length,
        wallet1aInClient1: ids1.includes(wallet1a.id().toString()),
        wallet1bInClient1: ids1.includes(wallet1b.id().toString()),
        wallet1cInClient1: ids1.includes(wallet1c.id().toString()),
        wallet2aInClient2: ids2.includes(wallet2a.id().toString()),
        wallet1aInClient2: ids2.includes(wallet1a.id().toString()),
      };
    });

    expect(result.accounts1Len).toBe(3);
    expect(result.accounts2Len).toBe(1);
    expect(result.wallet1aInClient1).toBe(true);
    expect(result.wallet1bInClient1).toBe(true);
    expect(result.wallet1cInClient1).toBe(true);
    expect(result.wallet2aInClient2).toBe(true);
    expect(result.wallet1aInClient2).toBe(false);
  });

  test("input notes are isolated between stores", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client1 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "NoteStore1"
      );
      const client2 = await window.WebClient.createClient(
        window.rpcUrl,
        undefined,
        undefined,
        "NoteStore2"
      );

      await Promise.all([client1.syncState(), client2.syncState()]);

      const notes1 = await client1.getInputNotes(
        new window.NoteFilter(window.NoteFilterTypes.All, null)
      );
      const notes2 = await client2.getInputNotes(
        new window.NoteFilter(window.NoteFilterTypes.All, null)
      );

      return {
        notes1Len: notes1.length,
        notes2Len: notes2.length,
      };
    });

    expect(result.notes1Len).toBe(result.notes2Len);
  });
});
