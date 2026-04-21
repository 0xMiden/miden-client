// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("basic fungible faucet", () => {
  test("creates a basic fungible faucet component from an account", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const newFaucet = await client.newFaucet(
        sdk.AccountStorageMode.tryFromStr("public"),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const basicFungibleFaucet =
        sdk.BasicFungibleFaucetComponent.fromAccount(newFaucet);

      return {
        symbol: basicFungibleFaucet.symbol().toString(),
        decimals: basicFungibleFaucet.decimals(),
        maxSupply: basicFungibleFaucet.maxSupply().toString(),
      };
    });
    expect(result.symbol).toEqual("DAG");
    expect(result.decimals).toEqual(8);
    expect(result.maxSupply).toEqual("10000000");
  });

  test("throws an error when creating a basic fungible faucet from a non-faucet account", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const account = await client.newWallet(
        sdk.AccountStorageMode.tryFromStr("public"),
        false,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      try {
        sdk.BasicFungibleFaucetComponent.fromAccount(account);
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });
    expect(result.threw).toBe(true);
    expect(result.message).toContain(
      "failed to get basic fungible faucet details from account"
    );
  });
});
