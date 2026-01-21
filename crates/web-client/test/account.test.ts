import { Page, expect } from "@playwright/test";
import test from "./playwright.global.setup";

// GET_ACCOUNT TESTS
// =======================================================================================================

interface GetAccountSuccessResult {
  commitmentOfCreatedAccount: string;
  commitmentOfGetAccountResult: string;
  isAccountType: boolean | undefined;
}

export const getAccountOneMatch = async (
  page: Page
): Promise<GetAccountSuccessResult> => {
  return await page.evaluate(async () => {
    const client = window.client;
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );

    const result = await client.getAccount(newAccount.id());

    return {
      commitmentOfCreatedAccount: newAccount.commitment().toHex(),
      commitmentOfGetAccountResult: result!.commitment().toHex(),
      isAccountType: result instanceof window.Account,
    };
  });
};

interface GetAccountFailureResult {
  commitmentOfGetAccountResult: string | undefined;
}

export const getAccountNoMatch = async (
  page: Page
): Promise<GetAccountFailureResult> => {
  return await page.evaluate(async () => {
    const client = window.client;
    const nonExistingAccountId = window.TestUtils.createMockAccountId();

    const result = await client.getAccount(nonExistingAccountId);

    return {
      commitmentOfGetAccountResult: result
        ? result.commitment().toHex()
        : undefined,
    };
  });
};

test.describe("get_account tests", () => {
  test("retrieves an existing account", async ({ page }) => {
    const result = await getAccountOneMatch(page);

    expect(result.commitmentOfCreatedAccount).toEqual(
      result.commitmentOfGetAccountResult
    );
    expect(result.isAccountType).toBe(true);
  });

  test("returns error attempting to retrieve a non-existing account", async ({
    page,
  }) => {
    const result = await getAccountNoMatch(page);

    expect(result.commitmentOfGetAccountResult).toBeUndefined();
  });
});

// GET_ACCOUNTS TESTS
// =======================================================================================================

interface GetAccountsSuccessResult {
  commitmentsOfCreatedAccounts: string[];
  commitmentsOfGetAccountsResult: string[];
  resultTypes: boolean[];
}

export const getAccountsManyMatches = async (
  page: Page
): Promise<GetAccountsSuccessResult> => {
  return await page.evaluate(async () => {
    const client = window.client;
    const newAccount1 = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const newAccount2 = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const commitmentsOfCreatedAccounts = [
      newAccount1.commitment().toHex(),
      newAccount2.commitment().toHex(),
    ];

    const result = await client.getAccounts();

    const commitmentsOfGetAccountsResult = [];
    const resultTypes = [];

    for (let i = 0; i < result.length; i++) {
      commitmentsOfGetAccountsResult.push(result[i].commitment().toHex());
      resultTypes.push(result[i] instanceof window.AccountHeader);
    }

    return {
      commitmentsOfCreatedAccounts: commitmentsOfCreatedAccounts,
      commitmentsOfGetAccountsResult: commitmentsOfGetAccountsResult,
      resultTypes: resultTypes,
    };
  });
};

export const getAccountsNoMatches = async (
  page: Page
): Promise<GetAccountsSuccessResult> => {
  return await page.evaluate(async () => {
    const client = window.client;

    const result = await client.getAccounts();

    const commitmentsOfGetAccountsResult = [];
    const resultTypes = [];

    for (let i = 0; i < result.length; i++) {
      commitmentsOfGetAccountsResult.push(result[i].commitment().toHex());
      resultTypes.push(result[i] instanceof window.AccountHeader);
    }

    return {
      commitmentsOfCreatedAccounts: [],
      commitmentsOfGetAccountsResult: commitmentsOfGetAccountsResult,
      resultTypes: resultTypes,
    };
  });
};

test.describe("getAccounts tests", () => {
  test("retrieves all existing accounts", async ({ page }) => {
    const result = await getAccountsManyMatches(page);

    for (let address of result.commitmentsOfGetAccountsResult) {
      expect(result.commitmentsOfCreatedAccounts.includes(address)).toBe(true);
    }
    expect(result.resultTypes).toEqual([true, true]);
  });

  test("returns empty array when no accounts exist", async ({ page }) => {
    const result = await getAccountsNoMatches(page);

    expect(result.commitmentsOfCreatedAccounts.length).toEqual(0);
    expect(result.commitmentsOfGetAccountsResult.length).toEqual(0);
    expect(result.resultTypes.length).toEqual(0);
  });
});

