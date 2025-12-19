import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  createNewFaucet,
  createNewWallet,
  isValidAddress,
  StorageMode,
} from "./webClientTestUtils";

// new_wallet tests
// =======================================================================================================

test.describe("new_wallet tests", () => {
  const testCases = [
    {
      description: "creates a new private, immutable wallet",
      storageMode: StorageMode.PRIVATE,
      mutable: false,
      authSchemeId: 0,
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
        isUpdatable: false,
      },
    },
    {
      description: "creates a new public, immutable wallet",
      storageMode: StorageMode.PUBLIC,
      mutable: false,
      authSchemeId: 0,
      expected: {
        isPublic: true,
        isPrivate: false,
        isNetwork: false,
        isUpdatable: false,
      },
    },
    {
      description: "creates a new private, mutable wallet",
      storageMode: StorageMode.PRIVATE,
      mutable: true,
      authSchemeId: 0,
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
        isUpdatable: true,
      },
    },
    {
      description: "creates a new public, mutable wallet",
      storageMode: StorageMode.PUBLIC,
      mutable: true,
      authSchemeId: 0,
      expected: {
        isPublic: true,
        isPrivate: false,
        isNetwork: false,
        isUpdatable: true,
      },
    },
  ];

  testCases.forEach(({ description, storageMode, mutable, expected }) => {
    test(description, async ({ page }) => {
      const result = await createNewWallet(page, {
        storageMode,
        mutable,
        authSchemeId: 0,
      });

      isValidAddress(result.id);
      expect(result.nonce).toEqual("0");
      isValidAddress(result.vaultCommitment);
      isValidAddress(result.storageCommitment);
      isValidAddress(result.codeCommitment);
      expect(result.isFaucet).toEqual(false);
      expect(result.isRegularAccount).toEqual(true);
      expect(result.isUpdatable).toEqual(expected.isUpdatable);
      expect(result.isPublic).toEqual(expected.isPublic);
      expect(result.isPrivate).toEqual(expected.isPrivate);
      expect(result.isNetwork).toEqual(expected.isNetwork);
      expect(result.isIdPublic).toEqual(expected.isPublic);
      expect(result.isIdPrivate).toEqual(expected.isPrivate);
      expect(result.isIdNetwork).toEqual(expected.isNetwork);
      expect(result.isNew).toEqual(true);
    });
  });

  test("Constructs the same account when given the same init seed", async ({
    page,
  }) => {
    const clientSeed1 = new Uint8Array(32);
    const clientSeed2 = new Uint8Array(32);
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(clientSeed1);
    crypto.getRandomValues(clientSeed2);
    crypto.getRandomValues(walletSeed);

    // Isolate the client instance both times to ensure the outcome is deterministic
    await createNewWallet(page, {
      storageMode: StorageMode.PUBLIC,
      mutable: false,
      authSchemeId: 0,
      clientSeed: clientSeed1,
      isolatedClient: true,
      walletSeed: walletSeed,
    });

    // This should fail, as the wallet is already tracked within the same browser context
    await expect(async () => {
      await createNewWallet(page, {
        storageMode: StorageMode.PUBLIC,
        mutable: false,
        authSchemeId: 0,
        clientSeed: clientSeed2,
        isolatedClient: true,
        walletSeed: walletSeed,
      });
    }).rejects.toThrowError(/failed to insert new wallet/);
  });
});

