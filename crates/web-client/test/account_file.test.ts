import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  setupWalletAndFaucet,
  clearStore,
  getAccount,
} from "./webClientTestUtils";

test.describe("AccountFile", () => {
  test("it serializes and deserializes an account file", async ({ page }) => {
    const { accountId } = await setupWalletAndFaucet(page);

    const accountFileBytes = await page.evaluate(async (accountId) => {
      const client = window.client;
      const accountIdObj = window.AccountId.fromHex(accountId);
      const accountFile = await client.exportAccountFile(accountIdObj);
      const bytes = Array.from(accountFile.serialize());
      return bytes;
    }, accountId);

    const reserializedBytes = await page.evaluate(async (bytes) => {
      const byteArray = new Uint8Array(bytes);
      const accountFile = window.AccountFile.deserialize(byteArray);
      const reserialized = Array.from(accountFile.serialize());
      return reserialized;
    }, accountFileBytes);

    expect(reserializedBytes).toEqual(accountFileBytes);

    await clearStore(page);

    await page.evaluate(async (bytes) => {
      const client = window.client;
      const accountFile = window.AccountFile.deserialize(new Uint8Array(bytes));
      await client.importAccountFile(accountFile);
    }, reserializedBytes);

    const account = await getAccount(page, accountId);

    expect(account).not.toBeNull();
    expect(account!.id).toBe(accountId);
  });
});
