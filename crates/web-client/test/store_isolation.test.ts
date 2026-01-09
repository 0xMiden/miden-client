import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("Store Isolation Tests", () => {
    test("creates separate stores for localhost using explicit store names", async ({ page }) => {
        const result = await page.evaluate(async () => {
            const MIDEN_NODE_PORT = 57291;
            const rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;

            // Create Client A
            const client1 = await window.WebClient.createClient(rpcUrl, undefined, undefined, "ClientA_Store");
            await client1.syncState();

            // Create Client B
            const client2 = await window.WebClient.createClient(rpcUrl, undefined, undefined, "ClientB_Store");
            await client2.syncState();

            // Check IndexedDB databases
            const databases = await window.indexedDB.databases();
            const names = databases.map((db) => db.name);

            return {
                names,
            };
        });

        console.log("Found databases:", result.names);

        expect(result.names).toContain("ClientA_Store");
        expect(result.names).toContain("ClientB_Store");
    });
});
