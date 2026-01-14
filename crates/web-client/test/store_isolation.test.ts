import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("Store Isolation Tests", () => {
  test("creates separate stores with isolated accounts", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const MIDEN_NODE_PORT = 57291;
      const rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;

      const client1 = await window.WebClient.createClient(
        rpcUrl,
        undefined,
        undefined,
        "Client1"
      );
      await client1.syncState();

      await client1.newWallet(window.AccountStorageMode.private(), true, 0);

      const client2 = await window.WebClient.createClient(
        rpcUrl,
        undefined,
        undefined,
        "Client2"
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
      };
    });

    expect(result.dbNames).toContain("MidenClientDB_Client1");
    expect(result.dbNames).toContain("MidenClientDB_Client2");

    expect(result.accounts1Len).toBe(1);
    expect(result.accounts2Len).toBe(0);
  });
});
