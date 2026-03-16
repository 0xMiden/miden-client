// @ts-nocheck
import { test, expect } from "./test-setup";
import { parseNetworkId } from "./test-helpers";

test.describe("Address instantiation tests", () => {
  test("Fail to instance address with wrong interface", async ({
    client,
    sdk,
  }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    expect(() =>
      sdk.Address.fromAccountId(newAccount.id(), "Does not exist")
    ).toThrow();
  });

  test("Fail to instance address with something that's not an account id", async ({
    sdk,
  }) => {
    expect(() =>
      sdk.Address.fromAccountId("notAnAccountId", "BasicWallet")
    ).toThrow();
  });

  test("Instance address with proper interface and read it", async ({
    client,
    sdk,
  }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.interface()).toBe("BasicWallet");
  });
});

test.describe("Bech32 tests", () => {
  test("to bech32 fails with non-valid-prefix", async ({ client, sdk }) => {
    expect(() => parseNetworkId(sdk, "non valid prefix")).toThrow();
  });

  test("encoding from bech32 and going back results in the same address", async ({
    client,
    sdk,
  }) => {
    const parsedNetworkId = parseNetworkId(sdk, "mtst");
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    const expectedBech32 = address.toBech32(parsedNetworkId);

    const parsedNetworkId2 = parseNetworkId(sdk, "mtst");
    const addressFromBech32 = sdk.Address.fromBech32(expectedBech32);
    const roundTripped = addressFromBech32.toBech32(parsedNetworkId2);
    expect(roundTripped).toBe(expectedBech32);
  });

  test("bech32 succeeds with mainnet prefix", async ({ client, sdk }) => {
    const parsedNetworkId = parseNetworkId(sdk, "mm");
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.toBech32(parsedNetworkId)).toHaveLength(47);
  });

  test("bech32 succeeds with testnet prefix", async ({ client, sdk }) => {
    const parsedNetworkId = parseNetworkId(sdk, "mtst");
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.toBech32(parsedNetworkId)).toHaveLength(49);
  });

  test("bech32 succeeds with dev prefix", async ({ client, sdk }) => {
    const parsedNetworkId = parseNetworkId(sdk, "mdev");
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.toBech32(parsedNetworkId)).toHaveLength(49);
  });

  test("bech32 succeeds with custom prefix", async ({ client, sdk }) => {
    const parsedNetworkId = parseNetworkId(sdk, "cstm");
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.toBech32(parsedNetworkId)).toHaveLength(49);
  });

  test("fromBech32 returns correct account id", async ({ client, sdk }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const accountId = newAccount.id();
    const asBech32 = accountId.toBech32(
      sdk.NetworkId.mainnet(),
      sdk.AccountInterface.BasicWallet
    );
    const fromBech32 = sdk.AccountId.fromBech32(asBech32).toString();
    expect(accountId.toString()).toBe(fromBech32);
  });
});

test.describe("Note tag tests", () => {
  test("note tag is returned and read", async ({ client, sdk }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
    expect(address.toNoteTag().asU32()).toBeTruthy();
  });
});

// ADDRESS INSERTION & DELETION TESTS
// =======================================================================================================

test.describe("Address insertion & deletion tests", () => {
  test("address can be removed and then re-inserted", async ({
    client,
    sdk,
  }) => {
    test.skip(true, "exportStore is browser-only");
  });
});