test.describe("get public account with details", () => {
  test("assets and storage with too many assets/entries are retrieved", async ({
    page,
  }) => {
    const [assetCount, balances, mapEntriesCount] = await page.evaluate(
      async () => {
        // This account is inserted into the genesis block when test node is started,
        // it starts with assets from 1500 faucets, the function "build_test_faucets_and_accoung"
        // is called when the node starts and does the setup for this account, you can find it
        // in: miden-client/crates/testing/node-builder/src/lib.rs
        const accountID = window.AccountId.fromHex(
          "0x0a0a0a0a0a0a0a100a0a0a0a0a0a0a"
        );
        await window.client.importAccountById(accountID);
        const account = await window.client.getAccount(accountID);
        const storage = account
          ?.storage()
          .getMapEntries("miden::test_account::map::too_many_entries");
        console.log("Storage length", storage?.length);
        const vault = account?.vault();
        const assets = vault?.fungibleAssets()!;
        const assetCount = assets.length;
        const balances = [];
        for (const asset of assets) {
          balances.push(vault?.getBalance(asset.faucetId()).toString());
        }
        const mapEntries = account
          ?.storage()
          .getMapEntries("miden::test_account::map::too_many_entries");
        return [assetCount, balances, mapEntries?.length];
      },
      {}
    );
    expect(assetCount).toBe(1501);
    expect(balances.every((balance) => balance === "100")).toBe(true);
    expect(mapEntriesCount).toBe(2000);
  });
});
test.describe("account public commitments", () => {
  test("properly stores public commitments", async ({ page }) => {
    const commitmentsCount = await page.evaluate(async () => {
      const newAccount = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const accountId = newAccount.id();

      const sk1 = window.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = window.AuthSecretKey.rpoFalconWithRNG(null);

      await window.client.addAccountSecretKeyToWebStore(accountId, sk1);
      await window.client.addAccountSecretKeyToWebStore(accountId, sk2);

      const commitments =
        await window.client.getPublicKeyCommitmentsOfAccount(accountId);

      return commitments.length;
    }, {});
    expect(commitmentsCount).toBe(3);
  });

  test("retrieve auth keys with pk commitments and verify signatures", async ({
    page,
  }) => {
    const allSksRetrieved = await page.evaluate(async () => {
      const accountId = window.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );

      const sk1 = window.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = window.AuthSecretKey.rpoFalconWithRNG(null);
      const sk3 = window.AuthSecretKey.rpoFalconWithRNG(null);

      await window.client.addAccountSecretKeyToWebStore(accountId, sk1);
      await window.client.addAccountSecretKeyToWebStore(accountId, sk2);
      await window.client.addAccountSecretKeyToWebStore(accountId, sk3);

      const commitments =
        await window.client.getPublicKeyCommitmentsOfAccount(accountId);

      let sk1Retrieved = false;
      let sk2Retrieved = false;
      let sk3Retrieved = false;

      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
      const signingInputs = window.SigningInputs.newBlind(message);

      for (const commitment of commitments) {
        const retrievedSk =
          await window.client.getAccountAuthByPubKeyCommitment(commitment);
        const signature = retrievedSk.signData(signingInputs);

        sk1Retrieved =
          sk1Retrieved || sk1.publicKey().verify(message, signature);
        sk2Retrieved =
          sk2Retrieved || sk2.publicKey().verify(message, signature);
        sk3Retrieved =
          sk3Retrieved || sk3.publicKey().verify(message, signature);
      }
      return sk1Retrieved && sk2Retrieved && sk3Retrieved;
    }, {});
    expect(allSksRetrieved).toBe(true);
  });

  test("non-registered account id does not have any commitments", async ({
    page,
  }) => {
    const allSksRetrieved = await page.evaluate(async () => {
      const accountId = window.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );
      const commitments =
        await window.client.getPublicKeyCommitmentsOfAccount(accountId);
      return commitments.length;
    }, {});
    expect(allSksRetrieved).toBe(0);
  });

  test("can retrieve pk commitment after wallet creation", async ({ page }) => {
    const allSksRetrieved = await page.evaluate(async () => {
      const account = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const commitments = await window.client.getPublicKeyCommitmentsOfAccount(
        account.id()
      );
      return commitments.length == 1;
    }, {});
    expect(allSksRetrieved).toBe(true);
  });

  test("separate account ids get their respective pk commitments", async ({
    page,
  }) => {
    const allSksRetrieved = await page.evaluate(async () => {
      const accountId1 = window.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );

      const sk1 = window.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = window.AuthSecretKey.rpoFalconWithRNG(null);

      await window.client.addAccountSecretKeyToWebStore(accountId1, sk1);
      await window.client.addAccountSecretKeyToWebStore(accountId1, sk2);

      const account1Commitments =
        await window.client.getPublicKeyCommitmentsOfAccount(accountId1);

      const accountId2 = window.AccountId.fromHex(
        "0x79817bcc6fb9f99027c2245f6979ef"
      );

      const sk3 = window.AuthSecretKey.rpoFalconWithRNG(null);

      await window.client.addAccountSecretKeyToWebStore(accountId2, sk3);

      const account2Commitments =
        await window.client.getPublicKeyCommitmentsOfAccount(accountId2);

      return account1Commitments.length == 2 && account2Commitments.length == 1;
    }, {});
    expect(allSksRetrieved).toBe(true);
  });
});

