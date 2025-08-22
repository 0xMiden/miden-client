// TODO: Rename this / figure out rebasing with the other featuer which has import tests

import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import {
  clearStore,
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccountBalance,
  setupWalletAndFaucet,
  StorageMode,
} from "./webClientTestUtils";

const exportDb = async () => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const db = await client.exportStore();
    const serialized = JSON.stringify(db);
    return serialized;
  });
};

const importDb = async (db: any) => {
  return await testingPage.evaluate(async (_db) => {
    const client = window.client;
    await client.forceImportStore(_db);
  }, db);
};

const getAccount = async (accountId: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const account = await client.getAccount(accountId);
    return {
      accountId: account?.id().toString(),
      accountCommitment: account?.commitment().toHex(),
    };
  }, accountId);
};

const exportAccount = async (accountId: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const accountBytes = client.exportAccountFile(accountId);
    return accountBytes;
  }, accountId);
};

const importAccount = async (accountBytes: any) => {
  return await testingPage.evaluate(async (_accountBytes) => {
    const client = window.client;
    await client.importAccountFile(_accountBytes);
    return;
  }, accountBytes);
};

describe("export and import the db", () => {
  it("export db with an account, find the account when re-importing", async () => {
    const { accountCommitment: initialAccountCommitment, accountId } =
      await setupWalletAndFaucet();
    const dbDump = await exportDb();

    await clearStore();

    await importDb(dbDump);

    const { accountCommitment } = await getAccount(accountId);

    expect(accountCommitment).to.equal(initialAccountCommitment);
  });
});

describe("export and import account", () => {
  it("should export and import a private account", async () => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PRIVATE;

    const initialWallet = await createNewWallet({
      storageMode,
      mutable,
      walletSeed,
    });
    const faucet = await createNewFaucet();

    const { targetAccountBalance: initialBalance } =
      await fundAccountFromFaucet(initialWallet.id, faucet.id);
    const { accountCommitment: initialCommitment } = await getAccount(
      initialWallet.id
    );
    const exportedAccount = await exportAccount(initialWallet.id);
    await clearStore();

    await importAccount(exportedAccount);

    const { accountCommitment: restoredCommitment } = await getAccount(
      initialWallet.id
    );

    const restoredBalance = await getAccountBalance(
      initialWallet.id,
      faucet.id
    );

    expect(restoredCommitment).to.equal(initialCommitment);
    expect(restoredBalance.toString()).to.equal(initialBalance);
  });
});
