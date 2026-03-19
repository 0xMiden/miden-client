// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("AccountFile", () => {
  test("it serializes and deserializes an account file", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { walletId } = await helpers.setupWalletAndFaucet();

      const accountIdObj = sdk.AccountId.fromHex(walletId);
      const accountFile = await client.exportAccountFile(accountIdObj);
      const bytes = accountFile.serialize();

      const deserialized = sdk.AccountFile.deserialize(new Uint8Array(bytes));
      const reserialized = deserialized.serialize();

      const bytesMatch =
        Array.from(reserialized).toString() === Array.from(bytes).toString();

      // Import into a fresh client
      const client2 = await helpers.createFreshMockClient();
      await client2.importAccountFile(deserialized);

      const account = await client2.getAccount(accountIdObj);
      const isDefined = account !== undefined && account !== null;
      const accountIdStr = account.id().toString();

      return { bytesMatch, isDefined, accountIdStr, walletId };
    });

    expect(result.bytesMatch).toBe(true);
    expect(result.isDefined).toBe(true);
    expect(result.accountIdStr).toBe(result.walletId);
  });
});