// GET_ACCOUNT_BY_PUBLIC_KEY TESTS
// =======================================================================================================

test.describe("getAccountByPublicKey tests", () => {
  test("finds wallet by public key after creation", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const commitments = await client.getPublicKeyCommitmentsOfAccount(
        wallet.id()
      );
      const secretKey = await client.getAccountAuthByPubKeyCommitment(
        commitments[0]
      );
      const publicKey = secretKey.publicKey();

      const foundAccount = await client.getAccountByPublicKey(publicKey);

      return {
        createdAccountId: wallet.id().toString(),
        foundAccountId: foundAccount?.id().toString(),
        found: foundAccount !== undefined,
      };
    });

    expect(result.found).toBe(true);
    expect(result.foundAccountId).toEqual(result.createdAccountId);
  });

  test("returns undefined for non-existent public key", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const randomSecretKey = window.AuthSecretKey.rpoFalconWithRNG(null);
      const randomPublicKey = randomSecretKey.publicKey();

      const foundAccount = await client.getAccountByPublicKey(randomPublicKey);

      return {
        found: foundAccount !== undefined,
      };
    });

    expect(result.found).toBe(false);
  });

  test("finds correct account among multiple accounts", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const wallet1 = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const wallet2 = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const commitments2 = await client.getPublicKeyCommitmentsOfAccount(
        wallet2.id()
      );
      const secretKey2 = await client.getAccountAuthByPubKeyCommitment(
        commitments2[0]
      );
      const publicKey2 = secretKey2.publicKey();

      const foundAccount = await client.getAccountByPublicKey(publicKey2);

      return {
        wallet1Id: wallet1.id().toString(),
        wallet2Id: wallet2.id().toString(),
        foundAccountId: foundAccount?.id().toString(),
      };
    });

    expect(result.foundAccountId).toEqual(result.wallet2Id);
    expect(result.foundAccountId).not.toEqual(result.wallet1Id);
  });

  test("finds account by additionally registered key", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );

      const additionalSecretKey = window.AuthSecretKey.ecdsaWithRNG(null);
      await client.addAccountSecretKeyToWebStore(
        wallet.id(),
        additionalSecretKey
      );

      const additionalPublicKey = additionalSecretKey.publicKey();
      const foundAccount =
        await client.getAccountByPublicKey(additionalPublicKey);

      return {
        walletId: wallet.id().toString(),
        foundAccountId: foundAccount?.id().toString(),
        found: foundAccount !== undefined,
      };
    });

    expect(result.found).toBe(true);
    expect(result.foundAccountId).toEqual(result.walletId);
  });

  test("finds faucet by public key", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const faucet = await client.newFaucet(
        window.AccountStorageMode.private(),
        false,
        "TST",
        8,
        BigInt(10000000),
        0
      );

      const commitments = await client.getPublicKeyCommitmentsOfAccount(
        faucet.id()
      );
      const secretKey = await client.getAccountAuthByPubKeyCommitment(
        commitments[0]
      );
      const publicKey = secretKey.publicKey();

      const foundAccount = await client.getAccountByPublicKey(publicKey);

      return {
        faucetId: faucet.id().toString(),
        foundAccountId: foundAccount?.id().toString(),
        foundIsFaucet: foundAccount?.isFaucet(),
      };
    });

    expect(result.foundAccountId).toEqual(result.faucetId);
    expect(result.foundIsFaucet).toBe(true);
  });
});
