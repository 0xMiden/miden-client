import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccount,
  getAccountBalance,
  StorageMode,
} from "./webClientTestUtils";

const importWalletFromSeed = async (
  page: Page,
  walletSeed: Uint8Array,
  mutable: boolean,
  authSchemeId: number
) => {
  return await page.evaluate(
    async ({ _walletSeed, mutable, authSchemeId }) => {
      const client = window.client;
      await client.syncState();
      const account = await client.importPublicAccountFromSeed(
        _walletSeed,
        mutable,
        authSchemeId
      );
      return {
        accountId: account.id().toString(),
        accountCommitment: account.commitment().toHex(),
      };
    },
    {
      _walletSeed: walletSeed,
      mutable,
      authSchemeId,
    }
  );
};

// Creates a new WebClient with a fresh IndexedDB store, simulating a clean
// start. This ensures both the store and the RPC client cache are empty.
const createFreshClient = async (page: Page) => {
  await page.evaluate(async () => {
    const freshStoreName = `test_fresh_${crypto.randomUUID().slice(0, 8)}`;
    const client = await window.WasmWebClient.createClient(
      window.rpcUrl,
      undefined,
      undefined,
      freshStoreName
    );
    window.client = client;
  });
};

const importAccountById = async (page: Page, accountId: string) => {
  return await page.evaluate(async (id) => {
    const client = window.client;
    const _accountId = window.AccountId.fromHex(id);
    await client.importAccountById(_accountId);
    const account = await client.getAccount(_accountId);
    return {
      accountId: account?.id().toString(),
      accountCommitment: account?.commitment().toHex(),
    };
  }, accountId);
};

test.describe("import from seed", () => {
  test("should import same public account from seed", async ({ page }) => {
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

    const result = await fundAccountFromFaucet(
      page,
      initialWallet.id,
      faucet.id
    );
    const initialBalance = result.targetAccountBalance;

    const { commitment: initialCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    // Deleting the account
    await createFreshClient(page);

    const { accountId: restoredAccountId } = await importWalletFromSeed(
      page,
      walletSeed,
      mutable,
      authSchemeId
    );

    expect(restoredAccountId).toEqual(initialWallet.id);

    const { commitment: restoredAccountCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredBalance!.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});

test.describe("import public account by id", () => {
  test("should import public account from id", async ({ page }) => {
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
    const { commitment: initialCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    await createFreshClient(page);

    const { accountId: restoredAccountId } = await importAccountById(
      page,
      initialWallet.id
    );
    expect(restoredAccountId).toEqual(initialWallet.id);

    const { commitment: restoredAccountCommitment } = await getAccount(
      page,
      initialWallet.id
    );
    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredBalance!.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});
