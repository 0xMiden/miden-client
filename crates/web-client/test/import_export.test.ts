// TODO: Rename this / figure out rebasing with the other featuer which has import tests

import { expect } from "chai";
import test from "./playwright.global.setup";
import { Page } from "@playwright/test";
import { clearStore, setupWalletAndFaucet } from "./webClientTestUtils";

const exportDb = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window.client;
    const db = await client.exportStore();
    const serialized = JSON.stringify(db);
    return serialized;
  });
};

const importDb = async (db: any, page: Page) => {
  return await page.evaluate(async (_db) => {
    const client = window.client;
    await client.forceImportStore(_db);
  }, db);
};

const getAccount = async (accountId: string, page: Page) => {
  return await page.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const account = await client.getAccount(accountId);
    return {
      accountId: account?.id().toString(),
      accountCommitment: account?.commitment().toHex(),
    };
  }, accountId);
};

test.describe("export and import the db", () => {
  test("export db with an account, find the account when re-importing", async ({
    page,
  }) => {
    const { accountCommitment: initialAccountCommitment, accountId } =
      await setupWalletAndFaucet(page);
    const dbDump = await exportDb(page);

    await clearStore(page);

    await importDb(dbDump, page);

    const { accountCommitment } = await getAccount(accountId, page);

    expect(accountCommitment).to.equal(initialAccountCommitment);
  });
});
