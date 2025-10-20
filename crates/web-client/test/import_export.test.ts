// TODO: Rename this / figure out rebasing with the other feature which has import tests

import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  clearStore,
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccountBalance,
  setupWalletAndFaucet,
  StorageMode,
} from "./webClientTestUtils";

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

const exportAccount = async (testingPage: Page, accountId: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const accountFile = await client.exportAccountFile(accountId);
    return Array.from(accountFile.serialize());
  }, accountId);
};

const importAccount = async (testingPage: Page, accountBytes: number[]) => {
  return await testingPage.evaluate(async (_accountBytes) => {
    const client = window.client;
    const bytes = new Uint8Array(_accountBytes);
    const accountFile = window.AccountFile.deserialize(bytes);
    await client.importAccountFile(accountFile);
  }, accountBytes);
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

    expect(accountCommitment).toEqual(initialAccountCommitment);
  });
});

test.describe("export and import account", () => {
  // Disabled due to out of bounds memory error:
  // https://github.com/0xMiden/miden-vm/issues/2303
  test.skip("should export and import a private account", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PRIVATE;

    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      walletSeed,
    });
    const faucet = await createNewFaucet(page);

    const { targetAccountBalance: initialBalance } =
      await fundAccountFromFaucet(page, initialWallet.id, faucet.id);
    const { accountCommitment: initialCommitment } = await getAccount(
      initialWallet.id,
      page
    );
    const exportedAccount = await exportAccount(page, initialWallet.id);
    await clearStore(page);

    await importAccount(page, exportedAccount);

    const { accountCommitment: restoredCommitment } = await getAccount(
      initialWallet.id,
      page
    );

    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredCommitment).toEqual(initialCommitment);
    expect(restoredBalance.toString()).toEqual(initialBalance);
  });
});
