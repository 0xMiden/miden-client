import { expect } from "chai";
import { Page } from "@playwright/test";
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
      true
    );
    const result = await client.getAccount(newAccount.id());

    return {
      commitmentOfCreatedAccount: newAccount.commitment().toHex(),
      commitmentOfGetAccountResult: result.commitment().toHex(),
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

    expect(result.commitmentOfCreatedAccount).to.equal(
      result.commitmentOfGetAccountResult
    );
    expect(result.isAccountType).to.be.true;
  });

  test("returns error attempting to retrieve a non-existing account", async ({
    page,
  }) => {
    const result = await getAccountNoMatch(page);

    expect(result.commitmentOfGetAccountResult).to.be.undefined;
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
      true
    );
    const newAccount2 = await client.newWallet(
      window.AccountStorageMode.private(),
      true
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
      expect(result.commitmentsOfCreatedAccounts.includes(address)).to.be.true;
    }
    expect(result.resultTypes).to.deep.equal([true, true]);
  });

  test("returns empty array when no accounts exist", async ({ page }) => {
    const result = await getAccountsNoMatches(page);

    expect(result.commitmentsOfCreatedAccounts.length).to.equal(0);
    expect(result.commitmentsOfGetAccountsResult.length).to.equal(0);
    expect(result.resultTypes.length).to.equal(0);
  });
});
