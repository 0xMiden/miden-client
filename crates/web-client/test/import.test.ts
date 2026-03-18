// @ts-nocheck
import { test, expect } from "./test-setup";
import {
  createIntegrationClient,
  integrationMint,
  integrationConsume,
} from "./test-helpers";

test.describe("import from seed", () => {
  test("should import same public account from seed", async ({ sdk }) => {
    test.slow();
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;

    const initialWallet = await client.newWallet(
      sdk.AccountStorageMode.public(),
      mutable,
      sdk.AuthScheme.AuthRpoFalcon512,
      walletSeed
    );
    const initialWalletId = initialWallet.id();

    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.public(),
      false,
      "DAG",
      8,
      sdk.u64(10000000),
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const faucetId = faucet.id();

    // Mint and consume to fund the wallet
    const { createdNoteId } = await integrationMint(
      client,
      sdk,
      initialWalletId,
      faucetId
    );
    const { targetAccountBalance: initialBalance } = await integrationConsume(
      client,
      sdk,
      initialWalletId,
      faucetId,
      createdNoteId
    );

    const initialAccount = await client.getAccount(initialWalletId);
    const initialCommitment = initialAccount.to_commitment().toHex();

    // Create a fresh client (separate store) and import the wallet from seed
    const result2 = await createIntegrationClient();
    test.skip(!result2, "requires running node");
    const { client: freshClient } = result2;

    await freshClient.syncState();
    const restoredAccount = await freshClient.importPublicAccountFromSeed(
      walletSeed,
      mutable,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const restoredAccountId = restoredAccount.id().toString();
    expect(restoredAccountId).toEqual(initialWalletId.toString());

    const restoredAccountObj = await freshClient.getAccount(
      sdk.AccountId.fromHex(restoredAccountId)
    );
    const restoredAccountCommitment = restoredAccountObj
      .to_commitment()
      .toHex();

    const restoredBalance = restoredAccountObj
      .vault()
      .getBalance(sdk.AccountId.fromHex(faucetId.toString()));

    expect(restoredBalance.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});

test.describe("import public account by id", () => {
  test("should import public account from id", async ({ sdk }) => {
    test.slow();
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;

    const initialWallet = await client.newWallet(
      sdk.AccountStorageMode.public(),
      mutable,
      sdk.AuthScheme.AuthRpoFalcon512,
      walletSeed
    );
    const initialWalletId = initialWallet.id();

    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.public(),
      false,
      "DAG",
      8,
      sdk.u64(10000000),
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const faucetId = faucet.id();

    // Mint and consume to fund the wallet
    const { createdNoteId } = await integrationMint(
      client,
      sdk,
      initialWalletId,
      faucetId
    );
    const { targetAccountBalance: initialBalance } = await integrationConsume(
      client,
      sdk,
      initialWalletId,
      faucetId,
      createdNoteId
    );

    const initialAccount = await client.getAccount(initialWalletId);
    const initialCommitment = initialAccount.to_commitment().toHex();

    // Create a fresh client (separate store) and import by account ID
    const result2 = await createIntegrationClient();
    test.skip(!result2, "requires running node");
    const { client: freshClient } = result2;

    const accountIdObj = sdk.AccountId.fromHex(initialWalletId.toString());
    await freshClient.importAccountById(accountIdObj);
    const restoredAccount = await freshClient.getAccount(accountIdObj);

    const restoredAccountId = restoredAccount.id().toString();
    expect(restoredAccountId).toEqual(initialWalletId.toString());

    const restoredAccountCommitment = restoredAccount.to_commitment().toHex();
    const restoredBalance = restoredAccount
      .vault()
      .getBalance(sdk.AccountId.fromHex(faucetId.toString()));

    expect(restoredBalance.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});
