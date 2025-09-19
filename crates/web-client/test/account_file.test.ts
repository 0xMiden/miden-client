import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import { setupWalletAndFaucet, clearStore, getAccount } from "./webClientTestUtils";

test.describe("AccountFile", () => {
  test("it serializes and deserializes an account file", async ({
    page,
  }) => {
    const { accountId } = await setupWalletAndFaucet(page);

    const accountFileBytes = await page.evaluate(async (accountId) => {
      const client = window.client;
      const accountIdObj = window.AccountId.fromHex(accountId);
      // Note: exportAccountFile returns Uint8Array of the serialized AccountFile
      return client.exportAccountFile(accountIdObj);
    }, accountId);

    const reserializedBytes = await page.evaluate(async (bytes) => {
      // Deserialize bytes to AccountFile object
      const accountFile = window.AccountFile.deserialize(bytes);

      // Serialize the AccountFile object back to bytes
      const reserialized = accountFile.serialize();
      return reserialized;
    }, accountFileBytes);

    expect(reserializedBytes).toEqual(accountFileBytes);

    await clearStore(page);

    await page.evaluate(async (bytes) => {
      const client = window.client;
      await client.importAccountFile(bytes);
    }, reserializedBytes);

    const account = await getAccount(page, accountId);

    expect(account).not.toBeNull();
    expect(account!.id).toBe(accountId);
  });
});
