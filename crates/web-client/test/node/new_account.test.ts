import { test, expect, sdk, createMockClient } from "./setup.ts";

function isValidAddress(address: string) {
  expect(address.startsWith("0x")).toBe(true);
}

async function createNewWallet(
  client: any,
  params: { storageMode: string; mutable: boolean }
) {
  const mode =
    params.storageMode === "public"
      ? sdk.AccountStorageMode.public()
      : sdk.AccountStorageMode.private();

  const wallet = await client.newWallet(
    mode,
    params.mutable,
    sdk.AuthScheme.AuthRpoFalcon512
  );

  return {
    id: wallet.id().toString(),
    nonce: wallet.nonce().toString(),
    vaultCommitment: wallet.vault().root().toHex(),
    storageCommitment: wallet.storage().commitment().toHex(),
    codeCommitment: wallet.code().commitment().toHex(),
    isFaucet: wallet.isFaucet(),
    isRegularAccount: wallet.isRegularAccount(),
    isUpdatable: wallet.isUpdatable(),
    isPublic: wallet.isPublic(),
    isPrivate: wallet.isPrivate(),
    isNetwork: wallet.isNetwork(),
    isIdPublic: wallet.id().isPublic(),
    isIdPrivate: wallet.id().isPrivate(),
    isIdNetwork: wallet.id().isNetwork(),
    isNew: wallet.isNew(),
  };
}

async function createNewFaucet(
  client: any,
  params: {
    storageMode: string;
    nonFungible: boolean;
    tokenSymbol: string;
    decimals: number;
    maxSupply: number;
  }
) {
  const mode =
    params.storageMode === "public"
      ? sdk.AccountStorageMode.public()
      : sdk.AccountStorageMode.private();

  const faucet = await client.newFaucet(
    mode,
    params.nonFungible,
    params.tokenSymbol,
    params.decimals,
    params.maxSupply,
    sdk.AuthScheme.AuthRpoFalcon512
  );

  return {
    id: faucet.id().toString(),
    nonce: faucet.nonce().toString(),
    vaultCommitment: faucet.vault().root().toHex(),
    storageCommitment: faucet.storage().commitment().toHex(),
    codeCommitment: faucet.code().commitment().toHex(),
    isFaucet: faucet.isFaucet(),
    isRegularAccount: faucet.isRegularAccount(),
    isUpdatable: faucet.isUpdatable(),
    isPublic: faucet.isPublic(),
    isPrivate: faucet.isPrivate(),
    isNetwork: faucet.isNetwork(),
    isIdPublic: faucet.id().isPublic(),
    isIdPrivate: faucet.id().isPrivate(),
    isIdNetwork: faucet.id().isNetwork(),
    isNew: faucet.isNew(),
  };
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
    test(description, async () => {
      const { client } = await createMockClient();
      const result = await createNewWallet(client, { storageMode, mutable });

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
      expected: {
        isPublic: false,
        isPrivate: true,
        isNetwork: false,
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
      test(description, async () => {
        const { client } = await createMockClient();
        const result = await createNewFaucet(client, {
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

  test("throws an error when attempting to create a non-fungible faucet", async () => {
    const { client } = await createMockClient();
    await expect(
      createNewFaucet(client, {
        storageMode: "public",
        nonFungible: true,
        tokenSymbol: "DAG",
        decimals: 8,
        maxSupply: 10000000,
      })
    ).rejects.toThrow("Non-fungible faucets are not supported yet");
  });

  test("throws an error when attempting to create a faucet with an invalid token symbol", async () => {
    const { client } = await createMockClient();
    await expect(
      createNewFaucet(client, {
        storageMode: "public",
        nonFungible: false,
        tokenSymbol: "INVALID_TOKEN",
        decimals: 8,
        maxSupply: 10000000,
      })
    ).rejects.toThrow(
      "token symbol should have length between 1 and 12 characters, but 13 was provided"
    );
  });
});
