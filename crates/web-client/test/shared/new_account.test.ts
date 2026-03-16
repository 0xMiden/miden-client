import { test, expect } from "./fixtures.ts";

function isValidAddress(address: string) {
  expect(address.startsWith("0x")).toBe(true);
}

// new_wallet tests
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
    test(description, async ({ ops }) => {
      const result = await ops.createNewWallet({ storageMode, mutable });

      isValidAddress(result.id);
      expect(result.nonce).toBe("0");
      isValidAddress(result.vaultCommitment);
      isValidAddress(result.storageCommitment);
      isValidAddress(result.codeCommitment);
      expect(result.isFaucet).toBe(false);
      expect(result.isRegularAccount).toBe(true);
      expect(result.isUpdatable).toBe(expected.isUpdatable);
      expect(result.isPublic).toBe(expected.isPublic);
      expect(result.isPrivate).toBe(expected.isPrivate);
      expect(result.isNetwork).toBe(expected.isNetwork);
      expect(result.isIdPublic).toBe(expected.isPublic);
      expect(result.isIdPrivate).toBe(expected.isPrivate);
      expect(result.isIdNetwork).toBe(expected.isNetwork);
      expect(result.isNew).toBe(true);
    });
  });
});

// new_faucet tests
test.describe("new_faucet tests", () => {
  const testCases = [
    {
      description: "creates a new private, fungible faucet",
      storageMode: "private",
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: 10000000,
      expected: { isPublic: false, isPrivate: true, isNetwork: false },
    },
    {
      description: "creates a new public, fungible faucet",
      storageMode: "public",
      nonFungible: false,
      tokenSymbol: "DAG",
      decimals: 8,
      maxSupply: 10000000,
      expected: { isPublic: true, isPrivate: false, isNetwork: false },
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
      test(description, async ({ ops }) => {
        const result = await ops.createNewFaucet({
          storageMode,
          nonFungible,
          tokenSymbol,
          decimals,
          maxSupply,
        });

        isValidAddress(result.id);
        expect(result.nonce).toBe("0");
        isValidAddress(result.vaultCommitment);
        isValidAddress(result.storageCommitment);
        isValidAddress(result.codeCommitment);
        expect(result.isFaucet).toBe(true);
        expect(result.isRegularAccount).toBe(false);
        expect(result.isUpdatable).toBe(false);
        expect(result.isPublic).toBe(expected.isPublic);
        expect(result.isPrivate).toBe(expected.isPrivate);
        expect(result.isNetwork).toBe(expected.isNetwork);
        expect(result.isIdPublic).toBe(expected.isPublic);
        expect(result.isIdPrivate).toBe(expected.isPrivate);
        expect(result.isIdNetwork).toBe(expected.isNetwork);
        expect(result.isNew).toBe(true);
      });
    }
  );
});
