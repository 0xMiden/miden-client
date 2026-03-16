// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("basic fungible faucet", () => {
  test("creates a basic fungible faucet component from an account", async ({
    client,
    sdk,
  }) => {
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

    expect(basicFungibleFaucet.symbol().toString()).toEqual("DAG");
    expect(basicFungibleFaucet.decimals()).toEqual(8);
    expect(basicFungibleFaucet.maxSupply().toString()).toEqual("10000000");
  });

  test("throws an error when creating a basic fungible faucet from a non-faucet account", async ({
    client,
    sdk,
  }) => {
    const account = await client.newWallet(
      sdk.AccountStorageMode.tryFromStr("public"),
      false,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    expect(() => sdk.BasicFungibleFaucetComponent.fromAccount(account)).toThrow(
      "failed to get basic fungible faucet details from account"
    );
  });
});
