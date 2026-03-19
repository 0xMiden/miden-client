// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("AccountReader tests", () => {
  test("creates account reader and reads account data correctly", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const account = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const reader = await client.accountReader(account.id());

      const nonce = await reader.nonce();
      const commitment = await reader.commitment();
      const isNew = (await reader.status()).isNew();
      const codeCommitment = await reader.codeCommitment();

      return {
        accountId: account.id().toString(),
        readerId: reader.accountId().toString(),
        accountNonce: account.nonce().toString(),
        readerNonce: nonce.toString(),
        accountCommitment: account.to_commitment().toHex(),
        readerCommitment: commitment.toHex(),
        accountCodeCommitment: account.code().commitment().toHex(),
        readerCodeCommitment: codeCommitment.toHex(),
        isNew,
      };
    });
    expect(result.accountId).toEqual(result.readerId);
    expect(result.accountNonce).toEqual(result.readerNonce);
    expect(result.accountCommitment).toEqual(result.readerCommitment);
    expect(result.accountCodeCommitment).toEqual(result.readerCodeCommitment);
    expect(result.isNew).toBe(true);
  });
});
