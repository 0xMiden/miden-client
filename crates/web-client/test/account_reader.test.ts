import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("AccountReader tests", () => {
  test("creates account reader and reads account data correctly", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const account = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const reader = client.accountReader(account.id());

      const nonce = await reader.nonce();
      const commitment = await reader.commitment();
      const isNew = (await reader.status()).isNew();
      const codeCommitment = await reader.codeCommitment();

      return {
        originalId: account.id().toString(),
        readerId: reader.accountId().toString(),
        accountNonce: account.nonce().toString(),
        readerNonce: nonce.toString(),
        accountCommitment: account.commitment().toHex(),
        readerCommitment: commitment.toHex(),
        accountCodeCommitment: account.code().commitment().toHex(),
        readerCodeCommitment: codeCommitment.toHex(),
        isNew,
      };
    });

    expect(result.originalId).toEqual(result.readerId);
    expect(result.accountNonce).toEqual(result.readerNonce);
    expect(result.accountCommitment).toEqual(result.readerCommitment);
    expect(result.accountCodeCommitment).toEqual(result.readerCodeCommitment);
    expect(result.isNew).toBe(true);
  });
});
