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

test.describe("get account with details", () => {
  test("get account with details", async ({ page }) => {
    const [assetCount, balances] = await page.evaluate(async () => {
      // This account is inserted into the genesis block when test node is started,
      // it starts with assets from 1500 faucets, the function "build_test_faucets_and_accoung"
      // is called when the node starts and does the setup for this account, you can find it
      // in: miden-client/crates/testing/node-builder/src/lib.rs
      const accountID = window.AccountId.fromHex(
        "0x0a0a0a0a0a0a0a100a0a0a0a0a0a0a"
      );
      await window.client.importAccountById(accountID);
      const account = await window.client.getAccount(accountID);
      const vault = account?.vault();
      const assets = vault?.fungibleAssets()!;
      const assetCount = assets.length;
      const balances = [];
      for (const asset of assets) {
        balances.push(vault?.getBalance(asset.faucetId()).toString());
      }
      return [assetCount, balances];
    }, {});
    expect(assetCount).toBe(1501);
    expect(balances.every((balance) => balance === "100")).toBe(true);
  });
});
