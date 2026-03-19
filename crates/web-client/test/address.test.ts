// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("Address instantiation tests", () => {
  test("Fail to instance address with wrong interface", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      try {
        sdk.Address.fromAccountId(newAccount.id(), "Does not exist");
        return { threw: false };
      } catch {
        return { threw: true };
      }
    });
    expect(result.threw).toBe(true);
  });

  test("Fail to instance address with something that's not an account id", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      try {
        sdk.Address.fromAccountId("notAnAccountId", "BasicWallet");
        return { threw: false };
      } catch {
        return { threw: true };
      }
    });
    expect(result.threw).toBe(true);
  });

  test("Instance address with proper interface and read it", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      // napi returns numeric enum (0), browser returns string ("BasicWallet")
      // Compare inside run where both values are on the same platform
      return {
        interfacesMatch:
          address.interface() === sdk.AccountInterface.BasicWallet,
      };
    });
    expect(result.interfacesMatch).toBe(true);
  });
});

test.describe("Bech32 tests", () => {
  test("to bech32 fails with non-valid-prefix", async ({ run }) => {
    const result = await run(async ({ helpers }) => {
      try {
        helpers.parseNetworkId("non valid prefix");
        return { threw: false };
      } catch {
        return { threw: true };
      }
    });
    expect(result.threw).toBe(true);
  });

  test("encoding from bech32 and going back results in the same address", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const parsedNetworkId = helpers.parseNetworkId("mtst");
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      const expectedBech32 = address.toBech32(parsedNetworkId);

      const parsedNetworkId2 = helpers.parseNetworkId("mtst");
      const addressFromBech32 = sdk.Address.fromBech32(expectedBech32);
      const roundTripped = addressFromBech32.toBech32(parsedNetworkId2);
      return { roundTripped, expectedBech32 };
    });
    expect(result.roundTripped).toBe(result.expectedBech32);
  });

  test("bech32 succeeds with mainnet prefix", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const parsedNetworkId = helpers.parseNetworkId("mm");
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      return { bech32Length: address.toBech32(parsedNetworkId).length };
    });
    expect(result.bech32Length).toEqual(47);
  });

  test("bech32 succeeds with testnet prefix", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const parsedNetworkId = helpers.parseNetworkId("mtst");
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      return { bech32Length: address.toBech32(parsedNetworkId).length };
    });
    expect(result.bech32Length).toEqual(49);
  });

  test("bech32 succeeds with dev prefix", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const parsedNetworkId = helpers.parseNetworkId("mdev");
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      return { bech32Length: address.toBech32(parsedNetworkId).length };
    });
    expect(result.bech32Length).toEqual(49);
  });

  test("bech32 succeeds with custom prefix", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const parsedNetworkId = helpers.parseNetworkId("cstm");
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      return { bech32Length: address.toBech32(parsedNetworkId).length };
    });
    expect(result.bech32Length).toEqual(49);
  });

  test("fromBech32 returns correct account id", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
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
      return {
        originalId: accountId.toString(),
        fromBech32,
      };
    });
    expect(result.originalId).toBe(result.fromBech32);
  });
});

test.describe("Note tag tests", () => {
  test("note tag is returned and read", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const address = sdk.Address.fromAccountId(newAccount.id(), "BasicWallet");
      return { noteTagU32: address.toNoteTag().asU32() };
    });
    expect(result.noteTagU32).toBeTruthy();
  });
});

// ADDRESS INSERTION & DELETION TESTS
// =======================================================================================================

test.describe("Address insertion & deletion tests", () => {
  test("address can be removed and then re-inserted", async ({ run }) => {
    test.skip(true, "exportStore is browser-only");
  });
});