// new_faucet tests
// =======================================================================================================
test.describe("new_faucet tests", () => {
  const testCases = [
    {
      description: "creates a new private, fungible faucet",
      storageMode: StorageMode.PRIVATE,
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: BigInt(10000000),
      authSchemeId: 0,
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
      storageMode: StorageMode.PUBLIC,
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: BigInt(10000000),
      authSchemeId: 0,
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
      authSchemeId,
      expected,
    }) => {
      test(description, async ({ page }) => {
        const result = await createNewFaucet(
          page,
          storageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          maxSupply,
          authSchemeId
        );

        isValidAddress(result.id);
        expect(result.nonce).toEqual("0");
        isValidAddress(result.vaultCommitment);
        isValidAddress(result.storageCommitment);
        isValidAddress(result.codeCommitment);
        expect(result.isFaucet).toEqual(true);
        expect(result.isRegularAccount).toEqual(false);
        expect(result.isUpdatable).toEqual(false);
        expect(result.isPublic).toEqual(expected.isPublic);
        expect(result.isPrivate).toEqual(expected.isPrivate);
        expect(result.isNetwork).toEqual(expected.isNetwork);
        expect(result.isIdPublic).toEqual(expected.isPublic);
        expect(result.isIdPrivate).toEqual(expected.isPrivate);
        expect(result.isIdNetwork).toEqual(expected.isNetwork);
        expect(result.isNew).toEqual(true);
      });
    }
  );

  test("throws an error when attempting to create a non-fungible faucet", async ({
    page,
  }) => {
    await expect(
      createNewFaucet(
        page,
        StorageMode.PUBLIC,
        true,
        "DAG",
        8,
        BigInt(10000000),
        0
      )
    ).rejects.toThrowError("Non-fungible faucets are not supported yet");
  });

  test("throws an error when attempting to create a faucet with an invalid token symbol", async ({
    page,
  }) => {
    await expect(
      createNewFaucet(
        page,
        StorageMode.PUBLIC,
        false,
        "INVALID_TOKEN",
        8,
        BigInt(10000000),
        0
      )
    ).rejects.toThrow(
      `token symbol should have length between 1 and 6 characters, but 13 was provided`
    );
  });
});

// AccountStorage.getMapEntries tests
// =======================================================================================================

test.describe("AccountStorage.getMapEntries tests", () => {
  test("returns undefined for missing/non-map slots and [] for empty maps", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const VALUE_SLOT_NAME = "miden::testing::account_storage_tests::value";
      const MAP_SLOT_NAME = "miden::testing::account_storage_tests::map";
      const MISSING_SLOT_NAME =
        "miden::testing::account_storage_tests::missing";

      const testPackageBytes =
        window.TestUtils.createMockSerializedLibraryPackage();
      const deserializedPackage = window.Package.deserialize(testPackageBytes);

      const valueSlot = window.StorageSlot.emptyValue(VALUE_SLOT_NAME);
      const storageMap = new window.StorageMap();
      const mapSlot = window.StorageSlot.map(MAP_SLOT_NAME, storageMap);

      const component = window.AccountComponent.fromPackage(
        deserializedPackage,
        new window.MidenArrays.StorageSlotArray([valueSlot, mapSlot])
      );

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const secretKey = window.AuthSecretKey.rpoFalconWithRNG(walletSeed);
      const authComponent =
        window.AccountComponent.createAuthComponentFromSecretKey(secretKey);

      const accountBuilderResult = new window.AccountBuilder(walletSeed)
        .accountType(window.AccountType.RegularAccountImmutableCode)
        .storageMode(window.AccountStorageMode.private())
        .withAuthComponent(authComponent)
        .withComponent(component)
        .build();

      await client.addAccountSecretKeyToWebStore(secretKey);
      await client.newAccount(accountBuilderResult.account, false);
      await client.syncState();

      const accountRecord = await client.getAccount(
        accountBuilderResult.account.id()
      );
      if (!accountRecord) {
        throw new Error("Account not found");
      }

      const storage = accountRecord.storage();

      const nonMapResult = storage.getMapEntries(VALUE_SLOT_NAME);
      const missingSlotResult = storage.getMapEntries(MISSING_SLOT_NAME);
      const emptyMapResult = storage.getMapEntries(MAP_SLOT_NAME);

      return {
        nonMap: nonMapResult,
        missing: missingSlotResult,
        emptyMap: emptyMapResult,
      };
    });

    expect(result.nonMap).toBeUndefined();
    expect(result.missing).toBeUndefined();
    expect(result.emptyMap).toEqual([]);
  });
});
