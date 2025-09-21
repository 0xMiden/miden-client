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
      expected: { isPublic: false, isUpdatable: false },
    },
    {
      description: "creates a new public, immutable wallet",
      storageMode: StorageMode.PUBLIC,
      mutable: false,
      expected: { isPublic: true, isUpdatable: false },
    },
    {
      description: "creates a new private, mutable wallet",
      storageMode: StorageMode.PRIVATE,
      mutable: true,
      expected: { isPublic: false, isUpdatable: true },
    },
    {
      description: "creates a new public, mutable wallet",
      storageMode: StorageMode.PUBLIC,
      mutable: true,
      expected: { isPublic: true, isUpdatable: true },
    },
  ];

  testCases.forEach(({ description, storageMode, mutable, expected }) => {
    test(description, async ({ page }) => {
      const result = await createNewWallet(page, { storageMode, mutable });

      isValidAddress(result.id);
      expect(result.nonce).toEqual("0");
      isValidAddress(result.vaultCommitment);
      isValidAddress(result.storageCommitment);
      isValidAddress(result.codeCommitment);
      expect(result.isFaucet).toEqual(false);
      expect(result.isRegularAccount).toEqual(true);
      expect(result.isUpdatable).toEqual(expected.isUpdatable);
      expect(result.isPublic).toEqual(expected.isPublic);
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
      clientSeed: clientSeed1,
      isolatedClient: true,
      walletSeed: walletSeed,
    });

    // This should fail, as the wallet is already tracked within the same browser context
    await expect(async () => {
      await createNewWallet(page, {
        storageMode: StorageMode.PUBLIC,
        mutable: false,
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
      expected: {
        isPublic: false,
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
      expected: {
        isPublic: true,
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
      test(description, async ({ page }) => {
        const result = await createNewFaucet(
          page,
          storageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          maxSupply
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
        BigInt(10000000)
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
        BigInt(10000000)
      )
    ).rejects.toThrow(
      `token symbol should have length between 1 and 6 characters, but 13 was provided`
    );
  });
});

// AccountStorage.getMapEntries tests
// =======================================================================================================

test.describe("AccountStorage.getMapEntries tests", () => {
  test("returns empty array for non-map storage slot", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      // Create a new wallet with private storage
      const account = await client.newWallet(
        window.AccountStorageMode.private(),
        true
      );

      // Get the account to access its storage
      const accountRecord = await client.getAccount(account.id());
      if (!accountRecord) {
        throw new Error("Account not found");
      }

      // Try to get map entries from slot 0 (should be empty for a new account)
      const mapEntries = accountRecord.storage().getMapEntries(0);

      return {
        entriesCount: mapEntries.length,
        entries: mapEntries.map((entry) => ({
          root: entry.root,
          key: entry.key,
          value: entry.value,
        })),
      };
    });

    // For a new account, slot 0 should be empty, so getMapEntries should return an empty array
    expect(result.entriesCount).toEqual(0);
    expect(result.entries).toEqual([]);
  });

  test("returns empty array for out of bounds index", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      // Create a new wallet
      const account = await client.newWallet(
        window.AccountStorageMode.private(),
        true
      );

      // Get the account to access its storage
      const accountRecord = await client.getAccount(account.id());
      if (!accountRecord) {
        throw new Error("Account not found");
      }

      // Try to get map entries from a high index (should be out of bounds)
      const mapEntries = accountRecord.storage().getMapEntries(255);

      return {
        entriesCount: mapEntries.length,
        entries: mapEntries,
      };
    });

    // For an out of bounds index, getMapEntries should return an empty array
    expect(result.entriesCount).toEqual(0);
    expect(result.entries).toEqual([]);
  });
});
