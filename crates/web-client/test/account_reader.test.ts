// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("AccountReader tests", () => {
  test("creates account reader and reads account data correctly", async ({
    client,
    sdk,
  }) => {
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

    expect(account.id().toString()).toEqual(reader.accountId().toString());
    expect(account.nonce().toString()).toEqual(nonce.toString());
    expect(account.to_commitment().toHex()).toEqual(commitment.toHex());
    expect(account.code().commitment().toHex()).toEqual(codeCommitment.toHex());
    expect(isNew).toBe(true);
  });
});
