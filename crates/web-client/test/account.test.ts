import { Page, expect } from "@playwright/test";
import test from "./playwright.global.setup";
import {
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccount,
  StorageMode,
} from "./webClientTestUtils";

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
  return await page.evaluate(async (page: Page) => {
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

test.describe("fetch account tests", () => {
  test("retrieves a public account", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PUBLIC;
    const authSchemeId = 0;
    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      authSchemeId,
      walletSeed,
    });
    const faucet = await createNewFaucet(page);

    const { targetAccountBalance: initialBalance } =
      await fundAccountFromFaucet(page, initialWallet.id, faucet.id);
    const { commitment } = await getAccount(page, initialWallet.id);

    const { isPublic, accountIsDefined, fetchedCommitment, balance } =
      await page.evaluate(async (accountId: string) => {
        const endpoint = new window.Endpoint(window.rpcUrl);
        const rpcClient = new window.RpcClient(endpoint);

        const fetchedAccount = await rpcClient.getAccountDetails(
          window.AccountId.fromHex(accountId)
        );
        return {
          isPublic: fetchedAccount.isPublic(),
          accountIsDefined: !!fetchedAccount.account(),
          fetchedCommitment: fetchedAccount.commitment().toHex(),
          balance: fetchedAccount.account()
            ? fetchedAccount
                .account()!
                .vault()
                .fungibleAssets()[0]
                .amount()
                .toString()
            : undefined,
        };
      }, initialWallet.id);
    expect(isPublic).toBe(true);
    expect(accountIsDefined).toBe(true);
    expect(fetchedCommitment).toBe(commitment);
    expect(balance).toBe(initialBalance);
  });

  test("retrieves a private account", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PRIVATE;
    const authSchemeId = 0;
    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      authSchemeId,
      walletSeed,
    });
    const faucet = await createNewFaucet(page);

    await fundAccountFromFaucet(page, initialWallet.id, faucet.id);

    const { commitment } = await getAccount(page, initialWallet.id);

    const { isPrivate, accountIsDefined, fetchedCommitment } =
      await page.evaluate(async (accountId: string) => {
        const endpoint = new window.Endpoint(window.rpcUrl);
        const rpcClient = new window.RpcClient(endpoint);

        const fetchedAccount = await rpcClient.getAccountDetails(
          window.AccountId.fromHex(accountId)
        );
        return {
          isPrivate: fetchedAccount.isPrivate(),
          accountIsDefined: !!fetchedAccount.account(),
          fetchedCommitment: fetchedAccount.commitment().toHex(),
        };
      }, initialWallet.id);
    expect(isPrivate).toBe(true);
    expect(accountIsDefined).toBe(false);
    expect(fetchedCommitment).toBe(commitment);
  });
});
