// @ts-nocheck
import { test, expect } from "./test-setup";
import { setupWalletAndFaucet, createFreshMockClient } from "./test-helpers";

test.describe("AccountFile", () => {
  test("it serializes and deserializes an account file", async ({
    client,
    sdk,
  }) => {
    const { walletId } = await setupWalletAndFaucet(client, sdk);

    const accountIdObj = sdk.AccountId.fromHex(walletId);
    const accountFile = await client.exportAccountFile(accountIdObj);
    const bytes = accountFile.serialize();

    const deserialized = sdk.AccountFile.deserialize(new Uint8Array(bytes));
    const reserialized = deserialized.serialize();
    expect(Array.from(reserialized)).toEqual(Array.from(bytes));

    // Import into a fresh client
    const client2 = await createFreshMockClient(sdk);
    await client2.importAccountFile(deserialized);

    const account = await client2.getAccount(accountIdObj);
    expect(account).toBeDefined();
    expect(account.id().toString()).toBe(walletId);
  });
});
