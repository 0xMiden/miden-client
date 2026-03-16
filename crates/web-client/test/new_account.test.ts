// @ts-nocheck
import { test, expect } from "./test-setup";

function isValidAddress(address: string) {
  expect(address.startsWith("0x")).toBe(true);
}

// new_wallet tests
// =======================================================================================================

test.describe("new_wallet tests", () => {
  const testCases = [
    {
      description: "creates a new private, immutable wallet",
      storageMode: "private",
      mutable: false,
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
        isUpdatable: false,
      },
    },
    {
      description: "creates a new public, immutable wallet",
      storageMode: "public",
      mutable: false,
      expected: {
        isPublic: true,
        isPrivate: false,
        isNetwork: false,
        isUpdatable: false,
      },
    },
    {
      description: "creates a new private, mutable wallet",
      storageMode: "private",
      mutable: true,
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
        isUpdatable: true,
      },
    },
    {
      description: "creates a new public, mutable wallet",
      storageMode: "public",
      mutable: true,
      expected: {
        isPublic: true,
        isPrivate: false,
        isNetwork: false,
        isUpdatable: true,
      },
    },
  ];

  testCases.forEach(({ description, storageMode, mutable, expected }) => {
    test(description, async ({ client, sdk }) => {
      const accountStorageMode =
        storageMode === "public"
          ? sdk.AccountStorageMode.public()
          : sdk.AccountStorageMode.private();

      const newWallet = await client.newWallet(
        accountStorageMode,
        mutable,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      isValidAddress(newWallet.id().toString());
      expect(newWallet.nonce().toString()).toEqual("0");
      isValidAddress(newWallet.vault().root().toHex());
      isValidAddress(newWallet.storage().commitment().toHex());
      isValidAddress(newWallet.code().commitment().toHex());
      expect(newWallet.isFaucet()).toEqual(false);
      expect(newWallet.isRegularAccount()).toEqual(true);
      expect(newWallet.isUpdatable()).toEqual(expected.isUpdatable);
      expect(newWallet.isPublic()).toEqual(expected.isPublic);
      expect(newWallet.isPrivate()).toEqual(expected.isPrivate);
      expect(newWallet.isNetwork()).toEqual(expected.isNetwork);
      expect(newWallet.id().isPublic()).toEqual(expected.isPublic);
      expect(newWallet.id().isPrivate()).toEqual(expected.isPrivate);
      expect(newWallet.id().isNetwork()).toEqual(expected.isNetwork);
      expect(newWallet.isNew()).toEqual(true);
    });
  });
});

// new_faucet tests
// =======================================================================================================
test.describe("new_faucet tests", () => {
  const testCases = [
    {
      description: "creates a new private, fungible faucet",
      storageMode: "private",
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: 10000000,
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
        isUpdatable: false,
        isRegularAccount: false,
        isFaucet: true,
      },
    },
    {
      description: "creates a new public, fungible faucet",
      storageMode: "public",
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: 10000000,
      expected: {
        isPublic: true,
        isPrivate: false,
        isNetwork: false,
        isUpdatable: false,
        isRegularAccount: false,
        isFaucet: true,
      },
    },
  ];

  testCases.forEach(
    ({
      description,
      storageMode,
      nonFungible,
      tokenSymbol,
      decimals,
      maxSupply,
      expected,
    }) => {
      test(description, async ({ client, sdk }) => {
        const accountStorageMode =
          storageMode === "public"
            ? sdk.AccountStorageMode.public()
            : sdk.AccountStorageMode.private();

        const newFaucet = await client.newFaucet(
          accountStorageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          sdk.u64(maxSupply),
          sdk.AuthScheme.AuthRpoFalcon512
        );

        isValidAddress(newFaucet.id().toString());
        expect(newFaucet.nonce().toString()).toEqual("0");
        isValidAddress(newFaucet.vault().root().toHex());
        isValidAddress(newFaucet.storage().commitment().toHex());
        isValidAddress(newFaucet.code().commitment().toHex());
        expect(newFaucet.isFaucet()).toEqual(true);
        expect(newFaucet.isRegularAccount()).toEqual(false);
        expect(newFaucet.isUpdatable()).toEqual(false);
        expect(newFaucet.isPublic()).toEqual(expected.isPublic);
        expect(newFaucet.isPrivate()).toEqual(expected.isPrivate);
        expect(newFaucet.isNetwork()).toEqual(expected.isNetwork);
        expect(newFaucet.id().isPublic()).toEqual(expected.isPublic);
        expect(newFaucet.id().isPrivate()).toEqual(expected.isPrivate);
        expect(newFaucet.id().isNetwork()).toEqual(expected.isNetwork);
        expect(newFaucet.isNew()).toEqual(true);
      });
    }
  );

  test("throws an error when attempting to create a non-fungible faucet", async ({
    client,
    sdk,
  }) => {
    await expect(
      client.newFaucet(
        sdk.AccountStorageMode.public(),
        true,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      )
    ).rejects.toThrowError("Non-fungible faucets are not supported yet");
  });

  test("throws an error when attempting to create a faucet with an invalid token symbol", async ({
    client,
    sdk,
  }) => {
    await expect(
      client.newFaucet(
        sdk.AccountStorageMode.public(),
        false,
        "INVALID_TOKEN",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      )
    ).rejects.toThrow(
      `token symbol should have length between 1 and 12 characters, but 13 was provided`
    );
  });
});

// AccountStorage.getMapEntries tests
// =======================================================================================================

test.describe("AccountStorage.getMapEntries tests", () => {
  test("returns undefined for invalid slot names", async ({ client, sdk }) => {
    const NON_MAP_SLOT_NAME =
      "miden::standards::auth::rpo_falcon512::public_key";
    const MISSING_SLOT_NAME = "miden::testing::account_storage_tests::missing";

    // Create a new wallet with private storage
    const account = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    // Get the account to access its storage
    const accountRecord = await client.getAccount(account.id());
    expect(accountRecord).toBeDefined();

    const storage = accountRecord.storage();

    const nonMapResult = storage.getMapEntries(NON_MAP_SLOT_NAME);
    const missingSlotResult = storage.getMapEntries(MISSING_SLOT_NAME);

    expect(nonMapResult).toBeUndefined();
    expect(missingSlotResult).toBeUndefined();
  });
});
