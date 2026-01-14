import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  setupWalletAndFaucet,
  clearStore,
  getAccount,
} from "./webClientTestUtils";

test.describe("AccountFile", () => {
  test("it serializes and deserializes an account file", async ({ page }) => {
    console.log(
      "Starting test: it serializes and deserializes an account file"
    );

    console.log("Step 1: Setting up wallet and faucet...");
    const { accountId } = await setupWalletAndFaucet(page);
    console.log("Step 1 complete. accountId:", accountId);

    console.log("Step 2: Exporting account file and serializing to bytes...");
    const accountFileBytes = await page.evaluate(async (accountId) => {
      debugger;
      const client = window.client;
      console.log(
        "Inside page.evaluate - exporting account file for:",
        accountId
      );
      const accountIdObj = window.AccountId.fromHex(accountId);
      console.log("Created AccountId object");
      const accountFile = await client.exportAccountFile(accountIdObj);
      console.log("Exported account file successfully");
      const bytes = Array.from(accountFile.serialize());
      console.log("Serialized account file, byte length:", bytes.length);
      return bytes;
    }, accountId);
    console.log(
      "Step 2 complete. accountFileBytes length:",
      accountFileBytes.length
    );

    console.log("Step 3: Deserializing and re-serializing bytes...");
    const reserializedBytes = await page.evaluate(async (bytes) => {
      console.log(
        "Inside page.evaluate - deserializing bytes, length:",
        bytes.length
      );
      const byteArray = new Uint8Array(bytes);
      // Deserialize bytes to AccountFile object
      const accountFile = window.AccountFile.deserialize(byteArray);
      console.log("Deserialized AccountFile successfully");

      // Serialize the AccountFile object back to bytes
      const reserialized = Array.from(accountFile.serialize());
      console.log(
        "Re-serialized AccountFile, byte length:",
        reserialized.length
      );
      return reserialized;
    }, accountFileBytes);
    console.log(
      "Step 3 complete. reserializedBytes length:",
      reserializedBytes.length
    );

    console.log("Step 4: Comparing original and reserialized bytes...");
    console.log(
      "Bytes match:",
      JSON.stringify(reserializedBytes) === JSON.stringify(accountFileBytes)
    );
    expect(reserializedBytes).toEqual(accountFileBytes);
    console.log("Step 4 complete. Bytes are equal.");

    console.log("Step 5: Clearing store...");
    await clearStore(page);
    console.log("Step 5 complete. Store cleared.");

    console.log("Step 6: Importing account file...");
    await page.evaluate(async (bytes) => {
      console.log(
        "Inside page.evaluate - importing account file, bytes length:",
        bytes.length
      );
      const client = window.client;
      const accountFile = window.AccountFile.deserialize(new Uint8Array(bytes));
      console.log("Deserialized AccountFile for import");
      await client.importAccountFile(accountFile);
      console.log("Import completed successfully");
    }, reserializedBytes);
    console.log("Step 6 complete. Account file imported.");

    console.log("Step 7: Getting account by id:", accountId);
    const account = await getAccount(page, accountId);
    console.log(
      "Step 7 complete. Account retrieved:",
      account ? "found" : "null"
    );
    if (account) {
      console.log("Account id:", account.id);
    }

    expect(account).not.toBeNull();
    expect(account!.id).toBe(accountId);
    console.log("Test completed successfully!");
  });
});
