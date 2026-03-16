// @ts-nocheck
import { test, expect } from "./test-setup";
import {
  setupWalletAndFaucet,
  mockMint,
  mockMintAndConsume,
} from "./test-helpers";

// GET_TRANSACTIONS TESTS
// =======================================================================================================

test.describe("get_transactions tests", () => {
  test("get_transactions retrieves all transactions successfully", async ({
    client,
    sdk,
  }) => {
    const { wallet, faucet } = await setupWalletAndFaucet(client, sdk);
    const { mintTransactionId, consumeTransactionId } =
      await mockMintAndConsume(client, sdk, wallet.id(), faucet.id());

    const transactions = await client.getTransactions(
      sdk.TransactionFilter.all()
    );
    const transactionIds = transactions.map((tx) => tx.id().toHex());
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );

    expect(transactionIds).toContain(mintTransactionId);
    expect(transactionIds).toContain(consumeTransactionId);
    expect(uncommitted.length).toEqual(0);
  });

  test("get_transactions retrieves uncommitted transactions successfully", async ({
    client,
    sdk,
  }) => {
    const { wallet, faucet } = await setupWalletAndFaucet(client, sdk);
    const { mintTransactionId, consumeTransactionId } =
      await mockMintAndConsume(client, sdk, wallet.id(), faucet.id());
    const { transactionId: uncommittedTransactionId } = await mockMint(
      client,
      sdk,
      wallet.id(),
      faucet.id(),
      { skipSync: true }
    );

    const transactions = await client.getTransactions(
      sdk.TransactionFilter.all()
    );
    const transactionIds = transactions.map((tx) => tx.id().toHex());
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );
    const uncommittedTransactionIds = uncommitted.map((tx) => tx.id().toHex());

    expect(transactionIds).toContain(mintTransactionId);
    expect(transactionIds).toContain(consumeTransactionId);
    expect(transactionIds).toContain(uncommittedTransactionId);
    expect(transactionIds.length).toEqual(3);

    expect(uncommittedTransactionIds).toContain(uncommittedTransactionId);
    expect(uncommittedTransactionIds.length).toEqual(1);
  });

  test("get_transactions retrieves no transactions successfully", async ({
    client,
    sdk,
  }) => {
    const transactions = await client.getTransactions(
      sdk.TransactionFilter.all()
    );
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );

    expect(transactions.length).toEqual(0);
    expect(uncommitted.length).toEqual(0);
  });

  test("get_transactions filters by specific transaction IDs successfully", async ({
    client,
    sdk,
  }) => {
    const { wallet, faucet } = await setupWalletAndFaucet(client, sdk);
    await mockMintAndConsume(client, sdk, wallet.id(), faucet.id());

    const allTransactions = await client.getTransactions(
      sdk.TransactionFilter.all()
    );
    const firstTransactionId = allTransactions[0].id();
    const firstTxIdHex = firstTransactionId.toHex();

    const filter = sdk.TransactionFilter.ids([firstTransactionId]);
    const filteredTransactions = await client.getTransactions(filter);
    const filteredTransactionIds = filteredTransactions.map((tx) =>
      tx.id().toHex()
    );

    expect(allTransactions.length).toEqual(2);
    expect(filteredTransactionIds.length).toEqual(1);
    expect(filteredTransactionIds).toContain(firstTxIdHex);
  });

  test("get_transactions filters expired transactions successfully", async ({
    client,
    sdk,
  }) => {
    const { wallet, faucet } = await setupWalletAndFaucet(client, sdk);

    const { transactionId: committedTransactionId } = await mockMint(
      client,
      sdk,
      wallet.id(),
      faucet.id()
    );

    const { transactionId: uncommittedTransactionId } = await mockMint(
      client,
      sdk,
      wallet.id(),
      faucet.id(),
      { skipSync: true }
    );

    const allTransactions = await client.getTransactions(
      sdk.TransactionFilter.all()
    );
    const allTransactionIds = allTransactions.map((tx) => tx.id().toHex());
    const currentBlockNum = allTransactions[0].blockNum();

    const futureBlockNum = currentBlockNum + 10;
    const futureExpiredTransactions = await client.getTransactions(
      sdk.TransactionFilter.expiredBefore(futureBlockNum)
    );
    const futureExpiredTransactionIds = futureExpiredTransactions.map((tx) =>
      tx.id().toHex()
    );

    const pastBlockNum = currentBlockNum - 10;
    const pastExpiredTransactions = await client.getTransactions(
      sdk.TransactionFilter.expiredBefore(pastBlockNum)
    );
    const pastExpiredTransactionIds = pastExpiredTransactions.map((tx) =>
      tx.id().toHex()
    );

    expect(futureExpiredTransactionIds.length).toEqual(1);
    expect(futureExpiredTransactionIds).toContain(uncommittedTransactionId);
    expect(pastExpiredTransactionIds.length).toEqual(0);
    expect(allTransactionIds.length).toEqual(2);
    expect(allTransactionIds).toContain(committedTransactionId);
    expect(allTransactionIds).toContain(uncommittedTransactionId);
  });
});

// COMPILE_TX_SCRIPT TESTS
// =======================================================================================================

test.describe("compile_tx_script tests", () => {
  test("compile_tx_script compiles script successfully", async ({
    client,
    sdk,
  }) => {
    await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const builder = await client.createCodeBuilder();
    const compiledScript = builder.compileNoteScript(`
      begin
        push.0 push.0
        assert_eq
      end
    `);

    expect(compiledScript.root().toHex().length).toBeGreaterThan(1);
  });

  test("compile_tx_script does not compile script successfully", async ({
    client,
    sdk,
  }) => {
    await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const builder = await client.createCodeBuilder();

    await expect(async () => {
      builder.compileNoteScript("fakeScript");
    }).rejects.toThrow(/failed to compile note script:/);
  });
});
